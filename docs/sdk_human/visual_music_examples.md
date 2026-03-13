# Visual Music Examples

Aurex now includes three small visual-music showcase pulses:

- **Electronic Megacity**: strong neon lighting pulses and active particles.
- **Jazz Atmosphere**: warm lounge mood with smoother atmospheric drift.
- **Ambient Dreamscape**: slow, fog-rich, minimal-structure world motion.

How they work:
1. The pulse defines a world identity (WorldBlueprint).
2. Generator parameters define base terrain/structure/atmosphere/lighting/particles/camera hints.
3. RhythmField music signals gently modulate those parameters.
4. The renderer draws the result through the same fixed pipeline.

These examples are deterministic: same pulse + same seed gives the same initial world setup.
