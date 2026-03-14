use aurex_midi::load_midi_timeline;

fn chunk(tag: [u8; 4], payload: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(8 + payload.len());
    out.extend_from_slice(&tag);
    out.extend_from_slice(&(payload.len() as u32).to_be_bytes());
    out.extend_from_slice(payload);
    out
}

fn push_vlq(mut value: u32, out: &mut Vec<u8>) {
    let mut bytes = [0_u8; 4];
    let mut idx = 3;
    bytes[idx] = (value & 0x7F) as u8;
    while {
        value >>= 7;
        value > 0
    } {
        idx = idx.saturating_sub(1);
        bytes[idx] = ((value & 0x7F) as u8) | 0x80;
    }
    out.extend_from_slice(&bytes[idx..=3]);
}

fn format0_basic_midi() -> Vec<u8> {
    let mut smf = Vec::new();
    smf.extend_from_slice(&chunk(
        *b"MThd",
        &[
            0x00, 0x00, // format 0
            0x00, 0x01, // one track
            0x01, 0xE0, // 480 TPB
        ],
    ));

    let mut track = Vec::new();
    track.extend_from_slice(&[0x00, 0xFF, 0x51, 0x03, 0x07, 0xA1, 0x20]); // tempo 500000
    track.extend_from_slice(&[0x00, 0xC0, 0x05]); // program change
    track.extend_from_slice(&[0x00, 0x90, 0x3C, 0x64]); // note on C4 vel 100
    push_vlq(240, &mut track);
    track.extend_from_slice(&[0x80, 0x3C, 0x40]); // note off
    track.extend_from_slice(&[0x00, 0xB0, 0x01, 0x40]); // control change mod wheel
    track.extend_from_slice(&[0x00, 0xFF, 0x2F, 0x00]); // end of track

    smf.extend_from_slice(&chunk(*b"MTrk", &track));
    smf
}

fn format1_multitrack_midi() -> Vec<u8> {
    let mut smf = Vec::new();
    smf.extend_from_slice(&chunk(
        *b"MThd",
        &[
            0x00, 0x01, // format 1
            0x00, 0x02, // two tracks
            0x01, 0xE0, // 480 TPB
        ],
    ));

    let mut tempo_track = Vec::new();
    tempo_track.extend_from_slice(&[0x00, 0xFF, 0x51, 0x03, 0x07, 0xA1, 0x20]); // tempo 500000
    push_vlq(480, &mut tempo_track);
    tempo_track.extend_from_slice(&[0xFF, 0x51, 0x03, 0x09, 0x27, 0xC0]); // tempo 600000
    tempo_track.extend_from_slice(&[0x00, 0xFF, 0x2F, 0x00]); // end

    let mut note_track = Vec::new();
    note_track.extend_from_slice(&[0x00, 0xC1, 0x2A]); // program change channel 1
    note_track.extend_from_slice(&[0x00, 0xB1, 0x07, 0x64]); // control change channel volume
    note_track.extend_from_slice(&[0x00, 0x91, 0x3E, 0x50]); // note on D4
    push_vlq(240, &mut note_track);
    note_track.extend_from_slice(&[0x81, 0x3E, 0x40]); // note off
    note_track.extend_from_slice(&[0x00, 0x91, 0x41, 0x45]); // note on F4
    push_vlq(240, &mut note_track);
    note_track.extend_from_slice(&[0x81, 0x41, 0x40]); // note off
    note_track.extend_from_slice(&[0x00, 0xFF, 0x2F, 0x00]); // end

    smf.extend_from_slice(&chunk(*b"MTrk", &tempo_track));
    smf.extend_from_slice(&chunk(*b"MTrk", &note_track));
    smf
}

#[test]
fn midi_file_parses() {
    let bytes = format0_basic_midi();
    let timeline = load_midi_timeline(&bytes).expect("format0 fixture should parse");

    assert_eq!(timeline.ticks_per_beat, 480);
    assert_eq!(timeline.tempo_map.len(), 1);
    assert_eq!(timeline.notes.len(), 1);
    assert_eq!(timeline.control_changes.len(), 1);
    assert_eq!(timeline.program_changes.len(), 1);
}

#[test]
fn tempo_map_is_deterministic() {
    let bytes = format1_multitrack_midi();
    let a = load_midi_timeline(&bytes)
        .expect("format1 fixture should parse")
        .tempo_map;
    let b = load_midi_timeline(&bytes)
        .expect("format1 fixture should parse")
        .tempo_map;

    assert_eq!(a, b);
    assert_eq!(a.len(), 2);
    assert!(a[0].tick <= a[1].tick);
}

#[test]
fn note_pairing_is_correct() {
    let bytes = format0_basic_midi();
    let timeline = load_midi_timeline(&bytes).expect("format0 fixture should parse");

    assert_eq!(timeline.notes.len(), 1);
    let note = timeline.notes[0];
    assert_eq!(note.pitch, 60);
    assert_eq!(note.velocity, 100);
    assert_eq!(note.channel, 0);
    assert_eq!(note.start_tick, 0);
    assert_eq!(note.end_tick, 240);
}

#[test]
fn timeline_construction_is_stable() {
    let bytes = format1_multitrack_midi();
    let a = load_midi_timeline(&bytes).expect("format1 fixture should parse");
    let b = load_midi_timeline(&bytes).expect("format1 fixture should parse");

    assert_eq!(a, b);
    assert!(
        a.notes
            .windows(2)
            .all(|w| w[0].start_tick <= w[1].start_tick)
    );
    assert!(a.control_changes.windows(2).all(|w| w[0].tick <= w[1].tick));
    assert!(a.program_changes.windows(2).all(|w| w[0].tick <= w[1].tick));
}
