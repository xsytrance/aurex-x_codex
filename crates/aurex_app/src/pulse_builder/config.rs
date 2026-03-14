use aurex_render::rhythm_field::VisualTheme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeometryStyle {
    City,
    Lounge,
    Dreamscape,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AtmosphereType {
    Clear,
    Hazy,
    Foggy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightingMode {
    NeonPulse,
    WarmGlow,
    DiffuseSoft,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructureSet {
    Dense,
    Sparse,
    Minimal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CameraRig {
    Orbit,
    Drift,
    Float,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PulseConfig {
    pub name: String,
    pub seed: u64,
    pub theme: VisualTheme,
    pub geometry_style: GeometryStyle,
    pub atmosphere_type: AtmosphereType,
    pub lighting_mode: LightingMode,
    pub structure_set: StructureSet,
    pub color_palette: String,
    pub camera_rig: CameraRig,
    pub rhythm_intensity: f32,
    pub particle_density_multiplier: f32,
}

impl Default for PulseConfig {
    fn default() -> Self {
        Self {
            name: "Untitled Pulse".to_string(),
            seed: 0,
            theme: VisualTheme::Electronic,
            geometry_style: GeometryStyle::City,
            atmosphere_type: AtmosphereType::Clear,
            lighting_mode: LightingMode::DiffuseSoft,
            structure_set: StructureSet::Sparse,
            color_palette: "default_palette".to_string(),
            camera_rig: CameraRig::Drift,
            rhythm_intensity: 0.5,
            particle_density_multiplier: 0.5,
        }
    }
}
