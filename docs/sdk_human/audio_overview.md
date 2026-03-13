# Aurex Audio Overview (Human)

Aurex audio is fully procedural: no sample packs are required for core runtime sound.

## How sound is made

The realtime engine combines small reusable synth modules:

- Oscillators (sine, triangle, saw, square, noise, supersaw)
- Filters (low/high/band-pass)
- ADSR envelopes
- Lightweight effects (delay, chorus, soft saturation)

These modules are combined into instruments like bass, pad, lead, hi-hat, and kick.

## Genre-aware style profiles

Aurex now supports style profiles to drive song generation by genre.
Each style defines:

- tempo range
- candidate musical scales
- default bass/pad/lead instruments
- drum pattern type

Example built-in genres include Electronic, Pop, HipHop, Rock, RnB, Jazz, Classical, Country, Reggae, and World.

When a song starts, Aurex deterministically:

1. picks a style from seed
2. picks BPM within style tempo range
3. picks a scale from style options
4. builds tracks with matching instrument presets and drum pattern

## Why this design

- deterministic behavior (same seed -> same song foundation)
- low CPU overhead
- no heap allocations in the audio callback
- easy expansion: add styles without redesigning runtime audio I/O

## Visual sync

When beats/notes trigger, audio emits compact events. The renderer drains events once per frame and uses beat energy + event reactions to drive glow, ring scale, particles, and camera behavior.


## Procedural vocal styles

Aurex now includes a sample-free vocal engine with several styles:

- Chant
- Choir pad
- R&B synth vocal
- Robot vocal
- Jazz-style scat

Vocal phrases are generated deterministically from phoneme sets, then shaped by formant filters (vowel-like resonances) and synth envelopes/effects.

Style profiles can optionally assign a vocal type (for example Electronic→Robot, Pop/RnB→RnbSynth, Jazz→Scat, World→Chant), so genre choice also influences vocal character.


## Song planner (full blueprint generation)

Aurex can now generate a complete song plan from a single seed.
The plan includes:

- genre/style
- BPM
- scale
- section structure (intro/verse/chorus/etc.)
- chord progression
- procedural title

This planner is deterministic and runs outside the realtime audio thread. It maps directly to style profile choices (instruments, drum pattern, and optional vocal type), creating a genre-aware foundation before audio playback begins.


## Lyrics and on-screen typography

Aurex now has a deterministic lyric engine that writes style-aware lyric lines from seed-driven templates and word banks.
Those lyrics are converted into beat-aligned syllable timelines, then consumed by renderer typography logic.

Typography styles (Neon, Pulse, Crystal, Circuit, Rune) are also selected from seed and react musically:

- Kick: brief text scale bump
- Snare: spark-like letter bursts
- Bass: stronger glow
- Pad: ambient glow lift
- Lead: subtle letter motion

This lyric/typography path runs outside the realtime audio callback.


## Experience planner

Aurex can now create a complete short-form procedural experience (about 30–90 seconds) from a single seed.

The generated plan includes:

- a title
- an exact duration
- a full song blueprint (style/BPM/scale/structure/chords)
- a typography style for on-screen lyrics
- a visual theme for scene mood

Because generation is deterministic, the same seed always produces the same audiovisual plan.


## Identity Engine and Creative Director

Aurex now adds an identity layer above songs and visuals so generated experiences feel authored rather than random.

### Identity Engine

`identity_engine` deterministically generates an `IdentityProfile` from seed:

- identity type (solo artist, collective, mythic entity, AI construct, anonymous order)
- generated name
- symbol motif
- tone
- color palette
- genre bias

### Creative Director

`creative_director::direct_experience(identity_seed, experience_seed)` combines identity + experience plan and aligns them.

Examples of alignment:

- tone influences visual theme and typography intensity
- genre bias nudges song style family selection
- identity name is stamped into the final experience title

This all occurs in planning systems outside realtime audio callbacks.


## Procedural World Generator

Aurex now includes a deterministic world blueprint pass in `aurex_render::world_generator`.

- `generate_world_blueprint(seed, theme)` creates lightweight parameters (no mesh/assets).
- Blueprint fields: theme, structure set, geometry style, atmosphere, lighting mode, color palette, and camera rig.
- Theme drives structure + lighting + camera defaults, while seed drives style/atmosphere/palette choices.
- `MockRenderer` stores `world_blueprint: Option<WorldBlueprint>` and can emit a debug summary for diagnostics.
- Generation runs outside the realtime audio callback, preserving CPAL determinism and callback allocation constraints.
