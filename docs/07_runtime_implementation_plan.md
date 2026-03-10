# Aurex X Runtime Implementation Plan (Phase 0 → Phase 2)

This document defines a clean Rust architecture plan for implementing the Aurex runtime incrementally.

It is intentionally implementation-focused but code-free, and follows the project constraints from vision, architecture, graphics/audio, runtime API, Prime Pulse, and AI handoff docs.

---

## 1) Proposed Rust Crate Structure

Use a Cargo workspace to separate platform/runtime concerns from game-facing APIs.

```text
/aurex-x
  Cargo.toml                    # workspace root
  /crates
    /aurex_app                  # executable entrypoint(s): editor/dev shell, runtime host
    /aurex_conductor            # orchestration, frame scheduling, deterministic timeline
    /aurex_ecs                  # ECS facade/types, schedules, deterministic system runner
    /aurex_render               # renderer frontend + frame graph + device setup
    /aurex_shape_synth          # procedural geometry generation (SSU)
    /aurex_materials            # procedural material definitions/parameter evaluation
    /aurex_lighting             # stylized dynamic light model + GPU light buffers
    /aurex_particles            # CPU/GPU particle simulation + emitters
    /aurex_postfx               # bloom, trails, fog, CRT, color grade
    /aurex_audio_asu            # synth engine, pattern sequencer, audio event bus
    /aurex_resonance            # system/profile resonance state + rules
    /aurex_library              # library world model and AXG catalog logic
    /aurex_trophies             # trophy definitions, progression, unlock evaluation
    /aurex_axg                  # AXG package format, manifest parser, validation
    /aurex_runtime_api          # stable API exposed to games/packages
    /aurex_core                 # shared math, ids, clocks, config, errors, telemetry
```

### Crate layering rules

- `aurex_core` may be depended on by all crates.
- Domain crates (`shape_synth`, `materials`, `audio_asu`, etc.) cannot depend on app/executable crates.
- `aurex_conductor` orchestrates other subsystems via traits/interfaces, not concrete app logic.
- `aurex_runtime_api` depends on stable facade traits, not internal implementation details.

---

## 2) Engine Module Layout (inside crates)

### `aurex_conductor`

- `clock` (fixed timestep + frame interpolation)
- `schedule` (ordered stages)
- `budget` (CPU/GPU budget envelopes)
- `orchestrator` (subsystem update/render dispatch)
- `sync` (barriers between simulation/audio/render)

### `aurex_ecs`

- `entity`, `component`, `storage`
- `query`
- `system`
- `schedule` (deterministic ordering)
- `snapshot` (debug deterministic captures)

### `aurex_render`

- `device` (window/surface/device init)
- `camera` (3D camera over 2.5D world)
- `frame_graph` (render pass dependencies)
- `passes`:
  - `geometry_pass`
  - `material_pass` (or material integrated into geometry shading)
  - `lighting_pass`
  - `particle_pass`
  - `postfx_pass`
  - `composite_pass`
- `debug` (timings, overlays, capture hooks)

### `aurex_shape_synth`

- `primitives` (circle, polygon, ring, tube, grid, extrusion)
- `parametric` (L-system/fractal/noise generators)
- `tessellation` (deterministic mesh generation)
- `instancing` (batch instance streams)

### `aurex_materials`

- `material_types` (flat/neon/chrome/crystal/noise/plasma/wireframe)
- `param_eval` (time/audio/reactive inputs)
- `gpu_bindings` (material uniforms/storage packing)

### `aurex_lighting`

- `light_types` (ambient/point/spot/pulse)
- `light_culling` (limit around max active lights)
- `light_animation` (audio-reactive pulse controls)

### `aurex_postfx`

- `bloom`
- `motion_trails`
- `fog`
- `color_grade`
- `distortion`
- `crt`

### `aurex_audio_asu`

- `synth`
- `drums`
- `sequencer`
- `events` (beat/kick/snare/drop timeline)
- `sync_bridge` (audio → conductor event bridge)

