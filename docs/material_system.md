# Aurex-X Procedural Material System

Aurex-X materials are fully procedural: no textures, no mesh assets, no baked lookups.
All color and lighting response are generated from math at runtime.

## Data Model (`aurex_scene::SdfMaterial`)

Each SDF object can define:

- `material_type`: built-in procedural shader family
- `base_color`: base RGB tint
- `emissive_strength`: additive glow contribution
- `roughness`: controls specular tightness
- `pattern`: optional pattern layer shared across material types
- `parameters`: map of custom floats for material-specific tuning

## Material Types

### SolidColor
Flat base color with optional shared pattern modulation.

### NeonGrid
Animated sine-based grid lines with pulsing emissive highlights.
Useful for tunnels, circuit floors, and synthwave edges.

Suggested parameters:
- `grid_scale`
- `pulse_speed`
- `line_width`

### Plasma
Animated trigonometric field combining multiple phase-shifted waves.
Produces flowing colorful blobs and emissive energy surfaces.

Suggested parameters:
- `speed`
- `frequency`

### FractalMetal
Noise-driven metallic variation with view-angle fresnel accents.
Designed for fractal/architectural surfaces.

### NoiseSurface
Dense fBm modulation for rough stone/terrain-like surfaces.

Suggested parameters:
- `noise_frequency`

### Holographic
Angle-dependent rainbow shift with emissive sheen.

Suggested parameters:
- `shift_speed`

### Lava
Flowing fBm noise blended into hot/cool gradients.
Designed for emissive molten effects.

### Wireframe
Procedural line lattice based on repeated local coordinates.

Suggested parameters:
- `wire_scale`
- `wire_width`

## Shared Pattern Layer (`SdfPattern`)

The `pattern` field applies an additional modulation layer:

- `None`
- `Bands`
- `Rings`
- `Checker`
- `Noise`

This allows quick style variation without changing material type.

## Renderer Evaluation API

Material sampling is implemented in:

```rust
evaluate_material(material, position, normal, time, scene_seed)
```

Inputs:
- surface position
- surface normal
- `RenderTime`
- scene seed

Outputs:
- RGB color
- emission scalar
- roughness scalar

## Time Animation

`RenderConfig` contains:

- `time: RenderTime { seconds }`

Time drives animated effects such as:
- Neon grid pulses
- Plasma waves
- Lava flow
- Holographic color shift

## Noise Utilities

`aurex_render_sdf::noise` provides deterministic functions used by materials/modifiers:

- `hash_noise`
- `value_noise`
- `fbm`

All functions are deterministic for identical inputs and seed.

## Emissive + Bloom Pre-pass

The renderer writes optional bloom pre-pass values in `RenderedFrame::bloom_prepass` when
`RenderConfig::output_bloom_prepass` is enabled.

This supports future post-processing pipelines while remaining fully procedural.

## Example JSON

```json
{
  "material": {
    "material_type": "NeonGrid",
    "base_color": { "x": 0.2, "y": 0.9, "z": 1.0 },
    "emissive_strength": 0.85,
    "roughness": 0.12,
    "pattern": "Bands",
    "parameters": {
      "grid_scale": 9.0,
      "pulse_speed": 3.8,
      "line_width": 0.06
    }
  }
}
```

## LLM Scene Authoring Guidance

When generating scene JSON:

1. Pick `material_type` based on mood (e.g., `NeonGrid` for techno, `Lava` for volcanic).
2. Set `base_color` as the dominant palette anchor.
3. Use `emissive_strength` for glow readability.
4. Keep `roughness` low for glossy/highlights, high for diffuse/chalky look.
5. Use `pattern` for quick structure overlays before introducing many parameters.
6. Add only a small parameter set first, then iterate.
