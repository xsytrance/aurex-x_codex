use aurex_scene::particle_swarm::ParticleSwarm;

#[test]
fn swarm_initialization_is_deterministic() {
    let a = ParticleSwarm::new(2026, 128);
    let b = ParticleSwarm::new(2026, 128);
    assert_eq!(a.particles(), b.particles());
}

#[test]
fn particle_motion_is_deterministic() {
    let mut a = ParticleSwarm::new(77, 64);
    let mut b = ParticleSwarm::new(77, 64);

    let targets = vec![[0.0_f32, 4.0, 0.0], [3.0, 5.0, -2.0], [-4.0, 3.0, 1.0]];
    a.set_targets(&targets);
    b.set_targets(&targets);

    for dt in [0.016_f32, 0.016, 0.033, 0.008, 0.025] {
        a.update(dt);
        b.update(dt);
    }

    assert_eq!(a.particles(), b.particles());
}

#[test]
fn target_assignment_is_stable() {
    let mut a = ParticleSwarm::new(14, 10);
    let mut b = ParticleSwarm::new(14, 10);
    let targets = vec![[1.0_f32, 2.0, 3.0], [-1.0, 0.5, 2.5], [4.0, 1.5, -3.5]];

    a.set_targets(&targets);
    b.set_targets(&targets);
    assert_eq!(a.particles(), b.particles());
}
