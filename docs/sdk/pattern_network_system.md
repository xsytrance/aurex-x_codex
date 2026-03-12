# Aurex-X Technical SDK: Procedural Pattern Network System

## Overview
Pattern networks are deterministic mathematical detail graphs that enrich surfaces without textures.

Core goals:
- symbolic/ornamental/faction-like identity
- music and rhythm reactivity
- reusable in CPU renderer now and future GPU backends

## Scene IR Types
`aurex_scene::patterns` introduces:
- `PatternNetwork`
- `PatternLayer`
- `PatternNode`
- `PatternBinding`
- `PatternComposeOp`
- `PatternPreset`
- `PatternContext`

Scene/material integration:
- `SdfScene.patterns: Vec<PatternNetwork>`
- `SdfMaterial.pattern_network: Option<PatternNetwork>`

## Pattern Catalog
Implemented node variants:
- `GridPattern`
- `HexPattern`
- `CircuitTracePattern`
- `WaveformPattern`
- `ConcentricPulsePattern`
- `GlyphStripePattern`
- `SpiralPattern`
- `FractalVeinPattern`
- `LatticePattern`
- `MosaicPattern`

Shared parameters (`PatternParams`):
- `scale`, `thickness`, `contrast`, `density`, `rotation`, `distortion`, `seed`

## Composition Graph
`PatternLayer` supports operations:
- `Add`
- `Multiply`
- `Mask`
- `Blend`
- `Max`
- `Min`
- `Invert`
- `Warp`

Bindings control source space and audio/rhythm reactivity:
- spaces: `World`, `Local`, `Surface`
- reactives: `Low`, `Mid`, `High`, `DominantFrequency`, `Beat`, `Measure`, `Phrase`, `Tempo`

## Material Integration
Pattern sampling affects:
- base color modulation
- emissive boost
- roughness modulation
- surface distortion signal (exposed in pattern sample)

Renderer applies pattern sampling in a deterministic shading order.

## Music/Rhythm Integration
Pattern context is fed from audio + rhythm features each frame:
- low/mid/high energies
- dominant frequency
- beat/measure/phrase
- beat phase
- tempo

## Generator Presets
Built-in presets attach visual identity by math-only graphs:
- `ElectronicCircuit`
- `PsySpiral`
- `PrimePulseTemple`
- `JazzLoungeGlow`
- `OperaCathedral`
- `ReggaeSunwave`
- `ClassicalOrnament`
- `HipHopSignal`

Generators can assign preset networks to their materials automatically.

## JSON Authoring Example
```json
{
  "sdf": {
    "patterns": [
      {
        "name": "circuit_identity",
        "preset": "ElectronicCircuit",
        "layers": []
      }
    ],
    "objects": [
      {
        "primitive": {"Sphere": {"radius": 1.0}},
        "material": {
          "material_type": "SpectralReactive",
          "pattern_network": {
            "layers": [
              {
                "node": {"CircuitTracePattern": {"scale": 2.0, "thickness": 0.08, "contrast": 1.2, "density": 1.0, "rotation": 0.0, "distortion": 0.1, "seed": 9}},
                "op": "Blend",
                "weight": 1.0,
                "binding": {"space": "World", "react_to": "High", "strength": 1.0}
              }
            ]
          }
        }
      }
    ]
  }
}
```
