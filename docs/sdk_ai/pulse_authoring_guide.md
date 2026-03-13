# AI Authoring Guide: Pulse JSON

## PulseDefinition
A Pulse definition includes:
- `metadata`
- `pulse_kind`
- `scene`
- optional `audio`
- optional `music`
- optional `timeline`
- optional `generators`
- optional `parameters`

## Metadata fields
- `title`
- `author`
- `description`
- `tags`
- `seed`
- `pulse_kind`
- `duration_hint`
- `interactivity` (`Interactive` | `Passive` | `Hybrid`)
- `prime_affinity` (maps to PrimeFaction resonance tracking)

## Scene source
Use either:
- inline `Scene` object
- path reference: `{ "scene_path": "..." }`

## Authoring rule
Reuse existing Aurex scene/audio/timeline systems. Do not create a new scene graph for pulses.

## PulseGraph authoring
Use a PulseGraph when multiple pulses should run as a deterministic flow.

Core graph schema:
- `name`
- `seed`
- `entry_node`
- `nodes[]` (`id`, `pulse_path`)
- `transitions[]`

Transition variants:
- `Manual` (`trigger`)
- `Timeline` (`after_seconds`)
- `MusicalCue` (`cue` + threshold/multiple)
- `GeneratorTrigger` (`event`)

Reference example:
- `examples/pulse_graphs/electronic_journey.graph.json`

## Boot World authoring
Boot World is authored as a normal pulse with an additional optional `boot_world` block:

- `seed`
- `districts[]` (`id`, `prime`, `center`, `radius`, `pulse_refs`)
- `portals[]` (`id`, `trigger`, `target_node`, `position`, `activation_radius`)

Portals should emit manual trigger strings that are present in PulseGraph transitions.

Reference:
- `examples/pulses/boot_world.pulse.json`

## Resonance tracker notes
Pulse `metadata.prime_affinity` feeds runtime resonance accumulation.

Tracked Prime factions:
- Pop
- Rock
- HipHopRap
- Electronic
- RnBSoulFunk
- Classical
- Jazz
- CountryFolk
- ReggaeCaribbeanAfrobeat
- WorldTraditional
- AmbientExperimental
- Kazoom

Boot World district entries and portal launches can further update resonance profile metrics.

## Living Boot Screen behavior
Living Boot state is derived from the runtime resonance profile and currently exposes:
- dominant/top-three prime presentation
- visual/audio bias weights
- deterministic idle warning + resonance-event-ready state

This is an extensible framework for future boot-screen personalization and boot-world evolution.

## Prime Pulse gate progression
Boot World runtime now tracks Prime Pulse gate layers from resonance profile data.

Key outputs:
- `prime_pulse_distance`
- `prime_pulse_layer`
- `prime_pulse_layers_unlocked`
- `prime_pulse_force_field_active`
- `prime_pulse_intensity`
- `prime_pulse_proximity`

This is currently logical gating + diagnostics only, intended for future Pulse Navigator and world mutation systems.
