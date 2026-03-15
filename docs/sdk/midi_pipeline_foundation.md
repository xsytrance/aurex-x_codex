# MIDI Pipeline Foundation (Phase 1)

Phase 1 introduces `aurex_midi` as an isolated, deterministic MIDI foundation module.

## Scope (Phase 1 only)

- deterministic MIDI data structures (`MidiFile`, `MidiTrack`, `MidiNote`)
- JSON-backed fixture parsing for development and test harnesses
- deterministic normalization/sorting of note events

## Out of scope (deferred)

- renderer integration
- automatic MIDI → scene generation
- automatic MIDI → Pulse generation

## Determinism guarantees

Given identical MIDI JSON input, normalization order and derived aggregate counts remain stable.
