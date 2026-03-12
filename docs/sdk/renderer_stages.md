# Renderer Stages (Technical SDK)

Renderer stage model (`aurex_render_sdf::stages`):
1. ScenePreprocess
2. EffectGraphEvaluation
3. GeometrySdf
4. MaterialPattern
5. LightingAtmosphere
6. PostProcessing

This explicit decomposition supports:
- profiling and diagnostics
- future deterministic parallel batching
- future GPU pass mapping
