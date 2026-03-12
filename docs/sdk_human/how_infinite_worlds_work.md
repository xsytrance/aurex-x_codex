# How Infinite Worlds Work

Aurex can make spaces feel endless by reusing the same geometry pattern across space.

- **Grid repetition** creates city-like blocks.
- **Axis repetition** creates endless tunnels.
- **Polar repetition** creates circular temple layouts.
- **Folding** mirrors space to build complex cathedral/fractal shapes from simple forms.

The trick is that Aurex transforms the sampled position before checking distance to a shape. So one object definition can appear many times without storing massive scene data.

These transforms are deterministic, so the same scene + time always renders the same result.
