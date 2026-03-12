# AI Authoring SDK: Transition Schema

## Transition object in demo timeline
```json
{
  "Transition": {
    "transition_type": "FractalZoom",
    "duration": 2.0,
    "intensity": 0.8,
    "auto": false,
    "spec": {
      "style": "FractalZoom",
      "duration": 2.0,
      "intensity": 0.8,
      "distortion": 0.2,
      "pattern_strength": 0.5,
      "harmonic_strength": 0.6,
      "progress_signal": "Phrase"
    }
  }
}
```

## Auto transition mode
```json
{"Transition": {"transition_type": "PatternMorph", "duration": 2.0, "intensity": 0.7, "auto": true}}
```

## Graph morph fields
Graph morphing is runtime-driven when both source/target scenes provide `effect_graph`.

Strategies:
- `NodeParameterBlend`
- `DistanceFieldBlend`
- `PatternCrossfade`
- `HarmonicPhaseBlend`
- `GeneratorMorph`

## Determinism
- keep numeric values explicit
- keep node IDs stable across morph targets
- use fixed durations and seed inputs
