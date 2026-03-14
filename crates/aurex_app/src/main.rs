mod pulse_builder;
mod pulse_sequence;
mod pulses;
mod timeline;

use aurex_audio::{
    AudioBackendMode, AudioBackendReadiness, MockAudioEngine, start_runtime_sine_output,
};
use aurex_conductor::{ConductorClock, ConductorStage, MAIN_LOOP_STAGES, execute_frame};
use aurex_ecs::{CommandBuffer, EcsCommand, EcsWorld, EntityId, Transform2p5D};
use aurex_lighting::{LightDescriptor, LightKind};
use aurex_postfx::BloomSettings;
use aurex_render::{
    BootAnimationConfig, BootAnimator, BootPostFxTrack, BootSequenceRecipe, BootStylePreset,
    BootStyleProfile, CameraRig, MockRenderer, RENDER_MAIN_STAGES, RenderBackendMode,
    RenderBackendReadiness, RenderBootstrapConfig, RenderBootstrapExecutor, RenderBootstrapPlan,
    RenderStage, RuntimeRenderDebugState, attempt_real_renderer_bootstrap, rasterize_boot_frame,
    run_real_renderer_event_loop_with_frame_hook, set_runtime_render_debug_state,
};
use aurex_shape_synth::{PrimitiveType, ShapeDescriptor};
use pulses::{
    ExamplePulseConfig,
    ambient_dreamscape::create_ambient_dreamscape_pulse,
    aurielle_intro::{create_aurielle_intro_pulse, create_aurielle_intro_pulse_at_time},
    electronic_megacity::{
        create_electronic_megacity_pulse, create_electronic_megacity_pulse_at_time,
    },
    jazz_atmosphere::create_jazz_atmosphere_pulse,
};
use timeline::{
    AudioAction, AudioCue, AudioTransport, EventScheduler, PulseTimeline, SceneManager,
    SceneVisualProfile, TimelineClock, TimelineEvent, TimelineEventKind, blend_scene_profiles,
    scripts::aurielle_intro::aurielle_intro_timeline,
};

const DEFAULT_PULSE_NAME: &str = "megacity";
const DEFAULT_SEED: u64 = 2026;
const AVAILABLE_PULSE_NAMES: [&str; 4] = ["megacity", "jazz", "ambient", "aurielle_intro"];

#[derive(Debug, Clone, PartialEq, Eq)]
struct RuntimeOptions {
    pulse_name: String,
    seed: u64,
}

#[derive(Debug, Clone)]
struct RuntimeTickReport {
    phase_change: Option<String>,
    triggered_events: Vec<String>,
    active_scenes: Vec<String>,
    resolved_profile: SceneVisualProfile,
}

fn available_pulse_names() -> &'static [&'static str] {
    &AVAILABLE_PULSE_NAMES
}

fn parse_runtime_options(args: impl IntoIterator<Item = String>) -> Result<RuntimeOptions, String> {
    let mut pulse_name: Option<String> = None;
    let mut seed = DEFAULT_SEED;

    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--seed" => {
                let value = iter.next().ok_or_else(|| {
                    "Missing value for --seed. Example: cargo run -- megacity --seed 42".to_string()
                })?;
                seed = value.parse::<u64>().map_err(|_| {
                    format!("Invalid --seed value '{value}'. Expected unsigned integer (u64).")
                })?;
            }
            value if value.starts_with("--") => {
                return Err(format!(
                    "Unknown option '{value}'. Supported option: --seed <u64>"
                ));
            }
            value => {
                if pulse_name.is_some() {
                    return Err(
                        "Too many positional arguments. Usage: cargo run -- [pulse] [--seed <u64>]"
                            .to_string(),
                    );
                }
                pulse_name = Some(value.to_string());
            }
        }
    }

    let pulse_name = pulse_name.unwrap_or_else(|| DEFAULT_PULSE_NAME.to_string());
    if !available_pulse_names().contains(&pulse_name.as_str()) {
        return Err(format!(
            "Unknown pulse '{pulse_name}'. Available pulses: {}",
            available_pulse_names().join(", ")
        ));
    }

    Ok(RuntimeOptions { pulse_name, seed })
}

fn create_initial_pulse(pulse_name: &str, seed: u64) -> ExamplePulseConfig {
    match pulse_name {
        "megacity" => create_electronic_megacity_pulse(seed),
        "jazz" => create_jazz_atmosphere_pulse(seed),
        "ambient" => create_ambient_dreamscape_pulse(seed),
        "aurielle_intro" => create_aurielle_intro_pulse(seed),
        _ => unreachable!("unsupported pulse should be rejected by parse_runtime_options"),
    }
}

