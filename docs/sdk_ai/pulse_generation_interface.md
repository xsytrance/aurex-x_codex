# AI Pulse Generation Interface

Use `PulseBuilder` as the canonical construction interface for generated showcase pulses.

## Input surface
Provide:
- pulse name
- visual theme (`Electronic`, `Jazz`, `Ambient`)
- seed
- structure density hint (`0.0..1.0`)
- lighting style (`Neon`, `Warm`, `Diffuse`)
- particle intensity hint (`0.0..1.0`)
- camera style (`Orbit`, `Drift`, `Float`)
- rhythm reactivity (`0.0..1.0`)

## Build contract
The builder must produce deterministic output by reusing existing functions:
- base stack output hints
- `sample_rhythm_field`
- `apply_rhythm_modulation`

No direct renderer-stage mutation is allowed.

## Target flow
Pulse Runtime
→ PulseBuilder output
→ WorldBlueprint
→ GeneratorStack
→ RhythmField
→ Renderer Pipeline
