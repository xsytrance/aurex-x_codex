# Aurex-X SDF Composition and Scene Graph

Aurex-X now supports hierarchical SDF scene graphs with boolean composition and smooth blending.
This enables complex structures from small procedural building blocks.

## Scene Graph Overview

`SdfScene` supports a tree root:

- `root: SdfNode`

`SdfNode` variants:

- `Primitive { object }`
- `Transform { modifiers, child, bounds_radius }`
- `Group { children }`
- `Union { children }`
- `SmoothUnion { children, k }`
- `Subtract { base, subtract }`
- `Intersect { children }`
- `Blend { children, weights }`
- `Empty`

Nodes are recursively evaluated.

## Boolean Composition

For child distances `d1, d2`:

- `Union` → `min(d1, d2)`
- `Intersect` → `max(d1, d2)`
- `Subtract` → `max(d1, -d2)`
- `Blend` → weighted average distance

`Subtract` supports one base node and multiple subtract nodes (unioned implicitly by min distance).

## Smooth Blending

`SmoothUnion` blends hard CSG seams into organic transitions using smooth-min:

```text
smooth_min(a, b, k)
```

`k` controls blend width:
- small `k` => sharper union
- larger `k` => softer transition

Renderer utilities also provide `smooth_max` for smooth subtract/intersection style logic.

## Transform Stacks

Nested transforms are represented with `Transform` nodes and local modifier lists.

Example:

```text
Transform(Translate)
  Transform(Rotate)
    Transform(Repeat)
      Primitive(Sphere)
```

Each transform affects its full subtree.

## Bounding Heuristics

Simple pruning support is included via optional `bounds_radius` fields on `SdfObject` and `Transform`:

- Bounding sphere estimate: `length(local_point) - radius`
- In unions/groups, nodes can early-out when a hint distance is already better.
- This keeps implementation simple and deterministic while providing a path toward BVH/acceleration later.

## JSON Example

```json
{
  "root": {
    "SmoothUnion": {
      "k": 0.35,
      "children": [
        { "Primitive": { "object": { "primitive": { "Sphere": { "radius": 1.0 } } } } },
        {
          "Transform": {
            "modifiers": [{ "Translate": { "offset": { "x": 1.0, "y": 0.0, "z": 0.0 } } }],
            "child": { "Primitive": { "object": { "primitive": { "Box": { "size": { "x": 0.5, "y": 0.5, "z": 0.5 } } } } }
          }
        }
      ]
    }
  }
}
```

## Authoring Tips

1. Start with a `Union` of 2–3 simple primitives.
2. Introduce `Transform`+`Repeat` for structural rhythm.
3. Use `SmoothUnion` for organic joins.
4. Carve space with `Subtract`.
5. Use `bounds_radius` aggressively on large repeated nodes.
