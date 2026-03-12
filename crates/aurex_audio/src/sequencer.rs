use serde::{Deserialize, Serialize};

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
    pub tracks: Vec<AudioTrack>,
}

impl AudioSequence {
    pub fn beat_time_seconds(&self) -> f32 {
        60.0 / self.bpm.max(1.0)
    }

    pub fn sample_energy(&self, t: f32) -> f32 {
        let beat = t / self.beat_time_seconds();
        let mut energy = 0.0;
        for tr in &self.tracks {
            energy += track_energy(tr, beat) * tr.volume;
        }
        energy
    }
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
