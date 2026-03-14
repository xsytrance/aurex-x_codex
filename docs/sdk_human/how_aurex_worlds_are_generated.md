# How Aurex Worlds Are Generated

Aurex builds large worlds from reusable procedural rules.

High-level flow:
1. **WorldBlueprint** describes the world at a high level (style, structure intent, and behavior targets).
2. **GeneratorStack** deterministically expands that blueprint into concrete scene parameters.
3. **Renderer pipeline** executes the generated scene using fixed stages.

What each layer does:
- **WorldBlueprint** = high-level descriptor.
- **GeneratorStack** = deterministic parameter generator.
- **Renderer** = execution pipeline.

World shaping systems include:
- **Cone marching + LOD** for scalable distance cost.
- **Volumetrics** for atmosphere and depth.
- **Particles** for motion accents.
- **Fractal recursion** for dense detail.
- **Prime generators** for musical world personality.

Generator stacks let multiple generators layer into one world (base form, structures, details, particles, rhythm-reactive overlays).

Example stacks include:
- `ElectronicCityStack`
- `JazzImprovisationStack`
- `RockMountainStack`

This keeps worlds expressive and repeatable without giant hand-authored maps.
