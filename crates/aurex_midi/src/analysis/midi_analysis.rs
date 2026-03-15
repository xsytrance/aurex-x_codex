use crate::{
    MidiTimeline,
    analysis::{
        density::{compute_note_density, compute_velocity_average},
        pitch::compute_pitch_range,
        rhythm::generate_beat_grid,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PitchRange {
    pub min: u8,
    pub max: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MidiAnalysis {
    pub bpm: f32,
    pub note_density: f32,
    pub pitch_range: PitchRange,
    pub beat_grid: Vec<u64>,
    pub rhythmic_intensity: f32,
}

pub fn analyze_timeline(timeline: &MidiTimeline) -> MidiAnalysis {
    let microseconds_per_beat = timeline
        .tempo_map
        .first()
        .map(|t| t.microseconds_per_beat)
        .unwrap_or(500_000);
    let bpm = 60_000_000.0 / microseconds_per_beat as f32;

    let note_density = compute_note_density(timeline);
    let pitch_range = compute_pitch_range(&timeline.notes);
    let beat_grid = generate_beat_grid(timeline);
    let rhythmic_intensity = note_density * compute_velocity_average(timeline);

    MidiAnalysis {
        bpm,
        note_density,
        pitch_range,
        beat_grid,
        rhythmic_intensity,
    }
}
