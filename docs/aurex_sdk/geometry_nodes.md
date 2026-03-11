# Geometry Node Catalog (Initial)

This catalog defines initial node types for Scene IR authoring.

## TunnelGenerator

Creates a tunnel-like procedural form suitable for forward-travel scenes.

- `radius` (float, range `1.0` - `20.0`): tunnel base radius.
- `twist` (float, range `0.0` - `5.0`): twist intensity along axis.
- `glow` (float, range `0.0` - `10.0`): emissive intensity hint.

## ParticleEmitter

Emits particles from a source or volume.

- `spawn_rate` (float, range `0.0` - `1000.0`): particles emitted per second.
- `speed` (float, range `0.0` - `50.0`): initial particle speed.
- `lifetime` (float, range `0.1` - `20.0`): particle lifetime in seconds.

## LightRig

Describes a grouped lighting setup.

- `intensity` (float, range `0.0` - `20.0`): master light output multiplier.
- `temperature` (float, range `1000.0` - `20000.0`): color temperature in kelvin.
- `strobe` (float, range `0.0` - `1.0`): strobe amount.

## GridFloor

Defines a stylized infinite or bounded floor grid.

- `scale` (float, range `0.1` - `50.0`): cell scaling factor.
- `line_width` (float, range `0.001` - `1.0`): line thickness.
- `pulse` (float, range `0.0` - `10.0`): pulse amplitude for animation.
