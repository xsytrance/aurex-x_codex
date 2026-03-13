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

impl RuntimeAudioOutput {
    pub fn set_pulse(&self, pulse: f32) {
        self.pulse
            .store(pulse.clamp(0.0, 1.0).to_bits(), Ordering::Relaxed);
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
            let mut synth = RuntimeSynthState::default();
            device
                .build_output_stream(
                    &default_config.config(),
                    move |data: &mut [f32], _| {
                        write_trance_data(data, channels, sample_rate, &mut synth, &pulse)
                    },
                    err_fn,
                    None,
                )
                .map_err(|e| format!("build_output_stream(f32) failed: {e}"))?
        }
        cpal::SampleFormat::I16 => {
            let pulse = Arc::clone(&pulse);
            let mut synth = RuntimeSynthState::default();
            device
                .build_output_stream(
                    &default_config.config(),
                    move |data: &mut [i16], _| {
                        write_trance_data(data, channels, sample_rate, &mut synth, &pulse)
                    },
                    err_fn,
                    None,
                )
                .map_err(|e| format!("build_output_stream(i16) failed: {e}"))?
        }
        cpal::SampleFormat::U16 => {
            let pulse = Arc::clone(&pulse);
            let mut synth = RuntimeSynthState::default();
            device
                .build_output_stream(
                    &default_config.config(),
                    move |data: &mut [u16], _| {
                        write_trance_data(data, channels, sample_rate, &mut synth, &pulse)
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

#[derive(Debug, Clone, Copy)]
struct NoteEvent {
    note: Option<u8>,
    velocity: f32,
    instrument: u8,
    param: f32,
}

#[derive(Debug, Clone)]
struct Row {
    events: Vec<NoteEvent>,
}

#[derive(Debug, Clone)]
struct Pattern {
    rows: Vec<Row>,
}

#[derive(Debug, Clone)]
struct Song {
    patterns: Vec<Pattern>,
    pattern_order: Vec<usize>,
}

#[derive(Debug, Clone)]
struct Sequencer {
    bpm: f32,
    row_index: usize,
    pattern_order_index: usize,
    row_timer: f32,
    rows_per_beat: usize,
    song: Song,
}

impl Sequencer {
    fn new(song: Song) -> Self {
        Self {
            bpm: 138.0,
            row_index: 0,
            pattern_order_index: 0,
            row_timer: 0.0,
            rows_per_beat: 4,
            song,
        }
    }

    fn seconds_per_row(&self) -> f32 {
        60.0 / (self.bpm * self.rows_per_beat as f32)
    }

    fn current_pattern(&self) -> &Pattern {
        let idx = self.song.pattern_order[self.pattern_order_index];
        &self.song.patterns[idx]
    }

    fn step_row(&mut self) -> &Row {
        self.row_index += 1;
        if self.row_index >= self.current_pattern().rows.len().max(1) {
            self.row_index = 0;
            self.pattern_order_index =
                (self.pattern_order_index + 1) % self.song.pattern_order.len().max(1);
        }
        &self.current_pattern().rows[self.row_index]
    }

    fn row(&self) -> &Row {
        &self.current_pattern().rows[self.row_index]
    }
}

#[derive(Debug, Clone)]
struct RuntimeSynthState {
    sequencer: Sequencer,
    rng: u32,
    kick_phase: f32,
    kick_freq: f32,
    kick_env: f32,
    snare_env: f32,
    hat_env: f32,
    bass_phase: f32,
    bass_freq: f32,
    bass_env: f32,
    pad_a_phase: f32,
    pad_b_phase: f32,
    pad_freq: f32,
    pad_env: f32,
    lead_phase: f32,
    lead_freq: f32,
    lead_env: f32,
    started: bool,
}

impl Default for RuntimeSynthState {
    fn default() -> Self {
        Self {
            sequencer: Sequencer::new(tracker_song()),
            rng: 0xA9E3_1234,
            kick_phase: 0.0,
            kick_freq: 120.0,
            kick_env: 0.0,
            snare_env: 0.0,
            hat_env: 0.0,
            bass_phase: 0.0,
            bass_freq: 55.0,
            bass_env: 0.0,
            pad_a_phase: 0.0,
            pad_b_phase: 0.0,
            pad_freq: 220.0,
            pad_env: 0.0,
            lead_phase: 0.0,
            lead_freq: 440.0,
            lead_env: 0.0,
            started: false,
        }
    }
}

fn midi_to_hz(note: u8) -> f32 {
    440.0 * 2.0_f32.powf((note as f32 - 69.0) / 12.0)
}

fn tracker_song() -> Song {
    let empty = || Row { events: vec![] };
    let mut rows = vec![empty(); 32];

    for step in [0usize, 4, 8, 12, 16, 20, 24, 28] {
        rows[step].events.push(NoteEvent {
            note: None,
            velocity: 1.0,
            instrument: 0,
            param: 0.0,
        });
    }
    for step in [4usize, 12, 20, 28] {
        rows[step].events.push(NoteEvent {
            note: None,
            velocity: 0.65,
            instrument: 2,
            param: 0.0,
        });
    }
    for step in [8usize, 24] {
        rows[step].events.push(NoteEvent {
            note: None,
            velocity: 0.75,
            instrument: 1,
            param: 0.0,
        });
    }

    let bass_notes = [36u8, 36, 43, 41, 36, 36, 38, 31];
    for (i, note) in bass_notes.iter().enumerate() {
        rows[i * 4].events.push(NoteEvent {
            note: Some(*note),
            velocity: 0.8,
            instrument: 3,
            param: 0.0,
        });
    }

    for (step, note) in [(0usize, 60u8), (8, 63), (16, 67), (24, 70)] {
        rows[step].events.push(NoteEvent {
            note: Some(note),
            velocity: 0.45,
            instrument: 4,
            param: 0.0,
        });
    }

    for (step, note) in [(14usize, 72u8), (30, 74)] {
        rows[step].events.push(NoteEvent {
            note: Some(note),
            velocity: 0.35,
            instrument: 5,
            param: 0.0,
        });
    }

    Song {
        patterns: vec![Pattern { rows }],
        pattern_order: vec![0],
    }
}

fn trigger_event(state: &mut RuntimeSynthState, event: NoteEvent) {
    match event.instrument {
        0 => {
            state.kick_env = event.velocity.clamp(0.0, 1.0);
            state.kick_freq = 120.0 + event.param * 20.0;
            state.kick_phase = 0.0;
        }
        1 => {
            state.snare_env = event.velocity.clamp(0.0, 1.0);
        }
        2 => {
            state.hat_env = event.velocity.clamp(0.0, 1.0);
        }
        3 => {
            state.bass_env = event.velocity.clamp(0.0, 1.0);
            if let Some(note) = event.note {
                state.bass_freq = midi_to_hz(note);
            }
        }
        4 => {
            state.pad_env = event.velocity.clamp(0.0, 1.0);
            if let Some(note) = event.note {
                state.pad_freq = midi_to_hz(note);
            }
        }
        5 => {
            state.lead_env = event.velocity.clamp(0.0, 1.0);
            if let Some(note) = event.note {
                state.lead_freq = midi_to_hz(note);
            }
        }
        _ => {}
    }
}

fn rand_noise(state: &mut RuntimeSynthState) -> f32 {
    state.rng = state
        .rng
        .wrapping_mul(1_664_525)
        .wrapping_add(1_013_904_223);
    let n = ((state.rng >> 8) & 0x00FF_FFFF) as f32 / 0x00FF_FFFF as f32;
    n * 2.0 - 1.0
}

fn write_trance_data<T>(
    output: &mut [T],
    channels: usize,
    sample_rate: f32,
    synth: &mut RuntimeSynthState,
    pulse: &Arc<AtomicU32>,
) where
    T: cpal::Sample + cpal::FromSample<f32>,
{
    if !synth.started {
        let row_len = synth.sequencer.row().events.len();
        for i in 0..row_len {
            let ev = synth.sequencer.row().events[i];
            trigger_event(synth, ev);
        }
        synth.started = true;
    }

    let dt = 1.0 / sample_rate.max(1.0);
    let ch = channels.max(1);

    for frame in output.chunks_mut(ch) {
        let p = f32::from_bits(pulse.load(Ordering::Relaxed)).clamp(0.0, 1.0);

        synth.sequencer.row_timer += dt;
        while synth.sequencer.row_timer >= synth.sequencer.seconds_per_row() {
            synth.sequencer.row_timer -= synth.sequencer.seconds_per_row();
            synth.sequencer.step_row();
            let row_len = synth.sequencer.row().events.len();
            for i in 0..row_len {
                let ev = synth.sequencer.row().events[i];
                trigger_event(synth, ev);
            }
        }

        synth.kick_freq = (synth.kick_freq - 220.0 * dt).max(40.0);
        synth.kick_phase = (synth.kick_phase + (synth.kick_freq / sample_rate)).fract();
        let kick = (synth.kick_phase * std::f32::consts::TAU).sin() * synth.kick_env;
        synth.kick_env *= 0.9992;

        synth.bass_phase = (synth.bass_phase + (synth.bass_freq / sample_rate)).fract();
        let sub =
            (synth.bass_phase * std::f32::consts::TAU).sin() * synth.bass_env * (0.35 + 0.65 * p);
        synth.bass_env *= 0.9997;

        synth.pad_a_phase = (synth.pad_a_phase + (synth.pad_freq / sample_rate)).fract();
        synth.pad_b_phase = (synth.pad_b_phase + ((synth.pad_freq * 1.007) / sample_rate)).fract();
        let pad = ((synth.pad_a_phase * std::f32::consts::TAU).sin()
            + (synth.pad_b_phase * std::f32::consts::TAU).sin())
            * 0.5
            * synth.pad_env;
        synth.pad_env *= 0.99992;

        synth.lead_phase = (synth.lead_phase + (synth.lead_freq / sample_rate)).fract();
        let lead = (synth.lead_phase * std::f32::consts::TAU).sin() * synth.lead_env;
        synth.lead_env *= 0.9995;

        let snare = rand_noise(synth) * synth.snare_env;
        synth.snare_env *= 0.996;
        let hat = rand_noise(synth) * synth.hat_env;
        synth.hat_env *= 0.989;

        let mixed =
            (kick * 0.55 + sub * 0.45 + pad * 0.30 + lead * 0.22 + snare * 0.16 + hat * 0.12)
                .clamp(-0.95, 0.95);
        let v: T = T::from_sample(mixed);
        for s in frame.iter_mut() {
            *s = v;
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
    use std::sync::{
        Arc,
        atomic::{AtomicU32, Ordering},
    };

    use super::{
        AudioBackendMode, AudioBackendReadiness, AudioTransition, MockAudioEngine,
        analyze_procedural_audio, default_demo_audio_config, synthesize_mono_sample,
        write_trance_data,
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

    fn write_sine_data_for_test(
        output: &mut [f32],
        channels: usize,
        sample_rate: f32,
        phase: &mut f32,
        pulse: &Arc<AtomicU32>,
    ) {
        let p = f32::from_bits(pulse.load(Ordering::Relaxed)).clamp(0.0, 1.0);
        let freq = 220.0 + p * 90.0;
        let amp = 0.06 + p * 0.06;
        let phase_inc = freq / sample_rate;

        for frame in output.chunks_mut(channels.max(1)) {
            let sample = (*phase * std::f32::consts::TAU).sin() * amp;
            *phase = (*phase + phase_inc).fract();
            for s in frame.iter_mut() {
                *s = sample;
            }
        }
    }

    #[test]
    fn runtime_pulse_changes_generated_waveform() {
        let pulse = Arc::new(AtomicU32::new(0.0f32.to_bits()));
        let mut low_phase = 0.0_f32;
        let mut low = vec![0.0_f32; 16];
        write_sine_data_for_test(&mut low, 1, 48_000.0, &mut low_phase, &pulse);

        pulse.store(1.0f32.to_bits(), Ordering::Relaxed);
        let mut high_phase = 0.0_f32;
        let mut high = vec![0.0_f32; 16];
        write_sine_data_for_test(&mut high, 1, 48_000.0, &mut high_phase, &pulse);

        let low_energy: f32 = low.iter().map(|s| s.abs()).sum();
        let high_energy: f32 = high.iter().map(|s| s.abs()).sum();
        assert!(high_energy > low_energy);
        assert_ne!(low, high);
    }

    #[test]
    fn trance_writer_emits_kick_and_pulse_sensitive_sub() {
        let pulse = Arc::new(AtomicU32::new(0.0f32.to_bits()));
        let mut synth = super::RuntimeSynthState::default();
        let mut low = vec![0.0_f32; 4096];
        write_trance_data(&mut low, 1, 48_000.0, &mut synth, &pulse);

        pulse.store(1.0f32.to_bits(), Ordering::Relaxed);
        let mut high = vec![0.0_f32; 4096];
        write_trance_data(&mut high, 1, 48_000.0, &mut synth, &pulse);

        let low_energy: f32 = low.iter().map(|s| s.abs()).sum();
        let high_energy: f32 = high.iter().map(|s| s.abs()).sum();
        assert!(low_energy > 1.0);
        assert!(high_energy > 1.0);
        assert_ne!(low, high);
    }
}
