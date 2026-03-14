use super::ExamplePulseConfig;
use crate::pulse_builder::{
    AtmosphereType, CameraRig, GeometryStyle, LightingMode, PulseBuilder, StructureSet,
};
use aurex_render::rhythm_field::VisualTheme;

pub fn create_ambient_dreamscape_pulse(seed: u64) -> ExamplePulseConfig {
    PulseBuilder::new("Ambient Dreamscape")
        .seed(seed)
        .theme(VisualTheme::Ambient)
        .geometry_style(GeometryStyle::Dreamscape)
        .structures(StructureSet::Minimal)
        .lighting(LightingMode::DiffuseSoft)
        .atmosphere(AtmosphereType::Foggy)
        .color_palette("mist_blue_violet")
        .camera_rig(CameraRig::Float)
        .rhythm_intensity(0.6)
        .particle_density_multiplier(0.2)
        .build()
}
