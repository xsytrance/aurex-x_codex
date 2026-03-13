# Aurex Audio System (AI Authoring Reference)

This reference describes runtime synth constraints and composition patterns for AI tooling.

## Hard constraints

- Keep CPAL stream architecture unchanged.
- No dynamic allocation in audio callback.
- Keep synth state persistent across callbacks.
- Deterministic math only.

## Runtime toolkit API concepts

### Oscillators
`OscillatorType`:
- `Sine`
- `Triangle`
- `Saw`
- `Square`
- `Noise`
- `Supersaw` (internally detuned saw stack)

### Filters
`FilterType`:
- `LowPass`
- `HighPass`
- `BandPass`

Use stateful `FilterState::process(...)` sample-by-sample.

### Envelope
`Envelope` (ADSR + `value`) supports `note_on`, `note_off`, and `update(dt)`.

### Effects
- `DelayFx` (fixed delay buffer)
- `ChorusFx` (modulated delay with fixed buffer)
- soft clip via `saturate_soft`

### Instrument construction
`Instrument` bundles:
- oscillator
- optional filter
- envelope
- `effect_flags`
- gain/cutoff/resonance/drive

`InstrumentVoice` contains mutable runtime state and `sample(freq, sample_rate)`.

## Style profile system

Use `style_profile` module for deterministic genre scaffolding.

### Core types
- `StyleProfile`
- `ScaleType`
- `InstrumentPreset`
- `DrumPatternType`

### Deterministic style flow
1. `choose_style(seed)` -> genre profile
2. `choose_style_selection(seed)` -> profile + bpm + scale
3. `styled_audio_config(seed)` -> sequencer tracks and config

### Built-in style names
- Electronic
- Pop
- HipHop
- Rock
- RnB
- Jazz
- Classical
- Country
- Reggae
- World

## Sequencer/event linkage

In callback:
- sequencer step triggers voice note on/off and event bus pushes
- event types: kick/snare/hat + bass/pad/lead notes

In renderer:
- drain events once per frame
- map to beat-energy and visual systems
