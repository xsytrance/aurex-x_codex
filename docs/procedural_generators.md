# Aurex-X Procedural World Generators

Aurex-X supports high-level seed-driven world generators in `aurex_scene::generators`.
Generators expand into hierarchical SDF node graphs at scene build/render time.

Pipeline:

```text
Scene JSON
-> Generator Expansion
-> SDF Node Graph
-> Raymarch Renderer
```

## SceneGenerator

`SdfScene.generator` accepts one of:

- `Tunnel`
- `FractalTemple`
- `CircuitBoard`
- `ParticleGalaxy`

Expansion uses `SdfScene.seed` for deterministic layouts.

Same seed + same params => same world.
Different seed => different layout.

## Generators

### TunnelGenerator
Creates repeating tunnel segments using torus rings and transforms.

Parameters:
- `radius`
- `segment_count`
- `twist`
- `repeat_distance`

### FractalTempleGenerator
Creates a central fractal core plus a deterministic pillar grid.

Parameters:
- `grid_size`
- `pillar_height`
- `pillar_spacing`
- `fractal_scale`

### CircuitBoardGenerator
Creates circuitry-like worlds with traces and component towers.

Generates:
- trace lines
- capacitor-like towers
- resistor-like blocks
- grid pathways

Parameters:
- `grid_resolution`
- `component_density`
- `trace_width`
- `height_variation`

### ParticleGalaxyGenerator
Creates radial particle clusters with deterministic spread.

Parameters:
- `particle_count`
- `radius`
- `noise_spread`
- `rotation_speed`

## Timeline Integration

Generator parameters can be animated with timeline keyframes.

Supported timeline targets:

- `generator.tunnel.radius`
- `generator.tunnel.twist`
- `generator.temple.fractal_scale`
- `generator.circuit.component_density`
- `generator.galaxy.rotation_speed`
- `generator.galaxy.radius`

This enables effects like tunnel pulsing, fractal breathing, and galaxy spin-up.

## JSON Example

```json
{
  "sdf": {
    "seed": 321,
    "generator": {
      "Tunnel": {
        "radius": 1.8,
        "segment_count": 14,
        "twist": 0.1,
        "repeat_distance": 2.2
      }
    },
    "timeline": {
      "duration": 8.0,
      "loops": true,
      "keyframes": [
        {
          "time": 0.0,
          "target": "generator.tunnel.radius",
          "value": { "Float": { "value": 1.5 } },
          "interpolation": "Smoothstep"
        },
        {
          "time": 4.0,
          "target": "generator.tunnel.radius",
          "value": { "Float": { "value": 2.2 } },
          "interpolation": "EaseOut"
        }
      ]
    }
  }
}
```

## Visual Expectations

- **Tunnel**: rhythmic neon corridors.
- **FractalTemple**: monumental pillars around a central fractal core.
- **CircuitBoard**: techno-grid worlds with luminous traces.
- **ParticleGalaxy**: rotating starfield clusters and plasma particles.
