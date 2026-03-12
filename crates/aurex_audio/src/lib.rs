pub mod analysis;
pub mod sequencer;
pub mod synth;
pub mod voice;

use analysis::{AudioFeatures, analyze_sequence};
use aurex_core::Tick;
use sequencer::AudioSequence;
use serde::{Deserialize, Serialize};
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
