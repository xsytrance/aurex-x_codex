# AI Authoring: Cinematic Scene Schema

## Timeline camera fields
`SceneTimeline` supports:
- `cinematic_camera: CameraRig`
- `shot_sequence: ShotSequence`

## CameraRig tagged union
```json
{"type":"Orbit", "center":..., "radius":..., "speed":..., "height":..., "fov_degrees":..., "roll":..., "rhythm": {"tempo_sync":..., "beat_shake":...}}
```
Variants: `Orbit`, `Flythrough`, `TargetTracking`, `BezierPath`, `Rhythm`.

## Director schema
```json
{"shots": [{"start":0.0, "end":4.0, "label":"intro", "camera": {"type":"Orbit", ...}}]}
```

## Volumetric lighting extension
```json
"lighting": {
  "volumetric": {
    "scattering_steps": 12,
    "beam_falloff": 0.8,
    "beam_density": 0.12,
    "shaft_intensity": 0.7
  }
}
```

## Determinism constraints
- Keep camera inputs explicit and seed/time driven.
- Avoid non-deterministic ordering when generating shot lists.
- Use fixed-step volumetric sampling settings.

## Related schema
- Effect graph + automation + demo schema: `docs/sdk_ai/effect_graph_automation_demo_schema.md`
