# AI Authoring SDK: Pattern Scene Generation

Use this guide to generate valid pattern-rich scenes.

## Scene-level pattern networks
```json
"patterns": [
  {"name": "identity", "preset": "PsySpiral", "layers": []}
]
```

## Material-level pattern override
```json
"pattern_network": {
  "layers": [
    {
      "node": {"WaveformPattern": {"scale": 2.8, "thickness": 0.1, "contrast": 1.2, "density": 1.0, "rotation": 0.0, "distortion": 0.25, "seed": 14}},
      "op": "Blend",
      "weight": 1.0,
      "binding": {"space": "Surface", "react_to": "Beat", "strength": 1.0}
    },
    {
      "node": {"ConcentricPulsePattern": {"scale": 1.8, "thickness": 0.08, "contrast": 1.1, "density": 1.2, "rotation": 0.0, "distortion": 0.1, "seed": 31}},
      "op": "Warp",
      "weight": 0.8,
      "binding": {"space": "World", "react_to": "Low", "strength": 1.0}
    }
  ]
}
```

## Valid presets
- `ElectronicCircuit`
- `PsySpiral`
- `PrimePulseTemple`
- `JazzLoungeGlow`
- `OperaCathedral`
- `ReggaeSunwave`
- `ClassicalOrnament`
- `HipHopSignal`

## Valid composition ops
- `Add`, `Multiply`, `Mask`, `Blend`, `Max`, `Min`, `Invert`, `Warp`

## Reactivity bindings
- `Low`, `Mid`, `High`, `DominantFrequency`, `Beat`, `Measure`, `Phrase`, `Tempo`

## Prompt strategy
- For circuit identity: use `ElectronicCircuit` + `High` reactive traces.
- For tunnel trance identity: `PsySpiral` + `Beat`/`Phrase` warp layers.
- For ceremonial architecture: `PrimePulseTemple` or `OperaCathedral` + `Low`/`Chords`.