fn create_pulse_at_time(pulse_name: &str, seed: u64, elapsed_seconds: f32) -> ExamplePulseConfig {
    match pulse_name {
        "megacity" => create_electronic_megacity_pulse_at_time(seed, elapsed_seconds),
        "jazz" => create_jazz_atmosphere_pulse(seed),
        "ambient" => create_ambient_dreamscape_pulse(seed),
        "aurielle_intro" => create_aurielle_intro_pulse_at_time(seed, elapsed_seconds),
        _ => unreachable!("unsupported pulse should be rejected by parse_runtime_options"),
    }
}

#[derive(Debug)]
struct RuntimePulseLoop {
    pulse_name: String,
    seed: u64,
    pulse: ExamplePulseConfig,
    last_phase_name: Option<String>,
    clock: TimelineClock,
    timeline: PulseTimeline,
    scheduler: EventScheduler,
    scene_manager: SceneManager,
    audio_transport: AudioTransport,
    resolved_profile: SceneVisualProfile,
}

impl RuntimePulseLoop {
    fn new(pulse_name: &str, seed: u64) -> Self {
        let pulse = create_initial_pulse(pulse_name, seed);
        let last_phase_name = pulse.current_phase_name.clone();
        let timeline = select_timeline(pulse_name);
        let mut scene_manager = SceneManager::default();
        if let Some((scene_id, layer)) = initial_scene_for_pulse(pulse_name) {
            scene_manager.activate_scene(scene_id, layer);
        }

        Self {
            pulse_name: pulse_name.to_string(),
            seed,
            pulse,
            last_phase_name,
            clock: TimelineClock::new(1.0 / 60.0),
            timeline,
            scheduler: EventScheduler::new(),
            scene_manager,
            audio_transport: AudioTransport::default(),
            resolved_profile: SceneVisualProfile::default(),
        }
    }

    fn update(&mut self, wall_time_seconds: f32) -> RuntimeTickReport {
        let _steps = self.clock.advance_to(wall_time_seconds.max(0.0));
        let elapsed_seconds = self.clock.time_seconds;

        self.pulse = create_pulse_at_time(&self.pulse_name, self.seed, elapsed_seconds);
        let next_phase = self.pulse.current_phase_name.clone();

        let phase_change = if next_phase != self.last_phase_name {
            self.last_phase_name = next_phase.clone();
            next_phase
        } else {
            None
        };

        let due = self
            .scheduler
            .collect_due_events(&self.timeline, elapsed_seconds);

        let mut triggered_events = Vec::new();
        for event in due {
            match &event.kind {
                TimelineEventKind::ActivateScene { scene_id, layer } => {
                    self.scene_manager.activate_scene(scene_id.clone(), *layer);
                    triggered_events.push(format!("scene:{}", scene_id));
                }
                TimelineEventKind::StartTransition {
                    from_scene,
                    to_scene,
                    layer,
                    spec,
                } => {
                    self.scene_manager.start_transition(
                        from_scene.clone(),
                        to_scene.clone(),
                        *layer,
                        *spec,
                        elapsed_seconds,
                    );
                    triggered_events.push(format!("transition:{}->{}", from_scene, to_scene));
                }
                TimelineEventKind::AudioCue { cue_id, action } => {
                    self.audio_transport.apply_cue(AudioCue {
                        cue_id: cue_id.clone(),
                        action: *action,
                    });
                    triggered_events.push(format!("audio:{cue_id}:{action:?}"));
                }
                TimelineEventKind::Trigger { key } => {
                    triggered_events.push(format!("trigger:{key}"));
                }
            }
        }

        self.scene_manager.update(elapsed_seconds);
        self.resolved_profile = blend_scene_profiles(&self.scene_manager.layers);
        apply_scene_profile_to_pulse(
            &mut self.pulse,
            self.resolved_profile,
            &self.scene_manager.layers,
        );

        let active_scenes = self
            .scene_manager
            .layers
            .iter()
            .map(|layer| format!("{}@{}:{:.2}", layer.scene_id, layer.layer, layer.weight))
            .collect();

        RuntimeTickReport {
            phase_change,
            triggered_events,
            active_scenes,
            resolved_profile: self.resolved_profile,
        }
    }
}

fn select_timeline(pulse_name: &str) -> PulseTimeline {
    if pulse_name == "aurielle_intro" {
        aurielle_intro_timeline()
    } else {
        PulseTimeline::new(
            format!("{}_timeline", pulse_name),
            0.0,
            vec![TimelineEvent {
                id: 1,
                at_seconds: 0.0,
                priority: 0,
                kind: TimelineEventKind::AudioCue {
                    cue_id: "default_loop".to_string(),
                    action: AudioAction::Play,
                },
            }],
        )
    }
}

