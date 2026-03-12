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
