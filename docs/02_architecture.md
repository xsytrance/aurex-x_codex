# Aurex X System Architecture

Aurex is orchestrated by a central system called the Conductor.

The Conductor manages:

• frame scheduling
• subsystem synchronization
• performance budgets
• resonance tracking

Core runtime systems:

Conductor
├ ECS Runtime
├ Shape Synth Unit (SSU)
├ Material System
├ Particle Engine
├ Lighting Engine
├ Post Processing Pipeline
├ Aurex Sound Unit (ASU)
├ Resonance Engine
├ Library System
└ Trophy System

The ECS Runtime drives gameplay logic.

Entities consist of components such as:

transform
velocity
shape
material
particle emitter
audio emitter
collider

Systems operate on these components deterministically.

Rendering Pipeline:

Procedural Geometry
↓
Material Shading
↓
Dynamic Lighting
↓
Particles
↓
Post Processing
↓
Final Frame

All rendering is procedural and asset-light.

Aurex uses a 2.5D world model:

• gameplay occurs on a primary plane
• camera moves in full 3D space

This allows cinematic visual effects while keeping gameplay simple.

---

## Workspace and Crate Dependency Architecture (Phase M0 Baseline)

Current workspace members:

- `aurex_app` (runtime host executable)
- `aurex_core` (shared deterministic types)
- `aurex_conductor` (frame/tick orchestration)
- `aurex_ecs` (entity/component model scaffolding)
- `aurex_render` (camera/render contracts)
- `aurex_shape_synth` (procedural shape contracts)
- `aurex_lighting` (light contracts)
- `aurex_postfx` (postfx contracts)

Dependency direction (must remain acyclic):

```text
aurex_app
 ├── aurex_conductor
 │    └── aurex_core
 ├── aurex_ecs
 │    └── aurex_core
 ├── aurex_render
 │    └── aurex_core
 ├── aurex_shape_synth
 ├── aurex_lighting
 └── aurex_postfx
```

Architecture rule:

- `aurex_app` depends on runtime crates.
- Runtime crates must not depend on `aurex_app`.
- Cross-domain dependencies (e.g., render ↔ audio) should be mediated by conductor/runtime APIs rather than direct coupling.

---

## Conductor Stage Graph (Execution Contract)

Canonical main loop stages:

1. `PreTick`
2. `SimTick`
3. `AudioTick`
4. `RenderPrepare`
5. `Render`
6. `Present`
7. `PostFrame`

Stage ownership:

- `PreTick`: input capture, command buffering, profile/runtime flags.
- `SimTick`: ECS deterministic state updates.
- `AudioTick`: ASU sequencing/event extraction.
- `RenderPrepare`: gather visible state snapshots and build draw packets.
- `Render`: execute render pipeline stages.
- `Present`: swapchain/frame output.
- `PostFrame`: telemetry, budgets, degradation decisions.

Deterministic constraints:

- Fixed timestep simulation (`FixedDelta`).
- Stable query iteration order in ECS systems.
- Seeded random streams scoped by system and tick.
- Frame-to-frame behavior reproducibility from identical input and seed.
