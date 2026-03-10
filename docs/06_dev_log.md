# Aurex Development Log

Phase 1
Aurex concept defined as a procedural fantasy console.

Phase 2
Core architecture designed including ECS runtime and Conductor system.

Phase 3
Graphics stack defined:

procedural geometry
materials
lighting
particles
post processing

Phase 4
Audio synthesis system designed (ASU).

Phase 5
Prime resonance system introduced.

Phase 6
Prime Pulse pack-in experience designed.

Key features:

• first-person resonance exploration
• manifestation mechanic
• Prime faction zones
• lore fragment discovery
• BassLine subterranean realm
• pulse-driven environmental rhythm

Phase 7
Runtime implementation planning document added (docs/07_runtime_implementation_plan.md) to guide prototype milestones M0–M5.

Phase 8
Executed Milestone M0 (workspace foundation) by converting the repository into a multi-crate workspace and adding initial runtime scaffolding crates:

• aurex_app
• aurex_core
• aurex_conductor
• aurex_ecs
• aurex_render
• aurex_shape_synth
• aurex_lighting
• aurex_postfx

Phase 9
System architecture documentation expanded with:

• crate dependency graph and conductor stage contract (docs/02_architecture.md)
• runtime API v0 contract definitions (docs/04_runtime_api.md)
• initial ADR index and accepted architecture decisions (docs/08_architecture_decisions.md)

Phase 10
ADR-0005 accepted and documented:

• selected `wgpu` + `winit` as the M1 rendering backend baseline
• formalized backend isolation boundaries in architecture docs
• added an architecture acceptance gate for starting M1 implementation

Phase 11
ADR-0006 accepted and documented:

• selected Aurex-owned deterministic ECS authority path in `aurex_ecs`
• defined command-buffer mutation boundary recommendations for runtime API
• inserted M1.5 ECS hardening milestone before broader rendering feature expansion

Phase 12
Implemented M1.5 scaffolding in code:

• added deterministic ECS command buffer + sorted command application path in `aurex_ecs`
• added initial ECS invariants tests for ordering/reproducibility
• added conductor stage enum/constants in `aurex_conductor` aligned with architecture stage graph

Phase 13
Accelerated M1 bootstrap scaffolding:

• added `aurex_render` mock renderer bootstrap contracts (`RenderBootstrapConfig`, `RenderStage`, `RENDER_MAIN_STAGES`)
• added render frame stats tracking contract for stage execution verification
• integrated render stage/bootstrap diagnostics into `aurex_app` runtime output

Phase 14
Conductor/render integration scaffolding added:

• added `execute_frame` trace path in `aurex_conductor` for canonical stage execution verification
• added render backend mode/status contracts in `aurex_render` (`Mock`, `WgpuPlanned`)
• updated `aurex_app` diagnostics to confirm conductor-render stage handshake

Phase 15
Render backend transition contract scaffolding added:

• added explicit backend transition API in `aurex_render` (`transition_backend_mode`)
• added readiness state transitions for `Mock` -> `WgpuPlanned`
• expanded `aurex_app` runtime diagnostics to print backend transition lifecycle

Phase 16
First procedural boot animation prototype scaffolding added:

• added deterministic `BootAnimator` and `BootFrame` contracts in `aurex_render`
• added tests verifying same-seed determinism and cross-seed variation
• integrated boot animation frame generation diagnostics into `aurex_app` output

Phase 17
Boot timeline phase model added:

• added `BootPhase` and `BootTimeline` contracts for Ignition/PulseLock/Reveal sequencing
• added timeline phase-count verification test in `aurex_render`
• expanded `aurex_app` diagnostics to print boot phase distribution per generated sequence

Phase 18
Boot style profile layer added:

• added `BootStyleProfile` with phase-specific intensity/hue/distortion envelopes
• applied style envelopes into boot timeline generation (`styled_glow`, `styled_hue`, `distortion_weight`)
• expanded `aurex_app` diagnostics with averaged styled boot metrics
