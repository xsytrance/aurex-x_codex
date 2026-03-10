use aurex_conductor::{execute_frame, ConductorClock, ConductorStage, MAIN_LOOP_STAGES};
use aurex_ecs::{CommandBuffer, EcsCommand, EcsWorld, EntityId, Transform2p5D};
use aurex_lighting::{LightDescriptor, LightKind};
use aurex_postfx::BloomSettings;
use aurex_render::{
    BootAnimationConfig, BootAnimator, CameraRig, MockRenderer, RenderBackendMode,
    RenderBootstrapConfig, RenderStage, RENDER_MAIN_STAGES,
};
use aurex_shape_synth::{PrimitiveType, ShapeDescriptor};

fn main() {
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

    let boot_animator = BootAnimator::new(BootAnimationConfig {
        seed: 1337,
        frame_count: 12,
        ..BootAnimationConfig::default()
    });
    let boot_frames = boot_animator.generate_frames(clock.sim_tick.0);
    let boot_timeline = boot_animator.generate_timeline(clock.sim_tick.0);
    let (phase_ignition, phase_pulse_lock, phase_reveal) = boot_timeline.phase_counts();
    let avg_styled_glow = boot_timeline.frames.iter().map(|f| f.styled_glow).sum::<f32>() / boot_timeline.frames.len() as f32;
    let avg_distortion = boot_timeline.frames.iter().map(|f| f.distortion_weight).sum::<f32>() / boot_timeline.frames.len() as f32;
    let first_boot = &boot_frames[0];
    let last_boot = &boot_frames[boot_frames.len() - 1];

    println!("Aurex runtime scaffold initialized.");
    println!("frame={} tick={}", clock.frame_index.0, clock.sim_tick.0);
    println!("camera_fov={}", camera.fov_degrees);
    println!("shape_count={}", shapes.len());
    println!("light_kind={:?} bloom_intensity={}", light.kind, bloom.intensity);
    println!("conductor_stage_count={}", MAIN_LOOP_STAGES.len());
    println!("ecs_entity_count={}", world.entity_count());
    println!(
        "render_bootstrap={} {}x{}",
        renderer.config().app_name,
        renderer.config().viewport_width,
        renderer.config().viewport_height
    );
    println!("render_stage_count={}", RENDER_MAIN_STAGES.len());
    println!(
        "render_stages={:?}/{:?}/{:?}",
        RenderStage::RenderPrepare,
        RenderStage::Render,
        RenderStage::Present
    );
    println!(
        "render_frame_id={} render_stages_executed={}",
        render_stats.frame_id, render_stats.stages_executed
    );
    println!("conductor_trace_stages={}", trace.stages.len());
    println!("render_stages_seen_by_conductor={}", visited.len());
    println!(
        "render_backend_before={:?} backend_ready_before={}",
        backend_before.mode, backend_before.ready
    );
    println!("render_backend_transition={:?}", transition);
    println!(
        "render_backend_after={:?} backend_ready_after={}",
        backend_after.mode, backend_after.ready
    );
    println!("boot_frame_count={}", boot_frames.len());
    println!(
        "boot_first=tick:{} radius:{:.3} glow:{:.3} hue:{:.2}",
        first_boot.tick, first_boot.ring_radius, first_boot.glow, first_boot.hue_shift
    );
    println!(
        "boot_last=tick:{} radius:{:.3} glow:{:.3} hue:{:.2}",
        last_boot.tick, last_boot.ring_radius, last_boot.glow, last_boot.hue_shift
    );
    println!(
        "boot_phases=Ignition:{} PulseLock:{} Reveal:{}",
        phase_ignition, phase_pulse_lock, phase_reveal
    );
    println!(
        "boot_style_avg=glow:{:.3} distortion:{:.3}",
        avg_styled_glow, avg_distortion
    );
}
