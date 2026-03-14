#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RhythmFieldSnapshot {
    pub beat_phase: f32,
    pub bar_phase: f32,
    pub pulse: f32,
    pub bass_energy: f32,
    pub mid_energy: f32,
    pub high_energy: f32,
    pub intensity: f32,
    pub accent: f32,
}

impl RhythmFieldSnapshot {
    pub fn clamped(self) -> Self {
        Self {
            beat_phase: self.beat_phase.clamp(0.0, 1.0),
            bar_phase: self.bar_phase.clamp(0.0, 1.0),
            pulse: self.pulse.clamp(0.0, 1.0),
            bass_energy: self.bass_energy.clamp(0.0, 1.0),
            mid_energy: self.mid_energy.clamp(0.0, 1.0),
            high_energy: self.high_energy.clamp(0.0, 1.0),
            intensity: self.intensity.clamp(0.0, 1.0),
            accent: self.accent.clamp(0.0, 1.0),
        }
    }
}

impl Default for RhythmFieldSnapshot {
    fn default() -> Self {
        Self {
            beat_phase: 0.0,
            bar_phase: 0.0,
            pulse: 0.0,
            bass_energy: 0.0,
            mid_energy: 0.0,
            high_energy: 0.0,
            intensity: 0.0,
            accent: 0.0,
        }
    }
}
