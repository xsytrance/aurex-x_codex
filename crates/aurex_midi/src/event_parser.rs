use crate::smf_reader::{SmfFile, TrackEvent};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedEvent {
    pub tick: u64,
    pub track_index: usize,
    pub order_in_track: usize,
    pub event: TrackEvent,
}

pub fn flatten_events(smf: &SmfFile) -> Vec<ParsedEvent> {
    let mut out = Vec::new();
    for (track_index, track) in smf.tracks.iter().enumerate() {
        for (order_in_track, event) in track.events.iter().enumerate() {
            out.push(ParsedEvent {
                tick: event.tick,
                track_index,
                order_in_track,
                event: event.clone(),
            });
        }
    }

    out.sort_by(|a, b| {
        a.tick
            .cmp(&b.tick)
            .then(a.track_index.cmp(&b.track_index))
            .then(a.order_in_track.cmp(&b.order_in_track))
    });
    out
}
