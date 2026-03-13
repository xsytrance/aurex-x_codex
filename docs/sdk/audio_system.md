# Aurex-X Technical SDK: Audio System

## Architecture

`aurex_audio` now provides deterministic, seed-driven procedural audio:

- `synth` module (modular synth graph)
- `voice` module (formant-based vocal synthesis)
- `sequencer` module (pattern/track music sequencing)
- `analysis` module (audio features for visuals)

`aurex_scene::SdfScene` includes:

- `audio: Option<aurex_audio::ProceduralAudioConfig>`

## Core Rust Types

### Synth

- `synth::SynthNode`
  - `Oscillator`
  - `Noise`
  - `FMOperator`
  - `Filter`
  - `Envelope`
  - `Mixer`
  - `Delay`
  - `Reverb`
  - `Distortion`
  - `Chorus`
- `synth::OscillatorType`
  - `Sine`, `Square`, `Saw`, `Triangle`, `Noise`, `Fm`

Sampling API:

- `sample_synth(node, t, sample_rate, seed) -> f32`

### Voice

- `voice::VoiceSynth`
- `voice::Phoneme` (`AH`, `EH`, `OH`, `OO`, `EE`)
- `voice::FormantFilter` (`f1`, `f2`, `f3`)
- `voice::VoicePreset` (`Robot`, `Female`, `Male`, `Choir`, `Alien`)

### Sequencer

- `sequencer::AudioNote`
- `sequencer::AudioPattern`
- `sequencer::AudioTrack`
- `sequencer::AudioSequence`

### Analysis

- `analysis::AudioFeatures`
  - `kick_energy`
  - `bass_energy`
  - `mid_energy`
  - `high_energy`
  - `spectral_centroid`
  - `tempo`
- `analyze_sequence(sequence, t, seed)`
- `analyze_procedural_audio(config, t)`

## Scene JSON Schema (Audio)

```json
{
  "audio": {
    "tempo": 140.0,
    "seed": 13370,
    "tracks": [
      {
        "name": "kick",
        "volume": 1.0,
        "patterns": [
          {
            "loops": 8,
            "notes": [
              { "pitch": 36, "duration_beats": 0.5, "velocity": 1.0, "instrument": "kick" }
            ]
          }
        ]
      }
    ],
    "synth_graph": { "Oscillator": { "osc_type": "Saw", "frequency": 110.0, "amplitude": 0.5, "phase": 0.0 } },
    "voice": {
      "preset": "Robot",
      "phonemes": ["EH", "OH"],
      "base_pitch_hz": 165.0,
      "phoneme_duration": 0.22
    }
  }
}
```

## Renderer/Scene Integration

`aurex_render_sdf::scene_at_time` analyzes audio and feeds visual systems:

- key-light intensity modulation
- fog density modulation
- camera motion perturbation
- temporary audio field injection (`SceneField::Audio`)
- field/generator re-expansion after timeline + audio updates

## Determinism Rules

- Always set `audio.seed` and `scene.seed`.
- Use explicit BPM and finite pattern loops.
- Keep oscillator/filter params fixed or timeline-keyframed.

## Current Limitations / Behavior Notes

- Sequencer tracks currently feed **procedural energy/intensity** analysis and modulation; they are not yet a full per-instrument synthesis pipeline.
- DSP nodes are lightweight deterministic approximations for real-time portability:
  - `Filter` uses simplified saturation-style shaping, not a full biquad model.
  - `Delay` and `Reverb` are compact tap/feedback approximations, not convolution/FDN-quality spaces.
- Voice synthesis is a phoneme/formant approximation designed for stylized robotic/choir/alien textures rather than realistic speech.

### Authoring Guidance

- For tight tonal control, prefer `synth_graph` (`Oscillator`/`FMOperator` + `Envelope` + `Filter`) and treat tracks as rhythmic energy drivers.
- Keep `tempo`, loop counts, and seeds explicit for reproducibility across machines.
- Use moderate effect values (`distortion.drive <= 2`, `reverb.room_size <= 1.2`) to avoid overpowering audio-reactive visuals.

## Related Authoring Systems

- Pattern networks (surface/detail identity): `docs/sdk/pattern_network_system.md`
- Harmonic geometry: `docs/sdk/harmonic_system.md`
- Rhythm-space/time-warp: `docs/sdk/rhythm_space_system.md`


## Cinematic and render integration
- Camera and director system: `docs/sdk/cinematic_systems.md`
- Post/volumetric pipeline guidance: `docs/sdk/cinematic_systems.md`


## Style profiles

Aurex supports deterministic genre scaffolding in `aurex_audio::style_profile`:

- `choose_style(seed)` selects one profile from built-ins (Electronic, Pop, HipHop, Rock, RnB, Jazz, Classical, Country, Reggae, World).
- `choose_style_selection(seed)` deterministically selects profile + BPM + scale.
- `styled_audio_config(seed)` builds sequencer tracks and instrument mappings from that style.

This provides genre-aware generation while keeping runtime callback behavior deterministic and allocation-free.
