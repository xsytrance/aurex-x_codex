# How Pulse Music Works

Pulse music in Aurex is built from tempo and repeating patterns, not pre-recorded audio files.

- A **tempo clock** advances beats and bars.
- **Patterns** trigger notes and modulation events on specific steps.
- **Tracks** layer instruments (bass, lead, percussion, ambient textures).
- A **rhythm field** exposes beat and energy signals that visuals can use (`tempo`, beat/bar/phrase counters, spectral flux, groove vector).

In the default Pulse runtime wiring, rhythm can gently brighten scene ambient lighting on strong beats.

This keeps Pulse music procedural, reactive, and deterministic.


## RhythmField snapshot and world modulation
The music sequencer produces a deterministic RhythmField snapshot (beat/bar phase + energy signals).
That snapshot modulates existing world parameters (like lighting pulse, atmosphere drift, and particle intensity) without replacing the base world setup.

This means music adds motion and expression while the underlying world identity stays recognizable.