fn initial_scene_for_pulse(pulse_name: &str) -> Option<(&'static str, u8)> {
    match pulse_name {
        "megacity" => Some(("megacity_skyline", 0)),
        "jazz" => Some(("jazz_lounge", 0)),
        "ambient" => Some(("ambient_mist", 0)),
        _ => None,
    }
}

fn timeline_summary_lines(pulse_loop: &RuntimePulseLoop) -> Vec<String> {
    vec![
        format!(
            "Timeline: {} t={:.2}s frame={}",
            pulse_loop.timeline.name, pulse_loop.clock.time_seconds, pulse_loop.clock.frame_index
        ),
        format!(
            "Timeline Active Scenes: {}",
            pulse_loop.scene_manager.layers.len()
        ),
        format!(
            "Timeline Active Audio Tracks: {}",
            pulse_loop.audio_transport.active_tracks.join(",")
        ),
        format!(
            "Timeline Visual Profile: geom={:.2} particles={:.2} fog={:.2} glow={:.2} stars={} logo={}",
            pulse_loop.resolved_profile.geometry_density,
            pulse_loop.resolved_profile.particle_density,
            pulse_loop.resolved_profile.fog_density,
            pulse_loop.resolved_profile.glow_intensity,
            pulse_loop.resolved_profile.starfield_enabled,
            pulse_loop.resolved_profile.logo_enabled
        ),
    ]
}

fn apply_scene_profile_to_pulse(
    pulse: &mut ExamplePulseConfig,
    profile: SceneVisualProfile,
    layers: &[timeline::scene_manager::SceneLayerState],
) {
    let primary_scene = layers
        .iter()
        .max_by(|a, b| a.weight.total_cmp(&b.weight))
        .map(|l| l.scene_id.as_str())
        .unwrap_or("scene_default");

    pulse.world_blueprint.palette_hint = format!(
        "{}|{}|stars:{}|logo:{}",
        pulse.pulse_config.color_palette,
        primary_scene,
        profile.starfield_enabled,
        profile.logo_enabled
    );

    pulse.generator_output.structures.density = profile.geometry_density;
    pulse.generator_output.particles.density_multiplier = profile.particle_density;
    pulse.generator_output.atmosphere.fog_density = profile.fog_density;
    pulse.generator_output.lighting.flash_envelope = profile.glow_intensity;

    pulse.modulated_output.structures.density = (pulse.modulated_output.structures.density * 0.5
        + profile.geometry_density * 0.5)
        .clamp(0.0, 1.0);
    pulse.modulated_output.particles.density_multiplier =
        (pulse.modulated_output.particles.density_multiplier * 0.4
            + profile.particle_density * 0.6)
            .clamp(0.0, 1.0);
    pulse.modulated_output.atmosphere.fog_density =
        (pulse.modulated_output.atmosphere.fog_density * 0.4 + profile.fog_density * 0.6)
            .clamp(0.0, 1.0);
    pulse.modulated_output.lighting.flash_envelope =
        (pulse.modulated_output.lighting.flash_envelope * 0.3 + profile.glow_intensity * 0.7)
            .clamp(0.0, 1.0);

    pulse.rhythm_snapshot.intensity =
        (pulse.rhythm_snapshot.intensity * 0.5 + profile.glow_intensity * 0.5).clamp(0.0, 1.0);
    pulse.rhythm_snapshot.high_energy =
        (pulse.rhythm_snapshot.high_energy * 0.5 + profile.particle_density * 0.5).clamp(0.0, 1.0);
}

fn runtime_render_debug_state_for_loop(pulse_loop: &RuntimePulseLoop) -> RuntimeRenderDebugState {
    let dominant_scene = pulse_loop
        .scene_manager
        .layers
        .iter()
        .max_by(|a, b| a.weight.total_cmp(&b.weight))
        .map(|l| l.scene_id.clone())
        .unwrap_or_else(|| "unbound".to_string());

    RuntimeRenderDebugState {
        pulse_name: pulse_loop.pulse.pulse_name.clone(),
        scene_name: dominant_scene,
        theme_name: format!("{:?}", pulse_loop.pulse.world_blueprint.theme),
        profile_name: format!(
            "geom:{:.2}|particles:{:.2}|fog:{:.2}|glow:{:.2}",
            pulse_loop.resolved_profile.geometry_density,
            pulse_loop.resolved_profile.particle_density,
            pulse_loop.resolved_profile.fog_density,
            pulse_loop.resolved_profile.glow_intensity
        ),
        profile_geometry_density: pulse_loop.resolved_profile.geometry_density,
        profile_particle_density: pulse_loop.resolved_profile.particle_density,
        profile_fog_density: pulse_loop.resolved_profile.fog_density,
        profile_glow_intensity: pulse_loop.resolved_profile.glow_intensity,
        starfield_enabled: pulse_loop.resolved_profile.starfield_enabled,
        logo_enabled: pulse_loop.resolved_profile.logo_enabled,
        boot_active: pulse_loop.clock.time_seconds < 1.25,
    }
}

