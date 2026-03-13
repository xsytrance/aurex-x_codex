use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
pub struct RhythmField {
    pub tempo: f32,
    pub beat_phase: f32,
    pub beat_strength: f32,
    pub beat_index: u64,
    pub bar_index: u64,
    pub phrase_index: u64,
    pub bass_energy: f32,
    pub harmonic_energy: f32,
    pub spectral_flux: f32,
    pub groove_vector: [f32; 3],
}
