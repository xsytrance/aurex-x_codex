use aurex_scene::{Scene, SdfModifier, SdfNode};

use crate::pulse_blueprint::PulseBlueprint;

#[derive(Debug, Clone)]
pub struct BeatDriver {
    beat_ticks: Vec<u64>,
    bpm: f32,
    seconds_per_beat: f32,
    current_beat: usize,
    elapsed_seconds: f32,
}

impl BeatDriver {
    pub fn new(blueprint: &PulseBlueprint) -> Self {
        let bpm = blueprint.bpm.max(1.0);
        Self {
            beat_ticks: blueprint.beat_ticks.clone(),
            bpm,
            seconds_per_beat: 60.0 / bpm,
            current_beat: 0,
            elapsed_seconds: 0.0,
        }
    }

    pub fn update(&mut self, delta_seconds: f32) -> bool {
        self.elapsed_seconds = (self.elapsed_seconds + delta_seconds.max(0.0)).max(0.0);
        let mut triggered = false;

        while self.current_beat + 1 < self.beat_ticks.len() {
            let next_boundary = (self.current_beat + 1) as f32 * self.seconds_per_beat;
            if self.elapsed_seconds < next_boundary {
                break;
            }
            self.current_beat += 1;
            triggered = true;
        }

        triggered
    }

    pub fn current_beat(&self) -> usize {
        self.current_beat
    }

    pub fn bpm(&self) -> f32 {
        self.bpm
    }
}

pub fn apply_beat_pulse(scene: &mut Scene, intensity: f32) {
    let pulse = intensity.clamp(0.0, 2.0);
    let offset_delta = 0.02 + pulse * 0.03;
    let emissive_delta = 0.04 + pulse * 0.12;

    let SdfNode::Union { children } = &mut scene.sdf.root else {
        return;
    };

    for node in children {
        let SdfNode::Transform {
            modifiers, child, ..
        } = node
        else {
            continue;
        };

        for modifier in modifiers {
            if let SdfModifier::Translate { offset } = modifier {
                offset.y += offset_delta;
            }
        }

        if let SdfNode::Primitive { object } = child.as_mut() {
            object.material.emissive_strength =
                (object.material.emissive_strength + emissive_delta).clamp(0.0, 3.0);
        }
    }
}
