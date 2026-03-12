use serde::{Deserialize, Serialize};

pub const GOLDEN_RATIO: f32 = 1.618_034;
pub const FIBONACCI_PHRASE_LENGTHS_BARS: [u32; 4] = [5, 8, 13, 21];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GoldenTempoMode {
    #[serde(default = "default_tempo_drift")]
    pub tempo_drift: f32,
}

const fn default_tempo_drift() -> f32 {
    0.061_8
}

impl Default for GoldenTempoMode {
    fn default() -> Self {
        Self {
            tempo_drift: default_tempo_drift(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AudioNote {
    pub pitch: i32,
    pub duration_beats: f32,
    pub velocity: f32,
    pub instrument: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AudioPattern {
    pub notes: Vec<AudioNote>,
    pub loops: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AudioTrack {
    pub name: String,
    pub patterns: Vec<AudioPattern>,
    pub volume: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AudioSequence {
    pub bpm: f32,
    #[serde(default)]
    pub golden_tempo_mode: Option<GoldenTempoMode>,
    pub tracks: Vec<AudioTrack>,
}

impl AudioSequence {
    pub fn beat_time_seconds(&self) -> f32 {
        60.0 / self.bpm.max(1.0)
    }

    pub fn tempo_at_time(&self, t: f32, seed: u32) -> f32 {
        if let Some(mode) = &self.golden_tempo_mode {
            let base_bpm = self.bpm.max(1.0);
            let beat = t / (60.0 / base_bpm);
            let phrase_idx = golden_phrase_index(beat);
            let phrase_bars =
                FIBONACCI_PHRASE_LENGTHS_BARS[phrase_idx % FIBONACCI_PHRASE_LENGTHS_BARS.len()];
            let phrase_beats = (phrase_bars * 4) as f32;
            let phrase_phase = (beat / phrase_beats).fract();

            let seed_phase = (seed as f32 * 0.754_877_7).fract() * std::f32::consts::TAU;
            let curve_a = (std::f32::consts::TAU * phrase_phase).sin();
            let curve_b =
                (std::f32::consts::TAU * (phrase_phase / GOLDEN_RATIO) + seed_phase).cos();
            let modulation = (curve_a * 0.7 + curve_b * 0.3) * (GOLDEN_RATIO - 1.0);
            let drift = 1.0 + modulation * mode.tempo_drift;
            (base_bpm * drift).max(1.0)
        } else {
            self.bpm.max(1.0)
        }
    }

    pub fn phrase_at_beat(&self, beat: f32) -> u32 {
        if self.golden_tempo_mode.is_some() {
            golden_phrase_index(beat) as u32
        } else {
            ((beat.floor().max(0.0) as u32) / 4) / 4
        }
    }

    pub fn sample_energy(&self, t: f32) -> f32 {
        let beat = t / (60.0 / self.tempo_at_time(t, 0));
        let mut energy = 0.0;
        for tr in &self.tracks {
            energy += track_energy(tr, beat) * tr.volume;
        }
        energy
    }
}

fn golden_phrase_index(beat: f32) -> usize {
    let mut measure = ((beat / 4.0).floor().max(0.0)) as u32;
    let mut phrase_idx = 0usize;
    while measure >= FIBONACCI_PHRASE_LENGTHS_BARS[phrase_idx % FIBONACCI_PHRASE_LENGTHS_BARS.len()]
    {
        measure -= FIBONACCI_PHRASE_LENGTHS_BARS[phrase_idx % FIBONACCI_PHRASE_LENGTHS_BARS.len()];
        phrase_idx += 1;
    }
    phrase_idx
}

fn track_energy(track: &AudioTrack, beat: f32) -> f32 {
    let mut cursor = 0.0;
    let mut e = 0.0;
    for p in &track.patterns {
        for _ in 0..p.loops.max(1) {
            for n in &p.notes {
                let start = cursor;
                let end = cursor + n.duration_beats.max(0.01);
                if beat >= start && beat < end {
                    let phase = (beat - start) / (end - start);
                    e += (1.0 - phase).max(0.0) * n.velocity;
                }
                cursor = end;
            }
        }
    }
    e
}

#[cfg(test)]
mod tests {
    use super::{AudioSequence, FIBONACCI_PHRASE_LENGTHS_BARS, GoldenTempoMode};

    #[test]
    fn golden_tempo_mode_is_deterministic() {
        let seq = AudioSequence {
            bpm: 140.0,
            golden_tempo_mode: Some(GoldenTempoMode::default()),
            tracks: vec![],
        };

        let a = seq.tempo_at_time(3.7, 99);
        let b = seq.tempo_at_time(3.7, 99);
        assert_eq!(a, b);
        assert_ne!(a, seq.bpm);
    }

    #[test]
    fn golden_phrase_progression_uses_fibonacci_lengths() {
        let seq = AudioSequence {
            bpm: 128.0,
            golden_tempo_mode: Some(GoldenTempoMode::default()),
            tracks: vec![],
        };
        let bars: u32 = FIBONACCI_PHRASE_LENGTHS_BARS.iter().sum();
        let beat_after_one_cycle = (bars * 4) as f32;
        assert_eq!(seq.phrase_at_beat(0.0), 0);
        assert_eq!(seq.phrase_at_beat((5 * 4) as f32), 1);
        assert_eq!(seq.phrase_at_beat(((5 + 8) * 4) as f32), 2);
        assert_eq!(seq.phrase_at_beat(((5 + 8 + 13) * 4) as f32), 3);
        assert_eq!(seq.phrase_at_beat(beat_after_one_cycle), 4);
    }
}
