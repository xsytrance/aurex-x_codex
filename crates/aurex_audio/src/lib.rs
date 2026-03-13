pub mod analysis;
pub mod sequencer;
pub mod synth;
pub mod voice;

use analysis::{AudioFeatures, analyze_sequence};
use aurex_core::Tick;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use sequencer::AudioSequence;
use serde::{Deserialize, Serialize};
use std::sync::{
    Arc,
    atomic::{AtomicU32, Ordering},
};

use synth::{OscillatorType, SynthNode, sample_synth};
use voice::{Phoneme, VoicePreset, VoiceSynth};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioBackendMode {
    MockSilence,
    CpalPlanned,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioTransition {
    Noop,
    Transitioned,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioEngineStatus {
    pub mode: AudioBackendMode,
    pub ready: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioBackendReadiness {
    pub has_device_io: bool,
    pub has_stream_graph: bool,
    pub can_emit_sound: bool,
}

impl AudioBackendReadiness {
    pub fn for_mode(mode: AudioBackendMode) -> Self {
        match mode {
            AudioBackendMode::MockSilence => Self {
                has_device_io: false,
                has_stream_graph: false,
                can_emit_sound: false,
            },
            AudioBackendMode::CpalPlanned => Self {
                has_device_io: true,
                has_stream_graph: true,
                can_emit_sound: true,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AudioClock {
    pub tick: Tick,
}

impl Default for AudioClock {
    fn default() -> Self {
        Self { tick: Tick(0) }
    }
}

impl AudioClock {
    pub fn advance(&mut self) {
        self.tick.0 += 1;
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BeatEvent {
    pub tick: Tick,
    pub pulse: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProceduralAudioConfig {
    pub tempo: f32,
    #[serde(default)]
    pub tracks: Vec<sequencer::AudioTrack>,
    #[serde(default)]
    pub synth_graph: Option<SynthNode>,
    #[serde(default)]
    pub voice: Option<VoiceSynthConfig>,
    #[serde(default)]
    pub seed: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VoiceSynthConfig {
    pub preset: VoicePreset,
    pub phonemes: Vec<Phoneme>,
    pub base_pitch_hz: f32,
    pub phoneme_duration: f32,
}

impl VoiceSynthConfig {
    pub fn to_voice_synth(&self) -> VoiceSynth {
        VoiceSynth {
            preset: self.preset,
            sequence: self.phonemes.clone(),
            base_pitch_hz: self.base_pitch_hz,
            phoneme_duration: self.phoneme_duration,
        }
    }
}

pub fn synthesize_mono_sample(cfg: &ProceduralAudioConfig, t: f32, sample_rate: f32) -> f32 {
    let mut sample = 0.0;

    if let Some(graph) = &cfg.synth_graph {
        sample += sample_synth(graph, t, sample_rate, cfg.seed);
    }

    if let Some(v) = &cfg.voice {
        sample += v.to_voice_synth().sample(t) * 0.7;
    }

    if !cfg.tracks.is_empty() {
        let seq = AudioSequence {
            bpm: cfg.tempo,
            tracks: cfg.tracks.clone(),
        };
        sample += seq.sample_energy(t) * 0.3;
    }

    sample.clamp(-1.0, 1.0)
}

pub fn analyze_procedural_audio(cfg: &ProceduralAudioConfig, t: f32) -> AudioFeatures {
    let seq = AudioSequence {
        bpm: cfg.tempo,
        tracks: cfg.tracks.clone(),
    };
    analyze_sequence(&seq, t, cfg.seed)
}

#[derive(Debug, Clone)]
pub struct MockAudioEngine {
    mode: AudioBackendMode,
    ready: bool,
    clock: AudioClock,
}

impl Default for MockAudioEngine {
    fn default() -> Self {
        Self {
            mode: AudioBackendMode::MockSilence,
            ready: true,
            clock: AudioClock::default(),
        }
    }
}

impl MockAudioEngine {
    pub fn status(&self) -> AudioEngineStatus {
        AudioEngineStatus {
            mode: self.mode,
            ready: self.ready,
        }
    }

    pub fn transition_mode(&mut self, mode: AudioBackendMode) -> AudioTransition {
        if self.mode == mode {
            return AudioTransition::Noop;
        }

        self.mode = mode;
        self.ready = mode == AudioBackendMode::MockSilence;
        AudioTransition::Transitioned
    }

    pub fn next_beat(&mut self) -> BeatEvent {
        self.clock.advance();
        let phase = (self.clock.tick.0 % 8) as f32 / 8.0;
        let pulse = (phase * std::f32::consts::TAU).sin() * 0.5 + 0.5;
        BeatEvent {
            tick: self.clock.tick,
            pulse,
        }
    }
}

pub struct RuntimeAudioOutput {
    _stream: cpal::Stream,
    pulse: Arc<AtomicU32>,
}

#[derive(Clone)]
pub struct RuntimePulseControl {
    pulse: Arc<AtomicU32>,
}

impl RuntimePulseControl {
    pub fn set_pulse(&self, pulse: f32) {
        self.pulse
            .store(pulse.clamp(0.0, 1.0).to_bits(), Ordering::Relaxed);
    }
}

impl RuntimeAudioOutput {
    pub fn set_pulse(&self, pulse: f32) {
        self.pulse
            .store(pulse.clamp(0.0, 1.0).to_bits(), Ordering::Relaxed);
    }

    pub fn pulse_control(&self) -> RuntimePulseControl {
        RuntimePulseControl {
            pulse: Arc::clone(&self.pulse),
        }
    }
}

pub fn start_runtime_sine_output() -> Result<RuntimeAudioOutput, String> {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or_else(|| "no default audio output device available".to_string())?;

    let default_config = device
        .default_output_config()
        .map_err(|e| format!("default_output_config failed: {e}"))?;

    let sample_rate = default_config.sample_rate().0 as f32;
    let channels = default_config.channels() as usize;
    let pulse = Arc::new(AtomicU32::new(0.0f32.to_bits()));

    let err_fn = |err| eprintln!("audio_stream_error={err}");

    let stream = match default_config.sample_format() {
        cpal::SampleFormat::F32 => {
            let pulse = Arc::clone(&pulse);
            let mut synth_state = RuntimeBootSynthState::default();
            device
                .build_output_stream(
                    &default_config.config(),
                    move |data: &mut [f32], _| {
                        write_boot_data(data, channels, sample_rate, &mut synth_state, &pulse)
                    },
                    err_fn,
                    None,
                )
                .map_err(|e| format!("build_output_stream(f32) failed: {e}"))?
        }
        cpal::SampleFormat::I16 => {
            let pulse = Arc::clone(&pulse);
            let mut synth_state = RuntimeBootSynthState::default();
            device
                .build_output_stream(
                    &default_config.config(),
                    move |data: &mut [i16], _| {
                        write_boot_data(data, channels, sample_rate, &mut synth_state, &pulse)
                    },
                    err_fn,
                    None,
                )
                .map_err(|e| format!("build_output_stream(i16) failed: {e}"))?
        }
        cpal::SampleFormat::U16 => {
            let pulse = Arc::clone(&pulse);
            let mut synth_state = RuntimeBootSynthState::default();
            device
                .build_output_stream(
                    &default_config.config(),
                    move |data: &mut [u16], _| {
                        write_boot_data(data, channels, sample_rate, &mut synth_state, &pulse)
                    },
                    err_fn,
                    None,
                )
                .map_err(|e| format!("build_output_stream(u16) failed: {e}"))?
        }
        other => {
            return Err(format!("unsupported output sample format: {other:?}"));
        }
    };

    stream
        .play()
        .map_err(|e| format!("stream play failed: {e}"))?;

    Ok(RuntimeAudioOutput {
        _stream: stream,
        pulse,
    })
}

#[derive(Debug, Default, Clone, Copy)]
struct RuntimeBootSynthState {
    sub_phase: f32,
    pad_phase: f32,
    harmonic_phase: f32,
    width_phase: f32,
    thump_phase: f32,
    thump_env: f32,
    last_pulse: f32,
}

fn write_boot_data<T>(
    output: &mut [T],
    channels: usize,
    sample_rate: f32,
    state: &mut RuntimeBootSynthState,
    pulse: &Arc<AtomicU32>,
) where
    T: cpal::Sample + cpal::FromSample<f32>,
{
    let pulse_value = f32::from_bits(pulse.load(Ordering::Relaxed)).clamp(0.0, 1.0);
    let sub_freq_hz = 44.0 + pulse_value * 20.0;
    let pad_freq_hz = 92.0 + pulse_value * 46.0;
    let harmonic_freq_hz = pad_freq_hz * (1.75 + pulse_value * 0.2);
    let beat_hz = 0.75 + pulse_value * 1.6;
    let master_amp = 0.05 + pulse_value * 0.03;

    let sub_inc = sub_freq_hz / sample_rate;
    let pad_inc = pad_freq_hz / sample_rate;
    let harmonic_inc = harmonic_freq_hz / sample_rate;
    let width_inc = (0.07 + pulse_value * 0.08) / sample_rate;
    let beat_inc = beat_hz / sample_rate;

    for frame in output.chunks_mut(channels.max(1)) {
        state.sub_phase = (state.sub_phase + sub_inc).fract();
        state.pad_phase = (state.pad_phase + pad_inc).fract();
        state.harmonic_phase = (state.harmonic_phase + harmonic_inc).fract();
        state.width_phase = (state.width_phase + width_inc).fract();

        let next_thump = (state.thump_phase + beat_inc).fract();
        let crossed_beat = next_thump < state.thump_phase;
        state.thump_phase = next_thump;

        if crossed_beat || (pulse_value - state.last_pulse) > 0.18 {
            state.thump_env = 1.0;
        }

        let sub_sine = (state.sub_phase * std::f32::consts::TAU).sin();
        let sub_tri = 4.0 * (state.sub_phase - 0.5).abs() - 1.0;
        let sub_layer = sub_sine * 0.7 + sub_tri * 0.3;

        let pad = (state.pad_phase * std::f32::consts::TAU).sin() * 0.7
            + (state.harmonic_phase * std::f32::consts::TAU).sin() * (0.12 + pulse_value * 0.16);

        let thump = (state.sub_phase * std::f32::consts::TAU).sin() * state.thump_env;
        state.thump_env *= 0.9992 - pulse_value * 0.00015;
        state.thump_env = state.thump_env.clamp(0.0, 1.0);

        let center = (sub_layer * 0.62 + pad * 0.24 + thump * 0.16) * master_amp;
        let width = ((state.width_phase * std::f32::consts::TAU).sin() * 0.5 + 0.5)
            * (0.008 + pulse_value * 0.012);
        let left = (center + width).clamp(-0.18, 0.18);
        let right = (center - width).clamp(-0.18, 0.18);
        state.last_pulse = pulse_value;

        if channels >= 2 {
            frame[0] = T::from_sample(left);
            frame[1] = T::from_sample(right);
            for chan in frame.iter_mut().skip(2) {
                *chan = T::from_sample(center);
            }
        } else {
            frame[0] = T::from_sample(center);
        }
    }
}

pub fn default_demo_audio_config(seed: u32) -> ProceduralAudioConfig {
    ProceduralAudioConfig {
        tempo: 140.0,
        tracks: vec![sequencer::AudioTrack {
            name: "bassline".into(),
            patterns: vec![sequencer::AudioPattern {
                notes: vec![
                    sequencer::AudioNote {
                        pitch: 36,
                        duration_beats: 0.5,
                        velocity: 0.9,
                        instrument: "fm_bass".into(),
                    },
                    sequencer::AudioNote {
                        pitch: 38,
                        duration_beats: 0.5,
                        velocity: 0.7,
                        instrument: "fm_bass".into(),
                    },
                ],
                loops: 8,
            }],
            volume: 0.8,
        }],
        synth_graph: Some(SynthNode::Mixer {
            inputs: vec![
                SynthNode::Oscillator {
                    osc_type: OscillatorType::Saw,
                    frequency: 110.0,
                    amplitude: 0.5,
                    phase: 0.0,
                },
                SynthNode::FMOperator {
                    carrier_freq: 220.0,
                    mod_freq: 55.0,
                    mod_index: 2.4,
                    amplitude: 0.3,
                },
            ],
            gain: 0.7,
        }),
        voice: Some(VoiceSynthConfig {
            preset: VoicePreset::Robot,
            phonemes: vec![Phoneme::EH, Phoneme::OH],
            base_pitch_hz: 160.0,
            phoneme_duration: 0.22,
        }),
        seed,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AudioBackendMode, AudioBackendReadiness, AudioTransition, MockAudioEngine,
        analyze_procedural_audio, default_demo_audio_config, synthesize_mono_sample,
    };

    #[test]
    fn readiness_contract_tracks_audio_mode() {
        let silent = AudioBackendReadiness::for_mode(AudioBackendMode::MockSilence);
        assert!(!silent.has_device_io);
        assert!(!silent.has_stream_graph);
        assert!(!silent.can_emit_sound);

        let planned = AudioBackendReadiness::for_mode(AudioBackendMode::CpalPlanned);
        assert!(planned.has_device_io);
        assert!(planned.has_stream_graph);
        assert!(planned.can_emit_sound);
    }

    #[test]
    fn transition_marks_cpal_planned_not_ready() {
        let mut engine = MockAudioEngine::default();
        let transitioned = engine.transition_mode(AudioBackendMode::CpalPlanned);

        assert_eq!(transitioned, AudioTransition::Transitioned);
        let status = engine.status();
        assert_eq!(status.mode, AudioBackendMode::CpalPlanned);
        assert!(!status.ready);
    }

    #[test]
    fn beat_sequence_progresses_tick_and_range() {
        let mut engine = MockAudioEngine::default();
        let first = engine.next_beat();
        let second = engine.next_beat();

        assert_eq!(first.tick.0, 1);
        assert_eq!(second.tick.0, 2);
        assert!((0.0..=1.0).contains(&first.pulse));
        assert!((0.0..=1.0).contains(&second.pulse));
    }

    #[test]
    fn procedural_audio_is_deterministic() {
        let cfg = default_demo_audio_config(77);
        let a = synthesize_mono_sample(&cfg, 0.42, 48_000.0);
        let b = synthesize_mono_sample(&cfg, 0.42, 48_000.0);
        assert_eq!(a, b);

        let fa = analyze_procedural_audio(&cfg, 1.0);
        let fb = analyze_procedural_audio(&cfg, 1.0);
        assert_eq!(fa, fb);
    }
}
