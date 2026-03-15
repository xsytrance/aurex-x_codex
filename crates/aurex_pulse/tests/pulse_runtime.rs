use aurex_pulse::PulseRuntime;

fn chunk(tag: [u8; 4], payload: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(8 + payload.len());
    out.extend_from_slice(&tag);
    out.extend_from_slice(&(payload.len() as u32).to_be_bytes());
    out.extend_from_slice(payload);
    out
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

    let track = vec![
        0x00, 0xFF, 0x51, 0x03, 0x07, 0xA1, 0x20, // tempo
        0x00, 0x90, 0x3C, 0x64, // note on
        0x81, 0x70, 0x80, 0x3C, 0x40, // delta 240 + note off
        0x00, 0xFF, 0x2F, 0x00, // end
    ];
    smf.extend_from_slice(&chunk(*b"MTrk", &track));
    smf
}

#[test]
fn pulse_runtime_is_deterministic() {
    let midi = format0_basic_midi();
    let runtime1 = PulseRuntime::from_midi_bytes(&midi).expect("runtime should parse midi");
    let runtime2 = PulseRuntime::from_midi_bytes(&midi).expect("runtime should parse midi");

    assert_eq!(runtime1.blueprint, runtime2.blueprint);

    let scene1 = runtime1.generate_scene();
    let scene2 = runtime2.generate_scene();
    assert_eq!(scene1, scene2);
}
