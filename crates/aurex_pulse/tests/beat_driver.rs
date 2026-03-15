use aurex_pulse::{beat_driver::BeatDriver, pulse_blueprint::PulseBlueprint};

fn sample_blueprint() -> PulseBlueprint {
    PulseBlueprint {
        bpm: 120.0,
        beat_ticks: vec![0, 480, 960, 1440, 1920],
        energy_level: 0.7,
        pitch_span: 24,
        density_level: 1.2,
    }
}

#[test]
fn beat_driver_progresses_monotonically() {
    let mut driver = BeatDriver::new(&sample_blueprint());
    assert!((driver.bpm() - 120.0).abs() < f32::EPSILON);

    assert_eq!(driver.current_beat(), 0);
    let _ = driver.update(0.5);
    assert_eq!(driver.current_beat(), 1);

    let _ = driver.update(0.5);
    assert_eq!(driver.current_beat(), 2);

    let _ = driver.update(0.5);
    assert_eq!(driver.current_beat(), 3);

    let _ = driver.update(0.5);
    assert_eq!(driver.current_beat(), 4);

    // No beats beyond beat_ticks len
    let _ = driver.update(100.0);
    assert_eq!(driver.current_beat(), 4);
}

#[test]
fn beat_driver_is_deterministic() {
    let blueprint = sample_blueprint();
    let mut a = BeatDriver::new(&blueprint);
    let mut b = BeatDriver::new(&blueprint);

    let deltas = [0.1_f32, 0.4, 0.12, 0.3, 0.08, 0.5, 0.5];
    for dt in deltas {
        assert_eq!(a.update(dt), b.update(dt));
        assert_eq!(a.current_beat(), b.current_beat());
    }
}

#[test]
fn beat_events_trigger_correctly() {
    let mut driver = BeatDriver::new(&sample_blueprint());

    // 0.49s should not cross first beat at 0.5s
    assert!(!driver.update(0.49));
    assert_eq!(driver.current_beat(), 0);

    // Crossing 0.5s triggers beat 1
    assert!(driver.update(0.02));
    assert_eq!(driver.current_beat(), 1);

    // Large delta can cross multiple beats, still reports trigger
    assert!(driver.update(1.2));
    assert!(driver.current_beat() >= 2);
}
