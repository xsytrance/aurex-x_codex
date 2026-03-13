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
    atomic::{AtomicU32, AtomicUsize, Ordering},
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioEvent {
    Kick,
    Snare,
    Hat,
    BassNote(u8),
    PadNote(u8),
    LeadNote(u8),
}

#[derive(Debug)]
pub struct AudioEventQueue {
    buffer: [AtomicU32; 256],
    write_index: AtomicUsize,
    read_index: AtomicUsize,
}

impl Default for AudioEventQueue {
    fn default() -> Self {
        Self {
            buffer: std::array::from_fn(|_| AtomicU32::new(0)),
            write_index: AtomicUsize::new(0),
            read_index: AtomicUsize::new(0),
        }
    }
}

impl AudioEventQueue {
    pub fn push(&self, event: AudioEvent) {
        let w = self.write_index.load(Ordering::Relaxed);
        let next = (w + 1) & 255;
        let r = self.read_index.load(Ordering::Acquire);
        if next == r {
            return;
        }

        self.buffer[w].store(encode_audio_event(event), Ordering::Relaxed);
        self.write_index.store(next, Ordering::Release);
    }

    pub fn drain_into(&self, out: &mut Vec<AudioEvent>) {
        let mut r = self.read_index.load(Ordering::Relaxed);
        let w = self.write_index.load(Ordering::Acquire);
        while r != w {
            let encoded = self.buffer[r].swap(0, Ordering::Relaxed);
            if let Some(event) = decode_audio_event(encoded) {
                out.push(event);
            }
            r = (r + 1) & 255;
        }
        self.read_index.store(r, Ordering::Release);
    }
}

fn encode_audio_event(event: AudioEvent) -> u32 {
    match event {
        AudioEvent::Kick => 1,
        AudioEvent::Snare => 2,
        AudioEvent::Hat => 3,
        AudioEvent::BassNote(note) => 0x10_0000 | note as u32,
        AudioEvent::PadNote(note) => 0x20_0000 | note as u32,
        AudioEvent::LeadNote(note) => 0x30_0000 | note as u32,
    }
}

fn decode_audio_event(encoded: u32) -> Option<AudioEvent> {
    match encoded {
        1 => Some(AudioEvent::Kick),
        2 => Some(AudioEvent::Snare),
        3 => Some(AudioEvent::Hat),
        0 => None,
        other => {
            let kind = other & 0xF0_0000;
            let note = (other & 0xFF) as u8;
            match kind {
                0x10_0000 => Some(AudioEvent::BassNote(note)),
                0x20_0000 => Some(AudioEvent::PadNote(note)),
                0x30_0000 => Some(AudioEvent::LeadNote(note)),
                _ => None,
            }
        }
    }
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
    events: Arc<AudioEventQueue>,
}

#[derive(Clone)]
pub struct RuntimePulseControl {
    pulse: Arc<AtomicU32>,
}

#[derive(Clone)]
pub struct RuntimeEventReader {
    events: Arc<AudioEventQueue>,
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

    pub fn event_reader(&self) -> RuntimeEventReader {
        RuntimeEventReader {
            events: Arc::clone(&self.events),
        }
    }

    pub fn drain_audio_events(&self, out: &mut Vec<AudioEvent>) {
        self.events.drain_into(out);
    }
}

