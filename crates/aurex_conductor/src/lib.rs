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
