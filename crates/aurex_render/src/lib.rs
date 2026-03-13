#[cfg(feature = "real_graphics")]
use winit::{dpi::PhysicalSize, event_loop::EventLoop, window::Window};

#[derive(Debug, Clone)]
pub struct CameraRig {
    pub eye: [f32; 3],
    pub target: [f32; 3],
    pub fov_degrees: f32,
}

impl Default for CameraRig {
    fn default() -> Self {
        Self {
            eye: [0.0, 6.0, 12.0],
            target: [0.0, 0.0, 0.0],
            fov_degrees: 60.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderStage {
    RenderPrepare,
    Render,
    Present,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderBackendMode {
    Mock,
    WgpuPlanned,
}

#[derive(Debug, Clone)]
pub struct RenderBootstrapConfig {
    pub app_name: String,
    pub viewport_width: u32,
    pub viewport_height: u32,
    pub backend_mode: RenderBackendMode,
}

impl RenderBootstrapConfig {
    pub fn with_backend_mode(mut self, mode: RenderBackendMode) -> Self {
        self.backend_mode = mode;
        self
    }
}

impl Default for RenderBootstrapConfig {
    fn default() -> Self {
        Self {
            app_name: "Aurex-X".to_string(),
            viewport_width: 1280,
            viewport_height: 720,
            backend_mode: RenderBackendMode::Mock,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderBackendReadiness {
    pub has_windowing: bool,
    pub has_gpu_backend: bool,
    pub can_present: bool,
}

impl RenderBackendReadiness {
    pub fn for_mode(mode: RenderBackendMode) -> Self {
        match mode {
            RenderBackendMode::Mock => Self {
                has_windowing: false,
                has_gpu_backend: false,
                can_present: false,
            },
            RenderBackendMode::WgpuPlanned => Self {
                has_windowing: true,
                has_gpu_backend: true,
                can_present: true,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderBootstrapStep {
    InitWindow,
    InitWgpuInstance,
    InitSurface,
    RequestDevice,
    ConfigureSwapchain,
    UploadBootScreenQuad,
    DrawBootScreen,
}

impl RenderBootstrapStep {
    pub fn as_str(&self) -> &'static str {
        match self {
            RenderBootstrapStep::InitWindow => "InitWindow",
            RenderBootstrapStep::InitWgpuInstance => "InitWgpuInstance",
            RenderBootstrapStep::InitSurface => "InitSurface",
            RenderBootstrapStep::RequestDevice => "RequestDevice",
            RenderBootstrapStep::ConfigureSwapchain => "ConfigureSwapchain",
            RenderBootstrapStep::UploadBootScreenQuad => "UploadBootScreenQuad",
            RenderBootstrapStep::DrawBootScreen => "DrawBootScreen",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderBootstrapTaskStatus {
    pub step: RenderBootstrapStep,
    pub ready: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderBootstrapPlan {
    pub tasks: Vec<RenderBootstrapTaskStatus>,
}

impl RenderBootstrapPlan {
    pub fn for_mode(mode: RenderBackendMode) -> Self {
        let ready = matches!(mode, RenderBackendMode::WgpuPlanned);
        let steps = [
            RenderBootstrapStep::InitWindow,
            RenderBootstrapStep::InitWgpuInstance,
            RenderBootstrapStep::InitSurface,
            RenderBootstrapStep::RequestDevice,
            RenderBootstrapStep::ConfigureSwapchain,
            RenderBootstrapStep::UploadBootScreenQuad,
            RenderBootstrapStep::DrawBootScreen,
        ];

        Self {
            tasks: steps
                .into_iter()
                .map(|step| RenderBootstrapTaskStatus { step, ready })
                .collect(),
        }
    }

    pub fn ready_count(&self) -> usize {
        self.tasks.iter().filter(|t| t.ready).count()
    }

    pub fn total_count(&self) -> usize {
        self.tasks.len()
    }

    pub fn summary(&self) -> String {
        self.tasks
            .iter()
            .map(|task| format!("{}:{}", task.step.as_str(), task.ready))
            .collect::<Vec<_>>()
            .join(",")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderBootstrapExecutor {
    plan: RenderBootstrapPlan,
    next_step_index: usize,
}

impl RenderBootstrapExecutor {
    pub fn new(mode: RenderBackendMode) -> Self {
        Self {
            plan: RenderBootstrapPlan::for_mode(mode),
            next_step_index: 0,
        }
    }

    pub fn execute_next(&mut self) -> Option<RenderBootstrapStep> {
        let step = self.plan.tasks.get(self.next_step_index).map(|t| t.step)?;
        self.next_step_index += 1;
        Some(step)
    }

    pub fn completed_count(&self) -> usize {
        self.next_step_index.min(self.plan.tasks.len())
    }

    pub fn total_count(&self) -> usize {
        self.plan.tasks.len()
    }

    pub fn last_completed_step(&self) -> Option<RenderBootstrapStep> {
        self.next_step_index
            .checked_sub(1)
            .and_then(|idx| self.plan.tasks.get(idx).map(|t| t.step))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RealRendererBootstrapResult {
    FeatureDisabled,
    AdapterUnavailable,
    DeviceRequestFailed,
    Ready,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RealRendererBootstrapStatus {
    pub result: RealRendererBootstrapResult,
    pub detail: String,
}

#[cfg(feature = "real_graphics")]
pub fn run_real_renderer_event_loop() -> Result<(), String> {
    run_real_renderer_event_loop_with_frame_hook(|_| {})
}

#[cfg(feature = "real_graphics")]
pub fn run_real_renderer_event_loop_with_frame_hook<F>(mut on_frame: F) -> Result<(), String>
where
    F: FnMut(f32) + 'static,
{
    use winit::event::{Event, WindowEvent};

    let event_loop =
        EventLoop::new().map_err(|err| format!("event loop initialization failed: {err}"))?;
    let window = event_loop
        .create_window(
            Window::default_attributes()
                .with_title("Aurex-X Boot")
                .with_inner_size(PhysicalSize::new(1280, 720)),
        )
        .map_err(|err| format!("window creation failed: {err}"))?;

    let instance = wgpu::Instance::default();
    let surface = instance
        .create_surface(&window)
        .map_err(|err| format!("surface creation failed: {err}"))?;
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: Some(&surface),
        force_fallback_adapter: false,
    }))
    .map_err(|err| format!("request_adapter failed: {err}"))?;

    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("Aurex-X Loop Device"),
        required_features: wgpu::Features::empty(),
        required_limits: wgpu::Limits::default(),
        memory_hints: wgpu::MemoryHints::default(),
        trace: wgpu::Trace::Off,
    }))
    .map_err(|err| format!("request_device failed: {err}"))?;

    let caps = surface.get_capabilities(&adapter);
    let format = caps
        .formats
        .first()
        .copied()
        .ok_or_else(|| "surface has no supported texture formats".to_string())?;
    let present_mode = caps
        .present_modes
        .iter()
        .copied()
        .find(|m| *m == wgpu::PresentMode::Fifo)
        .unwrap_or(wgpu::PresentMode::AutoVsync);
    let alpha_mode = caps
        .alpha_modes
        .first()
        .copied()
        .unwrap_or(wgpu::CompositeAlphaMode::Auto);

    let mut size = window.inner_size();
    let mut config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format,
        width: size.width.max(1),
        height: size.height.max(1),
        present_mode,
        alpha_mode,
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    };
    surface.configure(&device, &config);

    let animator = BootAnimator::with_style_and_recipe(
        BootAnimationConfig {
            seed: 1337,
            frame_count: 240,
            ..BootAnimationConfig::default()
        },
        BootStyleProfile::from_preset(BootStylePreset::NeonStorm),
        BootSequenceRecipe::GrandReveal,
    );
    let timeline_frames = animator.generate_frames(1);
    let boot_screen = animator
        .generate_timeline(1)
        .to_boot_screen_sequence("AUREX-X", "Prime Pulse online");
    let start_time = std::time::Instant::now();
    let mut last_frame_time = start_time;
    let mut scene_particles = vec![Particle::default(); 220];
    let mut particle_cursor = 0usize;
    let starfield = build_starfield(200);

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Aurex-X Boot Texture Shader"),
        source: wgpu::ShaderSource::Wgsl(
            r#"
@group(0) @binding(0)
var boot_tex: texture_2d<f32>;
@group(0) @binding(1)
var boot_sampler: sampler;

struct VsOut {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vid: u32) -> VsOut {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -3.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(3.0, 1.0),
    );
    var uvs = array<vec2<f32>, 3>(
        vec2<f32>(0.0, 2.0),
        vec2<f32>(0.0, 0.0),
        vec2<f32>(2.0, 0.0),
    );
    var out: VsOut;
    out.position = vec4<f32>(positions[vid], 0.0, 1.0);
    out.uv = uvs[vid];
    return out;
}

@fragment
fn fs_main(inf: VsOut) -> @location(0) vec4<f32> {
    return textureSample(boot_tex, boot_sampler, inf.uv);
}
"#
            .into(),
        ),
    });

    let texture_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Aurex-X Boot Texture BGL"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Aurex-X Boot Texture Pipeline Layout"),
        bind_group_layouts: &[&texture_bind_group_layout],
        push_constant_ranges: &[],
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Aurex-X Boot Texture Pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: config.format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    });

    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("Aurex-X Boot Sampler"),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    });

    let mut boot_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Aurex-X Boot Texture"),
        size: wgpu::Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let mut boot_texture_view = boot_texture.create_view(&wgpu::TextureViewDescriptor::default());
    let mut boot_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Aurex-X Boot Texture BG"),
        layout: &texture_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&boot_texture_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
    });

    event_loop
        .run(|event, target| {
            target.set_control_flow(winit::event_loop::ControlFlow::Poll);
            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => target.exit(),
                    WindowEvent::Resized(new_size) => {
                        size = new_size;
                        if size.width > 0 && size.height > 0 {
                            config.width = size.width;
                            config.height = size.height;
                            surface.configure(&device, &config);

                            boot_texture = device.create_texture(&wgpu::TextureDescriptor {
                                label: Some("Aurex-X Boot Texture"),
                                size: wgpu::Extent3d {
                                    width: config.width,
                                    height: config.height,
                                    depth_or_array_layers: 1,
                                },
                                mip_level_count: 1,
                                sample_count: 1,
                                dimension: wgpu::TextureDimension::D2,
                                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                                usage: wgpu::TextureUsages::TEXTURE_BINDING
                                    | wgpu::TextureUsages::COPY_DST,
                                view_formats: &[],
                            });
                            boot_texture_view =
                                boot_texture.create_view(&wgpu::TextureViewDescriptor::default());
                            boot_bind_group =
                                device.create_bind_group(&wgpu::BindGroupDescriptor {
                                    label: Some("Aurex-X Boot Texture BG"),
                                    layout: &texture_bind_group_layout,
                                    entries: &[
                                        wgpu::BindGroupEntry {
                                            binding: 0,
                                            resource: wgpu::BindingResource::TextureView(
                                                &boot_texture_view,
                                            ),
                                        },
                                        wgpu::BindGroupEntry {
                                            binding: 1,
                                            resource: wgpu::BindingResource::Sampler(&sampler),
                                        },
                                    ],
                                });
                        }
                    }
                    WindowEvent::RedrawRequested => {
                        if config.width == 0 || config.height == 0 {
                            return;
                        }

                        on_frame(start_time.elapsed().as_secs_f32());

                        let frame = match surface.get_current_texture() {
                            Ok(frame) => frame,
                            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                                surface.configure(&device, &config);
                                return;
                            }
                            Err(wgpu::SurfaceError::OutOfMemory) => {
                                target.exit();
                                return;
                            }
                            Err(wgpu::SurfaceError::Timeout) => return,
                            Err(wgpu::SurfaceError::Other) => return,
                        };

                        let now = std::time::Instant::now();
                        let elapsed = start_time.elapsed().as_secs_f32();
                        let dt = (now - last_frame_time).as_secs_f32().clamp(0.0, 0.05);
                        last_frame_time = now;
                        let runtime_pulse =
                            (elapsed * std::f32::consts::TAU * 0.6).sin() * 0.5 + 0.5;
                        let timeline_idx = ((elapsed * 24.0) as usize) % timeline_frames.len();
                        let boot = &timeline_frames[timeline_idx];
                        let screen = &boot_screen.frames[timeline_idx % boot_screen.frames.len()];

                        let cpu_frame = if elapsed < 5.0 {
                            let mut frame = rasterize_boot_frame(boot, config.width, config.height);
                            overlay_boot_caption(
                                &mut frame.rgba,
                                config.width,
                                config.height,
                                screen,
                            );
                            frame
                        } else if elapsed < 6.0 {
                            let mut frame = rasterize_boot_frame(boot, config.width, config.height);
                            overlay_boot_caption(
                                &mut frame.rgba,
                                config.width,
                                config.height,
                                screen,
                            );
                            let fade = (1.0 - (elapsed - 5.0)).clamp(0.0, 1.0);
                            apply_fade_to_black(&mut frame.rgba, fade);
                            frame
                        } else if elapsed < 12.0 {
                            rasterize_reveal_frame(config.width, config.height, elapsed - 6.0, boot)
                        } else {
                            let scene_time = elapsed - 12.0;
                            let scene = select_demo_scene(scene_time);
                            let local_t = local_scene_time(scene_time);
                            rasterize_demo_scene(
                                config.width,
                                config.height,
                                scene,
                                local_t,
                                dt,
                                runtime_pulse,
                                &mut scene_particles,
                                &mut particle_cursor,
                                &starfield,
                            )
                        };

                        queue.write_texture(
                            wgpu::TexelCopyTextureInfo {
                                texture: &boot_texture,
                                mip_level: 0,
                                origin: wgpu::Origin3d::ZERO,
                                aspect: wgpu::TextureAspect::All,
                            },
                            &cpu_frame.rgba,
                            wgpu::TexelCopyBufferLayout {
                                offset: 0,
                                bytes_per_row: Some(config.width * 4),
                                rows_per_image: Some(config.height),
                            },
                            wgpu::Extent3d {
                                width: config.width,
                                height: config.height,
                                depth_or_array_layers: 1,
                            },
                        );

                        let swap_view = frame
                            .texture
                            .create_view(&wgpu::TextureViewDescriptor::default());
                        let mut encoder =
                            device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                                label: Some("Aurex-X Boot Loop Encoder"),
                            });

                        {
                            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                label: Some("Aurex-X Boot Loop Pass"),
                                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                    view: &swap_view,
                                    resolve_target: None,
                                    ops: wgpu::Operations {
                                        load: wgpu::LoadOp::Clear(wgpu::Color {
                                            r: 0.0,
                                            g: 0.0,
                                            b: 0.0,
                                            a: 1.0,
                                        }),
                                        store: wgpu::StoreOp::Store,
                                    },
                                })],
                                depth_stencil_attachment: None,
                                occlusion_query_set: None,
                                timestamp_writes: None,
                            });
                            pass.set_pipeline(&pipeline);
                            pass.set_bind_group(0, &boot_bind_group, &[]);
                            pass.draw(0..3, 0..1);
                        }

                        queue.submit([encoder.finish()]);
                        frame.present();
                    }
                    _ => {}
                },
                Event::AboutToWait => {
                    window.request_redraw();
                }
                _ => {}
            }
        })
        .map_err(|err| format!("event loop run failed: {err}"))
}

