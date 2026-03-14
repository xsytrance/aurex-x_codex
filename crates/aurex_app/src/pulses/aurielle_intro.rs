use super::ExamplePulseConfig;
use crate::pulse_builder::{
    AtmosphereType, CameraRig, GeometryStyle, LightingMode, PulseBuilder, StructureSet,
};
use crate::pulse_sequence::{PulsePhaseOverrides, PulseSequence};
use aurex_render::rhythm_field::VisualTheme;

pub fn create_aurielle_intro_pulse(seed: u64) -> ExamplePulseConfig {
    create_aurielle_intro_pulse_at_time(seed, 0.0)
}

pub fn create_aurielle_intro_pulse_at_time(seed: u64, elapsed_seconds: f32) -> ExamplePulseConfig {
    let sequence = PulseSequence::new()
        .add_phase("Silence", 2.0)
        .add_phase_with_overrides(
            "Aurielle Appears",
            4.0,
            PulsePhaseOverrides {
                lighting_override: Some(LightingMode::WarmGlow),
                atmosphere_override: Some(AtmosphereType::Hazy),
                particle_override: Some(0.6),
                camera_override: Some(CameraRig::Orbit),
                rhythm_intensity_override: Some(0.8),
                structure_override: Some(StructureSet::Sparse),
            },
        )
        .add_phase("Maestros Reveal", 4.0)
        .add_phase("Logo Formation", 3.0)
        .add_phase("Menu Transition", 5.0);

    PulseBuilder::new("Aurielle Intro")
        .seed(seed)
        .theme(VisualTheme::Electronic)
        .geometry_style(GeometryStyle::City)
        .structures(StructureSet::Dense)
        .lighting(LightingMode::NeonPulse)
        .atmosphere(AtmosphereType::Clear)
        .color_palette("aurielle_gold_cyan")
        .camera_rig(CameraRig::Orbit)
        .rhythm_intensity(0.9)
        .particle_density_multiplier(0.7)
        .sequence(sequence)
        .build_at_time(elapsed_seconds)
}
