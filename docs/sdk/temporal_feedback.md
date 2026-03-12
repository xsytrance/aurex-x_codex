# Temporal Feedback (Technical SDK)

Temporal feedback adds deterministic frame-history blending after post processing.

## Stage
Renderer pipeline now includes:
1. ScenePreprocess
2. EffectGraphEvaluation
3. GeometrySdf
4. MaterialPattern
5. LightingAtmosphere
6. PostProcessing
7. TemporalFeedback

## Runtime structures
Implemented in `aurex_render_sdf::temporal`:
- `TemporalBuffer`
- `TemporalFrame`
- `TemporalConfig`
- `TemporalBlendMode`
- `TemporalEffect`

Temporal buffers store previous frame color + depth and are persisted in `RendererState`.

## Blend modes
- `AdditiveTrail`
- `DecayTrail`
- `MotionEcho`
- `BeatEcho`
- `ColorSmear`

All modes are deterministic and audio-reactive via beat/measure/harmonic/frequency inputs.
