# Performance Architecture (Technical SDK)

Aurex-X renderer acceleration focuses on deterministic optimization tracks:

- adaptive raymarch stepping
- explicit stage separation
- deterministic per-frame caches
- diagnostics and render stats
- GPU-ready descriptor scaffolding

## Parallel-safety stance
- Current default path remains serial for strict behavior parity.
- Stages are explicit and structured to enable future parallel execution with deterministic ordering constraints.

## Spatial caching
Implemented cache structures in `aurex_render_sdf::cache`:
- `PatternSampleCache`
- `FieldSampleCache`
- `EffectGraphEvalCache`
- `SceneBoundsCache`

Pattern/field cache keys are generated from world-space sample positions (`SampleKey::from_world(sample_position, time, seed)`), not just camera origin. This produces realistic hit/miss behavior and preserves deterministic key generation via quantized spatial bins.

## Effect graph reuse across frames
Effect-graph evaluation reuse is now backed by persistent renderer state (`RendererState.effect_graph_cache`). Reuse checks remain deterministic (`scene_seed + time tick`), and cache access is thread-safe via synchronized renderer state.

`effect_graph_evals` in diagnostics reports actual per-frame evaluations and is not reset when copying other cache counters.

## Stage timing diagnostics
`render_sdf_scene_with_diagnostics` returns:
- raymarch step totals
- rays traced
- cache hit/miss counters
- effect graph eval count
- per-stage duration (ms)
- per-stage frame percentage
- total frame time (ms)

These are aligned to the stage model in `aurex_render_sdf::stages`.