fn runtime_diagnostics_report(selected_pulse: &ExamplePulseConfig) -> String {
    let mut clock = ConductorClock::default();
    let camera = CameraRig::default();

    let shapes = [
        ShapeDescriptor {
            primitive_type: PrimitiveType::Circle,
            seed: 7,
        },
        ShapeDescriptor {
            primitive_type: PrimitiveType::Ring,
            seed: 11,
        },
        ShapeDescriptor {
            primitive_type: PrimitiveType::Tube,
            seed: 13,
        },
    ];

    let light = LightDescriptor {
        kind: LightKind::Pulse,
        intensity: 0.85,
        color_rgb: [0.2, 0.7, 1.0],
    };

    let bloom = BloomSettings::default();

    let mut audio = MockAudioEngine::default();
    let audio_before = audio.status();
    let audio_transition = audio.transition_mode(AudioBackendMode::CpalPlanned);
    let audio_after = audio.status();
    let audio_probe = audio.next_beat();
    let audio_readiness = AudioBackendReadiness::for_mode(audio_after.mode);

    let mut world = EcsWorld::default();
    let mut commands = CommandBuffer::default();
    commands.push(EcsCommand::SpawnEntity {
        entity: EntityId(10),
    });
    commands.push(EcsCommand::SpawnEntity {
        entity: EntityId(3),
    });
    commands.push(EcsCommand::SetTransform {
        entity: EntityId(3),
        transform: Transform2p5D {
            position: [3.0, 0.0, 0.0],
            ..Transform2p5D::default()
        },
    });
    world.apply_commands(&mut commands);

    let mut renderer = MockRenderer::new(RenderBootstrapConfig::default());
    let render_stats = renderer.run_frame(&RENDER_MAIN_STAGES);

    renderer.set_rhythm_snapshot(selected_pulse.rhythm_snapshot);
    let renderer_world_debug = renderer.world_debug_summary();

    let mut visited = Vec::new();
    let trace = execute_frame(&mut clock, |stage| {
        if matches!(
            stage,
            ConductorStage::RenderPrepare | ConductorStage::Render | ConductorStage::Present
        ) {
            visited.push(stage);
        }
    });
    let backend_before = renderer.backend_status();
    let transition = renderer.transition_backend_mode(RenderBackendMode::WgpuPlanned);
    let backend_after = renderer.backend_status();
    let render_readiness = RenderBackendReadiness::for_mode(backend_after.mode);
    let render_bootstrap_plan = RenderBootstrapPlan::for_mode(backend_after.mode);
    let mut render_bootstrap_executor = RenderBootstrapExecutor::new(backend_after.mode);
    while render_bootstrap_executor.execute_next().is_some() {}
    let real_renderer_bootstrap = attempt_real_renderer_bootstrap();

    let boot_style = BootStyleProfile::from_preset(BootStylePreset::NeonStorm);
    let boot_recipe = BootSequenceRecipe::GrandReveal;
    let boot_animator = BootAnimator::with_style_and_recipe(
        BootAnimationConfig {
            seed: 1337,
            frame_count: 12,
            ..BootAnimationConfig::default()
        },
        boot_style.clone(),
        boot_recipe,
    );
    let boot_frames = boot_animator.generate_frames(clock.sim_tick.0);
    let first_boot = &boot_frames[0];
    let last_boot = &boot_frames[boot_frames.len() - 1];
    let boot_first_raster = rasterize_boot_frame(first_boot, 320, 180);
    let boot_last_raster = rasterize_boot_frame(last_boot, 320, 180);
    let boot_timeline = boot_animator.generate_timeline(clock.sim_tick.0);
    let (phase_ignition, phase_pulse_lock, phase_reveal) = boot_timeline.phase_counts();
    let boot_intents = boot_timeline.derive_render_intents();
    let boot_postfx = boot_timeline.aggregate_postfx();
    let postfx_track = BootPostFxTrack::from_timeline(&boot_timeline);
    let boot_screen = boot_timeline.to_boot_screen_sequence("AUREX-X", "Prime Pulse online");
    let first_postfx = postfx_track.snapshot_for_tick(first_boot.tick).unwrap();
    let latest_postfx = postfx_track.latest_snapshot().unwrap();
    let first_screen = boot_screen.frames.first().unwrap();
    let latest_screen = boot_screen.frames.last().unwrap();
    let avg_styled_glow = boot_timeline
        .frames
        .iter()
        .map(|f| f.styled_glow)
        .sum::<f32>()
        / boot_timeline.frames.len() as f32;
    let avg_distortion = boot_timeline
        .frames
        .iter()
        .map(|f| f.distortion_weight)
        .sum::<f32>()
        / boot_timeline.frames.len() as f32;
    let avg_phase_t = boot_timeline.frames.iter().map(|f| f.phase_t).sum::<f32>()
        / boot_timeline.frames.len() as f32;
    let avg_bloom_intent =
        boot_intents.iter().map(|i| i.bloom_weight).sum::<f32>() / boot_intents.len() as f32;
    let avg_fog_intent =
        boot_intents.iter().map(|i| i.fog_weight).sum::<f32>() / boot_intents.len() as f32;
    let peak_bloom_intent = boot_intents
        .iter()
        .map(|i| i.bloom_weight)
        .fold(0.0_f32, f32::max);
    let avg_color_shift =
        boot_intents.iter().map(|i| i.color_shift).sum::<f32>() / boot_intents.len() as f32;

    let mut lines = Vec::new();
    lines.push("Aurex runtime scaffold initialized.".to_string());
    lines.push(format!(
        "frame={} tick={}",
        clock.frame_index.0, clock.sim_tick.0
    ));
    lines.push(format!("camera_fov={}", camera.fov_degrees));
    lines.push(format!("shape_count={}", shapes.len()));
    lines.push(format!(
        "light_kind={:?} bloom_intensity={}",
        light.kind, bloom.intensity
    ));
    lines.push(format!("conductor_stage_count={}", MAIN_LOOP_STAGES.len()));
    lines.push(format!(
        "audio_backend_before={:?} audio_ready_before={}",
        audio_before.mode, audio_before.ready
    ));
    lines.push(format!("audio_backend_transition={:?}", audio_transition));
    lines.push(format!(
        "audio_backend_after={:?} audio_ready_after={}",
        audio_after.mode, audio_after.ready
    ));
    lines.push(format!(
        "audio_probe=tick:{} pulse:{:.3}",
        audio_probe.tick.0, audio_probe.pulse
    ));
    lines.push(format!(
        "audio_m1_readiness=device_io:{} stream_graph:{} can_emit_sound:{}",
        audio_readiness.has_device_io,
        audio_readiness.has_stream_graph,
        audio_readiness.can_emit_sound
    ));
    lines.push(format!("ecs_entity_count={}", world.entity_count()));
    lines.push(format!(
        "render_bootstrap={} {}x{}",
        renderer.config().app_name,
        renderer.config().viewport_width,
        renderer.config().viewport_height
    ));
    lines.push(format!("render_stage_count={}", RENDER_MAIN_STAGES.len()));
    lines.push(format!(
        "render_stages={:?}/{:?}/{:?}",
        RenderStage::RenderPrepare,
        RenderStage::Render,
        RenderStage::Present
    ));
    lines.push(format!(
        "render_frame_id={} render_stages_executed={}",
        render_stats.frame_id, render_stats.stages_executed
    ));
    lines.push(format!("conductor_trace_stages={}", trace.stages.len()));
    lines.push(format!("render_stages_seen_by_conductor={}", visited.len()));
    lines.push(format!(
        "render_backend_before={:?} backend_ready_before={}",
        backend_before.mode, backend_before.ready
    ));
    lines.push(format!("render_backend_transition={:?}", transition));
    lines.push(format!(
        "render_backend_after={:?} backend_ready_after={}",
        backend_after.mode, backend_after.ready
    ));
    lines.push(format!(
        "render_m1_readiness=windowing:{} gpu:{} can_present:{}",
        render_readiness.has_windowing,
        render_readiness.has_gpu_backend,
        render_readiness.can_present
    ));
    lines.push(format!(
        "render_bootstrap_ready_steps={}/{}",
        render_bootstrap_plan.ready_count(),
        render_bootstrap_plan.total_count()
    ));
    lines.push(format!(
        "render_bootstrap_step_map={}",
        render_bootstrap_plan.summary()
    ));
    lines.push(format!(
        "render_bootstrap_executor_progress={}/{}",
        render_bootstrap_executor.completed_count(),
        render_bootstrap_executor.total_count()
    ));
    lines.push(format!(
        "render_bootstrap_executor_last_step={}",
        render_bootstrap_executor
            .last_completed_step()
            .map(|step| step.as_str())
            .unwrap_or("None")
    ));
    lines.push(format!(
        "render_real_bootstrap={:?} detail:{}",
        real_renderer_bootstrap.result, real_renderer_bootstrap.detail
    ));

    lines.push(format!(
        "pulse_showcase_count={}",
        available_pulse_names().len()
    ));
    lines.push(format!("Pulse: {}", selected_pulse.pulse_name));
    lines.push(format!("Theme: {:?}", selected_pulse.world_blueprint.theme));
    lines.push(format!(
        "Geometry: {:?}",
        selected_pulse.pulse_config.geometry_style
    ));
    lines.push(format!(
        "Structures: {:?}",
        selected_pulse.pulse_config.structure_set
    ));
    lines.push(format!(
        "Geometry: structures_density={:.3}",
        selected_pulse.modulated_output.structures.density
    ));
    lines.push(format!(
        "Atmosphere: fog_density={:.3}",
        selected_pulse.modulated_output.atmosphere.fog_density
    ));
    lines.push(format!(
        "Rhythm snapshot: pulse={:.2} bass={:.2} intensity={:.2}",
        selected_pulse.rhythm_snapshot.pulse,
        selected_pulse.rhythm_snapshot.bass_energy,
        selected_pulse.rhythm_snapshot.intensity
    ));
    if let Some(duration) = selected_pulse.sequence_duration_seconds {
        lines.push(format!("Sequence Duration: {:.1}s", duration));
    }
    if let Some(phase) = selected_pulse.current_phase_name.as_deref() {
        lines.push(format!("Current Phase: {}", phase));
    }
    lines.push(format!(
        "renderer_world_debug_lines={}",
        renderer_world_debug.lines().count()
    ));
    lines.push(format!("boot_frame_count={}", boot_frames.len()));
    lines.push(format!(
        "boot_first=tick:{} radius:{:.3} glow:{:.3} hue:{:.2}",
        first_boot.tick, first_boot.ring_radius, first_boot.glow, first_boot.hue_shift
    ));
    lines.push(format!(
        "boot_last=tick:{} radius:{:.3} glow:{:.3} hue:{:.2}",
        last_boot.tick, last_boot.ring_radius, last_boot.glow, last_boot.hue_shift
    ));
    lines.push(format!(
        "boot_phases=Ignition:{} PulseLock:{} Reveal:{}",
        phase_ignition, phase_pulse_lock, phase_reveal
    ));
    lines.push(format!("boot_style_preset={:?}", boot_style.preset));
    lines.push(format!("boot_sequence_recipe={:?}", boot_recipe));
    lines.push(format!("boot_screen_title={}", boot_screen.title_text));
    lines.push(format!(
        "boot_screen_subtitle={}",
        boot_screen.subtitle_text
    ));
    lines.push(format!(
        "boot_style_avg=glow:{:.3} distortion:{:.3} phase_t:{:.3}",
        avg_styled_glow, avg_distortion, avg_phase_t
    ));
    lines.push(format!(
        "boot_intent_avg=bloom:{:.3} fog:{:.3} color_shift:{:.3}",
        avg_bloom_intent, avg_fog_intent, avg_color_shift
    ));
    lines.push(format!("boot_intent_peak_bloom={:.3}", peak_bloom_intent));
    lines.push(format!(
        "boot_postfx_avg=bloom:{:.3} fog:{:.3} distortion:{:.3} color_shift:{:.3}",
        boot_postfx.avg_bloom,
        boot_postfx.avg_fog,
        boot_postfx.avg_distortion,
        boot_postfx.avg_color_shift
    ));
    lines.push(format!(
        "boot_postfx_peak_bloom={:.3}",
        boot_postfx.peak_bloom
    ));
    lines.push(format!(
        "boot_screen_first=tick:{} progress:{:.3} glow:{:.3} glyphs:{}",
        first_screen.tick,
        first_screen.title_progress,
        first_screen.title_glow,
        first_screen.glyphs_lit
    ));
    lines.push(format!(
        "boot_screen_latest=tick:{} progress:{:.3} glow:{:.3} glyphs:{}",
        latest_screen.tick,
        latest_screen.title_progress,
        latest_screen.title_glow,
        latest_screen.glyphs_lit
    ));
    lines.push(format!(
        "boot_postfx_first=tick:{} bloom:{:.3} fog:{:.3}",
        first_postfx.tick, first_postfx.bloom_strength, first_postfx.fog_density
    ));
    lines.push(format!(
        "boot_postfx_latest=tick:{} bloom:{:.3} fog:{:.3}",
        latest_postfx.tick, latest_postfx.bloom_strength, latest_postfx.fog_density
    ));
    lines.push(format!(
        "boot_raster_first={}x{} lit:{} checksum:{}",
        boot_first_raster.width,
        boot_first_raster.height,
        boot_first_raster.lit_pixel_count(),
        boot_first_raster.checksum()
    ));
    lines.push(format!(
        "boot_raster_latest={}x{} lit:{} checksum:{}",
        boot_last_raster.width,
        boot_last_raster.height,
        boot_last_raster.lit_pixel_count(),
        boot_last_raster.checksum()
    ));

    lines.join("\n")
}