#[cfg(feature = "real_graphics")]
fn apply_fade_to_black(rgba: &mut [u8], fade: f32) {
    let k = fade.clamp(0.0, 1.0);
    for px in rgba.chunks_exact_mut(4) {
        px[0] = ((px[0] as f32) * k) as u8;
        px[1] = ((px[1] as f32) * k) as u8;
        px[2] = ((px[2] as f32) * k) as u8;
    }
}

#[cfg(feature = "real_graphics")]
fn rasterize_reveal_frame(
    width: u32,
    height: u32,
    reveal_t: f32,
    boot: &BootFrame,
) -> BootFramebuffer {
    let mut frame = BootFramebuffer {
        width,
        height,
        rgba: vec![0; (width as usize) * (height as usize) * 4],
    };
    for px in frame.rgba.chunks_exact_mut(4) {
        px[3] = 255;
    }

    let angle = reveal_t * 0.25;
    let x_offset = angle.sin() * 40.0;
    let scale = 1.0 + angle.cos() * 0.05;
    let pulse = (boot.glow * 0.5).clamp(0.0, 1.0);
    let glow = 0.7 + pulse * 0.8;

    let base_scale = ((width as f32 / 240.0) * scale).clamp(2.0, 7.0) as i32;
    let title = "AUREX-X";
    let title_w = text_pixel_width(title, base_scale);
    let title_h = 7 * base_scale;
    let x = ((width as i32 - title_w) / 2) + x_offset as i32;
    let y = ((height as i32 - title_h) / 2) - (angle.sin() * 10.0) as i32;

    let layers = [
        (3, [30, 120, 255], 0.10 * glow),
        (2, [75, 170, 255], 0.18 * glow),
        (1, [130, 215, 255], 0.30 * glow),
        (0, [205, 240, 255], 0.95),
    ];
    for (spread, color, intensity) in layers {
        draw_text(
            &mut frame.rgba,
            width,
            height,
            title,
            x - spread,
            y,
            base_scale,
            color,
            intensity.clamp(0.0, 1.0),
        );
        if spread > 0 {
            draw_text(
                &mut frame.rgba,
                width,
                height,
                title,
                x + spread,
                y,
                base_scale,
                color,
                intensity.clamp(0.0, 1.0),
            );
        }
    }

    frame
}

#[cfg(feature = "real_graphics")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DemoScene {
    Visualizer,
    StarfieldWarp,
    PulseReactorChamber,
    ParticleStorm,
    ReturnToTitle,
}

