# Example Pulses (Technical SDK)

Aurex includes small showcase pulses demonstrating the current architecture:

Pulse Runtime
→ Experience Planner
→ WorldBlueprint
→ GeneratorStack
→ RhythmField modulation
→ Renderer Pipeline

Renderer stages remain unchanged:
1. ScenePreprocess
2. EffectGraphEvaluation
3. GeometrySdf
4. MaterialPattern
5. LightingAtmosphere
6. Particles
7. PostProcessing
8. TemporalFeedback

## Included examples
Implemented in `crates/aurex_app/src/pulses/`:
- `electronic_megacity`
- `jazz_atmosphere`
- `ambient_dreamscape`

Each pulse builds:
- a `WorldBlueprint` (identity/theme/palette/camera intent)
- base `GeneratorStackOutput`
- deterministic `RhythmFieldSnapshot`
- modulated `GeneratorStackOutput`

## Modulation notes
RhythmField signals (`beat_phase`, `bar_phase`, `pulse`, `bass_energy`, `mid_energy`, `high_energy`, `intensity`, `accent`) modulate layer parameters with bounded additive deltas. Base world parameters are preserved.

## Developer usage
Call:
- `create_electronic_megacity_pulse(seed)`
- `create_jazz_atmosphere_pulse(seed)`
- `create_ambient_dreamscape_pulse(seed)`

These helpers are deterministic for the same seed and pulse type.


## Builder integration
Example pulses are created through `PulseBuilder` so developer-created pulses, future APL, and UI tooling can share one construction path.