fn main() {
    let options = match parse_runtime_options(std::env::args().skip(1)) {
        Ok(options) => options,
        Err(err) => {
            eprintln!("{err}");
            return;
        }
    };

    let mut pulse_loop = RuntimePulseLoop::new(&options.pulse_name, options.seed);
    let pulse = pulse_loop.pulse.clone();

    println!("Launching Pulse: {}", pulse.pulse_name);
    if let Some(duration) = pulse.sequence_duration_seconds {
        println!("Sequence Duration: {:.1}s", duration);
    }
    if let Some(phase) = pulse.current_phase_name.as_deref() {
        println!("Current Phase: {}", phase);
    }

    println!("{}", runtime_diagnostics_report(&pulse));
    for line in timeline_summary_lines(&pulse_loop) {
        println!("{line}");
    }

    let runtime_audio = match start_runtime_sine_output() {
        Ok(audio) => {
            println!("audio_runtime=started detail:cpal stream active");
            Some(audio)
        }
        Err(err) => {
            eprintln!("audio_runtime=error detail:{err}");
            None
        }
    };

    let runtime_audio = runtime_audio;
    set_runtime_render_debug_state(runtime_render_debug_state_for_loop(&pulse_loop));
    if let Err(err) = run_real_renderer_event_loop_with_frame_hook(move |t| {
        let tick = pulse_loop.update(t);
        set_runtime_render_debug_state(runtime_render_debug_state_for_loop(&pulse_loop));
        if let Some(phase_name) = tick.phase_change {
            println!("Phase Change: {}", phase_name);
            println!("{}", runtime_diagnostics_report(&pulse_loop.pulse));
        }
        for event in tick.triggered_events {
            println!("Timeline Event: {event}");
        }
        if !tick.active_scenes.is_empty() {
            println!("Timeline Scenes: {}", tick.active_scenes.join(" | "));
        }
        println!(
            "Timeline Visual Params: geom={:.2} particles={:.2} fog={:.2} glow={:.2} stars={} logo={}",
            tick.resolved_profile.geometry_density,
            tick.resolved_profile.particle_density,
            tick.resolved_profile.fog_density,
            tick.resolved_profile.glow_intensity,
            tick.resolved_profile.starfield_enabled,
            tick.resolved_profile.logo_enabled
        );

        if let Some(audio) = runtime_audio.as_ref() {
            let pulse =
                (pulse_loop.clock.time_seconds * std::f32::consts::TAU * 0.6).sin() * 0.5 + 0.5;
            audio.set_pulse(pulse);
        }
    }) && !err.contains("real_graphics feature is disabled")
    {
        eprintln!("render_real_loop=error detail:{err}");
    }
}

