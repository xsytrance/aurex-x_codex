#[derive(Debug, Clone, PartialEq)]
pub struct TimelineClock {
    pub time_seconds: f32,
    pub frame_index: u64,
    pub fixed_step_seconds: f32,
    accumulator_seconds: f32,
    pub playing: bool,
}

impl Default for TimelineClock {
    fn default() -> Self {
        Self {
            time_seconds: 0.0,
            frame_index: 0,
            fixed_step_seconds: 1.0 / 60.0,
            accumulator_seconds: 0.0,
            playing: true,
        }
    }
}

impl TimelineClock {
    pub fn new(fixed_step_seconds: f32) -> Self {
        Self {
            fixed_step_seconds: fixed_step_seconds.max(1.0 / 1000.0),
            ..Self::default()
        }
    }

    pub fn advance_to(&mut self, wall_time_seconds: f32) -> usize {
        if !self.playing {
            return 0;
        }

        if wall_time_seconds <= self.time_seconds {
            return 0;
        }

        let delta = wall_time_seconds - self.time_seconds;
        self.accumulator_seconds += delta;

        let mut stepped = 0;
        while self.accumulator_seconds >= self.fixed_step_seconds {
            self.time_seconds += self.fixed_step_seconds;
            self.frame_index += 1;
            self.accumulator_seconds -= self.fixed_step_seconds;
            stepped += 1;
        }

        stepped
    }
}

#[cfg(test)]
mod tests {
    use super::TimelineClock;

    #[test]
    fn deterministic_stepping_advances_expected_frames() {
        let mut clock = TimelineClock::new(0.5);
        let frames = clock.advance_to(2.2);
        assert_eq!(frames, 4);
        assert!((clock.time_seconds - 2.0).abs() < f32::EPSILON);
        assert_eq!(clock.frame_index, 4);
    }
}
