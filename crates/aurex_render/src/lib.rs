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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderBackendReadiness {
    pub has_windowing: bool,
    pub has_gpu_backend: bool,
    pub can_present: bool,
}

impl RenderBackendReadiness {
    pub fn for_mode(mode: RenderBackendMode) -> Self {
        match mode {
            RenderBackendMode::Mock => Self {
                has_windowing: false,
                has_gpu_backend: false,
                can_present: false,
            },
            RenderBackendMode::WgpuPlanned => Self {
                has_windowing: true,
                has_gpu_backend: true,
                can_present: true,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderBootstrapStep {
    InitWindow,
    InitWgpuInstance,
    InitSurface,
    RequestDevice,
    ConfigureSwapchain,
    UploadBootScreenQuad,
    DrawBootScreen,
}

impl RenderBootstrapStep {
    pub fn as_str(&self) -> &'static str {
        match self {
            RenderBootstrapStep::InitWindow => "InitWindow",
            RenderBootstrapStep::InitWgpuInstance => "InitWgpuInstance",
            RenderBootstrapStep::InitSurface => "InitSurface",
            RenderBootstrapStep::RequestDevice => "RequestDevice",
            RenderBootstrapStep::ConfigureSwapchain => "ConfigureSwapchain",
            RenderBootstrapStep::UploadBootScreenQuad => "UploadBootScreenQuad",
            RenderBootstrapStep::DrawBootScreen => "DrawBootScreen",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderBootstrapTaskStatus {
    pub step: RenderBootstrapStep,
    pub ready: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderBootstrapPlan {
    pub tasks: Vec<RenderBootstrapTaskStatus>,
}

impl RenderBootstrapPlan {
    pub fn for_mode(mode: RenderBackendMode) -> Self {
        let ready = matches!(mode, RenderBackendMode::WgpuPlanned);
        let steps = [
            RenderBootstrapStep::InitWindow,
            RenderBootstrapStep::InitWgpuInstance,
            RenderBootstrapStep::InitSurface,
            RenderBootstrapStep::RequestDevice,
            RenderBootstrapStep::ConfigureSwapchain,
            RenderBootstrapStep::UploadBootScreenQuad,
            RenderBootstrapStep::DrawBootScreen,
        ];

        Self {
            tasks: steps
                .into_iter()
                .map(|step| RenderBootstrapTaskStatus { step, ready })
                .collect(),
        }
    }

    pub fn ready_count(&self) -> usize {
        self.tasks.iter().filter(|t| t.ready).count()
    }

    pub fn total_count(&self) -> usize {
        self.tasks.len()
    }

    pub fn summary(&self) -> String {
        self.tasks
            .iter()
            .map(|task| format!("{}:{}", task.step.as_str(), task.ready))
            .collect::<Vec<_>>()
            .join(",")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderBootstrapExecutor {
    plan: RenderBootstrapPlan,
    next_step_index: usize,
}

impl RenderBootstrapExecutor {
    pub fn new(mode: RenderBackendMode) -> Self {
        Self {
            plan: RenderBootstrapPlan::for_mode(mode),
            next_step_index: 0,
        }
    }

    pub fn execute_next(&mut self) -> Option<RenderBootstrapStep> {
        let step = self.plan.tasks.get(self.next_step_index).map(|t| t.step)?;
        self.next_step_index += 1;
        Some(step)
    }

    pub fn completed_count(&self) -> usize {
        self.next_step_index.min(self.plan.tasks.len())
    }

    pub fn total_count(&self) -> usize {
        self.plan.tasks.len()
    }

    pub fn last_completed_step(&self) -> Option<RenderBootstrapStep> {
        self.next_step_index
            .checked_sub(1)
            .and_then(|idx| self.plan.tasks.get(idx).map(|t| t.step))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RealRendererBootstrapResult {
    FeatureDisabled,
    AdapterUnavailable,
    DeviceRequestFailed,
    Ready,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RealRendererBootstrapStatus {
    pub result: RealRendererBootstrapResult,
    pub detail: String,
}

pub fn attempt_real_renderer_bootstrap() -> RealRendererBootstrapStatus {
    #[cfg(feature = "real_graphics")]
    {
        return RealRendererBootstrapStatus {
            result: RealRendererBootstrapResult::AdapterUnavailable,
            detail: "real_graphics feature enabled; adapter probe not wired yet".to_string(),
        };
    }

    #[cfg(not(feature = "real_graphics"))]
    {
        RealRendererBootstrapStatus {
            result: RealRendererBootstrapResult::FeatureDisabled,
            detail: "build without real_graphics feature".to_string(),
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
    pub curve_exp: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootStylePreset {
    Classic,
    NeonStorm,
    CrystalPulse,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BootStyleProfile {
    pub ignition: PhaseStyle,
    pub pulse_lock: PhaseStyle,
    pub reveal: PhaseStyle,
    pub preset: BootStylePreset,
}

impl BootStyleProfile {
    pub fn from_preset(preset: BootStylePreset) -> Self {
        match preset {
            BootStylePreset::Classic => Self::default(),
            BootStylePreset::NeonStorm => Self {
                ignition: PhaseStyle {
                    intensity_mul: 0.95,
                    hue_bias: 24.0,
                    distortion_weight: 0.65,
                    curve_exp: 1.3,
                },
                pulse_lock: PhaseStyle {
                    intensity_mul: 1.3,
                    hue_bias: 42.0,
                    distortion_weight: 0.95,
                    curve_exp: 1.7,
                },
                reveal: PhaseStyle {
                    intensity_mul: 1.05,
                    hue_bias: 16.0,
                    distortion_weight: 0.45,
                    curve_exp: 1.1,
                },
                preset,
            },
            BootStylePreset::CrystalPulse => Self {
                ignition: PhaseStyle {
                    intensity_mul: 0.8,
                    hue_bias: -18.0,
                    distortion_weight: 0.4,
                    curve_exp: 0.9,
                },
                pulse_lock: PhaseStyle {
                    intensity_mul: 1.05,
                    hue_bias: -4.0,
                    distortion_weight: 0.55,
                    curve_exp: 1.2,
                },
                reveal: PhaseStyle {
                    intensity_mul: 1.2,
                    hue_bias: 8.0,
                    distortion_weight: 0.25,
                    curve_exp: 0.8,
                },
                preset,
            },
        }
    }

    pub fn style_for(&self, phase: BootPhase) -> PhaseStyle {
        match phase {
            BootPhase::Ignition => self.ignition,
            BootPhase::PulseLock => self.pulse_lock,
            BootPhase::Reveal => self.reveal,
        }
    }
}

impl Default for BootStyleProfile {
    fn default() -> Self {
        Self {
            ignition: PhaseStyle {
                intensity_mul: 0.85,
                hue_bias: -12.0,
                distortion_weight: 0.55,
                curve_exp: 1.0,
            },
            pulse_lock: PhaseStyle {
                intensity_mul: 1.15,
                hue_bias: 18.0,
                distortion_weight: 0.8,
                curve_exp: 1.4,
            },
            reveal: PhaseStyle {
                intensity_mul: 1.0,
                hue_bias: 4.0,
                distortion_weight: 0.3,
                curve_exp: 0.9,
            },
            preset: BootStylePreset::Classic,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootSequenceRecipe {
    Standard,
    QuickPulse,
    GrandReveal,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BootSequenceConfig {
    pub ignition_ratio: f32,
    pub pulse_lock_ratio: f32,
    pub reveal_ratio: f32,
    pub pulse_speed_mul: f32,
}

impl BootSequenceConfig {
    pub fn from_recipe(recipe: BootSequenceRecipe) -> Self {
        match recipe {
            BootSequenceRecipe::Standard => Self {
                ignition_ratio: 0.33,
                pulse_lock_ratio: 0.34,
                reveal_ratio: 0.33,
                pulse_speed_mul: 1.0,
            },
            BootSequenceRecipe::QuickPulse => Self {
                ignition_ratio: 0.22,
                pulse_lock_ratio: 0.5,
                reveal_ratio: 0.28,
                pulse_speed_mul: 1.2,
            },
            BootSequenceRecipe::GrandReveal => Self {
                ignition_ratio: 0.38,
                pulse_lock_ratio: 0.27,
                reveal_ratio: 0.35,
                pulse_speed_mul: 0.85,
            },
        }
    }
}

impl Default for BootSequenceConfig {
    fn default() -> Self {
        Self::from_recipe(BootSequenceRecipe::Standard)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BootTimelineFrame {
    pub phase: BootPhase,
    pub frame: BootFrame,
    pub phase_t: f32,
    pub styled_glow: f32,
    pub styled_hue: f32,
    pub distortion_weight: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BootRenderIntent {
    pub bloom_weight: f32,
    pub distortion_weight: f32,
    pub fog_weight: f32,
    pub color_shift: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BootPostFxSnapshot {
    pub tick: u64,
    pub bloom_strength: f32,
    pub fog_density: f32,
    pub distortion_amount: f32,
    pub color_grade_shift: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BootPostFxAggregate {
    pub avg_bloom: f32,
    pub avg_fog: f32,
    pub avg_distortion: f32,
    pub avg_color_shift: f32,
    pub peak_bloom: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BootPostFxTrack {
    pub snapshots: Vec<BootPostFxSnapshot>,
}

impl BootPostFxTrack {
    pub fn from_timeline(timeline: &BootTimeline) -> Self {
        Self {
            snapshots: timeline.to_postfx_snapshots(),
        }
    }

    pub fn snapshot_for_tick(&self, tick: u64) -> Option<BootPostFxSnapshot> {
        self.snapshots.iter().find(|s| s.tick == tick).copied()
    }

    pub fn latest_snapshot(&self) -> Option<BootPostFxSnapshot> {
        self.snapshots.last().copied()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BootScreenFrame {
    pub tick: u64,
    pub title_progress: f32,
    pub title_glow: f32,
    pub subtitle_opacity: f32,
    pub glyphs_lit: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BootScreenSequence {
    pub title_text: String,
    pub subtitle_text: String,
    pub frames: Vec<BootScreenFrame>,
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

    pub fn derive_render_intents(&self) -> Vec<BootRenderIntent> {
        self.frames
            .iter()
            .map(|f| {
                let (phase_bloom, phase_fog) = match f.phase {
                    BootPhase::Ignition => (0.85, 0.25),
                    BootPhase::PulseLock => (1.15, 0.5),
                    BootPhase::Reveal => (1.0, 0.7),
                };

                BootRenderIntent {
                    bloom_weight: f.styled_glow * phase_bloom,
                    distortion_weight: f.distortion_weight,
                    fog_weight: (0.2 + f.phase_t * 0.8) * phase_fog,
                    color_shift: f.styled_hue,
                }
            })
            .collect()
    }

    pub fn to_postfx_snapshots(&self) -> Vec<BootPostFxSnapshot> {
        let intents = self.derive_render_intents();
        self.frames
            .iter()
            .zip(intents.iter())
            .map(|(frame, intent)| BootPostFxSnapshot {
                tick: frame.frame.tick,
                bloom_strength: intent.bloom_weight,
                fog_density: intent.fog_weight,
                distortion_amount: intent.distortion_weight,
                color_grade_shift: intent.color_shift,
            })
            .collect()
    }

    pub fn to_boot_screen_sequence(
        &self,
        title_text: &str,
        subtitle_text: &str,
    ) -> BootScreenSequence {
        let glyph_count = title_text.chars().count().max(1);
        let frames = self
            .frames
            .iter()
            .map(|f| {
                let reveal_weight = match f.phase {
                    BootPhase::Ignition => 0.2,
                    BootPhase::PulseLock => 0.65,
                    BootPhase::Reveal => 1.0,
                };
                let title_progress = (f.phase_t * reveal_weight).clamp(0.0, 1.0);
                let glyphs_lit =
                    ((title_progress * glyph_count as f32).ceil() as usize).clamp(1, glyph_count);
                let title_glow = (f.styled_glow * (0.55 + reveal_weight * 0.45)).clamp(0.0, 2.0);
                let subtitle_opacity = (0.2 + title_progress * 0.8).clamp(0.0, 1.0);

                BootScreenFrame {
                    tick: f.frame.tick,
                    title_progress,
                    title_glow,
                    subtitle_opacity,
                    glyphs_lit,
                }
            })
            .collect();

        BootScreenSequence {
            title_text: title_text.to_string(),
            subtitle_text: subtitle_text.to_string(),
            frames,
        }
    }

    pub fn aggregate_postfx(&self) -> BootPostFxAggregate {
        let snapshots = self.to_postfx_snapshots();
        let len = snapshots.len().max(1) as f32;

        let avg_bloom = snapshots.iter().map(|s| s.bloom_strength).sum::<f32>() / len;
        let avg_fog = snapshots.iter().map(|s| s.fog_density).sum::<f32>() / len;
        let avg_distortion = snapshots.iter().map(|s| s.distortion_amount).sum::<f32>() / len;
        let avg_color_shift = snapshots.iter().map(|s| s.color_grade_shift).sum::<f32>() / len;
        let peak_bloom = snapshots
            .iter()
            .map(|s| s.bloom_strength)
            .fold(0.0_f32, f32::max);

        BootPostFxAggregate {
            avg_bloom,
            avg_fog,
            avg_distortion,
            avg_color_shift,
            peak_bloom,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BootAnimator {
    config: BootAnimationConfig,
    style: BootStyleProfile,
    sequence: BootSequenceConfig,
    recipe: BootSequenceRecipe,
}

impl BootAnimator {
    pub fn new(config: BootAnimationConfig) -> Self {
        Self {
            config,
            style: BootStyleProfile::default(),
            sequence: BootSequenceConfig::default(),
            recipe: BootSequenceRecipe::Standard,
        }
    }

    pub fn with_style(config: BootAnimationConfig, style: BootStyleProfile) -> Self {
        Self {
            config,
            style,
            sequence: BootSequenceConfig::default(),
            recipe: BootSequenceRecipe::Standard,
        }
    }

    pub fn with_style_and_recipe(
        config: BootAnimationConfig,
        style: BootStyleProfile,
        recipe: BootSequenceRecipe,
    ) -> Self {
        Self {
            config,
            style,
            sequence: BootSequenceConfig::from_recipe(recipe),
            recipe,
        }
    }

    pub fn recipe(&self) -> BootSequenceRecipe {
        self.recipe
    }

    pub fn generate_frames(&self, start_tick: u64) -> Vec<BootFrame> {
        (0..self.config.frame_count)
            .map(|i| {
                let t = i as f32 * self.config.pulse_speed * self.sequence.pulse_speed_mul;
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

        let ignition_end = ((total as f32 * self.sequence.ignition_ratio).round() as usize)
            .clamp(1, total.saturating_sub(2).max(1));
        let pulse_lock_end = ((total as f32
            * (self.sequence.ignition_ratio + self.sequence.pulse_lock_ratio))
            .round() as usize)
            .clamp(
                ignition_end + 1,
                total.saturating_sub(1).max(ignition_end + 1),
            );

        let ignition_span = ignition_end.max(1);
        let pulse_lock_span = pulse_lock_end.saturating_sub(ignition_end).max(1);
        let reveal_span = total.saturating_sub(pulse_lock_end).max(1);

        let frames = raw
            .into_iter()
            .enumerate()
            .map(|(idx, frame)| {
                let (phase, local_idx, span) = if idx < ignition_end {
                    (BootPhase::Ignition, idx, ignition_span)
                } else if idx < pulse_lock_end {
                    (BootPhase::PulseLock, idx - ignition_end, pulse_lock_span)
                } else {
                    (BootPhase::Reveal, idx - pulse_lock_end, reveal_span)
                };

                let phase_t = (local_idx as f32 / span as f32).clamp(0.0, 1.0);

                let phase_style = self.style.style_for(phase);
                let curve = phase_t.powf(phase_style.curve_exp.max(0.01));

                BootTimelineFrame {
                    phase,
                    phase_t,
                    styled_glow: frame.glow * (phase_style.intensity_mul + 0.1 * curve),
                    styled_hue: frame.hue_shift + phase_style.hue_bias * (0.6 + 0.4 * curve),
                    distortion_weight: phase_style.distortion_weight * (0.75 + 0.25 * curve),
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
    fn real_bootstrap_reports_feature_disabled_by_default() {
        let status = attempt_real_renderer_bootstrap();
        #[cfg(not(feature = "real_graphics"))]
        {
            assert_eq!(status.result, RealRendererBootstrapResult::FeatureDisabled);
            assert!(status.detail.contains("without real_graphics"));
        }
    }

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
    fn bootstrap_executor_advances_through_steps() {
        let mut executor = RenderBootstrapExecutor::new(RenderBackendMode::WgpuPlanned);

        assert_eq!(executor.completed_count(), 0);
        assert_eq!(executor.total_count(), 7);

        let mut last = None;
        while let Some(step) = executor.execute_next() {
            last = Some(step);
        }

        assert_eq!(executor.completed_count(), executor.total_count());
        assert_eq!(executor.last_completed_step(), last);
        assert_eq!(
            executor.last_completed_step(),
            Some(RenderBootstrapStep::DrawBootScreen)
        );
    }

    #[test]
    fn bootstrap_plan_matches_backend_mode() {
        let mock = RenderBootstrapPlan::for_mode(RenderBackendMode::Mock);
        assert_eq!(mock.ready_count(), 0);
        assert_eq!(mock.total_count(), 7);

        let planned = RenderBootstrapPlan::for_mode(RenderBackendMode::WgpuPlanned);
        assert_eq!(planned.ready_count(), planned.total_count());
        assert!(planned.summary().contains("DrawBootScreen:true"));
    }

    #[test]
    fn readiness_contract_tracks_backend_mode() {
        let mock = RenderBackendReadiness::for_mode(RenderBackendMode::Mock);
        assert!(!mock.has_windowing);
        assert!(!mock.has_gpu_backend);
        assert!(!mock.can_present);

        let planned = RenderBackendReadiness::for_mode(RenderBackendMode::WgpuPlanned);
        assert!(planned.has_windowing);
        assert!(planned.has_gpu_backend);
        assert!(planned.can_present);
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
        assert_eq!(
            renderer.backend_status().mode,
            RenderBackendMode::WgpuPlanned
        );
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
            style,
        )
        .generate_timeline(5);

        let first = &timeline.frames[0];
        let last = &timeline.frames[timeline.frames.len() - 1];

        assert_eq!(first.phase, BootPhase::Ignition);
        assert_eq!(last.phase, BootPhase::Reveal);
        assert!(first.distortion_weight > 0.0);
        assert!(last.distortion_weight > 0.0);
    }

    #[test]
    fn preset_selection_changes_styling() {
        let cfg = BootAnimationConfig {
            seed: 99,
            frame_count: 12,
            ..BootAnimationConfig::default()
        };

        let classic = BootAnimator::with_style(
            cfg.clone(),
            BootStyleProfile::from_preset(BootStylePreset::Classic),
        )
        .generate_timeline(0);

        let storm = BootAnimator::with_style(
            cfg,
            BootStyleProfile::from_preset(BootStylePreset::NeonStorm),
        )
        .generate_timeline(0);

        assert_ne!(classic.frames[0].styled_hue, storm.frames[0].styled_hue);
        assert_ne!(classic.frames[0].styled_glow, storm.frames[0].styled_glow);
    }

    #[test]
    fn render_intents_are_derived_for_each_frame() {
        let timeline = BootAnimator::with_style_and_recipe(
            BootAnimationConfig {
                seed: 44,
                frame_count: 12,
                ..BootAnimationConfig::default()
            },
            BootStyleProfile::from_preset(BootStylePreset::NeonStorm),
            BootSequenceRecipe::GrandReveal,
        )
        .generate_timeline(0);

        let intents = timeline.derive_render_intents();
        assert_eq!(intents.len(), timeline.frames.len());
        assert!(intents.iter().all(|i| i.bloom_weight > 0.0));
    }

    #[test]
    fn render_intent_values_stay_in_reasonable_ranges() {
        let timeline = BootAnimator::with_style_and_recipe(
            BootAnimationConfig {
                seed: 77,
                frame_count: 16,
                ..BootAnimationConfig::default()
            },
            BootStyleProfile::from_preset(BootStylePreset::CrystalPulse),
            BootSequenceRecipe::QuickPulse,
        )
        .generate_timeline(0);

        let intents = timeline.derive_render_intents();
        for i in intents {
            assert!(i.bloom_weight > 0.0);
            assert!(i.fog_weight >= 0.0);
            assert!(i.distortion_weight >= 0.0);
            assert!(i.color_shift.is_finite());
        }
    }

    #[test]
    fn boot_screen_sequence_tracks_title_reveal() {
        let timeline = BootAnimator::with_style_and_recipe(
            BootAnimationConfig {
                seed: 1337,
                frame_count: 12,
                ..BootAnimationConfig::default()
            },
            BootStyleProfile::from_preset(BootStylePreset::NeonStorm),
            BootSequenceRecipe::GrandReveal,
        )
        .generate_timeline(1);

        let sequence = timeline.to_boot_screen_sequence("AUREX-X", "Booting runtime");
        assert_eq!(sequence.title_text, "AUREX-X");
        assert_eq!(sequence.frames.len(), timeline.frames.len());

        let first = sequence.frames.first().unwrap();
        let last = sequence.frames.last().unwrap();
        assert!(first.glyphs_lit >= 1);
        assert!(last.glyphs_lit >= first.glyphs_lit);
        assert!(last.subtitle_opacity >= first.subtitle_opacity);
    }

    #[test]
    fn postfx_snapshot_and_aggregate_are_consistent() {
        let timeline = BootAnimator::with_style_and_recipe(
            BootAnimationConfig {
                seed: 101,
                frame_count: 12,
                ..BootAnimationConfig::default()
            },
            BootStyleProfile::from_preset(BootStylePreset::NeonStorm),
            BootSequenceRecipe::GrandReveal,
        )
        .generate_timeline(0);

        let snapshots = timeline.to_postfx_snapshots();
        let agg = timeline.aggregate_postfx();

        assert_eq!(snapshots.len(), 12);
        assert!(agg.avg_bloom > 0.0);
        assert!(agg.peak_bloom >= agg.avg_bloom);
        assert!(agg.avg_fog >= 0.0);
    }

    #[test]
    fn postfx_track_supports_tick_lookup() {
        let timeline = BootAnimator::with_style_and_recipe(
            BootAnimationConfig {
                seed: 303,
                frame_count: 10,
                ..BootAnimationConfig::default()
            },
            BootStyleProfile::from_preset(BootStylePreset::Classic),
            BootSequenceRecipe::Standard,
        )
        .generate_timeline(25);

        let track = BootPostFxTrack::from_timeline(&timeline);
        assert_eq!(track.snapshots.len(), 10);
        assert!(track.snapshot_for_tick(25).is_some());
        assert!(track.snapshot_for_tick(999).is_none());
        assert_eq!(track.latest_snapshot().unwrap().tick, 34);
    }

    #[test]
    fn recipe_changes_phase_distribution() {
        let cfg = BootAnimationConfig {
            seed: 12,
            frame_count: 12,
            ..BootAnimationConfig::default()
        };

        let standard = BootAnimator::with_style_and_recipe(
            cfg.clone(),
            BootStyleProfile::from_preset(BootStylePreset::Classic),
            BootSequenceRecipe::Standard,
        )
        .generate_timeline(0);

        let quick = BootAnimator::with_style_and_recipe(
            cfg,
            BootStyleProfile::from_preset(BootStylePreset::Classic),
            BootSequenceRecipe::QuickPulse,
        )
        .generate_timeline(0);

        assert_ne!(standard.phase_counts(), quick.phase_counts());
    }
}
