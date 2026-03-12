use std::collections::BTreeMap;

use aurex_audio::{ProceduralAudioConfig, sequencer as audio_seq};
use serde::{Deserialize, Serialize};

use crate::{
    instrument::InstrumentKind,
    pattern::{Pattern, PatternEvent},
    rhythm_field::RhythmField,
    tempo::TempoClock,
    track::Track,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MusicSequenceConfig {
    pub bpm: f32,
    #[serde(default = "default_ppq")]
    pub ppq: u32,
    #[serde(default)]
    pub tracks: Vec<Track>,
    #[serde(default)]
    pub seed: u32,
}

fn default_ppq() -> u32 {
    16
}

#[derive(Debug, Clone)]
pub struct EmittedEvent {
    pub track: String,
    pub event: PatternEvent,
}

pub struct MusicSequencer {
    pub clock: TempoClock,
    pub config: MusicSequenceConfig,
    pub emitted_events: Vec<EmittedEvent>,
    pub rhythm_field: RhythmField,
}

impl MusicSequencer {
    pub fn new(config: MusicSequenceConfig) -> Self {
        Self {
            clock: TempoClock::new(config.bpm, config.ppq),
            config,
            emitted_events: Vec::new(),
            rhythm_field: RhythmField::default(),
        }
    }

    pub fn update(&mut self, delta_seconds: f32) {
        let advanced = self.clock.advance(delta_seconds);
        self.emitted_events.clear();

        if advanced == 0 {
            self.update_rhythm_field();
            return;
        }

        let tick = self.clock.tick;
        for track in &self.config.tracks {
            emit_track_events(track, tick, self.config.ppq, &mut self.emitted_events);
        }
        self.update_rhythm_field();
    }

    pub fn to_procedural_audio_config(&self) -> ProceduralAudioConfig {
        let mut tracks = Vec::new();
        for t in &self.config.tracks {
            tracks.push(track_to_audio(t));
        }
        ProceduralAudioConfig {
            tempo: self.config.bpm,
            tracks,
            synth_graph: None,
            voice: None,
            seed: self.config.seed,
        }
    }

    fn update_rhythm_field(&mut self) {
        let beat_phase = self.clock.beat_phase;
        let beat_strength = (1.0 - beat_phase).powf(2.0);
        let mut bass_energy = 0.0;
        let mut harmonic_energy = 0.0;

        for ev in &self.emitted_events {
            if let PatternEvent::Note {
                pitch, velocity, ..
            } = ev.event
            {
                if pitch <= 48 {
                    bass_energy += velocity;
                }
                harmonic_energy += velocity;
            }
        }

        self.rhythm_field = RhythmField {
            beat_phase,
            beat_strength,
            bass_energy: bass_energy.clamp(0.0, 1.0),
            harmonic_energy: harmonic_energy.clamp(0.0, 1.0),
        };
    }
}

fn emit_track_events(track: &Track, tick: u64, ppq: u32, out: &mut Vec<EmittedEvent>) {
    let steps = track.pattern.steps.max(1);
    let ticks_per_step = ppq.max(1) as u64;
    let step_index = ((tick / ticks_per_step) % steps as u64) as u32;

    for event in &track.pattern.events {
        let event_step = match event {
            PatternEvent::Note { step, .. }
            | PatternEvent::Modulation { step, .. }
            | PatternEvent::GeneratorHook { step, .. } => *step,
        };
        if event_step == step_index {
            out.push(EmittedEvent {
                track: track.name.clone(),
                event: event.clone(),
            });
        }
    }
}

fn track_to_audio(track: &Track) -> audio_seq::AudioTrack {
    let mut notes = Vec::new();
    let mut step_notes: BTreeMap<u32, Vec<&PatternEvent>> = BTreeMap::new();
    for ev in &track.pattern.events {
        if matches!(ev, PatternEvent::Note { .. }) {
            let step = match ev {
                PatternEvent::Note { step, .. } => *step,
                _ => 0,
            };
            step_notes.entry(step).or_default().push(ev);
        }
    }

    for (_, events) in step_notes {
        for ev in events {
            if let PatternEvent::Note {
                pitch,
                duration_beats,
                velocity,
                ..
            } = ev
            {
                notes.push(audio_seq::AudioNote {
                    pitch: *pitch,
                    duration_beats: *duration_beats,
                    velocity: (*velocity * track.volume).clamp(0.0, 1.0),
                    instrument: track.instrument.as_audio_instrument().to_string(),
                });
            }
        }
    }

    audio_seq::AudioTrack {
        name: track.name.clone(),
        patterns: vec![audio_seq::AudioPattern { notes, loops: 16 }],
        volume: track.volume,
    }
}

pub fn default_electronic_sequence(seed: u32) -> MusicSequenceConfig {
    MusicSequenceConfig {
        bpm: 138.0,
        ppq: 16,
        seed,
        tracks: vec![Track {
            name: "electro_bass".into(),
            instrument: InstrumentKind::PulseSynth,
            volume: 0.9,
            pattern: Pattern {
                steps: 16,
                events: vec![
                    PatternEvent::Note {
                        step: 0,
                        pitch: 36,
                        duration_beats: 0.5,
                        velocity: 0.9,
                    },
                    PatternEvent::Note {
                        step: 4,
                        pitch: 38,
                        duration_beats: 0.5,
                        velocity: 0.8,
                    },
                    PatternEvent::Note {
                        step: 8,
                        pitch: 43,
                        duration_beats: 0.5,
                        velocity: 0.85,
                    },
                    PatternEvent::Note {
                        step: 12,
                        pitch: 38,
                        duration_beats: 0.5,
                        velocity: 0.75,
                    },
                ],
            },
        }],
    }
}
