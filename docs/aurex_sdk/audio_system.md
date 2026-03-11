# Audio System Notes

This document summarizes the current Aurex audio model and the intended sync contract with Scene IR.

## Current engine capabilities

The current audio layer centers on procedural sound generation and sequencing concepts, including:

- synth voice generation
- pattern-based triggering/sequencing
- timing and transport suitable for reactive visuals

## Scene-facing sync signals

Scene IR reserves `sync` bindings for three high-level signals:

- `kick`
- `snare`
- `bass`

These are optional string links to scene targets (for example, `node_id.parameter_name`) and act as placeholders for future runtime binding.

## Future plan

Planned expansion includes:

1. Explicit voice/bus naming in SDK docs.
2. Deterministic signal extraction API for beat/energy bands.
3. Runtime mapping layer between audio analysis signals and node parameters.
4. Tooling support so editors/LLMs can suggest valid sync targets.

This document is descriptive and does not change current renderer/audio runtime behavior.
