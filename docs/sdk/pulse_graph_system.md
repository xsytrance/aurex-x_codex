# Pulse Graph System (Technical SDK)

`PulseGraph` allows Aurex-X experiences to connect multiple pulse packages into a deterministic directed flow.

## Core types
Implemented in `crates/aurex_pulse/src/pulse_graph.rs`:

- `PulseGraph`
- `PulseNode`
- `PulseTransition`
- `PulseTransitionKind`
- `PulseGraphRunner`

## Transition kinds
- `Manual { trigger }`
- `Timeline { after_seconds }`
- `MusicalCue { cue }`
- `GeneratorTrigger { event }`

`MusicalCue` reads `RhythmField` values exposed by the active `PulseRunner`.

## Runner model
`PulseGraphRunner` wraps `PulseRunner` and orchestrates node switching:

1. load graph and entry node
2. initialize active pulse runner
3. update active runner
4. evaluate transitions in deterministic list order
5. shutdown old runner and initialize next runner when a transition fires

The renderer stage pipeline remains unchanged; PulseGraph only coordinates pulse-level lifecycle.

## Determinism
Given fixed graph seed, pulse definitions, and timestep progression, transition evaluation is deterministic.

## Example
`examples/pulse_graphs/electronic_journey.graph.json`
