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

## Style profiles (genre-aware song setup)

`crates/aurex_audio/src/style_profile.rs` adds deterministic style selection:

- `StyleProfile` fields:
  - name
  - tempo range (`tempo_min`..`tempo_max`)
  - scale options
  - bass/pad/lead instrument presets
  - drum pattern type
- `choose_style(seed)` chooses genre profile deterministically.
- `choose_style_selection(seed)` chooses style + BPM + scale deterministically.
- `styled_audio_config(seed)` builds a sequencer-ready `ProceduralAudioConfig` using selected profile.

Built-in style names:

- Electronic, Pop, HipHop, Rock, RnB, Jazz, Classical, Country, Reggae, World

## Event coupling

Audio callback emits:

- `Kick`, `Snare`, `Hat`
- `BassNote(u8)`, `PadNote(u8)`, `LeadNote(u8)`

Renderer drains these once per frame and updates beat/visual energy systems deterministically.


## Procedural vocal engine

`crates/aurex_audio/src/vocal_engine.rs` adds deterministic vocal synthesis without samples:

- vocal styles: `Chant`, `ChoirPad`, `RnbSynth`, `Robot`, `Scat`
- deterministic phrase generation from phoneme sets
- formant synthesis (`Formant`, vowel presets AH/OO/EE/OH)
- `VocalVoice` combines oscillator + formant filters + envelope + optional effects

Style profiles can attach a `vocal_type` so generated songs include genre-matched procedural vocals.
