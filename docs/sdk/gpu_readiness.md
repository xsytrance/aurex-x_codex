# GPU Readiness (Technical SDK)

Aurex-X now includes preparatory GPU descriptor scaffolding in `aurex_render_sdf::gpu`:
- `GpuSceneDescriptor`
- `GpuMaterialDescriptor`
- `GpuPatternDescriptor`
- `GpuPipelineDescriptor`

These descriptors isolate renderer orchestration from evaluation kernels and provide migration anchors for future compute/wgpu backends.

Current status:
- CPU renderer remains authoritative.
- Descriptor translation path exists for staged migration.
