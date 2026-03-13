# Aurex Audio Overview (Human)

Aurex audio is fully procedural: no sample packs are required for core runtime sound.

## How sound is made

The realtime engine combines small reusable synth modules:

- Oscillators (sine, triangle, saw, square, noise, supersaw)
- Filters (low/high/band-pass)
- ADSR envelopes
- Lightweight effects (delay, chorus, soft saturation)

These modules are combined into instruments like bass, pad, lead, hi-hat, and kick.

## Why this design

- deterministic behavior (same input -> same output)
- low CPU overhead
- no heap allocations in the audio callback
- easy to build new instruments by recombining modules

## Visual sync

When beats/notes trigger, audio emits compact events. The renderer drains events once per frame and uses beat energy + event reactions to drive glow, ring scale, particles, and camera behavior.
