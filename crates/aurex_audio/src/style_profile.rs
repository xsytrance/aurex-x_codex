use crate::{
    ProceduralAudioConfig,
    runtime_toolkit::Instrument,
    sequencer::{AudioNote, AudioPattern, AudioTrack},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScaleType {
    Major,
    Minor,
    Dorian,
    Mixolydian,
    Pentatonic,
    HarmonicMinor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstrumentPreset {
    TranceBass,
    SupersawPad,
    AnalogLead,
    NoiseHat,
    KickDrum,
}

impl InstrumentPreset {
    pub fn as_str(self) -> &'static str {
        match self {
            InstrumentPreset::TranceBass => "trance_bass",
            InstrumentPreset::SupersawPad => "supersaw_pad",
            InstrumentPreset::AnalogLead => "analog_lead",
            InstrumentPreset::NoiseHat => "noise_hat",
            InstrumentPreset::KickDrum => "kick_drum",
        }
    }

    pub fn to_instrument(self) -> Instrument {
        match self {
            InstrumentPreset::TranceBass => Instrument::trance_bass(),
            InstrumentPreset::SupersawPad => Instrument::supersaw_pad(),
            InstrumentPreset::AnalogLead => Instrument::analog_lead(),
            InstrumentPreset::NoiseHat => Instrument::noise_hat(),
            InstrumentPreset::KickDrum => Instrument::kick_drum(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrumPatternType {
    FourOnFloor,
    Backbeat,
    HalfTime,
    Shuffle,
    ReggaeSkank,
    WorldPulse,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StyleProfile {
    pub name: &'static str,
    pub tempo_min: f32,
    pub tempo_max: f32,
    pub scale_options: &'static [ScaleType],
    pub bass_instrument: InstrumentPreset,
    pub pad_instrument: InstrumentPreset,
    pub lead_instrument: InstrumentPreset,
    pub drum_pattern_type: DrumPatternType,
}

const MAJOR_MINOR: &[ScaleType] = &[ScaleType::Major, ScaleType::Minor];
const MINOR_DORIAN: &[ScaleType] = &[ScaleType::Minor, ScaleType::Dorian];
const PENTA_MINOR: &[ScaleType] = &[ScaleType::Pentatonic, ScaleType::Minor];
const MAJOR_MIXO: &[ScaleType] = &[ScaleType::Major, ScaleType::Mixolydian];
const JAZZ_SCALES: &[ScaleType] = &[ScaleType::Dorian, ScaleType::Mixolydian, ScaleType::Major];
const CLASSICAL_SCALES: &[ScaleType] =
    &[ScaleType::Major, ScaleType::Minor, ScaleType::HarmonicMinor];
const WORLD_SCALES: &[ScaleType] = &[
    ScaleType::Dorian,
    ScaleType::Pentatonic,
    ScaleType::HarmonicMinor,
];

const STYLE_PROFILES: [StyleProfile; 10] = [
    StyleProfile {
        name: "Electronic",
        tempo_min: 124.0,
        tempo_max: 142.0,
        scale_options: MINOR_DORIAN,
        bass_instrument: InstrumentPreset::TranceBass,
        pad_instrument: InstrumentPreset::SupersawPad,
        lead_instrument: InstrumentPreset::AnalogLead,
        drum_pattern_type: DrumPatternType::FourOnFloor,
    },
    StyleProfile {
        name: "Pop",
        tempo_min: 96.0,
        tempo_max: 124.0,
        scale_options: MAJOR_MINOR,
        bass_instrument: InstrumentPreset::TranceBass,
        pad_instrument: InstrumentPreset::SupersawPad,
        lead_instrument: InstrumentPreset::AnalogLead,
        drum_pattern_type: DrumPatternType::Backbeat,
    },
    StyleProfile {
        name: "HipHop",
        tempo_min: 72.0,
        tempo_max: 98.0,
        scale_options: PENTA_MINOR,
        bass_instrument: InstrumentPreset::TranceBass,
        pad_instrument: InstrumentPreset::SupersawPad,
        lead_instrument: InstrumentPreset::AnalogLead,
        drum_pattern_type: DrumPatternType::HalfTime,
    },
    StyleProfile {
        name: "Rock",
        tempo_min: 104.0,
        tempo_max: 148.0,
        scale_options: MAJOR_MINOR,
        bass_instrument: InstrumentPreset::TranceBass,
        pad_instrument: InstrumentPreset::AnalogLead,
        lead_instrument: InstrumentPreset::AnalogLead,
        drum_pattern_type: DrumPatternType::Backbeat,
    },
    StyleProfile {
        name: "RnB",
        tempo_min: 68.0,
        tempo_max: 92.0,
        scale_options: MINOR_DORIAN,
        bass_instrument: InstrumentPreset::TranceBass,
        pad_instrument: InstrumentPreset::SupersawPad,
        lead_instrument: InstrumentPreset::AnalogLead,
        drum_pattern_type: DrumPatternType::HalfTime,
    },
    StyleProfile {
        name: "Jazz",
        tempo_min: 108.0,
        tempo_max: 168.0,
        scale_options: JAZZ_SCALES,
        bass_instrument: InstrumentPreset::TranceBass,
        pad_instrument: InstrumentPreset::SupersawPad,
        lead_instrument: InstrumentPreset::AnalogLead,
        drum_pattern_type: DrumPatternType::Shuffle,
    },
    StyleProfile {
        name: "Classical",
        tempo_min: 60.0,
        tempo_max: 96.0,
        scale_options: CLASSICAL_SCALES,
        bass_instrument: InstrumentPreset::TranceBass,
        pad_instrument: InstrumentPreset::SupersawPad,
        lead_instrument: InstrumentPreset::AnalogLead,
        drum_pattern_type: DrumPatternType::WorldPulse,
    },
    StyleProfile {
        name: "Country",
        tempo_min: 92.0,
        tempo_max: 120.0,
        scale_options: MAJOR_MIXO,
        bass_instrument: InstrumentPreset::TranceBass,
        pad_instrument: InstrumentPreset::AnalogLead,
        lead_instrument: InstrumentPreset::AnalogLead,
        drum_pattern_type: DrumPatternType::Backbeat,
    },
    StyleProfile {
        name: "Reggae",
        tempo_min: 68.0,
        tempo_max: 92.0,
        scale_options: MAJOR_MIXO,
        bass_instrument: InstrumentPreset::TranceBass,
        pad_instrument: InstrumentPreset::SupersawPad,
        lead_instrument: InstrumentPreset::AnalogLead,
        drum_pattern_type: DrumPatternType::ReggaeSkank,
    },
    StyleProfile {
        name: "World",
        tempo_min: 78.0,
        tempo_max: 130.0,
        scale_options: WORLD_SCALES,
        bass_instrument: InstrumentPreset::TranceBass,
        pad_instrument: InstrumentPreset::SupersawPad,
        lead_instrument: InstrumentPreset::AnalogLead,
        drum_pattern_type: DrumPatternType::WorldPulse,
    },
];

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StyleSelection {
    pub profile: StyleProfile,
    pub bpm: f32,
    pub scale: ScaleType,
}

pub fn choose_style(seed: u64) -> StyleProfile {
    STYLE_PROFILES[(seed as usize) % STYLE_PROFILES.len()]
}

pub fn choose_style_selection(seed: u64) -> StyleSelection {
    let profile = choose_style(seed);
    let tempo_t = splitmix_f32(seed ^ 0x5EED_BAAD_77AA_1177);
    let bpm = profile.tempo_min + (profile.tempo_max - profile.tempo_min) * tempo_t;
    let scale_idx = ((splitmix_u64(seed ^ 0xA9E3_00FF_1CE0_F00D) as usize)
        % profile.scale_options.len())
    .max(0);

    StyleSelection {
        profile,
        bpm,
        scale: profile.scale_options[scale_idx],
    }
}

pub fn styled_audio_config(seed: u64) -> ProceduralAudioConfig {
    let sel = choose_style_selection(seed);
    let bass_name = sel.profile.bass_instrument.as_str();
    let pad_name = sel.profile.pad_instrument.as_str();
    let lead_name = sel.profile.lead_instrument.as_str();
    let drum_notes = drum_pattern_notes(sel.profile.drum_pattern_type);

    let mut bass_pitch = 36;
    if matches!(sel.scale, ScaleType::Major | ScaleType::Mixolydian) {
        bass_pitch = 38;
    }

    let tracks = vec![
        AudioTrack {
            name: format!("{}-bass", sel.profile.name.to_lowercase()),
            patterns: vec![AudioPattern {
                notes: vec![
                    AudioNote {
                        pitch: bass_pitch,
                        duration_beats: 0.5,
                        velocity: 0.88,
                        instrument: bass_name.into(),
                    },
                    AudioNote {
                        pitch: bass_pitch + 3,
                        duration_beats: 0.5,
                        velocity: 0.72,
                        instrument: bass_name.into(),
                    },
                ],
                loops: 8,
            }],
            volume: 0.85,
        },
        AudioTrack {
            name: format!("{}-pad", sel.profile.name.to_lowercase()),
            patterns: vec![AudioPattern {
                notes: vec![AudioNote {
                    pitch: bass_pitch + 12,
                    duration_beats: 2.0,
                    velocity: 0.52,
                    instrument: pad_name.into(),
                }],
                loops: 4,
            }],
            volume: 0.65,
        },
        AudioTrack {
            name: format!("{}-lead", sel.profile.name.to_lowercase()),
            patterns: vec![AudioPattern {
                notes: vec![
                    AudioNote {
                        pitch: bass_pitch + 24,
                        duration_beats: 0.25,
                        velocity: 0.52,
                        instrument: lead_name.into(),
                    },
                    AudioNote {
                        pitch: bass_pitch + 27,
                        duration_beats: 0.25,
                        velocity: 0.48,
                        instrument: lead_name.into(),
                    },
                ],
                loops: 4,
            }],
            volume: 0.45,
        },
        AudioTrack {
            name: format!("{}-drums", sel.profile.name.to_lowercase()),
            patterns: vec![AudioPattern {
                notes: drum_notes,
                loops: 8,
            }],
            volume: 0.78,
        },
    ];

    let mut synth_graph = crate::fallback_demo_audio_config(seed as u32)
        .synth_graph
        .expect("default graph exists");

    // style-flavor micro adjustments to graph gain without changing architecture
    if let crate::synth::SynthNode::Mixer { gain, .. } = &mut synth_graph {
        *gain = if sel.profile.name == "HipHop" || sel.profile.name == "RnB" {
            0.64
        } else {
            0.7
        };
    }

    let mut cfg = crate::fallback_demo_audio_config(seed as u32);
    cfg.tempo = sel.bpm;
    cfg.tracks = tracks;
    cfg.synth_graph = Some(synth_graph);
    cfg.seed = seed as u32;

    // touch instrument constructors to keep style→preset mapping explicit and compiled
    let _ = sel.profile.bass_instrument.to_instrument().effect_flags;
    let _ = sel.profile.pad_instrument.to_instrument().effect_flags;
    let _ = sel.profile.lead_instrument.to_instrument().effect_flags;

    cfg
}

fn drum_pattern_notes(pattern: DrumPatternType) -> Vec<AudioNote> {
    match pattern {
        DrumPatternType::FourOnFloor => vec![
            note(36, 0.5, 0.92, InstrumentPreset::KickDrum),
            note(42, 0.25, 0.52, InstrumentPreset::NoiseHat),
        ],
        DrumPatternType::Backbeat => vec![
            note(36, 0.5, 0.84, InstrumentPreset::KickDrum),
            note(40, 0.5, 0.72, InstrumentPreset::NoiseHat),
        ],
        DrumPatternType::HalfTime => vec![
            note(36, 1.0, 0.88, InstrumentPreset::KickDrum),
            note(42, 0.5, 0.45, InstrumentPreset::NoiseHat),
        ],
        DrumPatternType::Shuffle => vec![
            note(36, 0.66, 0.8, InstrumentPreset::KickDrum),
            note(42, 0.33, 0.5, InstrumentPreset::NoiseHat),
        ],
        DrumPatternType::ReggaeSkank => vec![
            note(36, 0.75, 0.64, InstrumentPreset::KickDrum),
            note(42, 0.25, 0.46, InstrumentPreset::NoiseHat),
        ],
        DrumPatternType::WorldPulse => vec![
            note(36, 0.5, 0.74, InstrumentPreset::KickDrum),
            note(42, 0.5, 0.42, InstrumentPreset::NoiseHat),
        ],
    }
}

fn note(pitch: i32, duration_beats: f32, velocity: f32, preset: InstrumentPreset) -> AudioNote {
    AudioNote {
        pitch,
        duration_beats,
        velocity,
        instrument: preset.as_str().into(),
    }
}

fn splitmix_u64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}

fn splitmix_f32(seed: u64) -> f32 {
    let x = splitmix_u64(seed);
    (x as f64 / u64::MAX as f64) as f32
}

#[cfg(test)]
mod tests {
    use super::{choose_style, choose_style_selection, styled_audio_config};

    #[test]
    fn style_choice_is_deterministic() {
        let a = choose_style(77);
        let b = choose_style(77);
        assert_eq!(a, b);
    }

    #[test]
    fn style_selection_keeps_bpm_in_range() {
        let s = choose_style_selection(1024);
        assert!(s.bpm >= s.profile.tempo_min);
        assert!(s.bpm <= s.profile.tempo_max);
    }

    #[test]
    fn styled_config_is_deterministic() {
        let a = styled_audio_config(99);
        let b = styled_audio_config(99);
        assert_eq!(a, b);
        assert!(!a.tracks.is_empty());
    }
}
