use crate::{RenderConfig, RenderedFrame, Scene, diagnostics::FrameDiagnostics};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RendererStage {
    ScenePreprocess,
    EffectGraphEvaluation,
    GeometrySdf,
    MaterialPattern,
    LightingAtmosphere,
    PostProcessing,
    TemporalFeedback,
}

pub const RENDERER_STAGES: [RendererStage; 7] = [
    RendererStage::ScenePreprocess,
    RendererStage::EffectGraphEvaluation,
    RendererStage::GeometrySdf,
    RendererStage::MaterialPattern,
    RendererStage::LightingAtmosphere,
    RendererStage::PostProcessing,
    RendererStage::TemporalFeedback,
];

pub struct StageExecutionResult {
    pub scene: Scene,
    pub diagnostics: FrameDiagnostics,
}

pub fn execute_scene_preprocess_stage(
    scene: &Scene,
    config: RenderConfig,
    mut diagnostics: FrameDiagnostics,
    preprocess: impl Fn(&Scene, RenderConfig) -> Scene,
) -> StageExecutionResult {
    diagnostics.stages.push("ScenePreprocess");
    let scene = preprocess(scene, config);
    StageExecutionResult { scene, diagnostics }
}

pub fn finalize_frame_stage(
    frame: RenderedFrame,
    mut diagnostics: FrameDiagnostics,
) -> (RenderedFrame, FrameDiagnostics) {
    diagnostics.stages.push("PostProcessing");
    (frame, diagnostics)
}
