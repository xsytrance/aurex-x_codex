# SDF System

`SdfScene` is the renderer-agnostic signed-distance scene contract.

## Structure
- `root_node`: `GeometryPipeline`
- `materials`: list of `SdfMaterial`
- `lighting`: `SdfLighting`

The structure is backend-neutral so raymarch, raster, or hybrid backends can interpret the same IR.

## Example JSON
```json
{
  "name": "minimal_sdf",
  "seed": 1337,
  "sdf_scene": {
    "root_node": {
      "base": {"type":"sphere","radius":{"kind":"value","value":1.0},"seed":null},
      "modifiers": []
    },
    "materials": [{"id":"core","albedo":[0.2,0.8,1.0],"roughness":0.3,"metallic":0.7,"emissive":[0.0,0.2,0.5]}],
    "lighting": {"ambient":[0.05,0.05,0.07],"key_lights":[]}
  },
  "audio_sync_bindings": {"reactive_parameters":[]}
}
```
