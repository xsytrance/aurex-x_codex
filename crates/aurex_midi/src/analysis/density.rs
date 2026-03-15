use crate::MidiTimeline;

pub fn compute_note_density(timeline: &MidiTimeline) -> f32 {
    let total_notes = timeline.notes.len() as f32;
    if timeline.notes.is_empty() {
        return 0.0;
    }

    let ticks_per_beat = u64::from(timeline.ticks_per_beat.max(1));
    let duration_tick = timeline.notes.iter().map(|n| n.end_tick).max().unwrap_or(0);
    let total_beats = ((duration_tick.saturating_add(ticks_per_beat - 1)) / ticks_per_beat).max(1);

    total_notes / total_beats as f32
}

pub fn compute_velocity_average(timeline: &MidiTimeline) -> f32 {
    if timeline.notes.is_empty() {
        return 0.0;
    }

    let velocity_sum: u64 = timeline.notes.iter().map(|n| u64::from(n.velocity)).sum();
    let avg = velocity_sum as f32 / timeline.notes.len() as f32;
    avg / 127.0
}
