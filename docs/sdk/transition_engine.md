# Transition Engine SDK (Technical)

## Modules
- `aurex_scene::transition`
- `aurex_scene::director_rules`
- `aurex_scene::effect_graph` morphing extensions

## Transition Core Types
- `TransitionEngine`
- `TransitionSpec`
- `TransitionState`
- `TransitionStyle`
- `TransitionContext`

Transition styles:
- PulseFlash
- PatternDissolve
- FractalZoom
- HarmonicSmear
- GeometryMelt
- TunnelSnap
- CathedralBloom
- RhythmStutter

All transitions are deterministic and seed-driven.

## Graph Morphing
`effect_graph` now includes:
- `GraphMorph`
- `GraphMorphSpec`
- `GraphMorphState`

Morph strategies:
- `NodeParameterBlend`
- `DistanceFieldBlend`
- `PatternCrossfade`
- `HarmonicPhaseBlend`
- `GeneratorMorph`

## Demo integration
`demo::Transition` supports:
- manual specs (`spec`)
- auto recommendations (`auto`) via `DirectorRuleSet`

Renderer transition path:
1. evaluate source scene block
2. evaluate target scene block
3. compute transition progress/context
4. blend via `TransitionEngine`
5. evaluate effect graph/automation/post

## Performance-related docs
- `docs/sdk/performance_architecture.md`
- `docs/sdk/renderer_stages.md`
- `docs/sdk/gpu_readiness.md`
