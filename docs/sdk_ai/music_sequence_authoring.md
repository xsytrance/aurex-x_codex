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
- `beat_phase`
- `beat_strength`
- `bass_energy`
- `harmonic_energy`

If pulse scene audio is not explicitly set, runtime can derive procedural audio track config from sequencer tracks.
