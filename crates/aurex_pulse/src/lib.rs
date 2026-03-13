pub mod boot_world;
pub mod diagnostics;
pub mod living_boot;
pub mod loader;
pub mod prime_pulse;
pub mod pulse_graph;
pub mod resonance;
pub mod runner;
pub mod schema;

#[cfg(test)]
mod tests {
    use aurex_render_sdf::RenderConfig;

    use std::path::PathBuf;

    use crate::{
        boot_world::{BootWorldGenerator, BootWorldState, District, PulsePortal},
        loader::load_pulse_from_str,
        pulse_graph::{
            PulseGraph, PulseGraphRunner, PulseNode, PulseTransition, PulseTransitionKind,
        },
        resonance::{PrimeFaction, ResonanceTracker},
        runner::{PulseRunner, PulseState},
    };

    #[test]
    fn pulse_runner_lifecycle_is_stable() {
        let pulse_json = r#"{
          "metadata": {
            "title": "Test Pulse",
            "author": "Aurex",
            "description": "test",
            "tags": ["test"],
            "seed": 42,
            "pulse_kind": "World",
            "duration_hint": 30.0,
            "interactivity": "Hybrid",
            "prime_affinity": "Electronic"
          },
          "pulse_kind": "World",
          "scene": {
            "sdf": {
              "seed": 1,
              "camera": {"position": {"x": 0.0, "y": 0.0, "z": -5.0}, "target": {"x": 0.0, "y": 0.0, "z": 0.0}, "fov_degrees": 60.0, "aspect_ratio": 1.7777},
              "lighting": {"ambient_light": 0.2, "key_lights": []},
              "root": {"Empty": null}
            }
          }
        }"#;

