# Aurex-X Technical SDK: Rhythm-Space & Time Warp System

## Overview
Rhythm-Space maps procedural beat structure into scene evolution and time behavior.

Per-frame pipeline:
1. `aurex_audio::analysis` computes rhythm state (`current_beat`, `current_measure`, `current_phrase`, `beat_phase`, `tempo`).
2. `aurex_render_sdf::scene_at_time` applies rhythm-space config.
3. Generators, particles, camera, and materials receive deterministic rhythm/time-warp modulation.

## Audio Rhythm Outputs
`AudioFeatures` now includes:
- `current_beat`
- `current_measure`
- `current_phrase`
- `beat_phase`
- `tempo`

## Rhythm Field
`aurex_scene::fields::RhythmField`:
- `beat_strength`
- `measure_strength`
- `phrase_strength`
- `tempo`

Sampling API:
- `sample_rhythm(field, time) -> f32`

`SceneField::Rhythm` contributes to `sample_fields` like other procedural fields.

## Scene Rhythm Config
`SdfScene.rhythm: Option<RhythmSpaceConfig>` with:
- `beat_geometry: bool`
- `echo_effect: bool`
- `particle_mode: Option<RhythmParticleMode>` (`Bass`, `Snare`, `Melody`)
- `time_warp: Option<TimeWarpConfig>`

`TimeWarpConfig`:
- `time_scale`
- `time_delay`
- `time_echo`
- `time_reverse`

## Runtime Effects
- **Beat geometry**: beat/measure/phrase pulses alter generator parameters.
- **Echo geometry**: strong beat pulses duplicate geometry briefly.
- **Particle rhythm**: emits rhythm/audio fields by particle mode.
- **Time warp**: warped time is injected into generator updates and material animation params (`rhythm_time`).

## Authoring Example
```json
{
  "sdf": {
    "rhythm": {
      "beat_geometry": true,
      "echo_effect": true,
      "particle_mode": "Bass",
      "time_warp": {
        "time_scale": 1.05,
        "time_delay": 0.08,
        "time_echo": 0.2,
        "time_reverse": false
      }
    }
  }
}
```