---

## 3) Responsibilities by Subsystem

- **Conductor**: Own frame lifecycle and stage execution ordering across simulation, audio, render, and platform systems.
- **ECS Runtime**: Deterministic state simulation and component/system composition.
- **Shape Synth Unit**: Generate all renderable geometric forms procedurally.
- **Material System**: Apply stylized procedural shading from parameterized recipes.
- **Lighting Engine**: Maintain stylized dynamic lighting model with strict light budgets.
- **Particle Engine**: Spawn/simulate/render particles linked to gameplay and audio events.
- **PostFX**: Apply stylized image-space effects to achieve demoscene look.
- **ASU**: Produce procedural music/sfx and publish time-accurate event markers.
- **Resonance Engine**: Track persistent behavior signatures and expose influence parameters.
- **Library/Trophy Systems**: Provide console-level UX/progression independent of individual AXG content.

---

## 4) Core Data Structures

Define stable, serializable runtime-facing structs early to reduce churn.

### Global timing and determinism

- `Tick(u64)` – fixed simulation tick id
- `FrameIndex(u64)` – presented frame id
- `FixedDelta` – canonical dt (e.g., 1/120 sec)
- `DeterminismSeed(u64)` – shared seed root per run/session/zone

### ECS primitives

- `EntityId` (generational id)
- `ComponentMask`
- `Transform2p5D`:
  - `position: Vec3` (z used for layering/depth/camera relation)
  - `rotation_yaw_pitch_roll: Vec3`
  - `scale: Vec3`
- `Velocity`, `ShapeInstance`, `MaterialHandle`, `ParticleEmitter`, `AudioEmitter`, `Collider`

### Shape/material model

- `ShapeDescriptor`:
  - `primitive_type`
  - `topology_params`
  - `deformation_params`
  - `seed`
- `MaterialDescriptor`:
  - `style`
  - `base_palette`
  - `reactive_params` (audio/resonance bindings)

### Lighting/postfx model

- `LightDescriptor` (ambient/point/spot/pulse + intensity/color/radius)
- `PostFxStack`:
  - `bloom: BloomSettings`
  - `trails: TrailSettings`
  - `fog: FogSettings`
  - `distortion: DistortionSettings`
  - `crt: CrtSettings`

### Audio event model

- `AudioEventType = Beat | Kick | Snare | Drop | Custom(String)`
- `AudioEvent { tick, sub_tick_phase, event_type, strength }`
- `AudioReactiveBus` ring-buffer for frame-coherent reads by render/gameplay systems

### Platform model

- `ResonanceProfile { faction_weights, history_window, active_theme }`
- `TrophyState { unlocked, progress, timestamps }`
- `AxgManifest { id, version, logic_entry, procedural_params, pattern_sets }`

---

## 5) Rendering Pipeline Stages (First Target + Growth Path)

For the first prototype, focus on minimal but correct sequencing.

### Stage A: Scene gather (CPU)

- Query ECS visible entities.
- Resolve `ShapeDescriptor + MaterialDescriptor + Transform` into draw packets.
- Build camera matrices from 3D camera controller.

### Stage B: Procedural geometry build

- Generate/refresh mesh buffers for active procedural shapes.
- Prefer cached deterministic mesh keys (`shape_hash`) to avoid regeneration when unchanged.

### Stage C: Material & lighting shading

- Run forward+stylized shading pass (initially forward, migrate to clustered/forward+ if needed).
- Upload bounded light set (target max ~12 active stylized lights).

### Stage D: Particles

- Simulate particle emitters (CPU in milestone 1).
- Render additive/alpha particle pass with depth-aware blending.

### Stage E: Post-processing

- Bright-pass + separable bloom.
- Optional simple trail accumulation buffer.
- Final tone/style composite (including CRT toggles).

### Stage F: Present

- Submit final color target to swapchain.

---

## 6) ECS Structure and Deterministic Scheduling

Use stage-based fixed update, with optional render interpolation.

### Deterministic schedule order (simulation tick)

