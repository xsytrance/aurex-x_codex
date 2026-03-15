use crate::MidiTimeline;

pub fn generate_beat_grid(timeline: &MidiTimeline) -> Vec<u64> {
    let ticks_per_beat = u64::from(timeline.ticks_per_beat.max(1));
    let last_tick = timeline
        .notes
        .iter()
        .map(|n| n.end_tick)
        .chain(timeline.control_changes.iter().map(|e| e.tick))
        .chain(timeline.program_changes.iter().map(|e| e.tick))
        .chain(timeline.tempo_map.iter().map(|e| e.tick))
        .max()
        .unwrap_or(0);

    let mut beat_grid = Vec::new();
    let mut tick = 0_u64;
    while tick <= last_tick {
        beat_grid.push(tick);
        tick = tick.saturating_add(ticks_per_beat);
        if tick == u64::MAX {
            break;
        }
    }

    if beat_grid.is_empty() {
        beat_grid.push(0);
    }

    beat_grid
}
