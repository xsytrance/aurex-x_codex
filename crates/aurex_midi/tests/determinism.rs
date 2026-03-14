use aurex_midi::MidiFile;

#[test]
fn fixture_json_parses_and_normalizes_stably() {
    let json = r#"{
      "ticks_per_quarter": 480,
      "tracks": [
        {
          "name": "bass",
          "notes": [
            { "start_tick": 480, "duration_ticks": 120, "key": 40, "velocity": 96, "channel": 1 },
            { "start_tick": 0, "duration_ticks": 240, "key": 36, "velocity": 100, "channel": 1 }
          ]
        }
      ]
    }"#;

    let a = MidiFile::from_json_str(json)
        .expect("fixture should parse")
        .normalized();
    let b = MidiFile::from_json_str(json)
        .expect("fixture should parse")
        .normalized();

    assert_eq!(a, b);
    assert_eq!(a.note_count(), 2);
    assert_eq!(a.tracks[0].notes[0].start_tick, 0);
}
