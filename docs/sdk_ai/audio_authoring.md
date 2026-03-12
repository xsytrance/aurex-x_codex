# AI Authoring SDK: Procedural Audio

Use this guide to generate valid Aurex-X audio JSON.

## Minimum Audio Block

```json
{
  "audio": {
    "tempo": 140.0,
    "seed": 77,
    "tracks": [],
    "synth_graph": null,
    "voice": null
  }
}
```

## Valid Voice Presets

- `Robot`
- `Female`
- `Male`
- `Choir`
- `Alien`

## Valid Phonemes

- `AH`, `EH`, `OH`, `OO`, `EE`

## SynthNode JSON Variants

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

## Suggested Parameter Ranges

- `tempo`: 60-200
- `golden_tempo_mode.tempo_drift`: 0.02-0.12 (optional)
- oscillator `frequency`: 20-4000
- oscillator `amplitude`: 0-1
- filter `cutoff`: 20-20000
- envelope ADSR: 0.001-4.0
- distortion `drive`: 0-4
- reverb `room_size`: 0-2


## Optional Golden Tempo Mode

You can enable deterministic golden-ratio tempo evolution by setting:

```json
"golden_tempo_mode": { "tempo_drift": 0.0618 }
```

Behavior:
- Phrase bars follow Fibonacci lengths: `5, 8, 13, 21` (repeating).
- Tempo modulation is deterministic from `seed`, phrase phase, and `phi = 1.61803398875`.
- If omitted, tempo remains fixed at `audio.tempo`.

## Audio-Visual Integration Tips

- Use kick-heavy patterns for pulse fields.
- Use bass energy to modulate fog density.
- Use high energy for sparkle/light intensity.
- Pair `voice.preset = Alien` with psychedelic tunnel scenes.

## Complete Example (Audio + Visual)

```json
{
  "sdf": {
    "seed": 444,
    "fields": [
      { "Audio": { "band": "Bass", "strength": 1.2, "radius": 24.0 } }
    ],
    "generator": {
      "Tunnel": { "radius": 1.9, "segment_count": 18, "twist": 0.12, "repeat_distance": 2.0 }
    },
    "audio": {
      "tempo": 148.0,
      "seed": 444,
      "tracks": [
        {
          "name": "bass",
          "volume": 0.9,
          "patterns": [
            {
              "loops": 8,
              "notes": [
                { "pitch": 40, "duration_beats": 0.25, "velocity": 0.8, "instrument": "bass" }
              ]
            }
          ]
        }
      ],
      "voice": {
        "preset": "Alien",
        "phonemes": ["AH", "EE", "OH", "OO"],
        "base_pitch_hz": 190.0,
        "phoneme_duration": 0.18
      }
    }
  }
}
```

## Current Limitations / Behavior Notes

- `tracks` currently contribute strong timing/energy signals and macro musical motion; they are not a full sample-accurate instrument renderer.
- `Filter`, `Delay`, and `Reverb` nodes are deterministic simplified DSP models.
- `voice` is stylized phoneme/formant synthesis and should be authored as texture-first audio.

## Authoring Strategy for Reliable Results

- Prefer `synth_graph` for stable pitched tone design.
- Use `tracks` to drive `kick/bass/mid/high` energies for visual reactivity.
- Use conservative ranges first, then iterate:
  - `distortion.drive`: `0.2-1.8`
  - `reverb.room_size`: `0.4-1.2`
  - `envelope.release`: `0.05-0.8`
- Keep `seed` fixed while iterating prompts/configs to compare changes deterministically.

## Discoverability

For pattern-rich visual authoring alongside audio:
- `docs/sdk_ai/pattern_scene_generation.md`
- `docs/sdk_ai/harmonic_scene_generation.md`
- `docs/sdk_ai/rhythm_scene_generation.md`

