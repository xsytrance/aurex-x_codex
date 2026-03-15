use aurex_pulse::demo_sequencer::{DemoSequencer, DemoStageType};

#[test]
fn sequencer_progresses_through_stages() {
    let mut sequencer = DemoSequencer::new();
    assert_eq!(sequencer.current_stage_type(), DemoStageType::Bootstrap);

    let _ = sequencer.update(2.6);
    assert_eq!(
        sequencer.current_stage_type(),
        DemoStageType::ParticleFormation
    );

    let _ = sequencer.update(3.1);
    assert_eq!(sequencer.current_stage_type(), DemoStageType::LogoAssembly);

    let _ = sequencer.update(100.0);
    assert_eq!(
        sequencer.current_stage_type(),
        DemoStageType::RuntimeHandover
    );
}

#[test]
fn sequencer_is_deterministic() {
    let mut a = DemoSequencer::new();
    let mut b = DemoSequencer::new();

    let deltas = [0.16_f32, 0.24, 0.5, 1.2, 0.75, 2.0, 1.4];
    for dt in deltas {
        assert_eq!(a.update(dt), b.update(dt));
        assert_eq!(a.current_stage_type(), b.current_stage_type());
    }
}

#[test]
fn stage_transitions_occur_correctly() {
    let mut sequencer = DemoSequencer::new();

    assert_eq!(sequencer.update(2.49), None);
    assert_eq!(sequencer.current_stage_type(), DemoStageType::Bootstrap);

    let next = sequencer.update(0.02);
    assert_eq!(next, Some(DemoStageType::ParticleFormation));
    assert_eq!(
        sequencer.current_stage_type(),
        DemoStageType::ParticleFormation
    );

    let next = sequencer.update(3.0);
    assert_eq!(next, Some(DemoStageType::LogoAssembly));
}
