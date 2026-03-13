# Renderer Stages (Technical SDK)

Renderer stage model (`aurex_render_sdf::stages`):
1. ScenePreprocess
2. EffectGraphEvaluation
3. GeometrySdf
4. MaterialPattern
5. LightingAtmosphere
6. PostProcessing

This decomposition supports:
- profiling and diagnostics
- future deterministic parallel batching
- future GPU pass mapping

## Timing instrumentation
Each stage records elapsed time in diagnostics:
- `FrameDiagnostics.stage_durations_ms`
- `FrameDiagnostics.stage_percentages`
- `FrameDiagnostics.total_frame_time_ms`

When `AUREX_DIAGNOSTICS=1`, the example runner prints stage duration and percentage of total frame time for each stage.
