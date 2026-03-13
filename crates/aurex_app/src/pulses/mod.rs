pub mod ambient_dreamscape;
pub mod electronic_megacity;
pub mod jazz_atmosphere;

use aurex_render::rhythm_field::{
    GeneratorStackOutput, RhythmFieldSnapshot, SequencerState, VisualTheme,
    apply_rhythm_modulation, sample_rhythm_field,
};

#[derive(Debug, Clone, PartialEq)]
pub struct WorldBlueprint {
    pub name: &'static str,
    pub theme: VisualTheme,
    pub palette_hint: &'static str,
    pub camera_motion: &'static str,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExamplePulseConfig {
    pub pulse_name: &'static str,
    pub world_blueprint: WorldBlueprint,
    pub generator_output: GeneratorStackOutput,
    pub rhythm_snapshot: RhythmFieldSnapshot,
    pub modulated_output: GeneratorStackOutput,
}

pub fn build_example_pulse(
    pulse_name: &'static str,
    world_blueprint: WorldBlueprint,
    seed: u64,
    time: f32,
    sequencer_state: SequencerState,
    generator_output: GeneratorStackOutput,
) -> ExamplePulseConfig {
    let rhythm_snapshot = sample_rhythm_field(seed, time, sequencer_state);
    let modulated_output =
        apply_rhythm_modulation(&rhythm_snapshot, &generator_output, world_blueprint.theme);

    ExamplePulseConfig {
        pulse_name,
        world_blueprint,
        generator_output,
        rhythm_snapshot,
        modulated_output,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ambient_dreamscape::create_ambient_dreamscape_pulse,
        electronic_megacity::create_electronic_megacity_pulse,
        jazz_atmosphere::create_jazz_atmosphere_pulse,
    };

    #[test]
    fn pulse_initialization_is_deterministic_for_same_seed() {
        let ea = create_electronic_megacity_pulse(77);
        let eb = create_electronic_megacity_pulse(77);
        assert_eq!(ea, eb);

        let ja = create_jazz_atmosphere_pulse(77);
        let jb = create_jazz_atmosphere_pulse(77);
        assert_eq!(ja, jb);

        let aa = create_ambient_dreamscape_pulse(77);
        let ab = create_ambient_dreamscape_pulse(77);
        assert_eq!(aa, ab);
    }
}
