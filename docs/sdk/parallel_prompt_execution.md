# Parallel Prompt Execution Plan (RhythmField / Pulse Workstream)

This document operationalizes the parallel execution strategy so multi-agent or multi-branch work can proceed safely without breaking deterministic RhythmField runtime behavior.

## Scope
Applies to the following work areas:
- `crates/aurex_music`
- `crates/aurex_pulse`
- `docs/sdk`, `docs/sdk_ai`, `docs/sdk_human`

## Phase 0: Contract freeze (serial)
Before splitting work, freeze these interfaces:

1. `RhythmField` contract fields:
   - `tempo`
   - `beat_phase`
   - `beat_strength`
   - `beat_index`
   - `bar_index`
   - `phrase_index`
   - `bass_energy`
   - `harmonic_energy`
   - `spectral_flux`
   - `groove_vector`

2. Pulse diagnostics minimum rhythm summary:
   - `beat_phase`
   - `bar_index`
   - `bass_energy`

3. Minimal runtime integration constraint:
   - keep renderer stage pipeline unchanged
   - only light-touch modulation through pulse runtime context

## Phase 1: Parallel tracks

### Track A (core rhythm engine)
Owner scope:
- `crates/aurex_music/src/rhythm_field.rs`
- `crates/aurex_music/src/sequencer.rs`
- `crates/aurex_music/src/lib.rs` tests

Acceptance checks:
- `cargo test -p aurex_music`

### Track B (pulse runtime integration)
Owner scope:
- `crates/aurex_pulse/src/runner.rs`
- `crates/aurex_pulse/src/diagnostics.rs`
- `crates/aurex_pulse/examples/run_pulse.rs`
- `crates/aurex_pulse/src/lib.rs` tests

Acceptance checks:
- `cargo test -p aurex_pulse`

### Track C (documentation)
Owner scope:
- `docs/sdk/*`
- `docs/sdk_ai/*`
- `docs/sdk_human/*`

Acceptance checks:
- docs consistency with runtime behavior and required diagnostics fields

## Phase 2: Merge order
1. Merge Track A first (provider API).
2. Rebase Track B on Track A and merge.
3. Merge Track C last (or after A and before final gate if no conflicts).

## Phase 3: Integration gate (serial)
Run:
1. `cargo fmt --all -- --check`
2. `cargo test`
3. `cargo run -p aurex_pulse --example run_pulse examples/pulses/infinite_circuit_megacity.pulse.json`

## Automated helper
Use:
- `scripts/run_parallel_rhythm_checks.sh`

This script executes Track A + B tests in parallel and then runs the serial integration gate.
