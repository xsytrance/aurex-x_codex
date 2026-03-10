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
