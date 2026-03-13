use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PatternEvent {
    Note {
        step: u32,
        pitch: i32,
        duration_beats: f32,
        velocity: f32,
    },
    Modulation {
        step: u32,
        target: String,
        value: f32,
    },
    GeneratorHook {
        step: u32,
        hook: String,
        amount: f32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Pattern {
    pub steps: u32,
    #[serde(default)]
    pub events: Vec<PatternEvent>,
}
