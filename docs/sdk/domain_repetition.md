# Domain Repetition (Technical SDK)

Aurex supports deterministic domain manipulation for infinite procedural environments.

## Module
`aurex_render_sdf::domain` provides pure coordinate transforms used before SDF primitive evaluation.

## Repetition operators
- `repeat_grid(p, cell_size)` — wraps coordinates into a repeating 3D cell.
- `repeat_axis(p, spacing, axis)` — repeats along a single axis.
- `repeat_polar(p, sectors)` — folds around Y into radial sectors.
- `repeat_sphere(p, radius)` — radial repeat for spherical symmetry.

## Folding operators
- `fold_space(p)` — absolute fold on XYZ.
- `mirror_fold(p)` — mirror fold on X/Z plane pair.
- `kaleidoscope_fold(p, segments)` — mirror + polar fold composition.

## Integration
Domain operators are applied in modifier order before primitive distance sampling:

`world point -> domain transforms -> primitive SDF`

Supported SDF modifiers:
- `RepeatGrid`
- `RepeatAxis`
- `RepeatPolar`
- `RepeatSphere`
- `FoldSpace`
- `MirrorFold`
- `KaleidoscopeFold`

All operations are deterministic and scene-seed stable.
