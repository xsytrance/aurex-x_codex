# How Aurex Stays Fast

Aurex-X keeps visual richness by combining smarter math with deterministic controls:
- adaptive raymarch steps (bigger in empty space, finer near surfaces)
- stage-based renderer flow
- deterministic caching during frame evaluation
- optional diagnostics to understand scene cost

## Spatial caching in plain terms
The renderer now keys caches from where rays actually sample the world, not just camera position. That means cache hit rates better reflect real scene reuse (for example repeated structures) instead of artificial same-key collisions.

## Effect graph reuse
If the same scene seed and time tick are rendered again, Aurex can reuse prior effect-graph evaluation state. This avoids re-running expensive graph logic unnecessarily while keeping deterministic outputs.

## Stage timing diagnostics
Diagnostics can report exactly where frame time goes:
- preprocess
- effect graph evaluation
- geometry
- material/pattern
- lighting/atmosphere
- post processing

This makes performance tuning clearer without changing scene behavior.