#[cfg(feature = "real_graphics")]
#[derive(Debug, Clone, Copy)]
struct Particle {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    life: f32,
    max_life: f32,
    active: bool,
}

#[cfg(feature = "real_graphics")]
impl Default for Particle {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            vx: 0.0,
            vy: 0.0,
            life: 0.0,
            max_life: 1.0,
            active: false,
        }
    }
}

#[cfg(feature = "real_graphics")]
#[derive(Debug, Clone, Copy)]
struct Star {
    x: f32,
    y: f32,
    z: f32,
}

#[cfg(feature = "real_graphics")]
fn select_demo_scene(t: f32) -> DemoScene {
    let cycle = t.rem_euclid(63.0);
    if cycle < 10.0 {
        DemoScene::Visualizer
    } else if cycle < 23.0 {
        DemoScene::StarfieldWarp
    } else if cycle < 38.0 {
        DemoScene::PulseReactorChamber
    } else if cycle < 53.0 {
        DemoScene::ParticleStorm
    } else {
        DemoScene::ReturnToTitle
    }
}

#[cfg(feature = "real_graphics")]
fn local_scene_time(t: f32) -> f32 {
    let cycle = t.rem_euclid(63.0);
    if cycle < 10.0 {
        cycle
    } else if cycle < 23.0 {
        cycle - 10.0
    } else if cycle < 38.0 {
        cycle - 23.0
    } else if cycle < 53.0 {
        cycle - 38.0
    } else {
        cycle - 53.0
    }
}

#[cfg(feature = "real_graphics")]
fn build_starfield(count: usize) -> Vec<Star> {
    (0..count)
        .map(|i| {
            let x = seeded_unit(0xA0E5_5001_u64, i as u32) * 2.0 - 1.0;
            let y = seeded_unit(0xA0E5_6001_u64, i as u32) * 2.0 - 1.0;
            let z = 0.1 + seeded_unit(0xA0E5_7001_u64, i as u32) * 0.9;
            Star { x, y, z }
        })
        .collect()
}

#[cfg(feature = "real_graphics")]
fn rasterize_demo_scene(
    width: u32,
    height: u32,
    scene: DemoScene,
    t: f32,
    dt: f32,
    pulse: f32,
    particles: &mut [Particle],
    cursor: &mut usize,
    stars: &[Star],
) -> BootFramebuffer {
    let mut frame = BootFramebuffer {
        width,
        height,
        rgba: vec![0; (width as usize) * (height as usize) * 4],
    };
    for px in frame.rgba.chunks_exact_mut(4) {
        px[3] = 255;
    }

    paint_scene_background(&mut frame.rgba, width, height, t, scene, pulse);
    match scene {
        DemoScene::Visualizer => {
            draw_reactor_rings(&mut frame.rgba, width, height, t, pulse, 2);
            update_particles(
                particles,
                dt,
                cursor,
                6,
                pulse,
                width,
                height,
                t * 2.0,
                18.0,
            );
            draw_particles(
                &mut frame.rgba,
                width,
                height,
                particles,
                [255, 210, 145],
                1.0,
            );
            draw_title_centered(&mut frame.rgba, width, height, "AUREX-X", 4, pulse, t);
        }
        DemoScene::StarfieldWarp => {
            draw_starfield(&mut frame.rgba, width, height, stars, t, pulse);
            draw_title_centered(&mut frame.rgba, width, height, "AUREX-X", 3, pulse * 0.7, t);
        }
        DemoScene::PulseReactorChamber => {
            draw_reactor_rings(&mut frame.rgba, width, height, t * 0.7, pulse, 4);
            draw_title_centered(&mut frame.rgba, width, height, "AUREX-X", 5, pulse, t);
        }
        DemoScene::ParticleStorm => {
            update_particles(
                particles,
                dt,
                cursor,
                10,
                pulse,
                width,
                height,
                t * 3.0,
                75.0,
            );
            swirl_particles(particles, dt, width, height, t);
            draw_particles(
                &mut frame.rgba,
                width,
                height,
                particles,
                [180, 225, 255],
                1.3,
            );
            draw_title_centered(&mut frame.rgba, width, height, "AUREX-X", 3, pulse * 0.6, t);
        }
        DemoScene::ReturnToTitle => {
            let fade = (1.0 - (t / 10.0)).clamp(0.0, 1.0);
            update_particles(
                particles,
                dt,
                cursor,
                (4.0 * fade) as usize,
                pulse,
                width,
                height,
                t,
                20.0 * fade,
            );
            draw_particles(
                &mut frame.rgba,
                width,
                height,
                particles,
                [120, 190, 255],
                fade,
            );
            draw_title_centered(&mut frame.rgba, width, height, "AUREX-X", 6, pulse, t * 0.6);
        }
    }

    frame
}

#[cfg(feature = "real_graphics")]
fn paint_scene_background(
    rgba: &mut [u8],
    width: u32,
    height: u32,
    t: f32,
    scene: DemoScene,
    pulse: f32,
) {
    let (base_r, base_g, base_b) = match scene {
        DemoScene::Visualizer => (8.0, 16.0, 32.0),
        DemoScene::StarfieldWarp => (4.0, 7.0, 14.0),
        DemoScene::PulseReactorChamber => (10.0, 14.0, 24.0),
        DemoScene::ParticleStorm => (7.0, 10.0, 22.0),
        DemoScene::ReturnToTitle => (4.0, 9.0, 18.0),
    };
    for y in 0..height {
        for x in 0..width {
            let idx = ((y * width + x) * 4) as usize;
            let nx = x as f32 / width as f32;
            let ny = y as f32 / height as f32;
            let vignette = (1.0 - ((nx - 0.5).powi(2) + (ny - 0.5).powi(2)) * 1.8).clamp(0.0, 1.0);
            let drift = ((nx * 5.0 + ny * 3.0 + t * 0.35).sin() * 0.5 + 0.5) * 8.0;
            let glow = 1.0 + pulse * 0.25;
            rgba[idx] = ((base_r + drift) * vignette * glow).clamp(0.0, 255.0) as u8;
            rgba[idx + 1] = ((base_g + drift * 1.2) * vignette * glow).clamp(0.0, 255.0) as u8;
            rgba[idx + 2] = ((base_b + drift * 1.4) * vignette * glow).clamp(0.0, 255.0) as u8;
        }
    }
}

#[cfg(feature = "real_graphics")]
fn draw_reactor_rings(
    rgba: &mut [u8],
    width: u32,
    height: u32,
    t: f32,
    pulse: f32,
    ring_count: usize,
) {
    let cx = width as f32 * 0.5;
    let cy = height as f32 * 0.52;
    let min_dim = width.min(height) as f32;
    for y in 0..height {
        for x in 0..width {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            let r = (dx * dx + dy * dy).sqrt();
            let mut lum = 0.0;
            for i in 0..ring_count {
                let base = min_dim * (0.10 + i as f32 * 0.08);
                let orbit = (t * (0.6 + i as f32 * 0.2)).sin() * 8.0;
                let d = (r - (base + orbit)).abs();
                lum += (1.0 - d / 7.0).clamp(0.0, 1.0) * (0.22 + pulse * 0.25);
            }
            if lum > 0.01 {
                blend_pixel(
                    rgba,
                    width,
                    height,
                    x as i32,
                    y as i32,
                    [70, 170, 255],
                    (lum * 180.0).clamp(0.0, 255.0) as u8,
                );
            }
        }
    }
}

#[cfg(feature = "real_graphics")]
fn draw_title_centered(
    rgba: &mut [u8],
    width: u32,
    height: u32,
    text: &str,
    scale: i32,
    pulse: f32,
    t: f32,
) {
    let angle = t * 0.25;
    let orbit_x = (angle.sin() * 22.0) as i32;
    let perspective = 1.0 + angle.cos() * 0.05;
    let s = ((scale as f32) * perspective).clamp(1.0, 12.0) as i32;
    let w = text_pixel_width(text, s);
    let h = 7 * s;
    let x = (width as i32 - w) / 2 + orbit_x;
    let y = (height as i32 - h) / 2;
    let glow = 0.65 + pulse * 0.35;
    draw_text(
        rgba,
        width,
        height,
        text,
        x - 2,
        y,
        s,
        [60, 145, 255],
        0.20 * glow,
    );
    draw_text(
        rgba,
        width,
        height,
        text,
        x,
        y,
        s,
        [210, 240, 255],
        0.85 * glow,
    );
}

