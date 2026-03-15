# Engine Architecture (Technical SDK)

Aurex-X currently follows a deterministic planning-to-render flow:

```text
Pulse
  ↓
Experience Planner
  ↓
WorldBlueprint
  ↓
GeneratorStack
  ↓
RhythmField
  ↓
Modulation Pass
  ↓
Renderer Pipeline
```

## Layer responsibilities
- **Pulse**: runtime package that owns lifecycle, timing, and diagnostics orchestration.
- **Experience Planner**: maps pulse intent to visual direction (`RenderTheme`) and world intent.
- **WorldBlueprint**: high-level world descriptor (what should exist and how it should feel), without low-level render execution details.
- **GeneratorStack**: deterministic parameter and structure generator that expands layered generator specs into concrete scene data.
- **RhythmField**: deterministic music-state snapshot that produces normalized modulation signals.
- **Modulation Pass**: applies bounded deltas to GeneratorStack output parameters.
- **Renderer Pipeline**: executes scene evaluation and image synthesis in fixed stage order.

## Current renderer pipeline stages
1. ScenePreprocess
2. EffectGraphEvaluation
3. GeometrySdf
4. MaterialPattern
5. LightingAtmosphere
6. Particles
7. PostProcessing
8. TemporalFeedback

## Integration notes
- `GeneratorStack` output is currently used for deterministic parameter/scene-node generation only.
- Runtime rhythm signals (`RhythmField`) modulate generation inputs; they do not add or reorder pipeline stages.
- Future RhythmField extensions should continue feeding generator/parameter inputs upstream of renderer execution.

## Runtime stabilization contracts
- Runtime debug and isolation environment flags are documented in `docs/sdk/runtime_debug_flags.md`.
- GeometrySdf mode contract is additive and explicit: `flat`, `safe`, `legacy` (default `safe`).
