use super::{ExamplePulseConfig, WorldBlueprint, build_example_pulse};
use aurex_render::rhythm_field::{
    AtmosphereLayerParams, CameraLayerHints, GeneratorStackOutput, LightingLayerParams,
    ParticleLayerParams, SequencerState, StructureLayerParams, TerrainLayerParams, VisualTheme,
};

pub fn create_electronic_megacity_pulse(seed: u64) -> ExamplePulseConfig {
    let world_blueprint = WorldBlueprint {
        name: "Electronic Megacity",
        theme: VisualTheme::Electronic,
        palette_hint: "neon_cyan_magenta",
        camera_motion: "slow_orbital_center",
    };

    let stack_output = GeneratorStackOutput {
        terrain: TerrainLayerParams {
            amplitude_hint: 0.62,
            roughness_hint: 0.58,
        },
        structures: StructureLayerParams {
            density: 0.86,
            emissive: 0.72,
        },
        atmosphere: AtmosphereLayerParams {
            hue_drift: 0.48,
            fog_density: 0.3,
        },
        lighting: LightingLayerParams {
            flash_envelope: 0.8,
            exposure: 0.7,
        },
        particles: ParticleLayerParams {
            density_multiplier: 0.82,
            brightness: 0.74,
        },
        camera_hints: CameraLayerHints {
            drift: 0.45,
            fov_bias: 0.6,
        },
    };

    let sequencer_state = SequencerState {
        bpm: 132.0,
        beat_index: 12,
        bar_index: 3,
        bass_energy: 0.76,
        mid_energy: 0.58,
        high_energy: 0.71,
    };

    build_example_pulse(
        "Electronic Megacity",
        world_blueprint,
        seed,
        2.75,
        sequencer_state,
        stack_output,
    )
}
