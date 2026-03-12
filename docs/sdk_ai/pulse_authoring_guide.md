# AI Authoring Guide: Pulse JSON

## PulseDefinition
A Pulse definition includes:
- `metadata`
- `pulse_kind`
- `scene`
- optional `audio`
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
- `prime_affinity`

## Scene source
Use either:
- inline `Scene` object
- path reference: `{ "scene_path": "..." }`

## Authoring rule
Reuse existing Aurex scene/audio/timeline systems. Do not create a new scene graph for pulses.
