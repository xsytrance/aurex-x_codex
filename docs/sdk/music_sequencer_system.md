# Procedural Music Sequencer System (Technical SDK)

Aurex introduces a deterministic procedural music sequencer that orchestrates (not replaces) the existing procedural audio engine.

## Crate
`crates/aurex_music`

## Core components
- `TempoClock` (`tempo.rs`)
- `Pattern` + `PatternEvent` (`pattern.rs`)
- `Track` (`track.rs`)
- `InstrumentKind` (`instrument.rs`)
- `MusicSequencer` + `MusicSequenceConfig` (`sequencer.rs`)
- `RhythmField` (`rhythm_field.rs`)

## Integration with Pulses
`PulseDefinition` now supports optional `music: MusicSequenceConfig`.

`PulseRunner` behavior:
- initialize sequencer if `music` exists
- update sequencer each `update(dt)`
- expose sequencer-derived `RhythmField`
- if scene audio is absent, derive `ProceduralAudioConfig` from sequencer tracks

## Determinism
- tempo clock progression is time-step driven
- pattern scheduling is step/tick based
- event emission is deterministic given same seed + dt stream

## RhythmField signal glossary
`RhythmField` is the real-time modulation surface produced by the sequencer and consumed by Pulse runtime systems.

- `tempo`: current BPM from `TempoClock`
- `beat_phase`: normalized phase within the current beat (`0.0..1.0`)
- `beat_strength`: transient-like beat emphasis scalar
- `beat_index`: absolute beat counter
- `bar_index`: absolute bar counter
- `phrase_index`: higher-level musical phrase counter
- `bass_energy`: low-frequency energy estimate
- `harmonic_energy`: mid/high harmonic energy estimate
- `spectral_flux`: frame-to-frame spectral change estimate
- `groove_vector`: compact multi-axis groove descriptor for downstream modulation

These values are deterministic for a fixed pulse definition and fixed simulation timestep stream.
