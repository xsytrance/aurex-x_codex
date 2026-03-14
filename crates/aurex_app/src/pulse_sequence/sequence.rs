use super::phase::{PulsePhase, PulsePhaseOverrides};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct PulseSequence {
    pub phases: Vec<PulsePhase>,
}

impl PulseSequence {
    pub fn new() -> Self {
        Self { phases: Vec::new() }
    }

    pub fn add_phase(mut self, name: impl Into<String>, duration_seconds: f32) -> Self {
        self.phases.push(PulsePhase::new(name, duration_seconds));
        self
    }

    pub fn add_phase_with_overrides(
        mut self,
        name: impl Into<String>,
        duration_seconds: f32,
        overrides: PulsePhaseOverrides,
    ) -> Self {
        self.phases
            .push(PulsePhase::new(name, duration_seconds).with_overrides(overrides));
        self
    }

    pub fn total_duration(&self) -> f32 {
        self.phases.iter().map(|p| p.duration_seconds).sum()
    }

    pub fn phase_at_time(&self, time: f32) -> &PulsePhase {
        assert!(
            !self.phases.is_empty(),
            "PulseSequence must contain at least one phase"
        );

        let mut t = time.max(0.0);
        let total = self.total_duration();
        if t >= total {
            return self.phases.last().expect("phase vector non-empty");
        }

        for phase in &self.phases {
            if t <= phase.duration_seconds {
                return phase;
            }
            t -= phase.duration_seconds;
        }

        self.phases.last().expect("phase vector non-empty")
    }
}

#[cfg(test)]
mod tests {
    use super::PulseSequence;
    use crate::pulse_sequence::phase::PulsePhaseOverrides;

    #[test]
    fn phase_selection_is_deterministic() {
        let seq = PulseSequence::new()
            .add_phase("Silence", 3.0)
            .add_phase("Reveal", 5.0)
            .add_phase("Logo", 2.0);

        let a = seq.phase_at_time(4.2).name.clone();
        let b = seq.phase_at_time(4.2).name.clone();
        assert_eq!(a, b);
    }

    #[test]
    fn phase_clamps_to_last_when_time_exceeds_duration() {
        let seq = PulseSequence::new()
            .add_phase("Silence", 1.0)
            .add_phase_with_overrides("Final", 2.0, PulsePhaseOverrides::default());

        assert_eq!(seq.phase_at_time(100.0).name, "Final");
    }
}
