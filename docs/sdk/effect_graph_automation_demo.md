# Effect Graph + Automation + Demo Sequencer (Technical SDK)

## `aurex_scene::effect_graph`
- Core types: `EffectGraph`, `EffectNode`, `EffectNodeId`, `EffectConnection`, `EffectContext`.
- Nodes are evaluated deterministically in node-id order.
- Supports generator, geometry-modifier, pattern, field, material, lighting, volumetric, and post-oriented node kinds.
- Graph shape supports linear pipelines and branch metadata through `connections`.

## `aurex_scene::automation`
- Core types: `AutomationTrack`, `AutomationCurve`, `AutomationTarget`, `AutomationBinding`.
- Sources include time/beat/measure/phrase/tempo/frequency-domain values.
- Curves: linear, smoothstep, sine, deterministic noise, exponential.
- Bindings can target generator, material, lighting, and camera-facing parameters.

## `aurex_scene::demo`
- Core types: `Demo`, `DemoTimeline`, `DemoBlock`, `Transition`.
- Timeline entries alternate scene blocks and transitions.
- Transition types: fade, pulse flash, pattern morph, fractal zoom, geometry dissolve.
- Runtime application is deterministic and time-normalized with loop-safe lookup.

## Scene IR extensions
`SdfScene` now supports:
- `effect_graph`
- `automation_tracks`
- `demo_sequence`

These are optional and backward-compatible with existing scene JSON.

## Related
- Transition engine and director rules: `docs/sdk/transition_engine.md`
