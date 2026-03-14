use std::collections::{BTreeMap, VecDeque};

use crate::{
    ControlChangeEvent, MidiNote, ProgramChangeEvent, event_parser::ParsedEvent,
    smf_reader::TrackEventKind,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct NoteKey {
    channel: u8,
    pitch: u8,
}

#[derive(Debug, Clone, Copy)]
struct PendingNote {
    start_tick: u64,
    velocity: u8,
}

pub fn extract_notes_and_controls(
    events: &[ParsedEvent],
) -> (
    Vec<MidiNote>,
    Vec<ControlChangeEvent>,
    Vec<ProgramChangeEvent>,
) {
    let mut open_notes: BTreeMap<NoteKey, VecDeque<PendingNote>> = BTreeMap::new();
    let mut notes = Vec::new();
    let mut control_changes = Vec::new();
    let mut program_changes = Vec::new();

    for parsed in events {
        match &parsed.event.kind {
            TrackEventKind::NoteOn {
                channel,
                key,
                velocity,
            } => {
                open_notes
                    .entry(NoteKey {
                        channel: *channel,
                        pitch: *key,
                    })
                    .or_default()
                    .push_back(PendingNote {
                        start_tick: parsed.tick,
                        velocity: *velocity,
                    });
            }
            TrackEventKind::NoteOff { channel, key, .. } => {
                let note_key = NoteKey {
                    channel: *channel,
                    pitch: *key,
                };
                let Some(queue) = open_notes.get_mut(&note_key) else {
                    continue;
                };
                let Some(pending) = queue.pop_front() else {
                    continue;
                };
                let end_tick = parsed.tick.max(pending.start_tick);
                notes.push(MidiNote {
                    pitch: *key,
                    velocity: pending.velocity,
                    start_tick: pending.start_tick,
                    end_tick,
                    channel: *channel,
                });
            }
            TrackEventKind::ControlChange {
                channel,
                controller,
                value,
            } => {
                control_changes.push(ControlChangeEvent {
                    tick: parsed.tick,
                    channel: *channel,
                    controller: *controller,
                    value: *value,
                });
            }
            TrackEventKind::ProgramChange { channel, program } => {
                program_changes.push(ProgramChangeEvent {
                    tick: parsed.tick,
                    channel: *channel,
                    program: *program,
                });
            }
            TrackEventKind::TempoMeta { .. } => {}
        }
    }

    for (key, queue) in open_notes {
        for pending in queue {
            notes.push(MidiNote {
                pitch: key.pitch,
                velocity: pending.velocity,
                start_tick: pending.start_tick,
                end_tick: pending.start_tick,
                channel: key.channel,
            });
        }
    }

    notes.sort_by(|a, b| {
        a.start_tick
            .cmp(&b.start_tick)
            .then(a.end_tick.cmp(&b.end_tick))
            .then(a.channel.cmp(&b.channel))
            .then(a.pitch.cmp(&b.pitch))
            .then(a.velocity.cmp(&b.velocity))
    });

    control_changes.sort_by(|a, b| {
        a.tick
            .cmp(&b.tick)
            .then(a.channel.cmp(&b.channel))
            .then(a.controller.cmp(&b.controller))
            .then(a.value.cmp(&b.value))
    });

    program_changes.sort_by(|a, b| {
        a.tick
            .cmp(&b.tick)
            .then(a.channel.cmp(&b.channel))
            .then(a.program.cmp(&b.program))
    });

    (notes, control_changes, program_changes)
}
