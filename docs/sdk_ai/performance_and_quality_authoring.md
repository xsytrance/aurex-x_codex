# AI Authoring: Performance and Quality

## Render config controls
Runtime controls include adaptive raymarch and quality settings:
- `adaptive_raymarch`
- `min_step_scale`
- `max_step_scale`
- `far_field_boost`
- quality multipliers (`raymarch_quality`, `pattern_quality`, `field_quality`, `volumetric_quality`, `transition_quality`, `post_quality`)

## Diagnostics mode
Use renderer diagnostics mode to get deterministic per-frame counters:
- raymarch steps
- rays traced
- cache hit/miss counts

## Authoring guidance
- Keep high-density pattern/field scenes paired with moderate quality settings for iteration loops.
- Raise quality for final showcase renders.
