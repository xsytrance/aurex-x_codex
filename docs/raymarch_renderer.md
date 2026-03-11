# Aurex-X Raymarch SDF Renderer

This document describes the first Aurex-X SDF rendering backend in `aurex_render_sdf`.

## Pipeline Overview

```text
Scene JSON
-> Scene IR (`aurex_scene::Scene`)
-> Geometry modifier pipeline
-> Primitive SDF evaluation
-> Raymarching integration
-> Lighting + shadow rays
-> RGBA framebuffer
```

## Raymarch Algorithm

For each pixel:

1. Build camera ray from `position`, `target`, `fov_degrees`, `aspect_ratio`.
2. Repeatedly sample signed distance at `p = origin + direction * t`.
3. Advance `t += distance`.
4. Stop when:
   - `distance < surface_epsilon` (surface hit), or
   - `t > max_distance`, or
   - iteration reaches `max_steps`.

Renderer control parameters:

- `max_steps`: Upper bound for SDF iterations.
- `max_distance`: Clamp for ray travel distance.
- `surface_epsilon`: Surface hit threshold.

These controls are exposed by `aurex_render_sdf::RenderConfig`.

## SDF Primitive Evaluation

`aurex_scene::SdfPrimitive` is evaluated with the following signed-distance functions:

- Sphere
- Box
- Torus
- Plane
- Cylinder
- Capsule
- Mandelbulb fractal
- Noise field

All primitive evaluators return a scalar signed distance, where negative values are inside.

## Geometry Modifier Order

Modifiers are applied in declared pipeline order to the sampled point before primitive evaluation.

Supported modifiers:

- Repeat
- Twist
- Bend
- Scale
- Rotate
- Translate
- NoiseDisplacement
- Mirror

Because order is preserved, different ordering intentionally changes shape results.

## Lighting System

Lighting consumes `aurex_scene::SdfLighting` and `KeyLight`:

- Ambient term: `ambient_light`
- Diffuse Lambert term per key light
- Basic shadow ray attenuation (`soft_shadow`)

Final color:

```text
ambient + Σ(material_color * key_light_color * intensity * lambert * shadow)
```

## GPU Note

The first backend is CPU raymarching for correctness and deterministic output while still depending on `wgpu` for renderer backend expansion. The crate provides a `wgpu_backend_marker()` function to keep the dependency integrated and ready for the planned fullscreen shader path.