#[cfg(feature = "real_graphics")]
fn update_particles(
    particles: &mut [Particle],
    dt: f32,
    cursor: &mut usize,
    spawn_count: usize,
    pulse: f32,
    width: u32,
    height: u32,
    t: f32,
    speed: f32,
) {
    let cx = width as f32 * 0.5;
    let cy = height as f32 * 0.5;
    for p in particles.iter_mut() {
        if p.active {
            p.x += p.vx * dt;
            p.y += p.vy * dt;
            p.life += dt;
            if p.life >= p.max_life {
                p.active = false;
            }
        }
    }

    let actual_spawn = ((spawn_count as f32) * (0.4 + pulse * 0.9)).round() as usize;
    for i in 0..actual_spawn.min(particles.len()) {
        let idx = (*cursor + i) % particles.len();
        let angle = (t * 1.3 + i as f32 * 0.618).rem_euclid(std::f32::consts::TAU);
        let radius = 36.0 + pulse * 18.0;
        particles[idx] = Particle {
            x: cx + angle.cos() * radius,
            y: cy + angle.sin() * radius,
            vx: angle.cos() * speed + (seeded_unit(0xABCD5000, idx as u32) - 0.5) * 8.0,
            vy: angle.sin() * speed + (seeded_unit(0xABCD6000, idx as u32) - 0.5) * 8.0,
            life: 0.0,
            max_life: 0.8 + seeded_unit(0xABCD7000, idx as u32) * 0.7,
            active: true,
        };
    }
    *cursor = (*cursor + actual_spawn) % particles.len().max(1);
}

#[cfg(feature = "real_graphics")]
fn swirl_particles(particles: &mut [Particle], dt: f32, width: u32, height: u32, t: f32) {
    let cx = width as f32 * 0.5;
    let cy = height as f32 * 0.5;
    for p in particles.iter_mut().filter(|p| p.active) {
        let dx = p.x - cx;
        let dy = p.y - cy;
        let spin = (t * 0.8 + (dx + dy) * 0.01).sin() * 25.0;
        p.vx += (-dy.signum() * spin) * dt;
        p.vy += (dx.signum() * spin) * dt;
    }
}

#[cfg(feature = "real_graphics")]
fn draw_particles(
    rgba: &mut [u8],
    width: u32,
    height: u32,
    particles: &[Particle],
    color: [u8; 3],
    brightness: f32,
) {
    for p in particles.iter().filter(|p| p.active) {
        let life_t = (1.0 - p.life / p.max_life).clamp(0.0, 1.0);
        let alpha = (life_t * 180.0 * brightness).clamp(0.0, 255.0) as u8;
        let x = p.x as i32;
        let y = p.y as i32;
        for oy in -1..=1 {
            for ox in -1..=1 {
                let a = if ox == 0 && oy == 0 { alpha } else { alpha / 3 };
                blend_pixel(rgba, width, height, x + ox, y + oy, color, a);
            }
        }
    }
}

#[cfg(feature = "real_graphics")]
fn draw_starfield(rgba: &mut [u8], width: u32, height: u32, stars: &[Star], t: f32, pulse: f32) {
    let cx = width as f32 * 0.5;
    let cy = height as f32 * 0.5;
    let speed = 0.22 + pulse * 0.95;
    for (i, star) in stars.iter().enumerate() {
        let mut z = star.z - (t * speed).fract();
        if z <= 0.05 {
            z += 1.0;
        }
        let depth = (1.0 - z).clamp(0.0, 1.0);
        let sx = cx + (star.x / z) * (width as f32 * 0.45);
        let sy = cy + (star.y / z) * (height as f32 * 0.45);
        let alpha = (65.0 + depth * 190.0).clamp(0.0, 255.0) as u8;
        let color = [
            (150.0 + depth * 70.0) as u8,
            (170.0 + depth * 60.0) as u8,
            (220.0 + (i % 17) as f32) as u8,
        ];
        blend_pixel(rgba, width, height, sx as i32, sy as i32, color, alpha);
    }
}

#[cfg(not(feature = "real_graphics"))]
pub fn run_real_renderer_event_loop() -> Result<(), String> {
    Err("real_graphics feature is disabled".to_string())
}

#[cfg(not(feature = "real_graphics"))]
pub fn run_real_renderer_event_loop_with_frame_hook<F>(_on_frame: F) -> Result<(), String>
where
    F: FnMut(f32) + 'static,
{
    Err("real_graphics feature is disabled".to_string())
}

#[cfg(feature = "real_graphics")]
fn overlay_boot_caption(rgba: &mut [u8], width: u32, height: u32, frame: &BootScreenFrame) {
    if width < 64 || height < 32 {
        return;
    }

    let w = width as i32;
    let h = height as i32;
    let title_scale = (width as f32 / 360.0).clamp(1.0, 3.0) as i32;
    let subtitle_scale = (title_scale - 1).max(1);

    let title = "AUREX-X";
    let subtitle = "Prime Pulse online";

    let title_w = text_pixel_width(title, title_scale);
    let subtitle_w = text_pixel_width(subtitle, subtitle_scale);

    let title_x = (w - title_w) / 2;
    let title_y = (h as f32 * 0.14) as i32;
    let subtitle_x = (w - subtitle_w) / 2;
    let subtitle_y = title_y + 10 * title_scale;

    let glow = frame.title_glow.clamp(0.0, 2.0);
    let title_intensity = (frame.title_progress * 1.2).clamp(0.0, 1.0);

    draw_text(
        rgba,
        width,
        height,
        title,
        title_x,
        title_y,
        title_scale,
        [
            (110.0 + glow * 70.0) as u8,
            (175.0 + glow * 45.0) as u8,
            255,
        ],
        title_intensity,
    );

    draw_text(
        rgba,
        width,
        height,
        subtitle,
        subtitle_x,
        subtitle_y,
        subtitle_scale,
        [85, 135, 210],
        (0.45 + 0.55 * frame.title_progress).clamp(0.0, 1.0),
    );

    let bar_x0 = (width as f32 * 0.14) as u32;
    let bar_x1 = (width as f32 * 0.86) as u32;
    let bar_y0 = (height as f32 * 0.86) as u32;
    let bar_y1 = (height as f32 * 0.91) as u32;
    let bar_w = bar_x1.saturating_sub(bar_x0).max(1);
    let fill = (bar_w as f32 * frame.title_progress.clamp(0.0, 1.0)) as u32;

    for y in bar_y0..bar_y1.min(height) {
        for x in bar_x0..bar_x1.min(width) {
            let idx = ((y * width + x) * 4) as usize;
            let t = ((x - bar_x0) as f32 / bar_w as f32).clamp(0.0, 1.0);
            let active = x <= bar_x0.saturating_add(fill);
            let segment = (((x - bar_x0) / 10) % 2) == 0;
            let scan = (((x + y) as f32 * 0.03 + frame.title_glow * 4.0).sin() * 0.5 + 0.5)
                .clamp(0.0, 1.0);

            let (r, g, b) = if active {
                let boost = if segment { 1.0 } else { 0.82 };
                (
                    ((95.0 + 150.0 * t) * boost * (0.75 + 0.25 * scan)).clamp(0.0, 255.0) as u8,
                    ((150.0 + 90.0 * t) * boost * (0.75 + 0.25 * scan)).clamp(0.0, 255.0) as u8,
                    ((225.0 + 25.0 * t) * boost).clamp(0.0, 255.0) as u8,
                )
            } else {
                (22, 30, 45)
            };
            rgba[idx] = rgba[idx].saturating_add(r / 2);
            rgba[idx + 1] = rgba[idx + 1].saturating_add(g / 2);
            rgba[idx + 2] = rgba[idx + 2].saturating_add(b / 2);
            rgba[idx + 3] = 255;
        }
    }
}

#[cfg(feature = "real_graphics")]
fn text_pixel_width(text: &str, scale: i32) -> i32 {
    text.chars()
        .map(|c| if c == ' ' { 4 } else { 6 })
        .sum::<i32>()
        * scale
}

#[cfg(feature = "real_graphics")]
fn draw_text(
    rgba: &mut [u8],
    width: u32,
    height: u32,
    text: &str,
    mut x: i32,
    y: i32,
    scale: i32,
    color: [u8; 3],
    intensity: f32,
) {
    for ch in text.chars() {
        if ch == ' ' {
            x += 4 * scale;
            continue;
        }
        let glyph = glyph_5x7(ch);
        for (row, bits) in glyph.iter().enumerate() {
            for col in 0..5 {
                if (bits >> (4 - col)) & 1 == 1 {
                    for sy in 0..scale {
                        for sx in 0..scale {
                            blend_pixel(
                                rgba,
                                width,
                                height,
                                x + col * scale + sx,
                                y + row as i32 * scale + sy,
                                color,
                                (220.0 * intensity).clamp(0.0, 255.0) as u8,
                            );
                        }
                    }
                }
            }
        }
        x += 6 * scale;
    }
}

