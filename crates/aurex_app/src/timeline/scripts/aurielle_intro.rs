use crate::timeline::{
    AudioAction, Easing, PulseTimeline, TimelineEvent, TimelineEventKind, TransitionMode,
    TransitionSpec,
};

pub fn aurielle_intro_timeline() -> PulseTimeline {
    PulseTimeline::new(
        "aurielle_intro",
        21.0,
        vec![
            TimelineEvent {
                id: 1,
                at_seconds: 0.0,
                priority: 0,
                kind: TimelineEventKind::ActivateScene {
                    scene_id: "boot_pulse".to_string(),
                    layer: 0,
                },
            },
            TimelineEvent {
                id: 2,
                at_seconds: 0.0,
                priority: 1,
                kind: TimelineEventKind::AudioCue {
                    cue_id: "intro_boot".to_string(),
                    action: AudioAction::Play,
                },
            },
            TimelineEvent {
                id: 3,
                at_seconds: 3.0,
                priority: 0,
                kind: TimelineEventKind::StartTransition {
                    from_scene: "boot_pulse".to_string(),
                    to_scene: "aurex_logo".to_string(),
                    layer: 0,
                    spec: TransitionSpec {
                        mode: TransitionMode::Fade,
                        duration_seconds: 1.0,
                        easing: Easing::SmoothStep,
                    },
                },
            },
            TimelineEvent {
                id: 4,
                at_seconds: 6.0,
                priority: 0,
                kind: TimelineEventKind::ActivateScene {
                    scene_id: "rings".to_string(),
                    layer: 1,
                },
            },
            TimelineEvent {
                id: 5,
                at_seconds: 9.0,
                priority: 0,
                kind: TimelineEventKind::StartTransition {
                    from_scene: "rings".to_string(),
                    to_scene: "particle_swirl".to_string(),
                    layer: 1,
                    spec: TransitionSpec {
                        mode: TransitionMode::Crossfade,
                        duration_seconds: 1.5,
                        easing: Easing::Linear,
                    },
                },
            },
            TimelineEvent {
                id: 6,
                at_seconds: 12.0,
                priority: 0,
                kind: TimelineEventKind::ActivateScene {
                    scene_id: "starfield_expansion".to_string(),
                    layer: 0,
                },
            },
            TimelineEvent {
                id: 7,
                at_seconds: 16.0,
                priority: 0,
                kind: TimelineEventKind::Trigger {
                    key: "aurielle_reveal".to_string(),
                },
            },
            TimelineEvent {
                id: 8,
                at_seconds: 16.0,
                priority: 1,
                kind: TimelineEventKind::AudioCue {
                    cue_id: "reveal_stinger".to_string(),
                    action: AudioAction::PlayOnce,
                },
            },
            TimelineEvent {
                id: 9,
                at_seconds: 16.0,
                priority: 2,
                kind: TimelineEventKind::StartTransition {
                    from_scene: "starfield_expansion".to_string(),
                    to_scene: "aurielle_reveal_scene".to_string(),
                    layer: 0,
                    spec: TransitionSpec {
                        mode: TransitionMode::Dissolve,
                        duration_seconds: 2.0,
                        easing: Easing::SmoothStep,
                    },
                },
            },
            TimelineEvent {
                id: 10,
                at_seconds: 18.5,
                priority: 0,
                kind: TimelineEventKind::StartTransition {
                    from_scene: "aurielle_reveal_scene".to_string(),
                    to_scene: "aurielle_overlay".to_string(),
                    layer: 2,
                    spec: TransitionSpec {
                        mode: TransitionMode::AdditiveOverlay,
                        duration_seconds: 1.0,
                        easing: Easing::Linear,
                    },
                },
            },
            TimelineEvent {
                id: 11,
                at_seconds: 20.5,
                priority: 1,
                kind: TimelineEventKind::AudioCue {
                    cue_id: "intro_boot".to_string(),
                    action: AudioAction::Stop,
                },
            },
        ],
    )
}
