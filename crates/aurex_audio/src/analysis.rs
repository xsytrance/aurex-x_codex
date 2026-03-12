use crate::sequencer::AudioSequence;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct AudioFeatures {
    pub kick_energy: f32,
    pub bass_energy: f32,
    pub mid_energy: f32,
    pub high_energy: f32,
    pub spectral_centroid: f32,
    pub tempo: f32,
}

pub fn analyze_sequence(seq: &AudioSequence, t: f32, seed: u32) -> AudioFeatures {
    let e = seq.sample_energy(t).max(0.0);
    let kick = envelope(t, 2.0, seed) * e;
    let bass = envelope(t + 0.17, 1.2, seed ^ 11) * e;
    let mid = envelope(t + 0.31, 3.3, seed ^ 37) * e * 0.8;
    let high = envelope(t + 0.53, 6.0, seed ^ 73) * e * 0.6;

    let sum = kick + bass + mid + high + 1e-6;
    let centroid = (kick * 80.0 + bass * 180.0 + mid * 1200.0 + high * 4200.0) / sum;

    AudioFeatures {
        kick_energy: kick,
        bass_energy: bass,
        mid_energy: mid,
        high_energy: high,
        spectral_centroid: centroid,
        tempo: seq.bpm,
    }
}

fn envelope(t: f32, freq: f32, seed: u32) -> f32 {
    let n = (std::f32::consts::TAU * freq * t + (seed as f32 * 0.13)).sin();
    n.abs()
}
