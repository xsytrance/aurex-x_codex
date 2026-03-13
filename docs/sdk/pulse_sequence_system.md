# Pulse Sequence System (Technical SDK)

`PulseSequence` adds deterministic, phase-level pulse evolution on top of the existing architecture.

## Scope
Current system exposes **phase-level authoring** only.
Design intentionally leaves room for future track-based sequencing.

## Location
`crates/aurex_app/src/pulse_sequence/`
- `phase.rs`
- `sequence.rs`

## Core types
- `PulsePhase`
  - `name`
  - `duration_seconds`
  - optional overrides (`lighting`, `atmosphere`, `particles`, `camera`, `rhythm intensity`, `structures`)
- `PulseSequence`
  - `phases: Vec<PulsePhase>`

## Deterministic sampling
`phase_at_time(time)` deterministically resolves the active phase.
- negative time clamps to first phase window
- overflow clamps to final phase
- same sequence + same time -> same phase

## Integration with PulseBuilder
`PulseBuilder::sequence(sequence)` optionally applies phase overrides before base world parameter generation and RhythmField modulation.
If no sequence is set, builder behavior is unchanged.

## Architecture placement
Pulse Runtime
→ PulseBuilder
→ WorldBlueprint
→ GeneratorStack
→ RhythmField modulation
→ Renderer Pipeline

Renderer stages remain unchanged.