#[cfg(test)]
mod tests {
    use super::{
        DEFAULT_SEED, RuntimePulseLoop, available_pulse_names, create_pulse_at_time,
        parse_runtime_options, runtime_diagnostics_report, runtime_render_debug_state_for_loop,
    };

    #[test]
    fn diagnostics_report_matches_expected_snapshot() {
        let pulse = create_pulse_at_time("megacity", DEFAULT_SEED, 0.0);
        let report = runtime_diagnostics_report(&pulse);
        assert!(report.contains("Aurex runtime scaffold initialized."));
        assert!(report.contains("render_stages=RenderPrepare/Render/Present"));
        assert!(report.contains(&format!(
            "pulse_showcase_count={}",
            available_pulse_names().len()
        )));
        assert!(report.contains("Pulse: Electronic Megacity"));
        assert!(report.contains("Theme: Electronic"));
        assert!(report.contains("Rhythm snapshot: pulse="));
        assert!(report.contains("Sequence Duration:"));
        assert!(report.contains("Current Phase:"));
    }

    #[test]
    fn runtime_options_support_known_pulse_names() {
        let options = parse_runtime_options(vec![
            "aurielle_intro".to_string(),
            "--seed".to_string(),
            "42".to_string(),
        ])
        .expect("aurielle_intro should be accepted");
        assert_eq!(options.pulse_name, "aurielle_intro");
        assert_eq!(options.seed, 42);
    }

