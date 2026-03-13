use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct TempoClock {
    pub bpm: f32,
    pub ppq: u32,
    pub time_seconds: f32,
    pub beat_position: f32,
    pub beat_phase: f32,
    pub bar_index: u64,
    pub tick: u64,
}

impl TempoClock {
    pub fn new(bpm: f32, ppq: u32) -> Self {
        Self {
            bpm: bpm.max(1.0),
            ppq: ppq.max(1),
            time_seconds: 0.0,
            beat_position: 0.0,
            beat_phase: 0.0,
            bar_index: 0,
            tick: 0,
        }
    }

    pub fn advance(&mut self, delta_seconds: f32) -> u64 {
        let prev_tick = self.tick;
        self.time_seconds = (self.time_seconds + delta_seconds).max(0.0);
        let beats_per_second = self.bpm / 60.0;
        self.beat_position = self.time_seconds * beats_per_second;
        self.beat_phase = self.beat_position.fract();
        self.bar_index = (self.beat_position / 4.0).floor() as u64;
        self.tick = (self.beat_position * self.ppq as f32).floor() as u64;
        self.tick.saturating_sub(prev_tick)
    }
}
