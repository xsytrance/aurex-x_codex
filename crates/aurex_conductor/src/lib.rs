use aurex_core::{FixedDelta, FrameIndex, Tick};

#[derive(Debug, Clone)]
pub struct ConductorClock {
    pub sim_tick: Tick,
    pub frame_index: FrameIndex,
    pub fixed_delta: FixedDelta,
}

impl Default for ConductorClock {
    fn default() -> Self {
        Self {
            sim_tick: Tick(0),
            frame_index: FrameIndex(0),
            fixed_delta: FixedDelta::default(),
        }
    }
}

impl ConductorClock {
    pub fn advance_frame(&mut self) {
        self.frame_index.0 += 1;
        self.sim_tick.0 += 1;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConductorStage {
    PreTick,
    SimTick,
    AudioTick,
    RenderPrepare,
    Render,
    Present,
    PostFrame,
}

pub const MAIN_LOOP_STAGES: [ConductorStage; 7] = [
    ConductorStage::PreTick,
    ConductorStage::SimTick,
    ConductorStage::AudioTick,
    ConductorStage::RenderPrepare,
    ConductorStage::Render,
    ConductorStage::Present,
    ConductorStage::PostFrame,
];

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct FrameExecutionTrace {
    pub stages: Vec<ConductorStage>,
}

pub fn execute_frame(clock: &mut ConductorClock, mut visit: impl FnMut(ConductorStage)) -> FrameExecutionTrace {
    let mut trace = FrameExecutionTrace::default();

    for stage in MAIN_LOOP_STAGES {
        visit(stage);
        trace.stages.push(stage);
    }

    clock.advance_frame();
    trace
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_execution_is_stable() {
        let mut clock = ConductorClock::default();
        let trace = execute_frame(&mut clock, |_| {});

        assert_eq!(trace.stages, MAIN_LOOP_STAGES);
        assert_eq!(clock.frame_index.0, 1);
        assert_eq!(clock.sim_tick.0, 1);
    }
}
