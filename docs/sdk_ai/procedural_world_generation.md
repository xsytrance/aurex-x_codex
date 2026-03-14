# AI Authoring: Procedural World Generation

Use generator families and rendering systems to build rich deterministic worlds.

## Planning-to-render model
- `WorldBlueprint`: high-level world descriptor.
- `GeneratorStack`: deterministic parameter/structure generator.
- Renderer pipeline: execution path.

A practical bridge is:
`ExperiencePlanner -> RenderTheme + WorldBlueprint -> GeneratorStack -> Renderer`.

## Prime generators
- `ElectronicCity`
- `JazzImprovisation`
- `RockAmpMountain`
- `PopStageWorld`
- `ReggaeIsland`

## Rendering systems to combine
- Cone marching (faster empty-space traversal)
- LOD (reduced far-detail cost)
- Volumetric atmosphere (fog/shafts/cloud density)
- Particle overlays (audio-reactive motion accents)
- Fractal recursion (KIFS-style complexity)

## Authoring approach
1. Prefer `generator_stack` for hybrid worlds.
2. Start with `BaseGenerator`, then add `StructureLayer` and `DetailLayer`.
3. Add `ParticleLayer` for animated accents.
4. Add `RhythmModulationLayer` when music-reactive geometry is desired.
5. Add domain repetition/folding for macro scale.
6. Layer temporal feedback for motion memory.
7. Tune diagnostics-guided performance (LOD + step reduction).

## Generator stack schema
`"generator_stack": { "layers": [ ... ] }`

Layer variants:
- `BaseGenerator`
- `StructureLayer`
- `DetailLayer`
- `ParticleLayer`
- `RhythmModulationLayer`

Each layer wraps an existing `SceneGenerator`, so older generator definitions remain compatible.

## RhythmField scope note
Current stack output remains deterministic parameter generation only. RhythmField is used as modulation input for generator behavior and renderer configuration, while renderer stage order stays fixed.
