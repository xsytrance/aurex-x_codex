use crate::RenderConfig;

#[derive(Debug, Clone, Default)]
pub struct GpuSceneDescriptor {
    pub object_count: u32,
    pub field_count: u32,
    pub pattern_count: u32,
}

#[derive(Debug, Clone, Default)]
pub struct GpuMaterialDescriptor {
    pub material_count: u32,
}

#[derive(Debug, Clone, Default)]
pub struct GpuPatternDescriptor {
    pub layer_count: u32,
}

#[derive(Debug, Clone, Default)]
pub struct GpuPipelineDescriptor {
    pub width: u32,
    pub height: u32,
    pub max_steps: u32,
}

pub fn gpu_pipeline_descriptor_from_config(config: RenderConfig) -> GpuPipelineDescriptor {
    GpuPipelineDescriptor {
        width: config.width,
        height: config.height,
        max_steps: config.max_steps,
    }
}
