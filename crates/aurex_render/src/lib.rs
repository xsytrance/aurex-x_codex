#[derive(Debug, Clone)]
pub struct CameraRig {
    pub eye: [f32; 3],
    pub target: [f32; 3],
    pub fov_degrees: f32,
}

impl Default for CameraRig {
    fn default() -> Self {
        Self {
            eye: [0.0, 6.0, 12.0],
            target: [0.0, 0.0, 0.0],
            fov_degrees: 60.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderStage {
    RenderPrepare,
    Render,
    Present,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderBackendMode {
    Mock,
    WgpuPlanned,
}

#[derive(Debug, Clone)]
pub struct RenderBootstrapConfig {
    pub app_name: String,
    pub viewport_width: u32,
    pub viewport_height: u32,
    pub backend_mode: RenderBackendMode,
}

impl RenderBootstrapConfig {
    pub fn with_backend_mode(mut self, mode: RenderBackendMode) -> Self {
        self.backend_mode = mode;
        self
    }
}

impl Default for RenderBootstrapConfig {
    fn default() -> Self {
        Self {
            app_name: "Aurex-X".to_string(),
            viewport_width: 1280,
            viewport_height: 720,
            backend_mode: RenderBackendMode::Mock,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RenderFrameStats {
    pub frame_id: u64,
    pub stages_executed: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RenderBackendStatus {
    pub mode: RenderBackendMode,
    pub ready: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendTransition {
    Noop,
    Transitioned,
}

#[derive(Debug)]
pub struct MockRenderer {
    config: RenderBootstrapConfig,
    frames_rendered: u64,
    backend_ready: bool,
}

impl MockRenderer {
    pub fn new(config: RenderBootstrapConfig) -> Self {
        let backend_ready = config.backend_mode == RenderBackendMode::Mock;
        Self {
            config,
            frames_rendered: 0,
            backend_ready,
        }
    }

    pub fn config(&self) -> &RenderBootstrapConfig {
        &self.config
    }

    pub fn backend_status(&self) -> RenderBackendStatus {
        RenderBackendStatus {
            mode: self.config.backend_mode,
            ready: self.backend_ready,
        }
    }

    pub fn transition_backend_mode(&mut self, mode: RenderBackendMode) -> BackendTransition {
        if self.config.backend_mode == mode {
            return BackendTransition::Noop;
        }

        self.config.backend_mode = mode;
        self.backend_ready = mode == RenderBackendMode::Mock;
        BackendTransition::Transitioned
    }

    pub fn run_frame(&mut self, stages: &[RenderStage]) -> RenderFrameStats {
        self.frames_rendered += 1;
        RenderFrameStats {
            frame_id: self.frames_rendered,
            stages_executed: stages.len(),
        }
    }
}

pub const RENDER_MAIN_STAGES: [RenderStage; 3] = [
    RenderStage::RenderPrepare,
    RenderStage::Render,
    RenderStage::Present,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_renderer_tracks_frame_progress() {
        let mut renderer = MockRenderer::new(RenderBootstrapConfig::default());

        let first = renderer.run_frame(&RENDER_MAIN_STAGES);
        let second = renderer.run_frame(&RENDER_MAIN_STAGES);

        assert_eq!(first.frame_id, 1);
        assert_eq!(second.frame_id, 2);
        assert_eq!(first.stages_executed, 3);
    }

    #[test]
    fn backend_status_reflects_mode() {
        let renderer = MockRenderer::new(RenderBootstrapConfig::default());
        let status = renderer.backend_status();

        assert_eq!(status.mode, RenderBackendMode::Mock);
        assert!(status.ready);
    }

    #[test]
    fn transition_to_wgpu_planned_sets_not_ready() {
        let mut renderer = MockRenderer::new(RenderBootstrapConfig::default());
        let t = renderer.transition_backend_mode(RenderBackendMode::WgpuPlanned);

        assert_eq!(t, BackendTransition::Transitioned);
        assert_eq!(renderer.backend_status().mode, RenderBackendMode::WgpuPlanned);
        assert!(!renderer.backend_status().ready);
    }
}
