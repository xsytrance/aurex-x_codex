mod pulse_builder;
mod pulse_sequence;
mod pulses;

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
    RenderStage, attempt_real_renderer_bootstrap, rasterize_boot_frame,
    run_real_renderer_event_loop_with_frame_hook,
};
use aurex_shape_synth::{PrimitiveType, ShapeDescriptor};
use pulses::{
    ambient_dreamscape::create_ambient_dreamscape_pulse,
    electronic_megacity::create_electronic_megacity_pulse,
    jazz_atmosphere::create_jazz_atmosphere_pulse,
};

fn runtime_diagnostics_report() -> String {
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

    let pulse_electronic = create_electronic_megacity_pulse(2026);
    let pulse_jazz = create_jazz_atmosphere_pulse(2026);
    let pulse_ambient = create_ambient_dreamscape_pulse(2026);

    renderer.set_rhythm_snapshot(pulse_electronic.rhythm_snapshot);
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

    lines.push(format!("pulse_showcase_count={}", 3));
    lines.push(format!("Pulse: {}", pulse_electronic.pulse_name));
    lines.push(format!(
        "Theme: {:?}",
        pulse_electronic.world_blueprint.theme
    ));
    lines.push(format!(
        "Geometry: {:?}",
        pulse_electronic.pulse_config.geometry_style
    ));
    lines.push(format!(
        "Structures: {:?}",
        pulse_electronic.pulse_config.structure_set
    ));
    lines.push(format!(
        "Geometry: structures_density={:.3}",
        pulse_electronic.modulated_output.structures.density
    ));
    lines.push(format!(
        "Atmosphere: fog_density={:.3}",
        pulse_electronic.modulated_output.atmosphere.fog_density
    ));
    lines.push(format!(
        "Rhythm snapshot: pulse={:.2} bass={:.2} intensity={:.2}",
        pulse_electronic.rhythm_snapshot.pulse,
        pulse_electronic.rhythm_snapshot.bass_energy,
        pulse_electronic.rhythm_snapshot.intensity
    ));
    if let Some(duration) = pulse_electronic.sequence_duration_seconds {
        lines.push(format!("Sequence Duration: {:.1}s", duration));
    }
    if let Some(phase) = pulse_electronic.current_phase_name.as_deref() {
        lines.push(format!("Current Phase: {}", phase));
    }
    lines.push(format!("Pulse: {}", pulse_jazz.pulse_name));
    lines.push(format!("Theme: {:?}", pulse_jazz.world_blueprint.theme));
    lines.push(format!("Pulse: {}", pulse_ambient.pulse_name));
    lines.push(format!("Theme: {:?}", pulse_ambient.world_blueprint.theme));
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
    println!("{}", runtime_diagnostics_report());

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
    if let Err(err) = run_real_renderer_event_loop_with_frame_hook(move |t| {
        if let Some(audio) = runtime_audio.as_ref() {
            let pulse = (t * std::f32::consts::TAU * 0.6).sin() * 0.5 + 0.5;
            audio.set_pulse(pulse);
        }
    }) && !err.contains("real_graphics feature is disabled")
    {
        eprintln!("render_real_loop=error detail:{err}");
    }
}

#[cfg(test)]
mod tests {
    use super::runtime_diagnostics_report;

    #[test]
    fn diagnostics_report_matches_expected_snapshot() {
        let report = runtime_diagnostics_report();
        assert!(report.contains("Aurex runtime scaffold initialized."));
        assert!(report.contains("render_stages=RenderPrepare/Render/Present"));
        assert!(report.contains("pulse_showcase_count=3"));
        assert!(report.contains("Pulse: Electronic Megacity"));
        assert!(report.contains("Theme: Electronic"));
        assert!(report.contains("Rhythm snapshot: pulse="));
        assert!(report.contains("Sequence Duration:"));
        assert!(report.contains("Current Phase:"));
    }
}
