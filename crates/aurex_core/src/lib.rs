#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Tick(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FrameIndex(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DeterminismSeed(pub u64);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FixedDelta {
    pub seconds: f32,
}

impl Default for FixedDelta {
    fn default() -> Self {
        Self { seconds: 1.0 / 120.0 }
    }
}
