use super::{ExamplePulseConfig, WorldBlueprint, build_example_pulse};
use aurex_render::rhythm_field::{
    AtmosphereLayerParams, CameraLayerHints, GeneratorStackOutput, LightingLayerParams,
    ParticleLayerParams, SequencerState, StructureLayerParams, TerrainLayerParams, VisualTheme,
};

pub fn create_jazz_atmosphere_pulse(seed: u64) -> ExamplePulseConfig {
    let world_blueprint = WorldBlueprint {
        name: "Jazz Atmosphere",
        theme: VisualTheme::Jazz,
        palette_hint: "warm_amber_teal",
        camera_motion: "gentle_drift_pan",
    };

    let stack_output = GeneratorStackOutput {
        terrain: TerrainLayerParams {
            amplitude_hint: 0.42,
            roughness_hint: 0.34,
        },
        structures: StructureLayerParams {
            density: 0.36,
            emissive: 0.28,
        },
        atmosphere: AtmosphereLayerParams {
            hue_drift: 0.66,
            fog_density: 0.74,
        },
        lighting: LightingLayerParams {
            flash_envelope: 0.24,
            exposure: 0.56,
        },
        particles: ParticleLayerParams {
            density_multiplier: 0.24,
            brightness: 0.32,
        },
        camera_hints: CameraLayerHints {
            drift: 0.62,
            fov_bias: 0.5,
        },
    };

    let sequencer_state = SequencerState {
        bpm: 96.0,
        beat_index: 9,
        bar_index: 4,
        bass_energy: 0.38,
        mid_energy: 0.64,
        high_energy: 0.29,
    };

    build_example_pulse(
        "Jazz Atmosphere",
        world_blueprint,
        seed,
        5.125,
        sequencer_state,
        stack_output,
    )
}