#[cfg(feature = "real_graphics")]
fn glyph_5x7(c: char) -> [u8; 7] {
    match c.to_ascii_uppercase() {
        'A' => [
            0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
        ],
        'E' => [
            0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111,
        ],
        'I' => [
            0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b11111,
        ],
        'L' => [
            0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111,
        ],
        'M' => [
            0b10001, 0b11011, 0b10101, 0b10001, 0b10001, 0b10001, 0b10001,
        ],
        'N' => [
            0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001, 0b10001,
        ],
        'O' => [
            0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
        ],
        'P' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000,
        ],
        'R' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001,
        ],
        'S' => [
            0b01111, 0b10000, 0b10000, 0b01110, 0b00001, 0b00001, 0b11110,
        ],
        'U' => [
            0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
        ],
        'X' => [
            0b10001, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001, 0b10001,
        ],
        '-' => [
            0b00000, 0b00000, 0b00000, 0b11111, 0b00000, 0b00000, 0b00000,
        ],
        _ => [0; 7],
    }
}

#[cfg(feature = "real_graphics")]
fn blend_pixel(
    rgba: &mut [u8],
    width: u32,
    height: u32,
    x: i32,
    y: i32,
    color: [u8; 3],
    alpha: u8,
) {
    if x < 0 || y < 0 || x as u32 >= width || y as u32 >= height {
        return;
    }
    let idx = (((y as u32) * width + x as u32) * 4) as usize;
    let a = alpha as u16;
    rgba[idx] = ((rgba[idx] as u16 * (255 - a) + color[0] as u16 * a) / 255) as u8;
    rgba[idx + 1] = ((rgba[idx + 1] as u16 * (255 - a) + color[1] as u16 * a) / 255) as u8;
    rgba[idx + 2] = ((rgba[idx + 2] as u16 * (255 - a) + color[2] as u16 * a) / 255) as u8;
    rgba[idx + 3] = 255;
}

