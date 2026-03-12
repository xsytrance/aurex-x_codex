# AI Authoring: Temporal Effects

Use `temporal_effects` in `sdf` scenes to enable frame-history blending.

## JSON fields
Each `TemporalEffect` supports:
- `blend_mode` (`AdditiveTrail`, `DecayTrail`, `MotionEcho`, `BeatEcho`, `ColorSmear`)
- `decay_rate`
- `feedback_strength`
- `beat_sync`
- `color_shift`

## Authoring examples
- Motion trails: `AdditiveTrail` with moderate decay.
- Rhythm echoes: `BeatEcho` with higher `beat_sync`.
- Synesthetic drift: `ColorSmear` with non-zero `color_shift`.

Combine with music-reactive scenes for strongest perceived visual rhythm.
