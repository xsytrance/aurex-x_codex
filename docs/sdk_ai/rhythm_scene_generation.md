# AI Authoring SDK: Rhythm Scene Generation

Use this guide to generate valid Rhythm-Space scenes.

## Rhythm block schema
```json
"rhythm": {
  "beat_geometry": true,
  "echo_effect": true,
  "particle_mode": "Bass",
  "time_warp": {
    "time_scale": 1.0,
    "time_delay": 0.0,
    "time_echo": 0.0,
    "time_reverse": false
  }
}
```

## Valid enums
- `particle_mode`: `Bass`, `Snare`, `Melody`

## Rhythm field example
```json
{ "Rhythm": { "beat_strength": 1.0, "measure_strength": 0.6, "phrase_strength": 0.3, "tempo": 140.0 } }
```

## Suggested ranges
- `time_scale`: `0.7 - 1.4`
- `time_delay`: `0.0 - 0.25`
- `time_echo`: `0.0 - 0.6`
- `beat_strength`: `0.4 - 1.4`

## Complete snippet
```json
{
  "sdf": {
    "rhythm": {
      "beat_geometry": true,
      "echo_effect": true,
      "particle_mode": "Melody",
      "time_warp": {"time_scale": 1.12, "time_delay": 0.04, "time_echo": 0.18, "time_reverse": false}
    },
    "fields": [
      {"Rhythm": {"beat_strength": 1.0, "measure_strength": 0.55, "phrase_strength": 0.25, "tempo": 148.0}}
    ]
  }
}
```
