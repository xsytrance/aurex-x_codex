use super::{ExamplePulseConfig, WorldBlueprint, build_example_pulse};
use aurex_render::rhythm_field::{
    AtmosphereLayerParams, CameraLayerHints, GeneratorStackOutput, LightingLayerParams,
    ParticleLayerParams, SequencerState, StructureLayerParams, TerrainLayerParams, VisualTheme,
};

pub fn create_ambient_dreamscape_pulse(seed: u64) -> ExamplePulseConfig {
    let world_blueprint = WorldBlueprint {
        name: "Ambient Dreamscape",
        theme: VisualTheme::Ambient,
        palette_hint: "mist_blue_violet",
        camera_motion: "slow_floating_motion",
    };

    let stack_output = GeneratorStackOutput {
        terrain: TerrainLayerParams {
            amplitude_hint: 0.34,
            roughness_hint: 0.28,
        },
        structures: StructureLayerParams {
            density: 0.14,
            emissive: 0.18,
        },
        atmosphere: AtmosphereLayerParams {
            hue_drift: 0.72,
            fog_density: 0.82,
        },
        lighting: LightingLayerParams {
            flash_envelope: 0.15,
            exposure: 0.42,
        },
        particles: ParticleLayerParams {
            density_multiplier: 0.2,
            brightness: 0.26,
        },
        camera_hints: CameraLayerHints {
            drift: 0.68,
            fov_bias: 0.44,
        },
    };

    let sequencer_state = SequencerState {
        bpm: 72.0,
        beat_index: 6,
        bar_index: 2,
        bass_energy: 0.27,
        mid_energy: 0.34,
        high_energy: 0.22,
    };

    build_example_pulse(
        "Ambient Dreamscape",
        world_blueprint,
        seed,
        8.0,
        sequencer_state,
        stack_output,
    )
}
