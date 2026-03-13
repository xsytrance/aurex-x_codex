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

## Sequencer/event linkage

In callback:
- sequencer step triggers voice note on/off and event bus pushes
- event types: kick/snare/hat + bass/pad/lead notes

In renderer:
- drain events once per frame
- map to beat-energy and visual systems

## Built-in defaults
Use provided constructors as base presets:
- `Instrument::trance_bass()`
- `Instrument::supersaw_pad()`
- `Instrument::analog_lead()`
- `Instrument::noise_hat()`
- `Instrument::kick_drum()`
