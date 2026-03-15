use crate::{TempoEvent, event_parser::ParsedEvent, smf_reader::TrackEventKind};

pub fn extract_tempo_map(events: &[ParsedEvent]) -> Vec<TempoEvent> {
    let mut tempo_map = Vec::new();
    for event in events {
        if let TrackEventKind::TempoMeta {
            microseconds_per_beat,
        } = &event.event.kind
        {
            tempo_map.push(TempoEvent {
                tick: event.tick,
                microseconds_per_beat: *microseconds_per_beat,
            });
        }
    }

    tempo_map.sort_by(|a, b| {
        a.tick
            .cmp(&b.tick)
            .then(a.microseconds_per_beat.cmp(&b.microseconds_per_beat))
    });

    if tempo_map.is_empty() {
        tempo_map.push(TempoEvent {
            tick: 0,
            microseconds_per_beat: 500_000,
        });
    }

    tempo_map
}
