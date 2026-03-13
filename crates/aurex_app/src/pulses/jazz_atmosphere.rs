use super::ExamplePulseConfig;
use crate::pulse_builder::{
    AtmosphereType, CameraRig, GeometryStyle, LightingMode, PulseBuilder, StructureSet,
};
use aurex_render::rhythm_field::VisualTheme;

pub fn create_jazz_atmosphere_pulse(seed: u64) -> ExamplePulseConfig {
    PulseBuilder::new("Jazz Atmosphere")
        .seed(seed)
        .theme(VisualTheme::Jazz)
        .geometry_style(GeometryStyle::Lounge)
        .structures(StructureSet::Sparse)
        .lighting(LightingMode::WarmGlow)
        .atmosphere(AtmosphereType::Hazy)
        .color_palette("warm_amber_teal")
        .camera_rig(CameraRig::Drift)
        .rhythm_intensity(0.72)
        .particle_density_multiplier(0.24)
        .build()
}
