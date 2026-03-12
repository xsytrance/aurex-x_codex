# AI Authoring SDK: Spatial Field Guide

Use this guide to generate valid Aurex-X JSON scenes with fields.

## Field JSON Patterns

```json
{ "Noise": { "scale": 0.2-4.0, "strength": 0.0-2.0, "octaves": 1-8, "speed": 0.0-4.0 } }
{ "Flow": { "direction": { "x": -2..2, "y": -2..2, "z": -2..2 }, "turbulence": 0.0-2.0, "strength": 0.0-2.0 } }
{ "Pulse": { "origin": { "x": -50..50, "y": -50..50, "z": -50..50 }, "frequency": 0.1-8.0, "amplitude": 0.0-3.0, "falloff": 0.01-1.0 } }
{ "Audio": { "band": "Kick|Snare|Bass|Mid|High", "strength": 0.0-3.0, "radius": 0.1-200.0 } }
```

## Generator + Field Combinations

- Tunnel + Noise + Flow => psychedelic corridors
- CircuitBoard + Pulse + Audio(Kick) => reactive techno grids
- FractalTemple + Pulse + Noise => breathing monumental structures
- ParticleGalaxy + Flow + Audio(Bass) => rotating cosmic storms

## Timeline Targets

Use keyframes with these targets:

- `field.noise.strength` (Float)
- `field.pulse.frequency` (Float)
- `field.flow.direction` (Vec3)
- `generator.tunnel.radius` (Float)
- `generator.circuit.component_density` (Float)
- `generator.temple.fractal_scale` (Float)
- `generator.galaxy.rotation_speed` (Float)

## Complete Example: Psytrance Dimension

```json
{
  "sdf": {
    "seed": 321,
    "fields": [
      { "Noise": { "scale": 1.4, "strength": 0.5, "octaves": 5, "speed": 1.1 } },
      { "Flow": { "direction": { "x": 0.7, "y": 0.1, "z": 1.0 }, "turbulence": 0.8, "strength": 0.6 } },
      { "Audio": { "band": "Bass", "strength": 1.0, "radius": 20.0 } }
    ],
    "generator": {
      "Tunnel": { "radius": 1.8, "segment_count": 14, "twist": 0.1, "repeat_distance": 2.2 }
    },
    "timeline": {
      "duration": 8.0,
      "loops": true,
      "keyframes": [
        { "time": 0.0, "target": "generator.tunnel.radius", "value": { "Float": { "value": 1.5 } }, "interpolation": "Smoothstep" },
        { "time": 4.0, "target": "generator.tunnel.radius", "value": { "Float": { "value": 2.2 } }, "interpolation": "EaseOut" },
        { "time": 8.0, "target": "generator.tunnel.radius", "value": { "Float": { "value": 1.5 } }, "interpolation": "Smoothstep" }
      ]
    }
  }
}
```

## Validation Checklist for AI

1. Always include `seed`.
2. Use valid enum casing (`"Tunnel"`, `"Noise"`, etc.).
3. Keep numeric ranges sane (see above).
4. If keyframing a target, ensure it exists in current scene intent.
5. Prefer small field count first (1-3), then iterate.
