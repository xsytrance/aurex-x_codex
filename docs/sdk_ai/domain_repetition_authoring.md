# AI Authoring: Domain Repetition

Use domain modifiers to author infinite-feeling spaces with small scene definitions.

## Modifiers
- `RepeatGrid { cell_size: Vec3 }`
- `RepeatAxis { spacing: f32, axis: "x"|"y"|"z" }`
- `RepeatPolar { sectors: u32 }`
- `RepeatSphere { radius: f32 }`
- `FoldSpace`
- `MirrorFold`
- `KaleidoscopeFold { segments: u32 }`

## Practical patterns
- Infinite tunnel: `RepeatAxis` on `z`, optional `Twist`.
- Circuit city: `RepeatGrid` with large `x/z` cells and low `y` repeat.
- Radial temple: `RepeatPolar` + `KaleidoscopeFold`.

## Guidance
- Keep modifier order intentional; transforms are applied sequentially.
- Use moderate cell sizes first, then tighten for denser repetition.
- Combine with fog/lighting to avoid visual monotony.
