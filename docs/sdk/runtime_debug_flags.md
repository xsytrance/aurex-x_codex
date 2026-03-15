# Runtime Debug Flags (Engineering Contract)

These environment flags are for **engineering diagnostics and isolation only**. Defaults are conservative and keep runtime behavior unchanged.

## Flags

- `AUREX_STOP_AFTER_PROCEDURAL_STAGE` (`bool`, default `false`)
- `AUREX_STOP_AFTER_SDF_STAGE` (`bool`, default `false`)
- `AUREX_BYPASS_PROCEDURAL_SETUP` (`bool`, default `false`)
- `AUREX_DIAGNOSTIC_GPU_TRIANGLE` (`bool`, default `false`)
- `AUREX_FORCE_FLAT_RENDER` (`bool`, default `false`)
- `AUREX_DISABLE_AUDIO` (`bool`, default `false`)
- `AUREX_DISABLE_GPU_ERROR_SCOPES` (`bool`, default `false`)
- `AUREX_LOG_ONLY_PROCEDURAL_TRANSITION` (`bool`, default `false`)
- `AUREX_SKIP_ROOT_TREE_BUILD` (`bool`, default `false`)
- `AUREX_SKIP_PROCEDURAL_CAMERA` (`bool`, default `false`)
- `AUREX_GEOMETRY_SDF_MODE` (`flat|safe|legacy`, default `safe`)

## Geometry mode contract

`AUREX_GEOMETRY_SDF_MODE` maps to:

- `flat`: bypass geometry hits (diagnostic fallback view)
- `safe`: bounded deterministic sphere + plane path (no dynamic scene tree traversal)
- `legacy`: existing full scene tree geometry traversal

Invalid values fall back to `safe`.
