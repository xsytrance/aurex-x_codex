use super::ExamplePulseConfig;
use crate::pulse_builder::{
    AtmosphereType, CameraRig, GeometryStyle, LightingMode, PulseBuilder, StructureSet,
};
use aurex_render::rhythm_field::VisualTheme;

pub fn create_electronic_megacity_pulse(seed: u64) -> ExamplePulseConfig {
    PulseBuilder::new("Electronic Megacity")
        .seed(seed)
        .theme(VisualTheme::Electronic)
        .geometry_style(GeometryStyle::City)
        .structures(StructureSet::Dense)
        .lighting(LightingMode::NeonPulse)
        .atmosphere(AtmosphereType::Clear)
        .color_palette("neon_cyan_magenta")
        .camera_rig(CameraRig::Orbit)
        .rhythm_intensity(1.0)
        .particle_density_multiplier(0.82)
        .build()
}
