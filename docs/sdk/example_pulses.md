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

## PulseBuilder usage
Example pulses are created through `PulseBuilder` (`crates/aurex_app/src/pulse_builder/`), which maps `PulseConfig` hints to existing systems.

Core `PulseConfig` hints:
- `name`, `seed`, `theme`
- `geometry_style`, `structure_set`
- `atmosphere_type`, `lighting_mode`
- `color_palette`, `camera_rig`
- optional modulation controls: `rhythm_intensity`, `particle_density_multiplier`

`build()` performs deterministic wiring only:
- create `WorldBlueprint`
- create base `GeneratorStackOutput`
- sample `RhythmFieldSnapshot`
- apply existing modulation

## Included examples
Implemented in `crates/aurex_app/src/pulses/`:
- `electronic_megacity`
- `jazz_atmosphere`
- `ambient_dreamscape`
- `aurielle_intro`

All are deterministic for the same pulse type + seed.

## Phase-based evolution
Example pulses can optionally attach a `PulseSequence` of named phases.
Phase overrides are deterministic bounded adjustments applied through PulseBuilder before RhythmField modulation.


## CLI selection
Run a specific pulse from the app crate:
- `cargo run -p aurex_app -- megacity`
- `cargo run -p aurex_app -- jazz`
- `cargo run -p aurex_app -- ambient`
- `cargo run -p aurex_app -- aurielle_intro`
- optional deterministic seed: `cargo run -p aurex_app -- megacity --seed 42`
