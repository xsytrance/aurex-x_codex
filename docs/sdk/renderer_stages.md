# Renderer Stages (Technical SDK)

Renderer stage model (`aurex_render_sdf::stages`):
1. ScenePreprocess
2. EffectGraphEvaluation
3. GeometrySdf
4. MaterialPattern
5. LightingAtmosphere
6. Particles
7. PostProcessing
8. TemporalFeedback

This decomposition supports:
- profiling and diagnostics
- deterministic stage ordering
- future GPU pass mapping without changing stage semantics

## Timing instrumentation
Each stage records elapsed time in diagnostics:
- `FrameDiagnostics.stage_durations_ms`
- `FrameDiagnostics.stage_percentages`
- `FrameDiagnostics.total_frame_time_ms`

When `AUREX_DIAGNOSTICS=1`, the example runner prints stage duration and percentage of total frame time for each stage.

## GeometrySdf operational modes

GeometrySdf now has an explicit mode contract used during stabilization:

- `flat`
- `safe` (default)
- `legacy`

See `docs/sdk/runtime_debug_flags.md` for environment controls and defaults.
