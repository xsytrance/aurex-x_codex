# Pulse Builder API (Technical SDK)

`PulseBuilder` provides a clean programmatic API for constructing showcase pulses using existing Aurex systems.

## API location
`crates/aurex_app/src/pulse_builder/`
- `mod.rs`
- `config.rs`
- `builder.rs`

## Core types
- `PulseConfig`
- `PulseBuilder`
- `GeometryStyle`
- `AtmosphereType`
- `LightingMode`
- `StructureSet`
- `CameraRig`
- output: `ExamplePulseConfig`

## PulseConfig fields
- `name`
- `seed`
- `theme`
- `geometry_style`
- `atmosphere_type`
- `lighting_mode`
- `structure_set`
- `color_palette`
- `camera_rig`
- optional controls: `rhythm_intensity`, `particle_density_multiplier`

## Example usage
```rust
let pulse = PulseBuilder::new("Electronic Megacity")
    .seed(42)
    .theme(VisualTheme::Electronic)
    .geometry_style(GeometryStyle::City)
    .structures(StructureSet::Dense)
    .lighting(LightingMode::NeonPulse)
    .atmosphere(AtmosphereType::Clear)
    .camera_rig(CameraRig::Orbit)
    .rhythm_intensity(1.0)
    .particle_density_multiplier(0.82)
    .sequence(sequence)
    .build();
```

## Build mapping
`build()` performs deterministic existing-system wiring:
1. construct world blueprint hints
2. construct base generator stack output
3. sample `RhythmField`
4. apply modulation

No renderer stage changes are introduced.


## Optional sequencing
Use `.sequence(PulseSequence)` to enable deterministic phase-level evolution.
Without a sequence, behavior is identical to previous PulseBuilder output.
