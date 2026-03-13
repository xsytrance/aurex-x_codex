# Procedural World System (Technical SDK)

The procedural world path is deterministic and layered, while preserving the existing renderer execution model.

## Data flow
1. `WorldBlueprint` defines high-level world intent.
2. `GeneratorStack` expands that intent into deterministic scene parameters and SDF node groups.
3. The renderer consumes the generated scene data through the fixed pipeline stages.

`GeneratorStack` is currently a **parameter/structure generation layer**. It does not execute rendering.

## GeneratorStack architecture
Worlds can be authored as a `generator_stack` (sequential layer execution) instead of a single generator:

- `BaseGenerator`
- `StructureLayer`
- `DetailLayer`
- `ParticleLayer`
- `RhythmModulationLayer`

Each layer implements the `SceneGeneratorLayer` trait, runs in stack order, and contributes to a final grouped SDF node.

## Rhythm modulation (current + forward path)
Rhythm-aware layers consume runtime modulation context (`runtime_context.rhythm_field`) to drive deterministic generator parameters.

Current behavior:
- RhythmField influences generated parameters/geometry shaping.
- Renderer stages remain unchanged.

Forward path:
- Future RhythmField expansion should continue as upstream modulation of generator and render configuration inputs, not as a new renderer architecture layer.

## Performance diagnostics
Frame diagnostics include:
- LOD activation counts
- average cone-march step reduction
- stage timings including `Particles`, `PostProcessing`, and `TemporalFeedback`

## Recursive micro/macro guidance
Use nested repetition + generator layering to compose multi-scale worlds (macro structures + micro motifs) while preserving determinism.

## RhythmField modulation pass
RhythmField is the deterministic music-to-world modulation layer between generator output and renderer execution:

Music Sequencer -> RhythmField -> Modulation Pass -> GeneratorStackOutput (modulated) -> Renderer Pipeline

The modulation pass applies bounded deltas to generator output parameters (terrain, structures, atmosphere, lighting, particles, camera hints). Base world identity remains intact.

