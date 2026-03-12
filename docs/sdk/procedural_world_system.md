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
