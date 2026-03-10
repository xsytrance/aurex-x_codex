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

## Recommended next ADRs

- ADR-0005: Rendering backend selection and portability constraints.
- ADR-0006: ECS implementation strategy (build vs adopt).
- ADR-0007: `.axg` format versioning and compatibility policy.
- ADR-0008: Audio clock authority and A/V sync boundary.
