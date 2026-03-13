# AI Authoring: Performance and Quality

## Render config controls
Runtime controls include adaptive raymarch and quality settings:
- `adaptive_raymarch`
- `min_step_scale`
- `max_step_scale`
- `far_field_boost`
- quality multipliers (`raymarch_quality`, `pattern_quality`, `field_quality`, `volumetric_quality`, `transition_quality`, `post_quality`)

## Diagnostics mode
Use diagnostics mode for deterministic per-frame counters:
- raymarch steps
- rays traced
- cache hit/miss counts
- effect graph evaluations
- per-stage timing (ms)
- per-stage frame percentage
- total frame time

## Spatial caching guidance
Cache behavior depends on world-space sample locality. Scenes with coherent spatial structure (repetition, smooth traversal) will show healthier cache reuse than chaotic camera paths.

## Effect-graph reuse guidance
When rendering repeated frames at identical seed/time-tick combinations, effect graph evaluation can be reused through persistent renderer state. This reduces overhead without introducing nondeterminism.

## Authoring guidance
- Keep high-density pattern/field scenes paired with moderate quality settings for iteration loops.
- Raise quality for final showcase renders.
- Use stage timing diagnostics to identify the dominant cost center before adjusting quality knobs.
