# Aurex Audio Engine (SDK)

Aurex uses a deterministic CPAL callback with persistent synth state. The stream architecture is unchanged: audio callback writes samples and event queue entries, while render-side code drains events.

## Module toolkit

`crates/aurex_audio/src/runtime_toolkit.rs` provides reusable building blocks:

- Oscillators: `Sine`, `Triangle`, `Saw`, `Square`, `Noise`, `Supersaw`
- Filters: `LowPass`, `HighPass`, `BandPass` with sample-by-sample state (`FilterState`)
- Envelope: ADSR `Envelope` (`attack/decay/sustain/release/value`)
- Effects:
  - `DelayFx` (fixed-size feedback delay line)
  - `ChorusFx` (modulated fixed-size delay)
  - soft saturation (`saturate_soft`)

All modules run allocation-free per sample.

## Instrument graph

`Instrument` defines a lightweight chain:

- oscillator type
- optional filter type
- envelope
- effect flags (`FX_DELAY`, `FX_CHORUS`, `FX_SATURATION`)
- gain/cutoff/resonance/drive

`InstrumentVoice` owns persistent runtime state (oscillator phase, supersaw phases, envelope state, filter state, delay/chorus buffers). Voices are triggered by sequencer steps and sampled per frame inside the CPAL callback.

## Built-in instruments

- `TranceBass`
- `SupersawPad`
- `AnalogLead`
- `NoiseHat`
- `KickDrum`

Runtime bass/pad/lead/hat/kick voices in `write_boot_data` use these definitions.

## Event coupling

Audio callback emits:

- `Kick`, `Snare`, `Hat`
- `BassNote(u8)`, `PadNote(u8)`, `LeadNote(u8)`

Renderer drains these once per frame and updates beat/visual energy systems deterministically.
