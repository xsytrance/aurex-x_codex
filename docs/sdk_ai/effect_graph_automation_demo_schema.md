# AI Authoring SDK: Effect Graph + Automation + Demo Schema

## Scene fields
At `sdf` level:
- `effect_graph`
- `automation_tracks`
- `demo_sequence`

## Effect graph schema
```json
"effect_graph": {
  "nodes": [{"id": 1, "node": "TunnelGenerator", "parameters": {}, "inputs": [], "outputs": ["geo"]}],
  "connections": [{"from": 1, "to": 2}]
}
```

## Automation schema
```json
"automation_tracks": [
  {
    "target": "LightingIntensity",
    "track": {
      "name": "phrase-lights",
      "source": "Phrase",
      "curve": "Sine",
      "amplitude": 0.7,
      "offset": 0.2,
      "frequency": 0.25
    }
  }
]
```

## Demo sequence schema
```json
"demo_sequence": {
  "timeline": {
    "entries": [
      {"SceneBlock": {"scene_reference": "examples/a.json", "duration": 8.0}},
      {"Transition": {"transition_type": "Fade", "duration": 2.0, "intensity": 0.8}}
    ]
  }
}
```

## Determinism rules
- Keep IDs stable for graph nodes.
- Keep automation source/curve parameters explicit.
- Keep transition durations/intensities numeric and fixed.
