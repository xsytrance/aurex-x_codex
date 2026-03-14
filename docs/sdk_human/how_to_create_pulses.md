# How to Create Pulses

Aurex pulses can now be assembled with a simple builder API.

## Steps
1. Pick a pulse name and visual theme.
2. Set world hints (structure density, lighting style, particles, camera style).
3. Set rhythm reactivity (how strongly music affects world parameters).
4. Build the pulse config.

The builder uses existing systems only:
- world blueprint identity
- generator stack parameters
- deterministic RhythmField modulation
- unchanged renderer pipeline

## Why this helps
This keeps pulse authoring consistent for:
- manual developer pulses
- future APL (Aurex Pulse Language)
- future web pulse creator tools

and still preserves deterministic behavior for the same seed and configuration.
