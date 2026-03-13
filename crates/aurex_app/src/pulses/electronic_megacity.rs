use super::ExamplePulseConfig;
use crate::pulse_builder::{CameraStyle, LightingStyle, PulseBuilder};
use aurex_render::rhythm_field::VisualTheme;

pub fn create_electronic_megacity_pulse(seed: u64) -> ExamplePulseConfig {
    PulseBuilder::new("Electronic Megacity")
        .theme(VisualTheme::Electronic)
        .seed(seed)
        .structure_density(0.86)
        .particle_intensity(0.82)
        .lighting_style(LightingStyle::Neon)
        .camera_style(CameraStyle::Orbit)
        .rhythm_reactivity(1.0)
        .build()
}
