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
