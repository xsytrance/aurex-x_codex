# How Aurex Stays Fast

Aurex-X keeps visual richness by combining smarter math with deterministic controls:
- adaptive raymarch steps (bigger in empty space, finer near surfaces)
- stage-based renderer flow
- small deterministic caches during frame evaluation
- optional diagnostics to understand scene cost

This keeps demos scalable while preserving the procedural aesthetic.
