# Pulse Runtime System (Technical SDK)

A **Pulse** is the primary executable Aurex-X experience unit (game, world, visual music, demo, or ambient runtime).

## Core types
- `PulseDefinition` (JSON schema)
- `PulseRunner` (lifecycle executor)

Implemented in `crates/aurex_pulse`.

## Lifecycle
1. `load`
2. `initialize`
3. `update`
4. `render`
5. `shutdown`

## Integration model
Pulse runtime **orchestrates existing systems** rather than replacing them:
- existing `SdfScene` scene graph
- existing generator/effect graph/automation/timeline/demo/camera stack
- existing renderer diagnostics

## Rendering pipeline
Pulse runtime invokes the existing renderer pipeline unchanged:
- ScenePreprocess
- EffectGraphEvaluation
- GeometrySdf
- MaterialPattern
- LightingAtmosphere
- Particles
- PostProcessing
- TemporalFeedback

## Diagnostics
Pulse-level diagnostics track lifecycle timing and frame counts, while preserving nested renderer diagnostics payloads.
