# Aurex-X Technical SDK: Field System

## Overview

`aurex_scene::fields` introduces spatial field forces sampled globally and used by:

- scene graph geometry evaluation
- procedural materials/emissive response
- atmospheric lighting (fog/scattering)
- generator expansion

## Rust Types

- `fields::SceneField`
  - `Noise(NoiseField)`
  - `Flow(FlowField)`
  - `Pulse(PulseField)`
  - `Audio(AudioField)`
- `fields::FieldSample { scalar, vector, energy }`
- sampling APIs:
  - `sample_field(field, position, time, scene_seed)`
  - `sample_fields(fields, position, time, scene_seed)`

`SdfScene` includes:

- `fields: Vec<fields::SceneField>`

## JSON Schema (Field Section)

```json
{
  "fields": [
    { "Noise": { "scale": 1.4, "strength": 0.5, "octaves": 5, "speed": 1.1 } },
    { "Flow": { "direction": { "x": 0.7, "y": 0.1, "z": 1.0 }, "turbulence": 0.8, "strength": 0.6 } },
    { "Pulse": { "origin": { "x": 0.0, "y": 0.0, "z": 0.0 }, "frequency": 2.4, "amplitude": 0.8, "falloff": 0.08 } },
    { "Audio": { "band": "Bass", "strength": 1.0, "radius": 20.0 } }
  ]
}
```

## Sampling Algorithm

At sample position `p` and time `t`:

1. Evaluate each field deterministically using `scene_seed`.
2. Accumulate:
   - `scalar` (domain/intensity)
   - `vector` (warp/flow direction)
   - `energy` (magnitude-like envelope)
3. Return combined `FieldSample`.

## Renderer Integration

Renderer applies field sample in multiple stages:

1. **Geometry/domain**: point warp before SDF distance evaluation.
2. **Material**: color/emissive modulation from `scalar` + `energy`.
3. **Lighting/atmosphere**: fog/scattering density boosted by field energy.
4. **Glow**: emissive accumulation amplified in energetic regions.

## Generator Integration

`generators::expand_generator(..., scene_fields)` allows generators to read fields while expanding.

Current interactions:

- Tunnel: radius perturbed by field scalar
- FractalTemple: pillar radius/height influenced by field sample
- CircuitBoard: component density and tower variation affected by field energy
- ParticleGalaxy: radial jitter influenced by field sample

## Timeline Integration

Renderer maps keyframe targets into mutable field params during `scene_at_time`:

- `field.noise.strength`
- `field.pulse.frequency`
- `field.flow.direction`

The updated field set is then used for generator expansion and final shading.
