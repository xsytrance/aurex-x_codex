use super::transition::TransitionSpec;

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineEvent {
    pub id: u64,
    pub at_seconds: f32,
    pub priority: u8,
    pub kind: TimelineEventKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TimelineEventKind {
    ActivateScene {
        scene_id: String,
        layer: u8,
    },
    StartTransition {
        from_scene: String,
        to_scene: String,
        layer: u8,
        spec: TransitionSpec,
    },
    AudioCue {
        cue_id: String,
        action: super::audio_transport::AudioAction,
    },
    Trigger {
        key: String,
    },
}
