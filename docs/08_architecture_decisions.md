# Aurex Architecture Decisions (ADR Index)

This document tracks architecture decisions that should remain stable across milestones.

---

## ADR-0001: Workspace-First Runtime Decomposition

Status: Accepted (Phase M0)

Decision:

Adopt a multi-crate workspace where orchestration, ECS, rendering contracts, procedural shape contracts, lighting, and post-processing contracts are separated into dedicated crates.

Rationale:

- Prevent early monolith coupling.
- Enforce explicit dependency directions.
- Make subsystem ownership and evolution clearer.

Tradeoffs:

- More crate boilerplate early.
- Cross-crate refactors require coordination.

---

## ADR-0002: Deterministic Simulation is Authoritative

Status: Accepted

Decision:

Use fixed-timestep deterministic simulation as the source of truth. Rendering may interpolate snapshots but cannot alter simulation state.

Rationale:

- Reproducibility for procedural systems.
- Stable behavior for resonance and progression systems.
- Future replay/debug tooling compatibility.

Tradeoffs:

- Requires discipline around time and randomness.
- Increased complexity for interpolation/latency handling.

---

## ADR-0003: Contract-First Rendering Integration

Status: Accepted (M0→M1)

Decision:

Define and validate rendering data contracts (`CameraRig`, `ShapeDescriptor`, `LightDescriptor`, `BloomSettings`) before selecting deeply-coupled rendering internals.

Rationale:

- Keeps runtime API stable while backend details evolve.
- Supports backend changes without breaking gameplay interfaces.

Tradeoffs:

- Slower initial visual progress compared to immediate backend-centric coding.

---

## ADR-0004: Conductor-Owned Stage Graph

Status: Accepted

Decision:

Conductor owns canonical stage order (`PreTick`→`SimTick`→`AudioTick`→`RenderPrepare`→`Render`→`Present`→`PostFrame`) and budget/degradation policy.

Rationale:

- Clear place for synchronization and performance governance.
- Prevents hidden side-channel updates across subsystems.

Tradeoffs:

- Requires explicit stage contracts per subsystem.

---

## ADR-0005: Rendering Backend and Portability Constraints

Status: Accepted (M1 Baseline)

Decision:

Adopt a custom Aurex renderer in `aurex_render` built on `wgpu` + `winit` as the primary runtime backend.

Portability target tiers:

- Tier 1: Linux, Windows, macOS (desktop runtime and dev workflow)
- Tier 2: Web (WASM) after desktop parity for core pipeline stages

Backend constraints:

- Keep graphics API details isolated in `aurex_render::device` and `aurex_render::passes`.
- Keep conductor/ECS/runtime APIs backend-agnostic.
- No gameplay-side dependency on GPU/shader internals.

Rationale:

- `wgpu` provides cross-platform backends with a single Rust-facing API.
- Supports required pipeline control for procedural geometry, stylized lighting, and post-processing.
- Preserves the contract-first architecture by isolating render implementation details.

Tradeoffs:

- More low-level implementation effort versus engine-level rendering abstractions.
- Some backend-specific visual differences must be normalized in shader/pipeline validation.

Implementation guidance:

- M1: establish `window + device + swapchain + camera uniform` path.
- M2: add procedural shape geometry pass.
- M3: integrate stylized light buffer path.
- M4: add bloom chain with deterministic parameter handling.

---

## ADR-0006: ECS Implementation Strategy and Determinism Constraints

Status: Accepted (M1 Baseline)

Decision:

Implement an in-house deterministic ECS core in `aurex_ecs` rather than adopting a general-purpose external ECS framework for runtime authority paths.

Scope boundary:

- Deterministic authority path (`SimTick`) uses Aurex-owned ECS storage/query/schedule.
- Optional tooling/editor adapters may bridge to external ECS ecosystems later, but must not replace deterministic runtime authority.

Design constraints:

- Stable entity iteration ordering by archetype/chunk and insertion rules.
- Fixed-stage schedule execution owned by Conductor.
- No wall-clock mutation in systems; all time from `Tick`/`FixedDelta`.
- Randomness provided only via seeded system streams.
- Snapshot extraction for render interpolation is explicit and read-only.

Rationale:

- Determinism is a primary project constraint (replayability, resonance consistency, procedural reproducibility).
- Runtime-specific requirements (2.5D-focused components, audio-reactive passes, small package constraints) benefit from tailored data layouts.
- Avoids accidental nondeterminism introduced by features optimized for generic high-throughput game workloads.

Tradeoffs:

- Higher implementation burden compared with adopting a full ECS ecosystem.
- Fewer out-of-the-box debugging tools initially.
- Requires deliberate benchmarking and profiling investment.

Implementation guidance:

- M1: finalize authority data model (`EntityId`, component stores, deterministic query order contract).
- M2: add deterministic schedule registry and phase barriers.
- M3: add snapshot/export pipeline for render and replay instrumentation.
- M4: add invariants tests (ordering, seed reproducibility, fixed-step equivalence).

---

## Recommended next ADRs

- ADR-0007: `.axg` format versioning and compatibility policy.
- ADR-0008: Audio clock authority and A/V sync boundary.
