use super::ExamplePulseConfig;
use crate::pulse_builder::{CameraStyle, LightingStyle, PulseBuilder};
use aurex_render::rhythm_field::VisualTheme;

pub fn create_jazz_atmosphere_pulse(seed: u64) -> ExamplePulseConfig {
    PulseBuilder::new("Jazz Atmosphere")
        .theme(VisualTheme::Jazz)
        .seed(seed)
        .structure_density(0.36)
        .particle_intensity(0.24)
        .lighting_style(LightingStyle::Warm)
        .camera_style(CameraStyle::Drift)
        .rhythm_reactivity(0.72)
        .build()
}
