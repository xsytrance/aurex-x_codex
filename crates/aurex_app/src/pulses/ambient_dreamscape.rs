use super::ExamplePulseConfig;
use crate::pulse_builder::{CameraStyle, LightingStyle, PulseBuilder};
use aurex_render::rhythm_field::VisualTheme;

pub fn create_ambient_dreamscape_pulse(seed: u64) -> ExamplePulseConfig {
    PulseBuilder::new("Ambient Dreamscape")
        .theme(VisualTheme::Ambient)
        .seed(seed)
        .structure_density(0.14)
        .particle_intensity(0.2)
        .lighting_style(LightingStyle::Diffuse)
        .camera_style(CameraStyle::Float)
        .rhythm_reactivity(0.6)
        .build()
}
