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


## Deterministic Song Planner

`crates/aurex_audio/src/song_planner.rs` provides an offline (non-callback) song blueprint generator.

- `SongSection`: `Intro`, `Verse`, `Chorus`, `Bridge`, `Breakdown`, `Drop`, `Outro`
- `SongStructure { sections }`
- `ChordProgression { chords }` using scale-degree chords (`I`, `ii`, `iii`, `IV`, `V`, `vi`, `vii°`)
- `SongPlan { title, bpm, scale, structure, chords, style }`

`generate_song_plan(seed)` deterministically selects:

1. style profile
2. BPM in style range
3. scale option
4. structure template
5. chord progression template
6. title from deterministic word pools

The planner runs outside the realtime audio callback and does not alter CPAL stream architecture.


## Lyric engine and synchronized timeline

`crates/aurex_audio/src/lyric_engine.rs` adds deterministic lyric planning outside realtime audio:

- `generate_lyrics(seed, style)` builds style-conditioned lyric lines from word banks.
- `build_lyric_timeline(lyrics, bpm)` converts words to beat-aligned syllables.

Generated timelines are intended for UI/renderer consumption and are not executed in the CPAL callback.

## Procedural typography renderer bridge

`crates/aurex_render/src/typography.rs` defines deterministic typography styles and lyric render events:

- `GlyphStyle` and `TypographyStyle`
- `choose_typography_style(seed)`
- `LyricRenderEvent` + `TimedLyricRenderEvent`

`MockRenderer` can ingest lyric timelines and expose currently active lyric text while applying BeatEnergy/music-event-driven typography reactions.


## Experience planner (30–90s procedural AV blueprint)

`crates/aurex_app/src/experience_planner.rs` adds deterministic high-level planning outside realtime threads.

- `ExperiencePlan { title, duration_seconds, song_plan, typography_style, visual_theme }`
- `VisualTheme`: `Reactor`, `Cathedral`, `DesertMonolith`, `StormField`, `NeonCity`
- `generate_experience_title(seed)` uses deterministic word pools
- `generate_experience(seed)` deterministically selects:
  - title
  - duration in `[30.0, 90.0]`
  - song plan (`SongPlan`)
  - typography style (`TypographyStyle`)
  - visual theme

No CPAL stream/callback architecture changes are involved.
