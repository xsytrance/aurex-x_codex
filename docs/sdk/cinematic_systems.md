# Cinematic Systems SDK (Technical)

## Camera System (`aurex_scene::camera`)
- Typed rigs: `Orbit`, `Flythrough`, `TargetTracking`, `BezierPath`, `Rhythm`.
- Deterministic sampling via `CameraRig::sample(base, time, duration, sync)`.
- Rhythm integration through `CameraSyncInput` (`beat`, `phrase`, `tempo`).

## Camera Director (`aurex_scene::director`)
- `CameraDirector` produces deterministic `ShotSequence`.
- Heuristics use scene scale, primitive structure (tunnel/particle signatures), and pattern density.
- Intended to keep procedural detail legible in dense scenes.

## Renderer Post Pipeline (`aurex_render_sdf::post`)
Stage order:
1. exposure adaptation
2. ACES tonemap
3. bloom hint/adaptation
4. vignette/chromatic stylization
5. deterministic grain
6. gamma output

## Volumetric Lighting
`SdfLighting.volumetric` parameters:
- `scattering_steps`
- `beam_falloff`
- `beam_density`
- `shaft_intensity`

Used by deterministic fixed-step scattering in `aurex_render_sdf`.

## Parallel Architecture Preparation
Renderer is now represented with explicit stage constants:
- Geometry
- MaterialShading
- PatternSampling
- PostProcessing

This keeps boundaries clear for future rayon task batching / GPU mapping.
