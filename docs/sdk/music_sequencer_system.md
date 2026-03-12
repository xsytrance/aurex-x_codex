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
