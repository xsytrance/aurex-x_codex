# Geometry Nodes

`aurex_scene::geometry_nodes::SceneGeometryNode` defines procedural SDF primitives for scene JSON.

## Primitives

### sphere
- **Description:** Round primitive useful for blobs, planets, and tunnel beads.
- **Parameters:** `radius`, `seed`.
- **Example JSON:**
```json
{ "type": "sphere", "radius": {"kind":"value","value":1.2}, "seed": 7 }
```
- **Visual effect:** Smooth closed surface with even curvature.

### torus
- **Description:** Ring shape for tunnels and orbital structures.
- **Parameters:** `major_radius`, `minor_radius`, `seed`.
- **Example JSON:**
```json
{ "type": "torus", "major_radius": {"kind":"value","value":2.4}, "minor_radius": {"kind":"value","value":0.25}, "seed": 11 }
```
- **Visual effect:** Donut-like ring, often repeated for demoscene corridors.

### box / plane / cylinder / capsule
- **Description:** Core constructive primitives.
- **Parameters:** `size`/`normal`/`height`/`radius` plus optional `seed`.
- **Visual effect:** Hard-edged forms, floors, pillars, and rounded columns.

### fractal_mandelbulb
- **Description:** Iterative fractal SDF primitive.
- **Parameters:** `power`, `iterations`, `bailout`, `scale`, `seed`.
- **Visual effect:** Dense recursive alien geometry.

### noise_field
- **Description:** Seeded volumetric pattern primitive.
- **Parameters:** `scale`, `amplitude`, `octaves`, `seed`.
- **Visual effect:** Organic fog-like or terrain-like displacement source.
