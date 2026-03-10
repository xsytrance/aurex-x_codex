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

#[derive(Debug, Clone)]
pub struct BootAnimationConfig {
    pub seed: u64,
    pub frame_count: u32,
    pub base_radius: f32,
    pub pulse_speed: f32,
}

impl Default for BootAnimationConfig {
    fn default() -> Self {
        Self {
            seed: 0xA9E3_0001_u64,
            frame_count: 16,
            base_radius: 1.0,
            pulse_speed: 0.35,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BootFrame {
    pub frame_index: u32,
    pub tick: u64,
    pub ring_radius: f32,
    pub glow: f32,
    pub hue_shift: f32,
    pub scanline_offset: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootPhase {
    Ignition,
    PulseLock,
    Reveal,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhaseStyle {
    pub intensity_mul: f32,
    pub hue_bias: f32,
    pub distortion_weight: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BootStyleProfile {
    pub ignition: PhaseStyle,
    pub pulse_lock: PhaseStyle,
    pub reveal: PhaseStyle,
}

impl Default for BootStyleProfile {
    fn default() -> Self {
        Self {
            ignition: PhaseStyle {
                intensity_mul: 0.85,
                hue_bias: -12.0,
                distortion_weight: 0.55,
            },
            pulse_lock: PhaseStyle {
                intensity_mul: 1.15,
                hue_bias: 18.0,
                distortion_weight: 0.8,
            },
            reveal: PhaseStyle {
                intensity_mul: 1.0,
                hue_bias: 4.0,
                distortion_weight: 0.3,
            },
        }
    }
}

impl BootStyleProfile {
    pub fn style_for(&self, phase: BootPhase) -> PhaseStyle {
        match phase {
            BootPhase::Ignition => self.ignition,
            BootPhase::PulseLock => self.pulse_lock,
            BootPhase::Reveal => self.reveal,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BootTimelineFrame {
    pub phase: BootPhase,
    pub frame: BootFrame,
    pub styled_glow: f32,
    pub styled_hue: f32,
    pub distortion_weight: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BootTimeline {
    pub frames: Vec<BootTimelineFrame>,
}

impl BootTimeline {
    pub fn phase_counts(&self) -> (usize, usize, usize) {
        let mut ignition = 0;
        let mut pulse_lock = 0;
        let mut reveal = 0;

        for f in &self.frames {
            match f.phase {
                BootPhase::Ignition => ignition += 1,
                BootPhase::PulseLock => pulse_lock += 1,
                BootPhase::Reveal => reveal += 1,
            }
        }

        (ignition, pulse_lock, reveal)
    }
}

#[derive(Debug, Clone)]
pub struct BootAnimator {
    config: BootAnimationConfig,
    style: BootStyleProfile,
}

impl BootAnimator {
    pub fn new(config: BootAnimationConfig) -> Self {
        Self {
            config,
            style: BootStyleProfile::default(),
        }
    }

    pub fn with_style(config: BootAnimationConfig, style: BootStyleProfile) -> Self {
        Self { config, style }
    }

    pub fn generate_frames(&self, start_tick: u64) -> Vec<BootFrame> {
        (0..self.config.frame_count)
            .map(|i| {
                let t = i as f32 * self.config.pulse_speed;
                let noise = seeded_unit(self.config.seed, i);
                let ring_radius = self.config.base_radius + (t.sin() * 0.18) + (noise * 0.07);
                let glow = 0.55 + (t.cos().abs() * 0.35) + (noise * 0.1);
                let hue_shift = (noise * 120.0) + (t.sin() * 35.0);
                let scanline_offset = ((i as f32 * 0.11) + noise).fract();

                BootFrame {
                    frame_index: i,
                    tick: start_tick + i as u64,
                    ring_radius,
                    glow,
                    hue_shift,
                    scanline_offset,
                }
            })
            .collect()
    }

    pub fn generate_timeline(&self, start_tick: u64) -> BootTimeline {
        let raw = self.generate_frames(start_tick);
        let total = raw.len().max(1);

        let frames = raw
            .into_iter()
            .enumerate()
            .map(|(idx, frame)| {
                let phase = if idx < total / 3 {
                    BootPhase::Ignition
                } else if idx < (2 * total) / 3 {
                    BootPhase::PulseLock
                } else {
                    BootPhase::Reveal
                };

                let phase_style = self.style.style_for(phase);

                BootTimelineFrame {
                    phase,
                    styled_glow: frame.glow * phase_style.intensity_mul,
                    styled_hue: frame.hue_shift + phase_style.hue_bias,
                    distortion_weight: phase_style.distortion_weight,
                    frame,
                }
            })
            .collect();

        BootTimeline { frames }
    }
}

fn seeded_unit(seed: u64, frame_index: u32) -> f32 {
    let mut x = seed ^ ((frame_index as u64 + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15));
    x ^= x >> 30;
    x = x.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x ^= x >> 27;
    x = x.wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^= x >> 31;
    (x as f64 / u64::MAX as f64) as f32
}

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

    #[test]
    fn boot_animation_is_deterministic_for_same_seed() {
        let animator = BootAnimator::new(BootAnimationConfig {
            seed: 42,
            frame_count: 8,
            ..BootAnimationConfig::default()
        });

        let a = animator.generate_frames(100);
        let b = animator.generate_frames(100);
        assert_eq!(a, b);
    }

    #[test]
    fn boot_timeline_covers_all_phases() {
        let timeline = BootAnimator::new(BootAnimationConfig {
            seed: 7,
            frame_count: 12,
            ..BootAnimationConfig::default()
        })
        .generate_timeline(0);

        let (ignition, pulse_lock, reveal) = timeline.phase_counts();
        assert!(ignition > 0);
        assert!(pulse_lock > 0);
        assert!(reveal > 0);
        assert_eq!(ignition + pulse_lock + reveal, 12);
    }

    #[test]
    fn boot_animation_changes_with_seed() {
        let a = BootAnimator::new(BootAnimationConfig {
            seed: 1,
            frame_count: 4,
            ..BootAnimationConfig::default()
        })
        .generate_frames(0);
        let b = BootAnimator::new(BootAnimationConfig {
            seed: 2,
            frame_count: 4,
            ..BootAnimationConfig::default()
        })
        .generate_frames(0);

        assert_ne!(a, b);
    }

    #[test]
    fn phase_style_profile_is_applied() {
        let style = BootStyleProfile::default();
        let timeline = BootAnimator::with_style(
            BootAnimationConfig {
                seed: 3,
                frame_count: 12,
                ..BootAnimationConfig::default()
            },
            style.clone(),
        )
        .generate_timeline(5);

        let first = &timeline.frames[0];
        let last = &timeline.frames[timeline.frames.len() - 1];

        assert_eq!(first.phase, BootPhase::Ignition);
        assert_eq!(first.distortion_weight, style.ignition.distortion_weight);
        assert_eq!(last.phase, BootPhase::Reveal);
        assert_eq!(last.distortion_weight, style.reveal.distortion_weight);
    }
}
