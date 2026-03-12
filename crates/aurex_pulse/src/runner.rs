use std::path::{Path, PathBuf};
use std::time::Instant;

use aurex_music::{rhythm_field::RhythmField, sequencer::MusicSequencer};
use aurex_render_sdf::{
    RenderConfig, RenderTime, RenderedFrame, render_sdf_scene_with_diagnostics,
};
use aurex_scene::{Scene, load_scene_from_json_path};

use crate::boot_world::{BootWorldGenerator, BootWorldState};
use crate::diagnostics::{PulseDiagnostics, RhythmSummary};
use crate::schema::{PulseDefinition, PulseSceneSource};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PulseState {
    Loaded,
    Initialized,
    Running,
    Shutdown,
}

pub struct PulseRunner {
    pub definition: PulseDefinition,
    pub scene: Scene,
    pub state: PulseState,
    pub runtime_seconds: f32,
    pub diagnostics: PulseDiagnostics,
    pub music_sequencer: Option<MusicSequencer>,
    pub rhythm_field: Option<RhythmField>,
    pub runtime_context: PulseRuntimeContext,
    pub boot_world: Option<BootWorldGenerator>,
    pub boot_world_state: Option<BootWorldState>,
    base_ambient_light: f32,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct PulseRuntimeContext {
    pub rhythm_field: Option<RhythmField>,
}

impl PulseRunner {
    pub fn load(
        definition: PulseDefinition,
        pulse_file_path: Option<&Path>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let load_start = Instant::now();
        let scene = resolve_scene(&definition, pulse_file_path)?;
        let base_ambient_light = scene.sdf.lighting.ambient_light;
        let mut runner = Self {
            definition,
            scene,
            state: PulseState::Loaded,
            runtime_seconds: 0.0,
            diagnostics: PulseDiagnostics::default(),
            music_sequencer: None,
            rhythm_field: None,
            runtime_context: PulseRuntimeContext::default(),
            boot_world: None,
            boot_world_state: None,
            base_ambient_light,
        };
        runner.diagnostics.lifecycle.load_ms = load_start.elapsed().as_secs_f64() * 1000.0;
        Ok(runner)
    }

    pub fn initialize(&mut self) {
        let init_start = Instant::now();
        if let Some(audio) = self.definition.audio.clone() {
            self.scene.sdf.audio = Some(audio);
        }
        if let Some(timeline) = self.definition.timeline.clone() {
            self.scene.sdf.timeline = Some(timeline);
        }
        if let Some(music) = self.definition.music.clone() {
            let seq = MusicSequencer::new(music);
            if self.scene.sdf.audio.is_none() {
                self.scene.sdf.audio = Some(seq.to_procedural_audio_config());
            }
            self.music_sequencer = Some(seq);
        }
        if let Some(boot_world) = self.definition.boot_world.clone() {
            self.boot_world = Some(boot_world);
            self.boot_world_state = Some(BootWorldState::new());
        }
        self.scene.sdf.seed = self.definition.metadata.seed;
        self.base_ambient_light = self.scene.sdf.lighting.ambient_light;
        self.state = PulseState::Initialized;
        self.diagnostics.lifecycle.initialize_ms = init_start.elapsed().as_secs_f64() * 1000.0;
    }

    pub fn update(&mut self, delta_seconds: f32) {
        let update_start = Instant::now();
        self.runtime_seconds = (self.runtime_seconds + delta_seconds).max(0.0);

        if let (Some(cfg), Some(state)) = (&self.boot_world, &mut self.boot_world_state) {
            state.update_player_position(cfg, self.scene.sdf.camera.position);
        }

        if let Some(seq) = &mut self.music_sequencer {
            seq.update(delta_seconds);
            self.rhythm_field = Some(seq.rhythm_field);
            self.runtime_context.rhythm_field = self.rhythm_field;
            self.scene.sdf.runtime_modulation =
                Some(aurex_scene::generators::RuntimeModulationContext {
                    rhythm_field: self.rhythm_field.map(|rf| {
                        aurex_scene::generators::RhythmFieldContext {
                            beat_phase: rf.beat_phase,
                            beat_strength: rf.beat_strength,
                            bass_energy: rf.bass_energy,
                            harmonic_energy: rf.harmonic_energy,
                        }
                    }),
                });
            self.diagnostics.rhythm_field = self.rhythm_field;
            self.diagnostics.rhythm_summary = self.rhythm_field.map(|rf| RhythmSummary {
                beat_phase: rf.beat_phase,
                bar_index: rf.bar_index,
                bass_energy: rf.bass_energy,
            });

            if let Some(rf) = self.rhythm_field {
                let pulse = (rf.beat_strength * 0.25 + rf.harmonic_energy * 0.15).clamp(0.0, 0.4);
                self.scene.sdf.lighting.ambient_light =
                    (self.base_ambient_light * (1.0 + pulse)).clamp(0.01, 2.0);
            }
        }
        self.state = PulseState::Running;
        self.diagnostics.lifecycle.update_ms += update_start.elapsed().as_secs_f64() * 1000.0;
    }

    pub fn render(&mut self, mut config: RenderConfig) -> RenderedFrame {
        let render_start = Instant::now();
        config.time = RenderTime {
            seconds: self.runtime_seconds,
        };
        let (frame, frame_diag) = render_sdf_scene_with_diagnostics(&self.scene, config);
        self.diagnostics.frames_rendered += 1;
        self.diagnostics.last_frame = Some(frame_diag);
        self.diagnostics.lifecycle.render_ms += render_start.elapsed().as_secs_f64() * 1000.0;
        frame
    }

    pub fn rhythm_field(&self) -> Option<RhythmField> {
        self.rhythm_field
    }

    pub fn runtime_context(&self) -> PulseRuntimeContext {
        self.runtime_context
    }

    pub fn shutdown(&mut self) {
        let shutdown_start = Instant::now();
        self.state = PulseState::Shutdown;
        self.diagnostics.lifecycle.shutdown_ms = shutdown_start.elapsed().as_secs_f64() * 1000.0;
    }
}

fn resolve_scene(
    definition: &PulseDefinition,
    pulse_file_path: Option<&Path>,
) -> Result<Scene, Box<dyn std::error::Error>> {
    match &definition.scene {
        PulseSceneSource::Inline(scene) => Ok(scene.clone()),
        PulseSceneSource::ScenePath { scene_path } => {
            let resolved = if Path::new(scene_path).is_absolute() {
                PathBuf::from(scene_path)
            } else if let Some(base) = pulse_file_path.and_then(|p| p.parent()) {
                base.join(scene_path)
            } else {
                PathBuf::from(scene_path)
            };
            Ok(load_scene_from_json_path(resolved)?)
        }
    }
}