1. Input capture (converted to deterministic command buffer)
2. Gameplay logic systems
3. Shape/material parameter systems
4. Audio sequencing update (authoritative tick alignment)
5. Audio event extraction (beat/kick/snare/drop)
6. Lighting/particle reactive systems
7. Physics-lite/collision (if enabled)
8. State commit + event log

### Render frame order

1. Read interpolated snapshot from ECS state
2. Render pipeline A→F
3. Publish profiling/budget telemetry

### Determinism safeguards

- Fixed timestep simulation only.
- Seeded random streams per system (`SystemRngKey`).
- No wall-clock reads inside gameplay systems.
- Stable iteration order for entity queries.

---

## 7) Conductor Orchestration Model

The Conductor is the runtime kernel coordinating all subsystems under explicit contracts.

### Conductor responsibilities

- Create subsystem graph at startup from config/profile.
- Own shared clocks (`sim_tick`, `audio_tick`, `render_frame`).
- Execute each stage in canonical order.
- Enforce performance budgets and degrade gracefully when exceeded.
- Publish runtime diagnostics.

### Proposed conductor stage graph

- `Boot`
- `LoadProfile + Resonance`
- `LoadAXG/PrimePulse`
- Main loop:
  - `PreTick`
  - `SimTick`
  - `AudioTick`
  - `RenderPrepare`
  - `Render`
  - `Present`
  - `PostFrame`

### Budget/degradation policy (early)

When over budget:

1. Reduce particle count ceiling.
2. Reduce bloom iterations/resolution.
3. Clamp dynamic light influence radius/count.
4. Last resort: lower internal render scale.

This keeps style continuity while preserving responsiveness.

---

## 8) First Milestone Implementation Steps (Minimal Renderer Prototype)

Target outcome: window + 3D camera + procedural shapes + dynamic lighting + bloom.

### Milestone M0 — Workspace foundation

- Convert repository to workspace structure with core crates:
  - `aurex_app`, `aurex_core`, `aurex_conductor`, `aurex_ecs`, `aurex_render`, `aurex_shape_synth`, `aurex_lighting`, `aurex_postfx`.
- Define shared type crates first (`aurex_core`) to avoid cyclic dependencies.
- Add architecture decision record for chosen graphics backend and ECS approach.

### Milestone M1 — Boot + window + camera

- Create runtime host executable.
- Initialize window/device/swapchain.
- Implement free-fly cinematic 3D camera with constrained defaults for 2.5D scenes.
- Draw debug grid/axes for spatial orientation.

### Milestone M2 — Procedural shape rendering

- Implement SSU primitives: circle, polygon, ring, tube (minimum set).
- Add deterministic tessellation with cached mesh keys.
- ECS integration: entities with `Transform2p5D + ShapeInstance` render correctly.

### Milestone M3 — Basic dynamic lighting

- Add ambient + point + pulse lights.
- Add per-frame light buffer upload and stylized shading parameters.
- Audio-reactive placeholder hook (synthetic pulse signal if ASU not integrated yet).

### Milestone M4 — Bloom post-processing

- Add bloom extraction + blur + composite.
- Expose tunable parameters via runtime config.
- Validate performance envelopes on reference hardware.

### Milestone M5 — Prototype validation & lock

- Determinism checks: same seed produces same geometry/animation paths.
- Frame capture and visual approval pass.
- Freeze prototype API surface for next phase (particles + ASU integration).

---

## Recommended Documentation Follow-ups

To keep documentation-driven development intact, update these docs as milestones progress:

- `docs/02_architecture.md`: add workspace/crate dependency diagram and conductor stage graph.
- `docs/03_graphics_audio.md`: add concrete first-pass renderer details (camera, light model, bloom constraints).
- `docs/04_runtime_api.md`: add minimal stable structs (`ShapeDescriptor`, `MaterialDescriptor`, `PostFxStack`, camera control API).
- `docs/06_dev_log.md`: log each milestone completion (M0–M5) and performance notes.

