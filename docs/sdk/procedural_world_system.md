# Procedural World System (Technical SDK)

This phase adds parallel procedural richness systems while preserving deterministic rendering:

- Cone marching acceleration (`cone_march.rs`)
- Distance/Fractal LOD (`lod.rs`)
- Volumetric atmosphere (`volumetric.rs`)
- Procedural particles (`particles.rs`)
- KIFS fractal helpers (`fractals.rs`)
- Prime-specific generators (`aurex_scene::generators`)

## Performance diagnostics
Frame diagnostics now include:
- LOD activation counts
- average cone-march step reduction
- stage timings including Particles, PostProcessing, TemporalFeedback

## Recursive micro/macro guidance
Use nested repetition + generator layering to create multi-scale worlds (macro structures with micro detail motifs) while keeping deterministic behavior.

## GeneratorStack architecture
Worlds can now be authored as a `generator_stack` (sequential layer execution) instead of a single generator:

- `BaseGenerator`
- `StructureLayer`
- `DetailLayer`
- `ParticleLayer`
- `RhythmModulationLayer`

Each layer implements the common `SceneGeneratorLayer` trait. The stack expands in order and combines outputs into a final SDF group node.

Rhythm-aware layers can read runtime modulation context (`runtime_context.rhythm_field`) via scene runtime modulation plumbing to produce beat/bass/harmonic driven geometry changes without modifying renderer stage order.
