# Pulse Sequences

Pulse Sequences let one pulse evolve through named story-like phases.

Example phase flow:
1. Silence
2. Aurielle Appears
3. Maestros Reveal
4. Logo Formation
5. Menu Transition

Each phase has a duration and can apply gentle style overrides (lighting mood, atmosphere density, camera feel, rhythm intensity).

This keeps the same base world identity while letting the experience progress over time in a deterministic way.

At runtime, elapsed time now advances the active phase automatically.
When a phase boundary is crossed, the app logs a clear phase transition message.

Try:
- `cargo run -p aurex_app -- aurielle_intro`

You will see launch diagnostics followed by `Phase Change: ...` logs as the intro evolves.

Current authoring is phase-based. Future versions can build track-style sequencing on top of this model.


The intro runtime also uses a deterministic `TimelineClock` + event scheduler so scene transitions and audio cues can be synchronized instead of hard cuts.
