# AI Authoring: Example Pulse Generation

Use these examples as templates when generating new showcase pulses.

## Required structure
For each pulse, provide:
- world blueprint identity (name/theme/palette/camera intent)
- base `GeneratorStackOutput`
- sequencer state used for `RhythmFieldSnapshot`
- modulated output from `apply_rhythm_modulation`

## Existing templates
- `create_electronic_megacity_pulse(seed)`
- `create_jazz_atmosphere_pulse(seed)`
- `create_ambient_dreamscape_pulse(seed)`

## Authoring constraints
- Keep modulation bounded and deterministic.
- Modulate parameters, do not replace the base world identity.
- Do not introduce new renderer stages or reorder pipeline execution.

## Typical mapping hints
- `pulse` → lighting flash envelope
- `bass_energy` → terrain amplitude hints
- `mid_energy`/`bar_phase` → atmosphere drift
- `intensity`/`high_energy` → particles
- `accent` → structure emissive accents
