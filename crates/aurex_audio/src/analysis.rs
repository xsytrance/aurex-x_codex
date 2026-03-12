use crate::sequencer::AudioSequence;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct AudioFeatures {
    pub kick_energy: f32,
    pub bass_energy: f32,
    pub mid_energy: f32,
    pub high_energy: f32,
    pub low_freq_energy: f32,
    pub mid_freq_energy: f32,
    pub high_freq_energy: f32,
    pub dominant_frequency: f32,
    pub harmonic_ratios: [f32; 3],
    pub current_beat: u32,
    pub current_measure: u32,
    pub current_phrase: u32,
    pub beat_phase: f32,
    pub spectral_centroid: f32,
    pub tempo: f32,
}

pub fn analyze_sequence(seq: &AudioSequence, t: f32, seed: u32) -> AudioFeatures {
    let e = seq.sample_energy(t).max(0.0);
    let kick = envelope(t, 2.0, seed) * e;
    let bass = envelope(t + 0.17, 1.2, seed ^ 11) * e;
    let mid = envelope(t + 0.31, 3.3, seed ^ 37) * e * 0.8;
    let high = envelope(t + 0.53, 6.0, seed ^ 73) * e * 0.6;

    let low_freq_energy = (kick * 0.7 + bass * 1.2).max(0.0);
    let mid_freq_energy = (mid * 1.1 + bass * 0.2).max(0.0);
    let high_freq_energy = (high * 1.3 + mid * 0.15).max(0.0);

    let sum = low_freq_energy + mid_freq_energy + high_freq_energy + 1e-6;
    let dominant_frequency =
        (low_freq_energy * 90.0 + mid_freq_energy * 660.0 + high_freq_energy * 3200.0) / sum;

    let r1 = (mid_freq_energy / low_freq_energy.max(1e-6)).clamp(0.0, 8.0);
    let r2 = (high_freq_energy / low_freq_energy.max(1e-6)).clamp(0.0, 8.0);
    let r3 = (high_freq_energy / mid_freq_energy.max(1e-6)).clamp(0.0, 8.0);

    let centroid = (kick * 80.0 + bass * 180.0 + mid * 1200.0 + high * 4200.0)
        / (kick + bass + mid + high + 1e-6);

    let beat_position = t / (60.0 / seq.bpm.max(1.0));
    let current_beat = beat_position.floor().max(0.0) as u32;
    let current_measure = current_beat / 4;
    let current_phrase = current_measure / 4;
    let beat_phase = beat_position.fract().clamp(0.0, 1.0);

    AudioFeatures {
        kick_energy: kick,
        bass_energy: bass,
        mid_energy: mid,
        high_energy: high,
        low_freq_energy,
        mid_freq_energy,
        high_freq_energy,
        dominant_frequency,
        harmonic_ratios: [r1, r2, r3],
        current_beat,
        current_measure,
        current_phrase,
        beat_phase,
        spectral_centroid: centroid,
        tempo: seq.bpm,
    }
}

fn envelope(t: f32, freq: f32, seed: u32) -> f32 {
    let n = (std::f32::consts::TAU * freq * t + (seed as f32 * 0.13)).sin();
    n.abs()
}

#[cfg(test)]
mod tests {
    use super::analyze_sequence;
    use crate::sequencer::AudioSequence;

    #[test]
    fn spectral_features_are_deterministic_and_bounded() {
        let seq = AudioSequence {
            bpm: 140.0,
            tracks: vec![],
        };
        let a = analyze_sequence(&seq, 1.2, 44);
        let b = analyze_sequence(&seq, 1.2, 44);
        assert_eq!(a, b);
        assert!(a.dominant_frequency >= 0.0);
        assert!(a.harmonic_ratios.iter().all(|r| *r >= 0.0 && *r <= 8.0));
        assert!(a.beat_phase >= 0.0 && a.beat_phase <= 1.0);
        assert_eq!(a.current_measure, a.current_beat / 4);
    }
}
