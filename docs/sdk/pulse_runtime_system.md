# Pulse Runtime System (Technical SDK)

A **Pulse** is the primary executable Aurex-X experience unit (game, world, visual music, demo, or ambient runtime).

## Core types
- `PulseDefinition` (JSON schema)
- `PulseRunner` (lifecycle executor)

Implemented in `crates/aurex_pulse`.

## Lifecycle
1. `load`
2. `initialize`
3. `update`
4. `render`
5. `shutdown`

## Integration model
Pulse runtime **orchestrates existing systems** rather than replacing them:
- existing `SdfScene` scene graph
- existing generator/effect graph/automation/timeline/demo/camera stack
- existing renderer diagnostics

## Rendering pipeline
Pulse runtime invokes the existing renderer pipeline unchanged:
- ScenePreprocess
- EffectGraphEvaluation
- GeometrySdf
- MaterialPattern
- LightingAtmosphere
- Particles
- PostProcessing
- TemporalFeedback

## Diagnostics
Pulse-level diagnostics track lifecycle timing and frame counts, while preserving nested renderer diagnostics payloads.

## Music sequencing integration
Pulse definitions may include `music: MusicSequenceConfig`.
The runtime initializes a sequencer and updates it every frame step, exposing RhythmField modulation signals while keeping the renderer pipeline unchanged.

Rhythm integration points:
- stores `RhythmField` in pulse runtime context each update
- publishes a diagnostics rhythm summary (`beat_phase`, `bar_index`, `bass_energy`)
- applies lightweight scene modulation by scaling `scene.sdf.lighting.ambient_light` from `beat_strength` and `harmonic_energy`

## PulseGraph orchestration
`aurex_pulse::pulse_graph` adds graph-level orchestration on top of `PulseRunner`:

- `PulseGraph`
- `PulseNode`
- `PulseTransition`
- `PulseGraphRunner`

`PulseGraphRunner` wraps and coordinates `PulseRunner` instances (it does not modify renderer stages).

Supported transition kinds:
- manual trigger
- timeline threshold (`after_seconds`)
- musical cue from `RhythmField`
- generator event trigger

Example graph: `examples/pulse_graphs/electronic_journey.graph.json`

## Boot World pulse hub
Pulse definitions may include optional `boot_world` metadata for hub-style experiences.

Boot World uses:
- `BootWorldGenerator`
- `District`
- `PulsePortal`
- `BootWorldState`

The runtime tracks district/portal proximity state, and portal systems can emit manual triggers into `PulseGraphRunner` for pulse launches.

## Resonance tracker
Pulse runtime can maintain a per-player `ResonanceTracker` profile keyed by `PrimeFaction`.

Sources:
- pulse `metadata.prime_affinity`
- Boot World district visits
- Boot World portal launches

Diagnostics expose `dominant_prime` and `top_three_primes` snapshots without affecting renderer diagnostics payloads.

## Living Boot Screen
Boot World includes optional Living Boot state derived from resonance profile.

- presentation updates: dominant/top-three + visual/audio bias weights
- idle updates: deterministic warning/event-ready state machine

Idle event rules are intentionally minimal:
- first long-idle threshold => warning only
- later long-idle thresholds => `resonance_event_ready = true`

No world mutation is performed yet; this only exposes deterministic state for future systems.
