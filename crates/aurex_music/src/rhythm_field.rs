use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
pub struct RhythmField {
    pub beat_phase: f32,
    pub beat_strength: f32,
    pub bass_energy: f32,
    pub harmonic_energy: f32,
}
