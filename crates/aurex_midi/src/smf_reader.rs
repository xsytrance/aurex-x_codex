use crate::timeline::MidiError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SmfFile {
    pub format: u16,
    pub ticks_per_beat: u16,
    pub tracks: Vec<SmfTrack>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SmfTrack {
    pub events: Vec<TrackEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrackEvent {
    pub tick: u64,
    pub kind: TrackEventKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrackEventKind {
    NoteOn {
        channel: u8,
        key: u8,
        velocity: u8,
    },
    NoteOff {
        channel: u8,
        key: u8,
        velocity: u8,
    },
    ControlChange {
        channel: u8,
        controller: u8,
        value: u8,
    },
    ProgramChange {
        channel: u8,
        program: u8,
    },
    TempoMeta {
        microseconds_per_beat: u32,
    },
}

pub fn parse_smf(bytes: &[u8]) -> Result<SmfFile, MidiError> {
    let mut cursor = Cursor::new(bytes);
    let header_id = cursor.read_exact(4)?;
    if header_id != b"MThd" {
        return Err(MidiError::InvalidHeaderChunk);
    }

    let header_len = cursor.read_u32_be()?;
    if header_len != 6 {
        return Err(MidiError::InvalidHeaderLength(header_len));
    }

    let format = cursor.read_u16_be()?;
    if format > 1 {
        return Err(MidiError::UnsupportedFormat(format));
    }
    let track_count = cursor.read_u16_be()?;
    let division = cursor.read_u16_be()?;
    if division & 0x8000 != 0 {
        return Err(MidiError::UnsupportedTimeDivision(division));
    }

    let mut tracks = Vec::with_capacity(usize::from(track_count));
    for _ in 0..track_count {
        let track_id = cursor.read_exact(4)?;
        if track_id != b"MTrk" {
            return Err(MidiError::InvalidTrackChunk);
        }
        let track_len = cursor.read_u32_be()?;
        let track_bytes = cursor.read_slice(track_len as usize)?;
        tracks.push(parse_track(track_bytes)?);
    }

    Ok(SmfFile {
        format,
        ticks_per_beat: division,
        tracks,
    })
}

fn parse_track(bytes: &[u8]) -> Result<SmfTrack, MidiError> {
    let mut cursor = Cursor::new(bytes);
    let mut tick = 0_u64;
    let mut running_status: Option<u8> = None;
    let mut events = Vec::new();

    while !cursor.is_eof() {
        let delta = read_vlq(&mut cursor)?;
        tick = tick.saturating_add(u64::from(delta));

        let first = cursor.read_u8()?;
        let status = if first & 0x80 != 0 {
            running_status = Some(first);
            first
        } else {
            let Some(prev) = running_status else {
                return Err(MidiError::RunningStatusWithoutPrevious);
            };
            cursor.rewind(1)?;
            prev
        };

        if status == 0xFF {
            let meta_type = cursor.read_u8()?;
            let len = read_vlq(&mut cursor)? as usize;
            let data = cursor.read_slice(len)?;
            if meta_type == 0x51 && data.len() == 3 {
                let microseconds_per_beat =
                    u32::from(data[0]) << 16 | u32::from(data[1]) << 8 | u32::from(data[2]);
                events.push(TrackEvent {
                    tick,
                    kind: TrackEventKind::TempoMeta {
                        microseconds_per_beat,
                    },
                });
            }
            if meta_type == 0x2F {
                break;
            }
            continue;
        }

        if status == 0xF0 || status == 0xF7 {
            let len = read_vlq(&mut cursor)? as usize;
            let _ = cursor.read_slice(len)?;
            continue;
        }

        let channel = status & 0x0F;
        match status & 0xF0 {
            0x80 => {
                let key = cursor.read_u8()?;
                let velocity = cursor.read_u8()?;
                events.push(TrackEvent {
                    tick,
                    kind: TrackEventKind::NoteOff {
                        channel,
                        key,
                        velocity,
                    },
                });
            }
            0x90 => {
                let key = cursor.read_u8()?;
                let velocity = cursor.read_u8()?;
                if velocity == 0 {
                    events.push(TrackEvent {
                        tick,
                        kind: TrackEventKind::NoteOff {
                            channel,
                            key,
                            velocity: 0,
                        },
                    });
                } else {
                    events.push(TrackEvent {
                        tick,
                        kind: TrackEventKind::NoteOn {
                            channel,
                            key,
                            velocity,
                        },
                    });
                }
            }
            0xB0 => {
                let controller = cursor.read_u8()?;
                let value = cursor.read_u8()?;
                events.push(TrackEvent {
                    tick,
                    kind: TrackEventKind::ControlChange {
                        channel,
                        controller,
                        value,
                    },
                });
            }
            0xC0 => {
                let program = cursor.read_u8()?;
                events.push(TrackEvent {
                    tick,
                    kind: TrackEventKind::ProgramChange { channel, program },
                });
            }
            0xA0 | 0xE0 => {
                let _ = cursor.read_u8()?;
                let _ = cursor.read_u8()?;
            }
            0xD0 => {
                let _ = cursor.read_u8()?;
            }
            _ => return Err(MidiError::UnsupportedStatus(status)),
        }
    }

    Ok(SmfTrack { events })
}

fn read_vlq(cursor: &mut Cursor<'_>) -> Result<u32, MidiError> {
    let mut value = 0_u32;
    for _ in 0..4 {
        let byte = cursor.read_u8()?;
        value = (value << 7) | u32::from(byte & 0x7F);
        if byte & 0x80 == 0 {
            return Ok(value);
        }
    }
    Err(MidiError::VlqTooLong)
}

struct Cursor<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, pos: 0 }
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.bytes.len()
    }

    fn read_exact(&mut self, len: usize) -> Result<&'a [u8], MidiError> {
        self.read_slice(len)
    }

    fn read_slice(&mut self, len: usize) -> Result<&'a [u8], MidiError> {
        let end = self.pos.saturating_add(len);
        if end > self.bytes.len() {
            return Err(MidiError::UnexpectedEof);
        }
        let slice = &self.bytes[self.pos..end];
        self.pos = end;
        Ok(slice)
    }

    fn read_u8(&mut self) -> Result<u8, MidiError> {
        let bytes = self.read_slice(1)?;
        Ok(bytes[0])
    }

    fn read_u16_be(&mut self) -> Result<u16, MidiError> {
        let bytes = self.read_slice(2)?;
        Ok(u16::from_be_bytes([bytes[0], bytes[1]]))
    }

    fn read_u32_be(&mut self) -> Result<u32, MidiError> {
        let bytes = self.read_slice(4)?;
        Ok(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    fn rewind(&mut self, count: usize) -> Result<(), MidiError> {
        if count > self.pos {
            return Err(MidiError::UnexpectedEof);
        }
        self.pos -= count;
        Ok(())
    }
}
