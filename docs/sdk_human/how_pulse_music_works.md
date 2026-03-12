# How Pulse Music Works

Pulse music in Aurex is built from tempo and repeating patterns, not pre-recorded audio files.

- A **tempo clock** advances beats and bars.
- **Patterns** trigger notes and modulation events on specific steps.
- **Tracks** layer instruments (bass, lead, percussion, ambient textures).
- A **rhythm field** exposes beat and energy signals that visuals can use (`tempo`, beat/bar/phrase counters, spectral flux, groove vector).

In the default Pulse runtime wiring, rhythm can gently brighten scene ambient lighting on strong beats.

This keeps Pulse music procedural, reactive, and deterministic.