        let pulse = load_pulse_from_str(pulse_json).expect("pulse should parse");
        let mut runner = PulseRunner::load(pulse, None).expect("pulse should load");
        assert_eq!(runner.state, PulseState::Loaded);
        runner.initialize();
        assert_eq!(runner.state, PulseState::Initialized);
        runner.update(1.0 / 60.0);
        assert_eq!(runner.state, PulseState::Running);
        let frame = runner.render(RenderConfig::default());
        assert_eq!(frame.width, RenderConfig::default().width);
        runner.shutdown();
        assert_eq!(runner.state, PulseState::Shutdown);
    }

    #[test]
    fn pulse_music_sequencer_updates_rhythm_field() {
        let pulse_json = r#"{
          "metadata": {
            "title": "Music Pulse",
            "author": "Aurex",
            "pulse_kind": "VisualMusic",
            "interactivity": "Passive"
          },
          "pulse_kind": "VisualMusic",
          "scene": {
            "sdf": {
              "seed": 2,
              "camera": {"position": {"x": 0.0, "y": 0.0, "z": -5.0}, "target": {"x": 0.0, "y": 0.0, "z": 0.0}, "fov_degrees": 60.0, "aspect_ratio": 1.7777},
              "lighting": {"ambient_light": 0.2, "key_lights": []},
              "root": {"Empty": null}
            }
          },
          "music": {
            "bpm": 128.0,
            "tracks": [
              {
                "name": "bass",
                "instrument": "PulseSynth",
                "pattern": {
                  "steps": 16,
                  "events": [{"Note": {"step": 0, "pitch": 36, "duration_beats": 0.5, "velocity": 0.9}}]
                },
                "volume": 1.0
              }
            ]
          }
        }"#;

        let pulse = load_pulse_from_str(pulse_json).expect("music pulse should parse");
        let mut runner = PulseRunner::load(pulse, None).expect("music pulse should load");
        runner.initialize();
        assert!(runner.music_sequencer.is_some());
        assert!(runner.scene.sdf.audio.is_some());
        let baseline_ambient = runner.scene.sdf.lighting.ambient_light;
        runner.update(1.0 / 60.0);
        let rhythm = runner
            .rhythm_field()
            .expect("rhythm field should be present");
        assert!(rhythm.tempo > 0.0);
        assert!(rhythm.bar_index <= rhythm.beat_index);
        assert!(runner.runtime_context().rhythm_field.is_some());
        assert!(runner.diagnostics.rhythm_summary.is_some());
        assert!(runner.diagnostics.dominant_prime.is_none());
        assert!(!runner.diagnostics.top_three_primes.is_empty());
        assert!(runner.scene.sdf.lighting.ambient_light >= baseline_ambient);
    }

    #[test]
    fn pulse_graph_transitions_by_timeline() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("workspace root parent")
            .parent()
            .expect("workspace root")
            .to_path_buf();
        let a_path = root
            .join("examples/pulses/infinite_circuit_megacity.pulse.json")
            .to_string_lossy()
            .to_string();
        let b_path = root
            .join("examples/pulses/jazz_improv_world.pulse.json")
            .to_string_lossy()
            .to_string();

        let graph = PulseGraph {
            name: "test_graph".into(),
            seed: 12,
            entry_node: "a".into(),
            nodes: vec![
                PulseNode {
                    id: "a".into(),
                    pulse_path: a_path,
                },
                PulseNode {
                    id: "b".into(),
                    pulse_path: b_path,
                },
            ],
            transitions: vec![PulseTransition {
                from: "a".into(),
                to: "b".into(),
                kind: PulseTransitionKind::Timeline { after_seconds: 0.0 },
            }],
        };
        let mut graph_runner = PulseGraphRunner::load(graph, None).expect("graph should load");
        assert_eq!(graph_runner.active_node_id, "a");
        graph_runner
            .update(1.0 / 60.0)
            .expect("graph update should work");
        assert_eq!(graph_runner.active_node_id, "b");
    }

    #[test]
    fn boot_world_portal_emits_manual_trigger_for_graph() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("workspace root parent")
            .parent()
            .expect("workspace root")
            .to_path_buf();
        let a_path = root
            .join("examples/pulses/infinite_circuit_megacity.pulse.json")
            .to_string_lossy()
            .to_string();
        let b_path = root
            .join("examples/pulses/jazz_improv_world.pulse.json")
            .to_string_lossy()
            .to_string();

        let graph = PulseGraph {
            name: "boot_graph".into(),
            seed: 7,
            entry_node: "hub".into(),
            nodes: vec![
                PulseNode {
                    id: "hub".into(),
                    pulse_path: a_path,
                },
                PulseNode {
                    id: "jazz".into(),
                    pulse_path: b_path,
                },
            ],
            transitions: vec![PulseTransition {
                from: "hub".into(),
                to: "jazz".into(),
                kind: PulseTransitionKind::Manual {
                    trigger: "portal_jazz".into(),
                },
            }],
        };

        let mut graph_runner = PulseGraphRunner::load(graph, None).expect("graph load");
        let boot_cfg = BootWorldGenerator {
            seed: 1,
            districts: vec![District {
                id: "jazz_district".into(),
                prime: PrimeFaction::Jazz,
                center: aurex_scene::Vec3::new(0.0, 0.0, 0.0),
                radius: 5.0,
                pulse_refs: vec!["jazz".into()],
            }],
            portals: vec![PulsePortal {
                id: "jazz_portal".into(),
                trigger: "portal_jazz".into(),
                target_node: "jazz".into(),
                position: aurex_scene::Vec3::new(0.0, 0.0, 0.0),
                activation_radius: 2.0,
            }],
        };
        let mut state = BootWorldState::new();
        state.update_player_position(&boot_cfg, aurex_scene::Vec3::new(0.5, 0.0, 0.0));
        let mut tracker = ResonanceTracker::default();
        state.emit_portal_triggers(&boot_cfg, &mut graph_runner, Some(&mut tracker));
        graph_runner.update(0.0).expect("update graph");
        assert_eq!(graph_runner.active_node_id, "jazz");
        assert_eq!(tracker.profile().pulse_count, 1);
    }
}
