use std::cmp::Ordering;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MidiFile {
    #[serde(default = "default_ticks_per_quarter")]
    pub ticks_per_quarter: u16,
    #[serde(default)]
    pub tracks: Vec<MidiTrack>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MidiTrack {
    pub name: String,
    #[serde(default)]
    pub notes: Vec<MidiNote>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct MidiNote {
    pub start_tick: u32,
    pub duration_ticks: u32,
    pub key: u8,
    pub velocity: u8,
    #[serde(default)]
    pub channel: u8,
}

impl MidiFile {
    pub fn from_json_str(contents: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(contents)
    }

    pub fn normalized(mut self) -> Self {
        for track in &mut self.tracks {
            track.notes.sort_by(compare_note_order);
        }
        self
    }

    pub fn note_count(&self) -> usize {
        self.tracks.iter().map(|t| t.notes.len()).sum()
    }
}

impl MidiNote {
    pub fn end_tick(self) -> u32 {
        self.start_tick.saturating_add(self.duration_ticks)
    }
}

fn compare_note_order(a: &MidiNote, b: &MidiNote) -> Ordering {
    a.start_tick
        .cmp(&b.start_tick)
        .then(a.channel.cmp(&b.channel))
        .then(a.key.cmp(&b.key))
        .then(a.velocity.cmp(&b.velocity))
        .then(a.duration_ticks.cmp(&b.duration_ticks))
}

const fn default_ticks_per_quarter() -> u16 {
    480
}

#[cfg(test)]
mod tests {
    use super::{MidiFile, MidiNote, MidiTrack};

    #[test]
    fn normalization_is_deterministic() {
        let file = MidiFile {
            ticks_per_quarter: 480,
            tracks: vec![MidiTrack {
                name: "lead".to_string(),
                notes: vec![
                    MidiNote {
                        start_tick: 240,
                        duration_ticks: 120,
                        key: 64,
                        velocity: 80,
                        channel: 0,
                    },
                    MidiNote {
                        start_tick: 0,
                        duration_ticks: 120,
                        key: 67,
                        velocity: 90,
                        channel: 0,
                    },
                ],
            }],
        };

        let a = file.clone().normalized();
        let b = file.normalized();
        assert_eq!(a, b);
    }

    #[test]
    fn end_tick_is_saturating() {
        let note = MidiNote {
            start_tick: u32::MAX,
            duration_ticks: 99,
            key: 60,
            velocity: 100,
            channel: 0,
        };
        assert_eq!(note.end_tick(), u32::MAX);
    }
}
