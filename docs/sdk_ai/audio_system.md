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


## Vocal synthesis system

Use `aurex_audio::vocal_engine` for deterministic procedural vocals.

### Core pieces
- `VocalType`: `Chant`, `ChoirPad`, `RnbSynth`, `Robot`, `Scat`
- phoneme sets:
  - `CHANT_PHONEMES = ["AH", "OH", "YA", "NA", "HE"]`
  - `SCAT_PHONEMES = ["BA", "DA", "DOO", "BEE", "SKA"]`
- `generate_phrase(seed, phonemes)` -> deterministic `Phrase`
- `Formant { frequency, bandwidth }` + vowel presets (AH/OO/EE/OH)
- `VocalVoice` sample path: oscillator -> formant filter -> envelope -> optional effects

### Style integration
`StyleProfile` includes optional `vocal_type`; `styled_audio_config(seed)` maps this to a `VoiceSynthConfig` for genre-aligned vocals.


## Song planner system

Use `aurex_audio::song_planner` for deterministic full-song blueprints.

### Core types
- `SongSection`
- `SongStructure`
- `Chord` and `ChordProgression`
- `SongPlan`

### Generation
- `generate_chord_progression(seed, scale)` yields deterministic degree progressions.
- `generate_song_plan(seed)` yields deterministic title/style/BPM/scale/structure/chords.

### Style integration
`SongPlan.style` is a `StyleProfile`, so the resulting plan carries:
- bass/pad/lead instrument presets
- drum pattern type
- optional vocal type

This system is offline/planning-stage only and must not run allocations in the realtime callback path.


## Lyric and typography synchronization

### Lyric engine (`aurex_audio::lyric_engine`)
- `generate_lyrics(seed, style)` -> deterministic `Lyrics`
- `build_lyric_timeline(lyrics, bpm)` -> beat-scheduled `LyricTimeline`
- `LyricTimeline` stores `LyricSyllable { text, beat_time }`

### Typography (`aurex_render::typography`)
- `choose_typography_style(seed)` -> deterministic `TypographyStyle`
- timeline entries map to `TimedLyricRenderEvent { beat_time, event }`
- renderer activates lyric text when timeline beat is reached

### Music-reactive text mapping
- Kick => scale boost
- Snare => spark intensity
- Bass => glow boost
- Pad => ambient boost
- Lead => letter motion

All of this is deterministic and outside the CPAL callback path.


## Experience planner integration

Use `aurex_app::experience_planner` for top-level deterministic AV planning.

### Core types
- `ExperiencePlan`
- `VisualTheme`

### Deterministic API
- `generate_experience_title(seed)`
- `generate_experience(seed)`

`generate_experience(seed)` composes:
- `SongPlan` from `aurex_audio::song_planner`
- `TypographyStyle` from `aurex_render::typography`
- a seed-selected visual theme
- duration clamped to 30–90 seconds

This planner is intentionally outside realtime audio callback execution.


## Identity + creative direction layer

Aurex includes an app-level deterministic cohesion system:

- `aurex_app::identity_engine`
- `aurex_app::creative_director`

### Identity Engine outputs
`IdentityProfile` includes:
- `IdentityType`
- generated name
- `SymbolType`
- `ToneType`
- 3-color palette
- `StyleBias`

### Creative Director flow
`direct_experience(identity_seed, experience_seed)`:
1. generate identity
2. generate base experience
3. adjust experience by identity tone + style bias

Adjustments can include visual theme override, typography modulation, and style-family reseeding for song plan alignment.

### Resonance seed model
Use two deterministic seeds:
- identity seed -> artistic persona
- experience seed -> baseline AV plan

This split keeps variation controllable while preserving deterministic reproducibility.


## Procedural World Generator

Aurex now includes a deterministic world blueprint pass in `aurex_render::world_generator`.

- `generate_world_blueprint(seed, theme)` creates lightweight parameters (no mesh/assets).
- Blueprint fields: theme, structure set, geometry style, atmosphere, lighting mode, color palette, and camera rig.
- Theme drives structure + lighting + camera defaults, while seed drives style/atmosphere/palette choices.
- `MockRenderer` stores `world_blueprint: Option<WorldBlueprint>` and can emit a debug summary for diagnostics.
- Generation runs outside the realtime audio callback, preserving CPAL determinism and callback allocation constraints.
