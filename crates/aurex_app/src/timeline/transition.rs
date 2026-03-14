#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransitionMode {
    Fade,
    Crossfade,
    Dissolve,
    AdditiveOverlay,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Easing {
    Linear,
    SmoothStep,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TransitionSpec {
    pub mode: TransitionMode,
    pub duration_seconds: f32,
    pub easing: Easing,
}

impl TransitionSpec {
    pub fn progress_at(&self, elapsed_seconds: f32) -> f32 {
        let t = if self.duration_seconds <= 0.0 {
            1.0
        } else {
            (elapsed_seconds / self.duration_seconds).clamp(0.0, 1.0)
        };
        match self.easing {
            Easing::Linear => t,
            Easing::SmoothStep => t * t * (3.0 - 2.0 * t),
        }
    }

    pub fn blend_weights(&self, elapsed_seconds: f32) -> (f32, f32) {
        let p = self.progress_at(elapsed_seconds);
        match self.mode {
            TransitionMode::Fade => (1.0 - p, p),
            TransitionMode::Crossfade => (1.0 - p, p),
            TransitionMode::Dissolve => (1.0 - p, p),
            TransitionMode::AdditiveOverlay => (1.0, p),
        }
    }
}
