# AI Authoring SDK: Harmonic Scene Generation

Use this guide to emit valid harmonic scene JSON.

## Harmonic bindings schema

```json
"harmonics": {
  "geometry": {"band": "Bass", "strength": 1.0},
  "materials": {"band": "High", "strength": 1.0},
  "particles": {"band": "Melody", "strength": 1.0},
  "fields": [
    {"center": {"x": 0.0, "y": 0.0, "z": 0.0}, "radius": 12.0, "falloff": 0.8, "strength": 1.0, "band": "Bass"}
  ]
}
```

## Valid harmonic bands

- `Bass`
- `Mid`
- `High`
- `Melody`
- `Chords`
- `Full`

## Harmonic particle generator

```json
"generator": {
  "HarmonicParticleField": {
    "particle_count": 64,
    "radius": 3.5,
    "thickness": 0.22,
    "mode": "ChordLattice"
  }
}
```

Modes:
- `BassBursts`
- `MelodySpirals`
- `ChordLattice`

## Spectral material example

```json
"material": {
  "material_type": "SpectralReactive",
  "base_color": {"x": 0.6, "y": 0.85, "z": 1.0},
  "emissive_strength": 0.4,
  "roughness": 0.2,
  "pattern": "Noise",
  "parameters": {}
}
```

## Authoring strategy

- Drive geometry with `Bass` or `Chords`.
- Drive materials with `High` for neon flicker.
- Drive particles with `Melody` for readable motion.
- Keep strengths in `0.5-1.4` for stable outputs.
