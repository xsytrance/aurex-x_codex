# AI Authoring: Example Pulse Generation

Use `PulseBuilder` as the canonical interface for generated showcase pulses.

## Builder contract
Provide `PulseConfig`-style hints:
- `name`, `seed`, `theme`
- `geometry_style`, `structure_set`
- `atmosphere_type`, `lighting_mode`
- `color_palette`, `camera_rig`
- `rhythm_intensity`, `particle_density_multiplier`

Builder output must remain deterministic and reuse existing systems:
1. world blueprint hints
2. base generator stack output
3. `sample_rhythm_field`
4. `apply_rhythm_modulation`

## Existing templates
- `create_electronic_megacity_pulse(seed)`
- `create_jazz_atmosphere_pulse(seed)`
- `create_ambient_dreamscape_pulse(seed)`

Do not introduce new renderer stages or alter stage order.

## Optional PulseSequence
Generated pulses may include a phase sequence for narrative progression.
Keep phases deterministic and use bounded overrides only.
Current interface is phase-level (track-level sequencing is future work).
