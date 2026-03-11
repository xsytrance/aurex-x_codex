# Scene IR Specification

The Scene IR is a minimal JSON format for defining demo scenes as pure data.

## Top-level object: `scene`

| Field | Type | Required | Description |
|---|---|---|---|
| `name` | string | yes | Stable scene identifier. |
| `nodes` | array of `SceneNode` | yes | Node instances and parameters. |
| `camera` | `CameraDefinition` | no | Camera path waypoints. |
| `sync` | `AudioSyncBindings` | no | Audio signal routing hints. |

## `nodes`

Each node entry has:

- `id: string` unique node reference within a scene.
- `node_type: string` node implementation/type name.
- `parameters: object<string, float>` numeric parameters.

## `camera`

`camera.path` is a list of 3D points:

```json
{
  "camera": {
    "path": [
      [0, 0, 10],
      [5, 2, 4],
      [12, 0, -6]
    ]
  }
}
```

## `sync`

`sync` maps high-level audio events to target paths (string handles), for example:

- `kick`
- `snare`
- `bass`

Each is optional.

## Complete example

```json
{
  "name": "neon_tunnel",
  "nodes": [
    {
      "id": "tunnel1",
      "node_type": "TunnelGenerator",
      "parameters": {
        "radius": 5.0,
        "twist": 0.2,
        "glow": 3.0
      }
    },
    {
      "id": "sparks",
      "node_type": "ParticleEmitter",
      "parameters": {
        "spawn_rate": 120.0,
        "speed": 3.5,
        "lifetime": 1.2
      }
    }
  ],
  "camera": {
    "path": [
      [0.0, 0.0, 10.0],
      [5.0, 2.0, 4.0],
      [12.0, 0.0, -6.0]
    ]
  },
  "sync": {
    "kick": "tunnel1.glow",
    "snare": "sparks.spawn_rate",
    "bass": "tunnel1.radius"
  }
}
```