impl RuntimeEventReader {
    pub fn drain_into(&self, out: &mut Vec<AudioEvent>) {
        self.events.drain_into(out);
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
    let events = Arc::new(AudioEventQueue::default());

    let err_fn = |err| eprintln!("audio_stream_error={err}");

    let stream = match default_config.sample_format() {
        cpal::SampleFormat::F32 => {
            let pulse = Arc::clone(&pulse);
            let events = Arc::clone(&events);
            let mut synth_state = RuntimeBootSynthState::default();
            device
                .build_output_stream(
                    &default_config.config(),
                    move |data: &mut [f32], _| {
                        write_boot_data(
                            data,
                            channels,
                            sample_rate,
                            &mut synth_state,
                            &pulse,
                            &events,
                        )
                    },
                    err_fn,
                    None,
                )
                .map_err(|e| format!("build_output_stream(f32) failed: {e}"))?
        }
        cpal::SampleFormat::I16 => {
            let pulse = Arc::clone(&pulse);
            let events = Arc::clone(&events);
            let mut synth_state = RuntimeBootSynthState::default();
            device
                .build_output_stream(
                    &default_config.config(),
                    move |data: &mut [i16], _| {
                        write_boot_data(
                            data,
                            channels,
                            sample_rate,
                            &mut synth_state,
                            &pulse,
                            &events,
                        )
                    },
                    err_fn,
                    None,
                )
                .map_err(|e| format!("build_output_stream(i16) failed: {e}"))?
        }
        cpal::SampleFormat::U16 => {
            let pulse = Arc::clone(&pulse);
            let events = Arc::clone(&events);
            let mut synth_state = RuntimeBootSynthState::default();
            device
                .build_output_stream(
                    &default_config.config(),
                    move |data: &mut [u16], _| {
                        write_boot_data(
                            data,
                            channels,
                            sample_rate,
                            &mut synth_state,
                            &pulse,
                            &events,
                        )
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
        events,
    })
}

#[derive(Debug, Default, Clone, Copy)]
struct RuntimeBootSynthState {
    sub_phase: f32,
    bass_phase: f32,
    bass_env: f32,
    bass_sweep: f32,
    pad_phases: [f32; 7],
    lead_phase: f32,
    width_phase: f32,
    hat_env: f32,
    noise_state: u32,
    seq_phase: f32,
    seq_step: u32,
    last_pulse: f32,
}

fn write_boot_data<T>(
    output: &mut [T],
    channels: usize,
    sample_rate: f32,
    state: &mut RuntimeBootSynthState,
    pulse: &Arc<AtomicU32>,
    events: &Arc<AudioEventQueue>,
) where
    T: cpal::Sample + cpal::FromSample<f32>,
{
    let pulse_value = f32::from_bits(pulse.load(Ordering::Relaxed)).clamp(0.0, 1.0);
    let sub_freq_hz = 42.0 + pulse_value * 18.0;
    let beat_hz = 1.4 + pulse_value * 1.8;
    let master_amp = 0.048 + pulse_value * 0.022;
    let bass_freq_hz = 53.0 + pulse_value * 16.0;
    let pad_base_hz = 108.0 + pulse_value * 28.0;
    let lead_hz = 216.0 + pulse_value * 44.0;

    let sub_inc = sub_freq_hz / sample_rate;
    let bass_inc = bass_freq_hz / sample_rate;
    let lead_inc = lead_hz / sample_rate;
    let width_inc = (0.07 + pulse_value * 0.08) / sample_rate;
    let beat_inc = beat_hz / sample_rate;

    if state.noise_state == 0 {
        state.noise_state = 0xA5A5_1F2Du32;
    }

    const BASS_NOTES: [u8; 8] = [36, 36, 39, 36, 43, 39, 36, 34];
    const PAD_NOTES: [u8; 4] = [48, 55, 53, 60];
    const LEAD_NOTES: [u8; 8] = [72, 74, 76, 79, 81, 79, 76, 74];

    for frame in output.chunks_mut(channels.max(1)) {
        state.sub_phase = (state.sub_phase + sub_inc).fract();
        state.bass_phase = (state.bass_phase + bass_inc).fract();
        state.lead_phase = (state.lead_phase + lead_inc).fract();
        state.width_phase = (state.width_phase + width_inc).fract();

        let next_seq = (state.seq_phase + beat_inc).fract();
        let crossed_step = next_seq < state.seq_phase;
        state.seq_phase = next_seq;

        if crossed_step {
            let step = state.seq_step;
            events.push(AudioEvent::Kick);
            if step % 2 == 1 {
                events.push(AudioEvent::Snare);
            }
            events.push(AudioEvent::Hat);
            if step % 2 == 0 {
                events.push(AudioEvent::Hat);
            }

            let bass_note = BASS_NOTES[(step as usize) % BASS_NOTES.len()];
            let pad_note = PAD_NOTES[((step / 2) as usize) % PAD_NOTES.len()];
            let lead_note = LEAD_NOTES[(step as usize) % LEAD_NOTES.len()];
            events.push(AudioEvent::BassNote(bass_note));
            events.push(AudioEvent::PadNote(pad_note));
            if step % 2 == 0 {
                events.push(AudioEvent::LeadNote(lead_note));
            }

            state.bass_env = 1.0;
            state.bass_sweep = 1.0;
            state.hat_env = 1.0;
            state.seq_step = (step + 1) & 15;
        }

        if (pulse_value - state.last_pulse) > 0.2 {
            state.hat_env = (state.hat_env + 0.35).clamp(0.0, 1.0);
        }

        let sub_sine = (state.sub_phase * std::f32::consts::TAU).sin();
        let sub_tri = 4.0 * (state.sub_phase - 0.5).abs() - 1.0;
        let sub_layer = sub_sine * 0.75 + sub_tri * 0.25;

        let detunes = [-0.021_f32, -0.013, -0.008, 0.0, 0.008, 0.013, 0.021];
        let mut supersaw = 0.0;
        for (idx, detune) in detunes.iter().enumerate() {
            let inc = (pad_base_hz * (1.0 + detune)) / sample_rate;
            state.pad_phases[idx] = (state.pad_phases[idx] + inc).fract();
            let saw = state.pad_phases[idx] * 2.0 - 1.0;
            supersaw += saw;
        }
        let pad = supersaw / 7.0;

        let bass_body = (state.bass_phase * std::f32::consts::TAU).sin();
        let bass_voice = bass_body * state.bass_env * (0.65 + 0.35 * state.bass_sweep);
        state.bass_env *= 0.9963 - pulse_value * 0.0009;
        state.bass_sweep *= 0.9925;

        state.noise_state = state
            .noise_state
            .wrapping_mul(1_664_525)
            .wrapping_add(1_013_904_223);
        let noise = ((state.noise_state >> 9) as f32 / (u32::MAX >> 9) as f32) * 2.0 - 1.0;
        let hat = noise * state.hat_env;
        state.hat_env *= 0.975 - pulse_value * 0.01;

        let lead = (state.lead_phase * std::f32::consts::TAU).sin() * (0.25 + pulse_value * 0.2);

        let center = (sub_layer * 0.42 + bass_voice * 0.24 + pad * 0.18 + hat * 0.05 + lead * 0.11)
            * master_amp;
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
        AudioBackendMode, AudioBackendReadiness, AudioEvent, AudioEventQueue, AudioTransition,
        MockAudioEngine, analyze_procedural_audio, default_demo_audio_config,
        synthesize_mono_sample,
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

    #[test]
    fn audio_event_queue_drains_in_fifo_order() {
        let queue = AudioEventQueue::default();
        queue.push(AudioEvent::Kick);
        queue.push(AudioEvent::BassNote(36));
        queue.push(AudioEvent::Hat);

        let mut events = Vec::new();
        queue.drain_into(&mut events);
        assert_eq!(
            events,
            vec![AudioEvent::Kick, AudioEvent::BassNote(36), AudioEvent::Hat]
        );
    }

    #[test]
    fn audio_event_queue_can_be_reused_after_drain() {
        let queue = AudioEventQueue::default();
        queue.push(AudioEvent::Snare);

        let mut events = Vec::new();
        queue.drain_into(&mut events);
        assert_eq!(events, vec![AudioEvent::Snare]);

        events.clear();
        queue.push(AudioEvent::PadNote(52));
        queue.drain_into(&mut events);
        assert_eq!(events, vec![AudioEvent::PadNote(52)]);
    }
}
