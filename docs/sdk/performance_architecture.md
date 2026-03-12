# Performance Architecture (Technical SDK)

Aurex-X renderer acceleration now focuses on deterministic optimization tracks:

- adaptive raymarch stepping
- explicit stage separation
- deterministic per-frame caches
- diagnostics and render stats
- GPU-ready descriptor scaffolding

## Parallel-safety stance
- Current default path remains serial for strict behavior parity.
- Stages are now explicit and structured to enable future parallel execution with deterministic ordering constraints.

## Caching
Implemented explicit cache structures in `aurex_render_sdf::cache`:
- `PatternSampleCache`
- `FieldSampleCache`
- `EffectGraphEvalCache`
- `SceneBoundsCache`

Caches are deterministic and invalidation-friendly.

## Diagnostics
`render_sdf_scene_with_diagnostics` returns frame diagnostics with:
- raymarch step totals
- rays traced
- cache hit/miss counters
