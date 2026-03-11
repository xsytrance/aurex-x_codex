# Audio Reactive System

`AudioSyncBindings` maps audio events to scene parameters deterministically.

## AudioReactiveParameter
- `node`: target node identifier
- `parameter`: parameter path
- `audio_source`: `kick|snare|bass|hi_hat|custom`
- `multiplier`: scalar response
- `offset`: additive baseline

## Example JSON
```json
{
  "reactive_parameters": [
    {"node":"TunnelGenerator","parameter":"radius","audio_source":"bass","multiplier":1.5,"offset":0.0},
    {"node":"CameraRig","parameter":"shake","audio_source":"snare","multiplier":0.5,"offset":0.1}
  ]
}
```

Expected visual effect: bass expands tunnel radius while snare drives camera jitter.
