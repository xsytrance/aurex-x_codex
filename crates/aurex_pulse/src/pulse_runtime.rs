use std::path::Path;

use aurex_midi::{analyze_timeline, load_midi_timeline};
use aurex_scene::{Scene, scene_generator};

use crate::beat_driver::{BeatDriver, apply_beat_pulse};
use crate::demo_sequencer::{DemoSequencer, apply_stage_effect};
use crate::pulse_blueprint::{PulseBlueprint, blueprint_from_midi_analysis};

pub struct PulseRuntime {
    pub blueprint: PulseBlueprint,
    beat_driver: BeatDriver,
    demo_sequencer: DemoSequencer,
}

impl PulseRuntime {
    pub fn from_midi_bytes(bytes: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        let timeline = load_midi_timeline(bytes)?;
        let analysis = analyze_timeline(&timeline);
        let blueprint = blueprint_from_midi_analysis(&analysis);
        let beat_driver = BeatDriver::new(&blueprint);
        let demo_sequencer = DemoSequencer::new();
        Ok(Self {
            blueprint,
            beat_driver,
            demo_sequencer,
        })
    }

    pub fn generate_scene(&self) -> Scene {
        let scene_blueprint = scene_generator::PulseBlueprint {
            bpm: self.blueprint.bpm,
            beat_ticks: self.blueprint.beat_ticks.clone(),
            energy_level: self.blueprint.energy_level,
            pitch_span: self.blueprint.pitch_span,
            density_level: self.blueprint.density_level,
        };
        scene_generator::generate_scene_from_blueprint(&scene_blueprint)
    }

    pub fn print_debug(&self) {
        println!("Pulse Runtime");
        println!("-------------");
        println!("BPM: {:.0}", self.blueprint.bpm);
        println!("Pitch span: {}", self.blueprint.pitch_span);
        println!("Density level: {:.3}", self.blueprint.density_level);
        println!("Energy level: {:.3}", self.blueprint.energy_level);
        println!("Beat count: {}", self.blueprint.beat_ticks.len());
    }

    pub fn update_scene_for_frame(&mut self, scene: &mut Scene, delta_seconds: f32) -> bool {
        let beat_triggered = self.beat_driver.update(delta_seconds);
        if beat_triggered {
            apply_beat_pulse(scene, self.blueprint.energy_level);
        }

        let transitioned_to = self.demo_sequencer.update(delta_seconds);
        let stage = transitioned_to.unwrap_or_else(|| self.demo_sequencer.current_stage_type());
        apply_stage_effect(stage, scene);

        beat_triggered
    }

    pub fn current_beat(&self) -> usize {
        self.beat_driver.current_beat()
    }

    pub fn load_runtime_from_midi_file(
        path: &Path,
    ) -> Result<PulseRuntime, Box<dyn std::error::Error>> {
        load_runtime_from_midi_file(path)
    }
}

pub fn load_runtime_from_midi_file(
    path: &Path,
) -> Result<PulseRuntime, Box<dyn std::error::Error>> {
    let bytes = std::fs::read(path)?;
    PulseRuntime::from_midi_bytes(&bytes)
}
