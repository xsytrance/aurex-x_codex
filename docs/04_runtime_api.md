# Aurex Runtime API

Games interact with Aurex through an ECS architecture.

Example entity:

Player
transform
velocity
shape
material
collider

Core operations:

create_entity()
add_component()
remove_component()

Rendering instructions:

spawn_shape()
apply_material()
emit_particles()

Camera API:

camera.position()
camera.rotate()
camera.zoom()
camera.look_at()

Audio API:

audio.play_pattern()
audio.trigger_event()

Post Processing Control:

post.bloom
post.motion_trails
post.fog
post.color_grade

Game packages are distributed as:

.axg

These contain:

manifest
logic
music patterns
procedural parameters

Typical game sizes are expected to be extremely small due to procedural content.

---

## Runtime API v0 Contract (Architecture-Level)

The following contracts define the first stable boundary between game logic and runtime internals.

### Deterministic Timing

- `Tick(u64)` – authoritative simulation step.
- `FrameIndex(u64)` – presented visual frame.
- `FixedDelta { seconds }` – fixed simulation interval.
- `DeterminismSeed(u64)` – session/world/zone seed root.

### ECS-Level Contracts

- `EntityId(u32)` – stable identity handle.
- `Transform2p5D { position, rotation_yaw_pitch_roll, scale }` – canonical world transform for 2.5D + 3D camera presentation.

### Rendering Contracts

- `CameraRig { eye, target, fov_degrees }` – baseline camera descriptor.
- `ShapeDescriptor { primitive_type, seed }` – procedural geometry descriptor.
- `PrimitiveType = Circle | Polygon | Ring | Tube`.
- `LightDescriptor { kind, intensity, color_rgb }` with `LightKind = Ambient | Point | Spot | Pulse`.
- `BloomSettings { intensity, low_frequency_boost }` – post-processing baseline.

### Execution Model Contract

Game-facing logic should target this update model:

1. Submit deterministic commands and gameplay intent.
2. Runtime executes fixed-step simulation tick.
3. Runtime emits audio/reactive events.
4. Runtime renders from current/interpolated snapshot.

This contract prevents games from coupling to renderer internals while preserving procedural flexibility.

---

## API Growth Recommendations (Next)

Recommended additions before external game packages are supported:

1. Command-buffer API for deterministic input and replay.
2. Material descriptor contract (style + reactive parameters).
3. Audio event bus contract (`beat`, `kick`, `snare`, `drop`, custom).
4. Error contract for invalid procedural descriptors.
5. Versioned runtime capability handshake for `.axg` compatibility.
