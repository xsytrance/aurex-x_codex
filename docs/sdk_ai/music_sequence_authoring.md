# AI Authoring: Music Sequence Config

Use `music` inside `PulseDefinition` to define procedural music.

## Minimal shape
- `bpm`
- optional `ppq` (default 16)
- `tracks[]`
  - `name`
  - `instrument` (`SineSynth`, `NoiseSynth`, `PulseSynth`, `Percussion`)
  - `pattern.steps`
  - `pattern.events`
    - `Note`
    - `Modulation`
    - `GeneratorHook`

## RhythmField outputs
Sequencer emits signals usable by runtime systems:
- `tempo`
- `beat_phase`
- `beat_strength`
- `beat_index`
- `bar_index`
- `phrase_index`
- `bass_energy`
- `harmonic_energy`
- `spectral_flux`
- `groove_vector`

Pulse runtime stores these signals in runtime context every frame and exposes diagnostics summary values (`beat_phase`, `bar_index`, `bass_energy`).

If pulse scene audio is not explicitly set, runtime can derive procedural audio track config from sequencer tracks.
