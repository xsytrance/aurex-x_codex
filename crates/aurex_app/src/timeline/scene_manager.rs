use super::transition::TransitionSpec;

#[derive(Debug, Clone, PartialEq)]
pub struct SceneLayerState {
    pub scene_id: String,
    pub layer: u8,
    pub weight: f32,
}

#[derive(Debug, Clone, PartialEq)]
struct ActiveTransition {
    from_scene: String,
    to_scene: String,
    layer: u8,
    spec: TransitionSpec,
    start_time_seconds: f32,
}

#[derive(Debug, Default, Clone)]
pub struct SceneManager {
    pub layers: Vec<SceneLayerState>,
    transitions: Vec<ActiveTransition>,
}

impl SceneManager {
    pub fn activate_scene(&mut self, scene_id: impl Into<String>, layer: u8) {
        let scene_id = scene_id.into();
        self.layers.retain(|s| s.layer != layer);
        self.layers.push(SceneLayerState {
            scene_id,
            layer,
            weight: 1.0,
        });
    }

    pub fn start_transition(
        &mut self,
        from_scene: impl Into<String>,
        to_scene: impl Into<String>,
        layer: u8,
        spec: TransitionSpec,
        start_time_seconds: f32,
    ) {
        self.transitions.push(ActiveTransition {
            from_scene: from_scene.into(),
            to_scene: to_scene.into(),
            layer,
            spec,
            start_time_seconds,
        });
    }

    pub fn update(&mut self, time_seconds: f32) {
        let mut completed = Vec::new();
        for (idx, t) in self.transitions.clone().into_iter().enumerate() {
            let elapsed = (time_seconds - t.start_time_seconds).max(0.0);
            let (from_w, to_w) = t.spec.blend_weights(elapsed);

            self.set_layer_weight(&t.from_scene, t.layer, from_w);
            self.set_or_insert(&t.to_scene, t.layer, to_w);

            if t.spec.progress_at(elapsed) >= 1.0 {
                completed.push(idx);
                self.layers
                    .retain(|l| !(l.layer == t.layer && l.scene_id == t.from_scene));
                self.set_or_insert(&t.to_scene, t.layer, 1.0);
            }
        }

        for idx in completed.into_iter().rev() {
            self.transitions.remove(idx);
        }
    }

    fn set_layer_weight(&mut self, scene_id: &str, layer: u8, weight: f32) {
        if let Some(scene) = self
            .layers
            .iter_mut()
            .find(|s| s.scene_id == scene_id && s.layer == layer)
        {
            scene.weight = weight;
        }
    }

    fn set_or_insert(&mut self, scene_id: &str, layer: u8, weight: f32) {
        if let Some(scene) = self
            .layers
            .iter_mut()
            .find(|s| s.scene_id == scene_id && s.layer == layer)
        {
            scene.weight = weight;
            return;
        }
        self.layers.push(SceneLayerState {
            scene_id: scene_id.to_string(),
            layer,
            weight,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::timeline::{Easing, TransitionMode};

    #[test]
    fn crossfade_reaches_target_scene() {
        let mut manager = SceneManager::default();
        manager.activate_scene("a", 0);
        manager.start_transition(
            "a",
            "b",
            0,
            TransitionSpec {
                mode: TransitionMode::Crossfade,
                duration_seconds: 2.0,
                easing: Easing::Linear,
            },
            1.0,
        );

        manager.update(2.0);
        assert!(manager.layers.iter().any(|s| s.scene_id == "b"));
        manager.update(3.1);
        assert!(
            manager
                .layers
                .iter()
                .any(|s| s.scene_id == "b" && s.weight == 1.0)
        );
        assert!(!manager.layers.iter().any(|s| s.scene_id == "a"));
    }
}
