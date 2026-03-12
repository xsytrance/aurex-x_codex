# Aurex-X Technical SDK: Harmonic Geometry System

## Overview

The harmonic system maps procedural audio features into geometry, material, and particle modulation.

Data flow each frame:
1. `aurex_audio::analysis::analyze_procedural_audio` computes band/spectrum features.
2. `aurex_render_sdf::scene_at_time` applies harmonic bindings.
3. Generator/material parameters are adjusted deterministically.
4. Scene is expanded and rendered.

## Scene IR Types

- `aurex_scene::harmonics::HarmonicField`
- `aurex_scene::harmonics::HarmonicBand`
- `aurex_scene::harmonics::HarmonicBinding`
- `aurex_scene::harmonics::SceneHarmonicsConfig`
- `SdfScene.harmonics: Option<SceneHarmonicsConfig>`

## Audio Spectrum Features

`AudioFeatures` includes:
- `low_freq_energy`
- `mid_freq_energy`
- `high_freq_energy`
- `dominant_frequency`
- `harmonic_ratios`

These values are deterministic for `(audio config, seed, t)`.

## Harmonic Geometry Controls

Harmonic bindings can modulate:
- tunnel radius/twist
- fractal temple scale/height
- circuit board variation/trace width
- particle radius/thickness

## Harmonic Particle System

`SceneGenerator::HarmonicParticleField` supports:
- `BassBursts`
- `MelodySpirals`
- `ChordLattice`

## Spectral Materials

`SdfMaterialType::SpectralReactive` reads harmonic parameters:
- `harmonic_low`
- `harmonic_mid`
- `harmonic_high`
- `harmonic_energy`
- `dominant_frequency`

and drives color/emission/roughness in shader evaluation.

## JSON Example

```json
{
  "sdf": {
    "harmonics": {
      "geometry": {"band": "Bass", "strength": 1.1},
      "materials": {"band": "High", "strength": 1.0},
      "particles": {"band": "Melody", "strength": 0.9}
    }
  }
}
```

## Determinism Rules

- Set both `scene.seed` and `audio.seed`.
- Use fixed loops and bounded parameter ranges.
- Avoid unbounded growth by clamping harmonic strengths in authoring.
