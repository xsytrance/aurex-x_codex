use std::{error::Error, fmt};

use crate::{
    MidiNote, event_parser::flatten_events, note_builder::extract_notes_and_controls,
    smf_reader::parse_smf, tempo_map::extract_tempo_map,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TempoEvent {
    pub tick: u64,
    pub microseconds_per_beat: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlChangeEvent {
    pub tick: u64,
    pub channel: u8,
    pub controller: u8,
    pub value: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgramChangeEvent {
    pub tick: u64,
    pub channel: u8,
    pub program: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MidiTimeline {
    pub ticks_per_beat: u16,
    pub tempo_map: Vec<TempoEvent>,
    pub notes: Vec<MidiNote>,
    pub control_changes: Vec<ControlChangeEvent>,
    pub program_changes: Vec<ProgramChangeEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MidiError {
    InvalidHeaderChunk,
    InvalidHeaderLength(u32),
    UnsupportedFormat(u16),
    UnsupportedTimeDivision(u16),
    InvalidTrackChunk,
    UnsupportedStatus(u8),
    RunningStatusWithoutPrevious,
    UnexpectedEof,
    VlqTooLong,
}

impl fmt::Display for MidiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidHeaderChunk => write!(f, "invalid SMF header chunk"),
            Self::InvalidHeaderLength(len) => write!(f, "invalid SMF header length: {len}"),
            Self::UnsupportedFormat(format) => write!(f, "unsupported SMF format: {format}"),
            Self::UnsupportedTimeDivision(division) => {
                write!(f, "unsupported SMF time division: {division}")
            }
            Self::InvalidTrackChunk => write!(f, "invalid SMF track chunk"),
            Self::UnsupportedStatus(status) => write!(f, "unsupported MIDI status byte: {status}"),
            Self::RunningStatusWithoutPrevious => {
                write!(f, "running status encountered without previous status")
            }
            Self::UnexpectedEof => write!(f, "unexpected EOF while parsing SMF"),
            Self::VlqTooLong => write!(f, "variable-length quantity is too long"),
        }
    }
}

impl Error for MidiError {}

pub fn load_midi_timeline(bytes: &[u8]) -> Result<MidiTimeline, MidiError> {
    let smf = parse_smf(bytes)?;
    let flattened = flatten_events(&smf);
    let tempo_map = extract_tempo_map(&flattened);
    let (notes, control_changes, program_changes) = extract_notes_and_controls(&flattened);

    Ok(MidiTimeline {
        ticks_per_beat: smf.ticks_per_beat,
        tempo_map,
        notes,
        control_changes,
        program_changes,
    })
}
