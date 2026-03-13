# Pulse Builder API (Technical SDK)

`PulseBuilder` provides a clean programmatic API for constructing showcase pulses using the existing Aurex pipeline.

## Purpose
`PulseBuilder` maps high-level pulse hints to existing engine systems:
- `WorldBlueprint` identity hints
- base `GeneratorStackOutput`
- deterministic `RhythmField` snapshot sampling
- bounded modulation via `apply_rhythm_modulation`

No new engine subsystem is introduced.

## API location
`crates/aurex_app/src/pulse_builder.rs`

## Core types
- `PulseConfig`
- `PulseBuilder`
- `LightingStyle`
- `CameraStyle`
- output: `ExamplePulseConfig`

`PulseConfig` fields:
- `name`
- `theme` (`VisualTheme`)
- `seed`
- `structure_density_hint`
- `lighting_style`
- `particle_intensity_hint`
- `camera_style`
- `rhythm_reactivity`

## Example usage
```rust
let pulse = PulseBuilder::new("Electronic Megacity")
    .theme(VisualTheme::Electronic)
    .seed(42)
    .structure_density(0.9)
    .particle_intensity(0.8)
    .lighting_style(LightingStyle::Neon)
    .camera_style(CameraStyle::Orbit)
    .rhythm_reactivity(1.0)
    .build();
```

## Architecture mapping
Pulse Runtime
→ Pulse (builder-generated config)
→ WorldBlueprint
→ GeneratorStack
→ RhythmField modulation
→ Renderer Pipeline

Renderer stages remain unchanged:
ScenePreprocess → EffectGraphEvaluation → GeometrySdf → MaterialPattern → LightingAtmosphere → Particles → PostProcessing → TemporalFeedback
