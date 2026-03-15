use aurex_midi::MidiAnalysis;

#[derive(Debug, Clone, PartialEq)]
pub struct PulseBlueprint {
    pub bpm: f32,
    pub beat_ticks: Vec<u64>,
    pub energy_level: f32,
    pub pitch_span: u8,
    pub density_level: f32,
}

pub fn blueprint_from_midi_analysis(analysis: &MidiAnalysis) -> PulseBlueprint {
    PulseBlueprint {
        bpm: analysis.bpm,
        beat_ticks: analysis.beat_grid.clone(),
        energy_level: analysis.rhythmic_intensity,
        pitch_span: analysis
            .pitch_range
            .max
            .saturating_sub(analysis.pitch_range.min),
        density_level: analysis.note_density,
    }
}

#[cfg(test)]
mod tests {
    use super::{PulseBlueprint, blueprint_from_midi_analysis};
    use aurex_midi::{MidiAnalysis, PitchRange};

    fn sample_analysis() -> MidiAnalysis {
        MidiAnalysis {
            bpm: 132.0,
            note_density: 1.5,
            pitch_range: PitchRange { min: 48, max: 72 },
            beat_grid: vec![0, 480, 960, 1440],
            rhythmic_intensity: 0.73,
        }
    }

    #[test]
    fn blueprint_generation_is_deterministic() {
        let analysis = sample_analysis();
        let a = blueprint_from_midi_analysis(&analysis);
        let b = blueprint_from_midi_analysis(&analysis);
        assert_eq!(a, b);
    }

    #[test]
    fn pitch_span_is_correct() {
        let analysis = sample_analysis();
        let blueprint = blueprint_from_midi_analysis(&analysis);
        assert_eq!(blueprint.pitch_span, 24);
    }

    #[test]
    fn energy_level_matches_analysis() {
        let analysis = sample_analysis();
        let blueprint: PulseBlueprint = blueprint_from_midi_analysis(&analysis);
        assert!((blueprint.energy_level - analysis.rhythmic_intensity).abs() < f32::EPSILON);
        assert!((blueprint.density_level - analysis.note_density).abs() < f32::EPSILON);
        assert!((blueprint.bpm - analysis.bpm).abs() < f32::EPSILON);
        assert_eq!(blueprint.beat_ticks, analysis.beat_grid);
    }
}
