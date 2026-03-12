# Aurex-X Advanced Lighting and Atmospherics

This document describes the deterministic, asset-free advanced lighting path in `aurex_render_sdf`.

## Features

- Soft shadows (raymarched)
- Ambient occlusion (normal-probe SDF AO)
- Volumetric fog
- Distance glow accumulation for emissive media
- Lightweight forward light scattering

All effects are procedural and deterministic for identical scene + time + config inputs.

## RenderConfig Controls

`RenderConfig` now includes:

- `shadow_steps`
- `shadow_softness`
- `ao_samples`
- `ao_strength`
- `enable_soft_shadows`
- `enable_ambient_occlusion`
- `enable_fog`
- `enable_scattering`

These allow quality/performance tuning without changing scene assets.

## Soft Shadows

Shadow raymarch formula:

```text
shadow = min(shadow, k * distance / t)
```

Where:
- `t` = traveled distance on shadow ray
- `distance` = SDF sample at that step
- `k` = `RenderConfig.shadow_softness`

This gives penumbra-like softness around occluders.

## Ambient Occlusion

AO samples several points along the hit normal and compares expected sample offsets vs. SDF distances:

```text
ao = 1 - strength * average(max(expected - sampled_distance, 0) / expected)
```

Crevices/intersections darken naturally.

## Volumetric Fog

Fog is configured in scene lighting:

- `fog_color`
- `fog_density`
- `fog_height_falloff`

Blend model:

```text
fog_factor = 1 - exp(-density * distance * exp(-height_falloff * height))
color = mix(surface_color, fog_color, fog_factor)
```

## Distance Glow

During primary raymarch, emissive strength contributes to an accumulated glow term:

```text
glow += emissive_strength * attenuation
```

This boosts neon/plasma readability in atmosphere-heavy scenes.

## Light Scattering

A lightweight volumetric pass samples along the camera ray and accumulates key-light forward phase contribution:

```text
scatter += light_intensity * phase_function * transmittance
```

Used for beam-like depth without heavy volumetric cost.

## Shading Order

Per hit:

1. Base material evaluation
2. Diffuse/specular key lighting
3. Ambient occlusion modulation
4. Emissive + distance glow
5. Light scattering add
6. Fog blending

Order is fixed for deterministic output.

## JSON Example (Lighting Section)

```json
{
  "lighting": {
    "ambient_light": 0.15,
    "fog_color": { "x": 0.05, "y": 0.12, "z": 0.22 },
    "fog_density": 0.09,
    "fog_height_falloff": 0.22,
    "key_lights": [
      {
        "direction": { "x": -0.3, "y": -1.0, "z": -0.2 },
        "intensity": 1.0,
        "color": { "x": 0.4, "y": 0.9, "z": 1.0 }
      }
    ]
  }
}
```
