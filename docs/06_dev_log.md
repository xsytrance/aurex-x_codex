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
