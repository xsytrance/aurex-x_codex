use aurex_scene::{Scene, SdfModifier, SdfNode};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DemoStage {
    pub start_time: f32,
    pub duration: f32,
    pub stage_type: DemoStageType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DemoStageType {
    Bootstrap,
    ParticleFormation,
    LogoAssembly,
    LogoReveal,
    EnergyPulse,
    SceneCollapse,
    RuntimeHandover,
}

#[derive(Debug, Clone)]
pub struct DemoSequencer {
    stages: Vec<DemoStage>,
    current_stage: usize,
    elapsed_time: f32,
}

impl DemoSequencer {
    pub fn new() -> Self {
        let sequence = [
            (DemoStageType::Bootstrap, 2.5_f32),
            (DemoStageType::ParticleFormation, 3.0_f32),
            (DemoStageType::LogoAssembly, 3.5_f32),
            (DemoStageType::LogoReveal, 2.5_f32),
            (DemoStageType::EnergyPulse, 2.0_f32),
            (DemoStageType::SceneCollapse, 2.5_f32),
            (DemoStageType::RuntimeHandover, 3.0_f32),
        ];

        let mut stages = Vec::with_capacity(sequence.len());
        let mut start = 0.0_f32;
        for (stage_type, duration) in sequence {
            stages.push(DemoStage {
                start_time: start,
                duration,
                stage_type,
            });
            start += duration;
        }

        Self {
            stages,
            current_stage: 0,
            elapsed_time: 0.0,
        }
    }

    pub fn update(&mut self, delta_time: f32) -> Option<DemoStageType> {
        self.elapsed_time = (self.elapsed_time + delta_time.max(0.0)).max(0.0);
        let mut transitioned_to: Option<DemoStageType> = None;

        while self.current_stage + 1 < self.stages.len() {
            let end_time = self.stages[self.current_stage].start_time
                + self.stages[self.current_stage].duration;
            if self.elapsed_time < end_time {
                break;
            }
            self.current_stage += 1;
            transitioned_to = Some(self.stages[self.current_stage].stage_type);
        }

        transitioned_to
    }

    pub fn current_stage_type(&self) -> DemoStageType {
        self.stages[self.current_stage].stage_type
    }
}

impl Default for DemoSequencer {
    fn default() -> Self {
        Self::new()
    }
}

pub fn apply_stage_effect(stage: DemoStageType, scene: &mut Scene) {
    match stage {
        DemoStageType::Bootstrap => {
            scene.sdf.lighting.ambient_light = 0.08;
        }
        DemoStageType::ParticleFormation => {
            scene.sdf.lighting.ambient_light = 0.14;
        }
        DemoStageType::LogoAssembly => {
            scene.sdf.lighting.ambient_light = 0.2;
            nudge_transforms(scene, 0.005);
        }
        DemoStageType::LogoReveal => {
            scene.sdf.lighting.ambient_light = 0.28;
            boost_emissive(scene, 0.12);
        }
        DemoStageType::EnergyPulse => {
            scene.sdf.lighting.ambient_light = 0.38;
            boost_emissive(scene, 0.2);
        }
        DemoStageType::SceneCollapse => {
            scene.sdf.lighting.ambient_light = 0.12;
            nudge_transforms(scene, -0.02);
        }
        DemoStageType::RuntimeHandover => {
            scene.sdf.lighting.ambient_light = 0.16;
            boost_emissive(scene, 0.05);
        }
    }
}

fn boost_emissive(scene: &mut Scene, amount: f32) {
    let SdfNode::Union { children } = &mut scene.sdf.root else {
        return;
    };
    for node in children {
        let SdfNode::Transform { child, .. } = node else {
            continue;
        };
        if let SdfNode::Primitive { object } = child.as_mut() {
            object.material.emissive_strength =
                (object.material.emissive_strength + amount).clamp(0.0, 3.0);
        }
    }
}

fn nudge_transforms(scene: &mut Scene, delta_y: f32) {
    let SdfNode::Union { children } = &mut scene.sdf.root else {
        return;
    };
    for node in children {
        let SdfNode::Transform { modifiers, .. } = node else {
            continue;
        };
        for modifier in modifiers {
            if let SdfModifier::Translate { offset } = modifier {
                offset.y += delta_y;
            }
        }
    }
}
