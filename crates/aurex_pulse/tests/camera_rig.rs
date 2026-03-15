use aurex_pulse::camera_rig::CameraRig;

#[test]
fn camera_orbit_is_deterministic() {
    let mut a = CameraRig::new();
    let mut b = CameraRig::new();

    let deltas = [0.1_f32, 0.2, 0.4, 0.15, 0.3, 0.5];
    for dt in deltas {
        a.update(dt);
        b.update(dt);
        assert_eq!(a.position, b.position);
        assert_eq!(a.time, b.time);
    }
}

#[test]
fn camera_time_progresses_monotonically() {
    let mut rig = CameraRig::new();
    let mut previous = rig.time;

    for dt in [0.0_f32, 0.016, 0.2, 0.75, 0.001] {
        rig.update(dt);
        assert!(rig.time >= previous);
        previous = rig.time;
    }

    rig.update(-1.0);
    assert!(rig.time >= previous);
}

#[test]
fn camera_position_updates_smoothly() {
    let mut rig = CameraRig::new();
    rig.update(1.0 / 60.0);
    let mut previous_position = rig.position;

    for _ in 0..120 {
        rig.update(1.0 / 60.0);

        let dx = (rig.position[0] - previous_position[0]).abs();
        let dz = (rig.position[2] - previous_position[2]).abs();

        assert!(dx < 1.0, "x delta should remain smooth: {dx}");
        assert!(dz < 1.0, "z delta should remain smooth: {dz}");

        previous_position = rig.position;
    }
}
