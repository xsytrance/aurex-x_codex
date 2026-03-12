pub mod diagnostics;
pub mod loader;
pub mod runner;
pub mod schema;

#[cfg(test)]
mod tests {
    use aurex_render_sdf::RenderConfig;

    use crate::{
        loader::load_pulse_from_str,
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
        runner.update(1.0 / 60.0);
        assert!(runner.rhythm_field().is_some());
    }
}
