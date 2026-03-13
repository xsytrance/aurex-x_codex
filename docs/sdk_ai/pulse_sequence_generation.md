# AI Authoring: Pulse Sequence Generation

Use `PulseSequence` for deterministic phase-level evolution.

## Generation pattern
1. Build a base pulse with `PulseBuilder`.
2. Attach a `PulseSequence` with ordered phases.
3. Optionally add bounded phase overrides.

## Constraints
- Keep overrides bounded and style-oriented.
- Do not replace base pulse identity each phase.
- Keep deterministic behavior: same seed + same sequence + same time -> same active phase.

## Current capability
Phase-level authoring only.
Design is intentionally compatible with future track-based sequencing.
