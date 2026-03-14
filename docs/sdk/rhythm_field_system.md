# RhythmField System (Technical SDK)

`RhythmField` is a deterministic musical modulation layer that sits between procedural generation and rendering.

## Flow
Music Sequencer
↓
RhythmField snapshot
↓
Modulation pass
↓
GeneratorStackOutput (modulated)
↓
Renderer pipeline

## Scope
- `RhythmField` converts timing/energy state into normalized signals (`0.0..1.0`).
- It modulates existing generator stack output parameters.
- It does **not** generate geometry directly.
- It does **not** add, remove, or reorder renderer stages.

## Snapshot signals
- `beat_phase`
- `bar_phase`
- `pulse`
- `bass_energy`
- `mid_energy`
- `high_energy`
- `intensity`
- `accent`

## Determinism
Snapshot sampling depends only on:
- seed
- sequencer state
- time
- deterministic math

Given identical inputs, output snapshots and modulation results are identical.

## Theme sensitivity
Modulation amount is scaled by `VisualTheme` weight mappings (for example, stronger lighting/particles in Electronic, stronger atmosphere in Jazz).

## Future expansion
The same deterministic signal layer can later drive:
- event triggers
- beat-synced structure animations
- pulse-reactive environments
without changing renderer stage architecture.