    #[test]
    fn runtime_options_default_to_megacity() {
        let options =
            parse_runtime_options(Vec::<String>::new()).expect("default options should parse");
        assert_eq!(options.pulse_name, "megacity");
        assert_eq!(options.seed, DEFAULT_SEED);
    }

    #[test]
    fn runtime_options_reject_unknown_pulse_name() {
        let err = parse_runtime_options(vec!["unknown".to_string()]).unwrap_err();
        assert!(err.contains("Unknown pulse 'unknown'"));
        assert!(err.contains("aurielle_intro"));
    }

    #[test]
    fn phase_changes_over_simulated_elapsed_time() {
        let mut pulse_loop = RuntimePulseLoop::new("aurielle_intro", DEFAULT_SEED);
        let mut seen_transitions = Vec::new();
        for t in [2.1, 6.2, 10.3, 13.5] {
            let tick = pulse_loop.update(t);
            if let Some(name) = tick.phase_change {
                seen_transitions.push(name);
            }
        }

        assert_eq!(
            seen_transitions,
            vec![
                "Aurielle Appears".to_string(),
                "Maestros Reveal".to_string(),
                "Logo Formation".to_string(),
                "Menu Transition".to_string(),
            ]
        );
    }

    #[test]
    fn phase_clamps_to_final_phase_after_total_duration() {
        let pulse = create_pulse_at_time("aurielle_intro", DEFAULT_SEED, 999.0);
        assert_eq!(pulse.current_phase_name.as_deref(), Some("Menu Transition"));
    }

