use aurex_scene::{
    SdfNode,
    scene_generator::{PulseBlueprint, generate_scene_from_blueprint},
};

fn sample_blueprint() -> PulseBlueprint {
    PulseBlueprint {
        bpm: 128.0,
        beat_ticks: vec![0, 480, 960, 1440],
        energy_level: 0.6,
        pitch_span: 24,
        density_level: 0.5,
    }
}

fn pillar_count(scene: &aurex_scene::Scene) -> usize {
    match &scene.sdf.root {
        SdfNode::Union { children } => children
            .iter()
            .filter(|n| matches!(n, SdfNode::Transform { .. }))
            .count(),
        _ => 0,
    }
}

fn first_pillar_half_height(scene: &aurex_scene::Scene) -> Option<f32> {
    let SdfNode::Union { children } = &scene.sdf.root else {
        return None;
    };

    for node in children {
        let SdfNode::Transform { child, .. } = node else {
            continue;
        };
        let SdfNode::Primitive { object } = child.as_ref() else {
            continue;
        };
        let aurex_scene::SdfPrimitive::Box { size } = object.primitive else {
            continue;
        };
        return Some(size.y);
    }

    None
}

#[test]
fn scene_generation_is_deterministic() {
    let blueprint = sample_blueprint();
    let scene1 = generate_scene_from_blueprint(&blueprint);
    let scene2 = generate_scene_from_blueprint(&blueprint);

    assert_eq!(scene1, scene2);
}

#[test]
fn pillar_count_scales_with_density() {
    let mut low = sample_blueprint();
    low.density_level = 0.2;
    let mut high = sample_blueprint();
    high.density_level = 0.9;

    let low_scene = generate_scene_from_blueprint(&low);
    let high_scene = generate_scene_from_blueprint(&high);

    assert!(pillar_count(&low_scene) < pillar_count(&high_scene));
}

#[test]
fn pillar_height_scales_with_pitch_span() {
    let mut low = sample_blueprint();
    low.pitch_span = 8;
    let mut high = sample_blueprint();
    high.pitch_span = 40;

    let low_scene = generate_scene_from_blueprint(&low);
    let high_scene = generate_scene_from_blueprint(&high);

    let low_h = first_pillar_half_height(&low_scene).expect("low scene has pillars");
    let high_h = first_pillar_half_height(&high_scene).expect("high scene has pillars");

    assert!(low_h < high_h);
}