pub fn attempt_real_renderer_bootstrap() -> RealRendererBootstrapStatus {
    #[cfg(feature = "real_graphics")]
    {
        let instance = wgpu::Instance::default();
        let adapter =
            match pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })) {
                Ok(adapter) => adapter,
                Err(err) => {
                    return RealRendererBootstrapStatus {
                        result: RealRendererBootstrapResult::AdapterUnavailable,
                        detail: format!("request_adapter failed: {err}"),
                    };
                }
            };

        if let Err(err) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("Aurex-X Bootstrap Device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            memory_hints: wgpu::MemoryHints::default(),
            trace: wgpu::Trace::Off,
        })) {
            return RealRendererBootstrapStatus {
                result: RealRendererBootstrapResult::DeviceRequestFailed,
                detail: format!("request_device failed: {err}"),
            };
        }

        return RealRendererBootstrapStatus {
            result: RealRendererBootstrapResult::Ready,
            detail: "adapter acquired; device initialized; surface configuration deferred to runtime loop".to_string(),
        };
    }

    #[cfg(not(feature = "real_graphics"))]
    {
        RealRendererBootstrapStatus {
            result: RealRendererBootstrapResult::FeatureDisabled,
            detail: "build without real_graphics feature".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BootFramebuffer {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

impl BootFramebuffer {
    pub fn pixel_count(&self) -> usize {
        self.rgba.len() / 4
    }

    pub fn lit_pixel_count(&self) -> usize {
        self.rgba
            .chunks_exact(4)
            .filter(|px| px[0] > 0 || px[1] > 0 || px[2] > 0)
            .count()
    }

    pub fn checksum(&self) -> u64 {
        self.rgba
            .iter()
            .enumerate()
            .fold(0_u64, |acc, (idx, byte)| {
                let weighted = (*byte as u64).wrapping_mul((idx as u64 % 251) + 1);
                acc.wrapping_mul(1_099_511_628_211).wrapping_add(weighted)
            })
    }
}

pub fn rasterize_boot_frame(frame: &BootFrame, width: u32, height: u32) -> BootFramebuffer {
    let mut rgba = vec![0_u8; width as usize * height as usize * 4];
    let cx = width as f32 * 0.5;
    let cy = height as f32 * 0.44;
    let min_dim = width.min(height) as f32;
    let ring_radius = (frame.ring_radius * min_dim * 0.22).max(1.0);
    let ring_thickness = (2.5 + frame.glow * 5.5).clamp(1.5, 14.0);

    let hue = ((frame.hue_shift + 360.0) % 360.0) / 360.0;
    let base_r = (0.28 + hue * 0.85).fract();
    let base_g = (0.52 + hue * 0.63).fract();
    let base_b = (0.92 + hue * 0.47).fract();

    let t = frame.frame_index as f32 * 0.08;
    let halo_radius = ring_radius * (1.23 + 0.04 * (t * 1.7).sin());
    let inner_radius = ring_radius * (0.84 + 0.03 * (t * 2.3).cos());

    for y in 0..height {
        for x in 0..width {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            let dist = (dx * dx + dy * dy).sqrt();

            let ring_delta = (dist - ring_radius).abs();
            let ring_core = (1.0 - (ring_delta / ring_thickness))
                .clamp(0.0, 1.0)
                .powf(0.7);

            let halo_delta = (dist - halo_radius).abs();
            let halo = (1.0 - halo_delta / (ring_thickness * 3.8))
                .clamp(0.0, 1.0)
                .powf(2.2)
                * (0.55 + frame.glow * 0.45);

            let inner_delta = (dist - inner_radius).abs();
            let inner_ring = (1.0 - inner_delta / (ring_thickness * 0.75)).clamp(0.0, 1.0)
                * (0.22 + 0.18 * (t * 2.1).sin().abs());

            let center_glow = (1.0 - dist / (ring_radius * 1.45))
                .clamp(0.0, 1.0)
                .powf(1.5)
                * (0.26 + frame.glow * 0.5);

            let mut intensity =
                (ring_core * 0.95 + halo * 0.72 + inner_ring + center_glow).clamp(0.0, 1.0);

            let vignette_dx = (x as f32 / width as f32 - 0.5) * 2.0;
            let vignette_dy = (y as f32 / height as f32 - 0.5) * 2.0;
            let vignette = (1.0 - (vignette_dx * vignette_dx + vignette_dy * vignette_dy) * 0.35)
                .clamp(0.45, 1.0);
            intensity *= vignette;

            let scan =
                0.9 + 0.1 * ((y as f32 * 0.22 + frame.scanline_offset * 12.0).sin() * 0.5 + 0.5);

            let idx = ((y * width + x) * 4) as usize;
            rgba[idx] = (base_r * intensity * scan * 255.0) as u8;
            rgba[idx + 1] = (base_g * intensity * scan * 255.0) as u8;
            rgba[idx + 2] = (base_b * intensity * scan * 255.0) as u8;
            rgba[idx + 3] = (intensity * 255.0) as u8;
        }
    }

    let spark_count = 28;
    for i in 0..spark_count {
        let fi = i as f32;
        let ang = fi / spark_count as f32 * std::f32::consts::TAU + t * 0.7;
        let radial = ring_radius * (1.02 + 0.16 * ((t * 0.9 + fi).sin() * 0.5 + 0.5));
        let sx = cx + radial * ang.cos();
        let sy = cy + radial * ang.sin();
        let sparkle = (0.4 + 0.6 * ((t * 2.0 + fi * 0.61).sin() * 0.5 + 0.5)).powf(2.0);
        let size = 1.0 + 1.8 * sparkle;

        for oy in -3..=3 {
            for ox in -3..=3 {
                let px = sx as i32 + ox;
                let py = sy as i32 + oy;
                if px < 0 || py < 0 || px as u32 >= width || py as u32 >= height {
                    continue;
                }
                let d = ((ox * ox + oy * oy) as f32).sqrt();
                let fall = (1.0 - d / (3.0 * size)).clamp(0.0, 1.0) * sparkle;
                if fall <= 0.0 {
                    continue;
                }
                let idx = (((py as u32) * width + px as u32) * 4) as usize;
                rgba[idx] = rgba[idx].saturating_add((70.0 * fall) as u8);
                rgba[idx + 1] = rgba[idx + 1].saturating_add((120.0 * fall) as u8);
                rgba[idx + 2] = rgba[idx + 2].saturating_add((210.0 * fall) as u8);
                rgba[idx + 3] = 255;
            }
        }
    }

    BootFramebuffer {
        width,
        height,
        rgba,
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RenderFrameStats {
    pub frame_id: u64,
    pub stages_executed: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RenderBackendStatus {
    pub mode: RenderBackendMode,
    pub ready: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendTransition {
    Noop,
    Transitioned,
}

#[derive(Debug)]
pub struct MockRenderer {
    config: RenderBootstrapConfig,
    frames_rendered: u64,
    backend_ready: bool,
}

impl MockRenderer {
    pub fn new(config: RenderBootstrapConfig) -> Self {
        let backend_ready = config.backend_mode == RenderBackendMode::Mock;
        Self {
            config,
            frames_rendered: 0,
            backend_ready,
        }
    }

    pub fn config(&self) -> &RenderBootstrapConfig {
        &self.config
    }

    pub fn backend_status(&self) -> RenderBackendStatus {
        RenderBackendStatus {
            mode: self.config.backend_mode,
            ready: self.backend_ready,
        }
    }

    pub fn transition_backend_mode(&mut self, mode: RenderBackendMode) -> BackendTransition {
        if self.config.backend_mode == mode {
            return BackendTransition::Noop;
        }

        self.config.backend_mode = mode;
        self.backend_ready = mode == RenderBackendMode::Mock;
        BackendTransition::Transitioned
    }

    pub fn run_frame(&mut self, stages: &[RenderStage]) -> RenderFrameStats {
        self.frames_rendered += 1;
        RenderFrameStats {
            frame_id: self.frames_rendered,
            stages_executed: stages.len(),
        }
    }
}

pub const RENDER_MAIN_STAGES: [RenderStage; 3] = [
    RenderStage::RenderPrepare,
    RenderStage::Render,
    RenderStage::Present,
];

#[derive(Debug, Clone)]
pub struct BootAnimationConfig {
    pub seed: u64,
    pub frame_count: u32,
    pub base_radius: f32,
    pub pulse_speed: f32,
}

impl Default for BootAnimationConfig {
    fn default() -> Self {
        Self {
            seed: 0xA9E3_0001_u64,
            frame_count: 16,
            base_radius: 1.0,
            pulse_speed: 0.35,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BootFrame {
    pub frame_index: u32,
    pub tick: u64,
    pub ring_radius: f32,
    pub glow: f32,
    pub hue_shift: f32,
    pub scanline_offset: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootPhase {
    Ignition,
    PulseLock,
    Reveal,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhaseStyle {
    pub intensity_mul: f32,
    pub hue_bias: f32,
    pub distortion_weight: f32,
    pub curve_exp: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootStylePreset {
    Classic,
    NeonStorm,
    CrystalPulse,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BootStyleProfile {
    pub ignition: PhaseStyle,
    pub pulse_lock: PhaseStyle,
    pub reveal: PhaseStyle,
    pub preset: BootStylePreset,
}

impl BootStyleProfile {
    pub fn from_preset(preset: BootStylePreset) -> Self {
        match preset {
            BootStylePreset::Classic => Self::default(),
            BootStylePreset::NeonStorm => Self {
                ignition: PhaseStyle {
                    intensity_mul: 0.95,
                    hue_bias: 24.0,
                    distortion_weight: 0.65,
                    curve_exp: 1.3,
                },
                pulse_lock: PhaseStyle {
                    intensity_mul: 1.3,
                    hue_bias: 42.0,
                    distortion_weight: 0.95,
                    curve_exp: 1.7,
                },
                reveal: PhaseStyle {
                    intensity_mul: 1.05,
                    hue_bias: 16.0,
                    distortion_weight: 0.45,
                    curve_exp: 1.1,
                },
                preset,
            },
            BootStylePreset::CrystalPulse => Self {
                ignition: PhaseStyle {
                    intensity_mul: 0.8,
                    hue_bias: -18.0,
                    distortion_weight: 0.4,
                    curve_exp: 0.9,
                },
                pulse_lock: PhaseStyle {
                    intensity_mul: 1.05,
                    hue_bias: -4.0,
                    distortion_weight: 0.55,
                    curve_exp: 1.2,
                },
                reveal: PhaseStyle {
                    intensity_mul: 1.2,
                    hue_bias: 8.0,
                    distortion_weight: 0.25,
                    curve_exp: 0.8,
                },
                preset,
            },
        }
    }

    pub fn style_for(&self, phase: BootPhase) -> PhaseStyle {
        match phase {
            BootPhase::Ignition => self.ignition,
            BootPhase::PulseLock => self.pulse_lock,
            BootPhase::Reveal => self.reveal,
        }
    }
}

impl Default for BootStyleProfile {
    fn default() -> Self {
        Self {
            ignition: PhaseStyle {
                intensity_mul: 0.85,
                hue_bias: -12.0,
                distortion_weight: 0.55,
                curve_exp: 1.0,
            },
            pulse_lock: PhaseStyle {
                intensity_mul: 1.15,
                hue_bias: 18.0,
                distortion_weight: 0.8,
                curve_exp: 1.4,
            },
            reveal: PhaseStyle {
                intensity_mul: 1.0,
                hue_bias: 4.0,
                distortion_weight: 0.3,
                curve_exp: 0.9,
            },
            preset: BootStylePreset::Classic,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootSequenceRecipe {
    Standard,
    QuickPulse,
    GrandReveal,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BootSequenceConfig {
    pub ignition_ratio: f32,
    pub pulse_lock_ratio: f32,
    pub reveal_ratio: f32,
    pub pulse_speed_mul: f32,
}

impl BootSequenceConfig {
    pub fn from_recipe(recipe: BootSequenceRecipe) -> Self {
        match recipe {
            BootSequenceRecipe::Standard => Self {
                ignition_ratio: 0.33,
                pulse_lock_ratio: 0.34,
                reveal_ratio: 0.33,
                pulse_speed_mul: 1.0,
            },
            BootSequenceRecipe::QuickPulse => Self {
                ignition_ratio: 0.22,
                pulse_lock_ratio: 0.5,
                reveal_ratio: 0.28,
                pulse_speed_mul: 1.2,
            },
            BootSequenceRecipe::GrandReveal => Self {
                ignition_ratio: 0.38,
                pulse_lock_ratio: 0.27,
                reveal_ratio: 0.35,
                pulse_speed_mul: 0.85,
            },
        }
    }
}

impl Default for BootSequenceConfig {
    fn default() -> Self {
        Self::from_recipe(BootSequenceRecipe::Standard)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BootTimelineFrame {
    pub phase: BootPhase,
    pub frame: BootFrame,
    pub phase_t: f32,
    pub styled_glow: f32,
    pub styled_hue: f32,
    pub distortion_weight: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BootRenderIntent {
    pub bloom_weight: f32,
    pub distortion_weight: f32,
    pub fog_weight: f32,
    pub color_shift: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BootPostFxSnapshot {
    pub tick: u64,
    pub bloom_strength: f32,
    pub fog_density: f32,
    pub distortion_amount: f32,
    pub color_grade_shift: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BootPostFxAggregate {
    pub avg_bloom: f32,
    pub avg_fog: f32,
    pub avg_distortion: f32,
    pub avg_color_shift: f32,
    pub peak_bloom: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BootPostFxTrack {
    pub snapshots: Vec<BootPostFxSnapshot>,
}

impl BootPostFxTrack {
    pub fn from_timeline(timeline: &BootTimeline) -> Self {
        Self {
            snapshots: timeline.to_postfx_snapshots(),
        }
    }

    pub fn snapshot_for_tick(&self, tick: u64) -> Option<BootPostFxSnapshot> {
        self.snapshots.iter().find(|s| s.tick == tick).copied()
    }

    pub fn latest_snapshot(&self) -> Option<BootPostFxSnapshot> {
        self.snapshots.last().copied()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BootScreenFrame {
    pub tick: u64,
    pub title_progress: f32,
    pub title_glow: f32,
    pub subtitle_opacity: f32,
    pub glyphs_lit: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BootScreenSequence {
    pub title_text: String,
    pub subtitle_text: String,
    pub frames: Vec<BootScreenFrame>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BootTimeline {
    pub frames: Vec<BootTimelineFrame>,
}

impl BootTimeline {
    pub fn phase_counts(&self) -> (usize, usize, usize) {
        let mut ignition = 0;
        let mut pulse_lock = 0;
        let mut reveal = 0;

        for f in &self.frames {
            match f.phase {
                BootPhase::Ignition => ignition += 1,
                BootPhase::PulseLock => pulse_lock += 1,
                BootPhase::Reveal => reveal += 1,
            }
        }

        (ignition, pulse_lock, reveal)
    }

    pub fn derive_render_intents(&self) -> Vec<BootRenderIntent> {
        self.frames
            .iter()
            .map(|f| {
                let (phase_bloom, phase_fog) = match f.phase {
                    BootPhase::Ignition => (0.85, 0.25),
                    BootPhase::PulseLock => (1.15, 0.5),
                    BootPhase::Reveal => (1.0, 0.7),
                };

                BootRenderIntent {
                    bloom_weight: f.styled_glow * phase_bloom,
                    distortion_weight: f.distortion_weight,
                    fog_weight: (0.2 + f.phase_t * 0.8) * phase_fog,
                    color_shift: f.styled_hue,
                }
            })
            .collect()
    }

    pub fn to_postfx_snapshots(&self) -> Vec<BootPostFxSnapshot> {
        let intents = self.derive_render_intents();
        self.frames
            .iter()
            .zip(intents.iter())
            .map(|(frame, intent)| BootPostFxSnapshot {
                tick: frame.frame.tick,
                bloom_strength: intent.bloom_weight,
                fog_density: intent.fog_weight,
                distortion_amount: intent.distortion_weight,
                color_grade_shift: intent.color_shift,
            })
            .collect()
    }

    pub fn to_boot_screen_sequence(
        &self,
        title_text: &str,
        subtitle_text: &str,
    ) -> BootScreenSequence {
        let glyph_count = title_text.chars().count().max(1);
        let frames = self
            .frames
            .iter()
            .map(|f| {
                let reveal_weight = match f.phase {
                    BootPhase::Ignition => 0.2,
                    BootPhase::PulseLock => 0.65,
                    BootPhase::Reveal => 1.0,
                };
                let title_progress = (f.phase_t * reveal_weight).clamp(0.0, 1.0);
                let glyphs_lit =
                    ((title_progress * glyph_count as f32).ceil() as usize).clamp(1, glyph_count);
                let title_glow = (f.styled_glow * (0.55 + reveal_weight * 0.45)).clamp(0.0, 2.0);
                let subtitle_opacity = (0.2 + title_progress * 0.8).clamp(0.0, 1.0);

                BootScreenFrame {
                    tick: f.frame.tick,
                    title_progress,
                    title_glow,
                    subtitle_opacity,
                    glyphs_lit,
                }
            })
            .collect();

        BootScreenSequence {
            title_text: title_text.to_string(),
            subtitle_text: subtitle_text.to_string(),
            frames,
        }
    }

    pub fn aggregate_postfx(&self) -> BootPostFxAggregate {
        let snapshots = self.to_postfx_snapshots();
        let len = snapshots.len().max(1) as f32;

        let avg_bloom = snapshots.iter().map(|s| s.bloom_strength).sum::<f32>() / len;
        let avg_fog = snapshots.iter().map(|s| s.fog_density).sum::<f32>() / len;
        let avg_distortion = snapshots.iter().map(|s| s.distortion_amount).sum::<f32>() / len;
        let avg_color_shift = snapshots.iter().map(|s| s.color_grade_shift).sum::<f32>() / len;
        let peak_bloom = snapshots
            .iter()
            .map(|s| s.bloom_strength)
            .fold(0.0_f32, f32::max);

        BootPostFxAggregate {
            avg_bloom,
            avg_fog,
            avg_distortion,
            avg_color_shift,
            peak_bloom,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BootAnimator {
    config: BootAnimationConfig,
    style: BootStyleProfile,
    sequence: BootSequenceConfig,
    recipe: BootSequenceRecipe,
}

impl BootAnimator {
    pub fn new(config: BootAnimationConfig) -> Self {
        Self {
            config,
            style: BootStyleProfile::default(),
            sequence: BootSequenceConfig::default(),
            recipe: BootSequenceRecipe::Standard,
        }
    }

    pub fn with_style(config: BootAnimationConfig, style: BootStyleProfile) -> Self {
        Self {
            config,
            style,
            sequence: BootSequenceConfig::default(),
            recipe: BootSequenceRecipe::Standard,
        }
    }

    pub fn with_style_and_recipe(
        config: BootAnimationConfig,
        style: BootStyleProfile,
        recipe: BootSequenceRecipe,
    ) -> Self {
        Self {
            config,
            style,
            sequence: BootSequenceConfig::from_recipe(recipe),
            recipe,
        }
    }

    pub fn recipe(&self) -> BootSequenceRecipe {
        self.recipe
    }

    pub fn generate_frames(&self, start_tick: u64) -> Vec<BootFrame> {
        (0..self.config.frame_count)
            .map(|i| {
                let t = i as f32 * self.config.pulse_speed * self.sequence.pulse_speed_mul;
                let noise = seeded_unit(self.config.seed, i);
                let ring_radius = self.config.base_radius + (t.sin() * 0.18) + (noise * 0.07);
                let glow = 0.55 + (t.cos().abs() * 0.35) + (noise * 0.1);
                let hue_shift = (noise * 120.0) + (t.sin() * 35.0);
                let scanline_offset = ((i as f32 * 0.11) + noise).fract();

                BootFrame {
                    frame_index: i,
                    tick: start_tick + i as u64,
                    ring_radius,
                    glow,
                    hue_shift,
                    scanline_offset,
                }
            })
            .collect()
    }

    pub fn generate_timeline(&self, start_tick: u64) -> BootTimeline {
        let raw = self.generate_frames(start_tick);
        let total = raw.len().max(1);

        let ignition_end = ((total as f32 * self.sequence.ignition_ratio).round() as usize)
            .clamp(1, total.saturating_sub(2).max(1));
        let pulse_lock_end = ((total as f32
            * (self.sequence.ignition_ratio + self.sequence.pulse_lock_ratio))
            .round() as usize)
            .clamp(
                ignition_end + 1,
                total.saturating_sub(1).max(ignition_end + 1),
            );

        let ignition_span = ignition_end.max(1);
        let pulse_lock_span = pulse_lock_end.saturating_sub(ignition_end).max(1);
        let reveal_span = total.saturating_sub(pulse_lock_end).max(1);

        let frames = raw
            .into_iter()
            .enumerate()
            .map(|(idx, frame)| {
                let (phase, local_idx, span) = if idx < ignition_end {
                    (BootPhase::Ignition, idx, ignition_span)
                } else if idx < pulse_lock_end {
                    (BootPhase::PulseLock, idx - ignition_end, pulse_lock_span)
                } else {
                    (BootPhase::Reveal, idx - pulse_lock_end, reveal_span)
                };

                let phase_t = (local_idx as f32 / span as f32).clamp(0.0, 1.0);

                let phase_style = self.style.style_for(phase);
                let curve = phase_t.powf(phase_style.curve_exp.max(0.01));

                BootTimelineFrame {
                    phase,
                    phase_t,
                    styled_glow: frame.glow * (phase_style.intensity_mul + 0.1 * curve),
                    styled_hue: frame.hue_shift + phase_style.hue_bias * (0.6 + 0.4 * curve),
                    distortion_weight: phase_style.distortion_weight * (0.75 + 0.25 * curve),
                    frame,
                }
            })
            .collect();

        BootTimeline { frames }
    }
}

fn seeded_unit(seed: u64, frame_index: u32) -> f32 {
    let mut x = seed ^ ((frame_index as u64 + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15));
    x ^= x >> 30;
    x = x.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x ^= x >> 27;
    x = x.wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^= x >> 31;
    (x as f64 / u64::MAX as f64) as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn real_bootstrap_reports_feature_disabled_by_default() {
        let status = attempt_real_renderer_bootstrap();
        #[cfg(not(feature = "real_graphics"))]
        {
            assert_eq!(status.result, RealRendererBootstrapResult::FeatureDisabled);
            assert!(status.detail.contains("without real_graphics"));
        }
    }

    #[test]
    fn mock_renderer_tracks_frame_progress() {
        let mut renderer = MockRenderer::new(RenderBootstrapConfig::default());

        let first = renderer.run_frame(&RENDER_MAIN_STAGES);
        let second = renderer.run_frame(&RENDER_MAIN_STAGES);

        assert_eq!(first.frame_id, 1);
        assert_eq!(second.frame_id, 2);
        assert_eq!(first.stages_executed, 3);
    }

    #[test]
    fn bootstrap_executor_advances_through_steps() {
        let mut executor = RenderBootstrapExecutor::new(RenderBackendMode::WgpuPlanned);

        assert_eq!(executor.completed_count(), 0);
        assert_eq!(executor.total_count(), 7);

        let mut last = None;
        while let Some(step) = executor.execute_next() {
            last = Some(step);
        }

        assert_eq!(executor.completed_count(), executor.total_count());
        assert_eq!(executor.last_completed_step(), last);
        assert_eq!(
            executor.last_completed_step(),
            Some(RenderBootstrapStep::DrawBootScreen)
        );
    }

    #[test]
    fn bootstrap_plan_matches_backend_mode() {
        let mock = RenderBootstrapPlan::for_mode(RenderBackendMode::Mock);
        assert_eq!(mock.ready_count(), 0);
        assert_eq!(mock.total_count(), 7);

        let planned = RenderBootstrapPlan::for_mode(RenderBackendMode::WgpuPlanned);
        assert_eq!(planned.ready_count(), planned.total_count());
        assert!(planned.summary().contains("DrawBootScreen:true"));
    }

    #[test]
    fn readiness_contract_tracks_backend_mode() {
        let mock = RenderBackendReadiness::for_mode(RenderBackendMode::Mock);
        assert!(!mock.has_windowing);
        assert!(!mock.has_gpu_backend);
        assert!(!mock.can_present);

        let planned = RenderBackendReadiness::for_mode(RenderBackendMode::WgpuPlanned);
        assert!(planned.has_windowing);
        assert!(planned.has_gpu_backend);
        assert!(planned.can_present);
    }

    #[test]
    fn backend_status_reflects_mode() {
        let renderer = MockRenderer::new(RenderBootstrapConfig::default());
        let status = renderer.backend_status();

        assert_eq!(status.mode, RenderBackendMode::Mock);
        assert!(status.ready);
    }

    #[test]
    fn transition_to_wgpu_planned_sets_not_ready() {
        let mut renderer = MockRenderer::new(RenderBootstrapConfig::default());
        let t = renderer.transition_backend_mode(RenderBackendMode::WgpuPlanned);

        assert_eq!(t, BackendTransition::Transitioned);
        assert_eq!(
            renderer.backend_status().mode,
            RenderBackendMode::WgpuPlanned
        );
        assert!(!renderer.backend_status().ready);
    }

    #[test]
    fn boot_animation_is_deterministic_for_same_seed() {
        let animator = BootAnimator::new(BootAnimationConfig {
            seed: 42,
            frame_count: 8,
            ..BootAnimationConfig::default()
        });

        let a = animator.generate_frames(100);
        let b = animator.generate_frames(100);
        assert_eq!(a, b);
    }

    #[test]
    fn boot_timeline_covers_all_phases() {
        let timeline = BootAnimator::new(BootAnimationConfig {
            seed: 7,
            frame_count: 12,
            ..BootAnimationConfig::default()
        })
        .generate_timeline(0);

        let (ignition, pulse_lock, reveal) = timeline.phase_counts();
        assert!(ignition > 0);
        assert!(pulse_lock > 0);
        assert!(reveal > 0);
        assert_eq!(ignition + pulse_lock + reveal, 12);
    }

    #[test]
    fn boot_animation_changes_with_seed() {
        let a = BootAnimator::new(BootAnimationConfig {
            seed: 1,
            frame_count: 4,
            ..BootAnimationConfig::default()
        })
        .generate_frames(0);
        let b = BootAnimator::new(BootAnimationConfig {
            seed: 2,
            frame_count: 4,
            ..BootAnimationConfig::default()
        })
        .generate_frames(0);

        assert_ne!(a, b);
    }

    #[test]
    fn phase_style_profile_is_applied() {
        let style = BootStyleProfile::default();
        let timeline = BootAnimator::with_style(
            BootAnimationConfig {
                seed: 3,
                frame_count: 12,
                ..BootAnimationConfig::default()
            },
            style,
        )
        .generate_timeline(5);

        let first = &timeline.frames[0];
        let last = &timeline.frames[timeline.frames.len() - 1];

        assert_eq!(first.phase, BootPhase::Ignition);
        assert_eq!(last.phase, BootPhase::Reveal);
        assert!(first.distortion_weight > 0.0);
        assert!(last.distortion_weight > 0.0);
    }

    #[test]
    fn preset_selection_changes_styling() {
        let cfg = BootAnimationConfig {
            seed: 99,
            frame_count: 12,
            ..BootAnimationConfig::default()
        };

        let classic = BootAnimator::with_style(
            cfg.clone(),
            BootStyleProfile::from_preset(BootStylePreset::Classic),
        )
        .generate_timeline(0);

        let storm = BootAnimator::with_style(
            cfg,
            BootStyleProfile::from_preset(BootStylePreset::NeonStorm),
        )
        .generate_timeline(0);

        assert_ne!(classic.frames[0].styled_hue, storm.frames[0].styled_hue);
        assert_ne!(classic.frames[0].styled_glow, storm.frames[0].styled_glow);
    }

    #[test]
    fn render_intents_are_derived_for_each_frame() {
        let timeline = BootAnimator::with_style_and_recipe(
            BootAnimationConfig {
                seed: 44,
                frame_count: 12,
                ..BootAnimationConfig::default()
            },
            BootStyleProfile::from_preset(BootStylePreset::NeonStorm),
            BootSequenceRecipe::GrandReveal,
        )
        .generate_timeline(0);

        let intents = timeline.derive_render_intents();
        assert_eq!(intents.len(), timeline.frames.len());
        assert!(intents.iter().all(|i| i.bloom_weight > 0.0));
    }

    #[test]
    fn render_intent_values_stay_in_reasonable_ranges() {
        let timeline = BootAnimator::with_style_and_recipe(
            BootAnimationConfig {
                seed: 77,
                frame_count: 16,
                ..BootAnimationConfig::default()
            },
            BootStyleProfile::from_preset(BootStylePreset::CrystalPulse),
            BootSequenceRecipe::QuickPulse,
        )
        .generate_timeline(0);

        let intents = timeline.derive_render_intents();
        for i in intents {
            assert!(i.bloom_weight > 0.0);
            assert!(i.fog_weight >= 0.0);
            assert!(i.distortion_weight >= 0.0);
            assert!(i.color_shift.is_finite());
        }
    }

    #[test]
    fn boot_screen_sequence_tracks_title_reveal() {
        let timeline = BootAnimator::with_style_and_recipe(
            BootAnimationConfig {
                seed: 1337,
                frame_count: 12,
                ..BootAnimationConfig::default()
            },
            BootStyleProfile::from_preset(BootStylePreset::NeonStorm),
            BootSequenceRecipe::GrandReveal,
        )
        .generate_timeline(1);

        let sequence = timeline.to_boot_screen_sequence("AUREX-X", "Booting runtime");
        assert_eq!(sequence.title_text, "AUREX-X");
        assert_eq!(sequence.frames.len(), timeline.frames.len());

        let first = sequence.frames.first().unwrap();
        let last = sequence.frames.last().unwrap();
        assert!(first.glyphs_lit >= 1);
        assert!(last.glyphs_lit >= first.glyphs_lit);
        assert!(last.subtitle_opacity >= first.subtitle_opacity);
    }

    #[test]
    fn postfx_snapshot_and_aggregate_are_consistent() {
        let timeline = BootAnimator::with_style_and_recipe(
            BootAnimationConfig {
                seed: 101,
                frame_count: 12,
                ..BootAnimationConfig::default()
            },
            BootStyleProfile::from_preset(BootStylePreset::NeonStorm),
            BootSequenceRecipe::GrandReveal,
        )
        .generate_timeline(0);

        let snapshots = timeline.to_postfx_snapshots();
        let agg = timeline.aggregate_postfx();

        assert_eq!(snapshots.len(), 12);
        assert!(agg.avg_bloom > 0.0);
        assert!(agg.peak_bloom >= agg.avg_bloom);
        assert!(agg.avg_fog >= 0.0);
    }

    #[test]
    fn postfx_track_supports_tick_lookup() {
        let timeline = BootAnimator::with_style_and_recipe(
            BootAnimationConfig {
                seed: 303,
                frame_count: 10,
                ..BootAnimationConfig::default()
            },
            BootStyleProfile::from_preset(BootStylePreset::Classic),
            BootSequenceRecipe::Standard,
        )
        .generate_timeline(25);

        let track = BootPostFxTrack::from_timeline(&timeline);
        assert_eq!(track.snapshots.len(), 10);
        assert!(track.snapshot_for_tick(25).is_some());
        assert!(track.snapshot_for_tick(999).is_none());
        assert_eq!(track.latest_snapshot().unwrap().tick, 34);
    }

    #[test]
    fn recipe_changes_phase_distribution() {
        let cfg = BootAnimationConfig {
            seed: 12,
            frame_count: 12,
            ..BootAnimationConfig::default()
        };

        let standard = BootAnimator::with_style_and_recipe(
            cfg.clone(),
            BootStyleProfile::from_preset(BootStylePreset::Classic),
            BootSequenceRecipe::Standard,
        )
        .generate_timeline(0);

        let quick = BootAnimator::with_style_and_recipe(
            cfg,
            BootStyleProfile::from_preset(BootStylePreset::Classic),
            BootSequenceRecipe::QuickPulse,
        )
        .generate_timeline(0);

        assert_ne!(standard.phase_counts(), quick.phase_counts());
    }

    #[test]
    fn rasterized_boot_frame_matches_requested_dimensions() {
        let frame = BootFrame {
            frame_index: 0,
            tick: 0,
            ring_radius: 0.95,
            glow: 0.8,
            hue_shift: 22.0,
            scanline_offset: 0.0,
        };

        let image = rasterize_boot_frame(&frame, 96, 54);
        assert_eq!(image.width, 96);
        assert_eq!(image.height, 54);
        assert_eq!(image.pixel_count(), 96 * 54);
        assert_eq!(image.rgba.len(), 96 * 54 * 4);
    }

    #[test]
    fn rasterized_boot_frame_checksum_is_deterministic() {
        let frame = BootFrame {
            frame_index: 7,
            tick: 7,
            ring_radius: 1.12,
            glow: 0.73,
            hue_shift: 214.0,
            scanline_offset: 0.0,
        };

        let a = rasterize_boot_frame(&frame, 80, 45);
        let b = rasterize_boot_frame(&frame, 80, 45);
        assert_eq!(a.checksum(), b.checksum());
    }

    #[test]
    fn rasterized_boot_frame_contains_visible_pixels() {
        let frame = BootFrame {
            frame_index: 3,
            tick: 3,
            ring_radius: 1.05,
            glow: 0.9,
            hue_shift: 128.0,
            scanline_offset: 0.0,
        };

        let image = rasterize_boot_frame(&frame, 128, 128);
        let lit = image.lit_pixel_count();

        assert!(lit > 0);
        assert!(lit < image.pixel_count());
    }
}