    #[test]
    fn phase_mapping_is_deterministic_for_fixed_times() {
        let first = create_pulse_at_time("aurielle_intro", DEFAULT_SEED, 6.2);
        let second = create_pulse_at_time("aurielle_intro", DEFAULT_SEED, 6.2);
        assert_eq!(first.current_phase_name, second.current_phase_name);
        assert_eq!(first.modulated_output, second.modulated_output);
    }

    #[test]
    fn timeline_scheduler_triggers_aurielle_reveal_event() {
        let mut pulse_loop = RuntimePulseLoop::new("aurielle_intro", DEFAULT_SEED);
        let tick = pulse_loop.update(16.1);
        assert!(
            tick.triggered_events
                .iter()
                .any(|event| event.contains("trigger:aurielle_reveal"))
        );
    }

    #[test]
    fn timeline_scene_layers_progress_with_events() {
        let mut pulse_loop = RuntimePulseLoop::new("aurielle_intro", DEFAULT_SEED);
        let _ = pulse_loop.update(9.2);
        assert!(
            pulse_loop
                .scene_manager
                .layers
                .iter()
                .any(|layer| layer.scene_id == "particle_swirl" || layer.scene_id == "rings")
        );
    }

    #[test]
    fn scene_transition_changes_visual_parameters_over_time() {
        let mut pulse_loop = RuntimePulseLoop::new("aurielle_intro", DEFAULT_SEED);
        let early = pulse_loop.update(6.1).resolved_profile;
        let later = pulse_loop.update(9.8).resolved_profile;
        assert_ne!(early.particle_density, later.particle_density);
        assert_ne!(early.fog_density, later.fog_density);
        assert_ne!(early.glow_intensity, later.glow_intensity);
    }

    #[test]
    fn boot_active_flag_switches_to_pulse_active_after_handoff() {
        let mut pulse_loop = RuntimePulseLoop::new("aurielle_intro", DEFAULT_SEED);
        let _ = pulse_loop.update(0.5);
        assert!(runtime_render_debug_state_for_loop(&pulse_loop).boot_active);
        let _ = pulse_loop.update(2.0);
        assert!(!runtime_render_debug_state_for_loop(&pulse_loop).boot_active);
    }

    #[test]
    fn visual_profile_application_is_deterministic_for_same_time() {
        let mut a = RuntimePulseLoop::new("aurielle_intro", DEFAULT_SEED);
        let mut b = RuntimePulseLoop::new("aurielle_intro", DEFAULT_SEED);
        let _ = a.update(9.8);
        let _ = b.update(9.8);
        assert_eq!(a.resolved_profile, b.resolved_profile);
        assert_eq!(
            a.pulse.world_blueprint.palette_hint,
            b.pulse.world_blueprint.palette_hint
        );
    }
}
