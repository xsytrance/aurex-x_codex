pub mod typography;
use typography::{
    LyricRenderEvent, TimedLyricRenderEvent, TypographyReactiveState, TypographyStyle,
    choose_typography_style,
};

#[cfg(feature = "real_graphics")]
use winit::{dpi::PhysicalSize, event_loop::EventLoop, window::Window};

pub type Vec3 = [f32; 3];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CameraRig {
    Orbit,
    PulseOrbit,
    Flyby,
    ReactorDive,
}

#[derive(Debug, Clone)]
pub struct CameraState {
    pub position: Vec3,
    pub target: Vec3,
    pub rig: CameraRig,
    pub fov_degrees: f32,
}

impl Default for CameraState {
    fn default() -> Self {
        Self {
            position: [0.0, 1.5, 6.0],
            target: [0.0, 0.0, 0.0],
            rig: CameraRig::Orbit,
            fov_degrees: 60.0,
        }
    }
}

impl CameraState {
    pub fn update_for_frame(&mut self, time: f32, pulse: f32) {
        let angle = time * 0.25;
        let base_radius = 4.0;

        match self.rig {
            CameraRig::Orbit => {
                self.position[0] = angle.cos() * base_radius;
                self.position[1] = 1.5;
                self.position[2] = angle.sin() * base_radius;
            }
            CameraRig::PulseOrbit => {
                let radius = base_radius + pulse * 0.5;
                self.position[0] = angle.cos() * radius;
                self.position[1] = 1.5;
                self.position[2] = angle.sin() * radius;
            }
            CameraRig::Flyby => {
                self.position[0] = (time * 0.5).sin() * 4.0;
                self.position[1] = 1.5;
                self.position[2] = -3.0 + time * 0.2;
            }
            CameraRig::ReactorDive => {
                self.position[0] = 0.0;
                self.position[1] = 1.1;
                self.position[2] = 6.0 - time * 0.1;
            }
        }

        self.target = [0.0, 0.0, 0.0];
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
    let mut frame_idx = 0usize;

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

                        let boot = &timeline_frames[frame_idx % timeline_frames.len()];
                        let screen = &boot_screen.frames[frame_idx % boot_screen.frames.len()];
                        frame_idx = frame_idx.wrapping_add(1);

                        let mut cpu_frame = rasterize_boot_frame(boot, config.width, config.height);
                        overlay_boot_caption(
                            &mut cpu_frame.rgba,
                            config.width,
                            config.height,
                            screen,
                        );

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

#[cfg(not(feature = "real_graphics"))]
pub fn run_real_renderer_event_loop() -> Result<(), String> {
    Err("real_graphics feature is disabled".to_string())
}

#[cfg(feature = "real_graphics")]
fn overlay_boot_caption(rgba: &mut [u8], width: u32, height: u32, frame: &BootScreenFrame) {
    if width < 32 || height < 16 {
        return;
    }

    let glyphs = frame.glyphs_lit.max(1) as usize;
    let total = 8usize;
    let lit_ratio =
        (frame.title_progress.clamp(0.0, 1.0) * glyphs as f32 / total as f32).clamp(0.0, 1.0);

    let y0 = (height as f32 * 0.82) as u32;
    let y1 = (height as f32 * 0.92) as u32;
    let x0 = (width as f32 * 0.16) as u32;
    let x1 = (width as f32 * 0.84) as u32;
    let span = (x1.saturating_sub(x0)).max(1);
    let lit_w = (span as f32 * lit_ratio) as u32;

    for y in y0..y1.min(height) {
        for x in x0..x1.min(width) {
            let i = ((y * width + x) * 4) as usize;
            let in_lit = x <= x0.saturating_add(lit_w);
            let glow = frame.title_glow.clamp(0.0, 2.0);
            let (r, g, b) = if in_lit {
                (
                    (120.0 + 100.0 * glow).clamp(0.0, 255.0) as u8,
                    (180.0 + 60.0 * glow).clamp(0.0, 255.0) as u8,
                    255u8,
                )
            } else {
                (40, 55, 75)
            };
            rgba[i] = rgba[i].saturating_add(r / 2);
            rgba[i + 1] = rgba[i + 1].saturating_add(g / 2);
            rgba[i + 2] = rgba[i + 2].saturating_add(b / 2);
            rgba[i + 3] = 255;
        }
    }
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
    let cy = height as f32 * 0.5;
    let min_dim = width.min(height) as f32;
    let ring_radius = (frame.ring_radius * min_dim * 0.24).max(1.0);
    let ring_thickness = (2.0 + frame.glow * 3.5).clamp(1.0, 9.0);

    let hue = ((frame.hue_shift + 360.0) % 360.0) / 360.0;
    let base_r = (0.35 + hue).fract();
    let base_g = (0.65 + hue * 0.75).fract();
    let base_b = (0.95 + hue * 0.5).fract();

    for y in 0..height {
        for x in 0..width {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            let dist = (dx * dx + dy * dy).sqrt();
            let ring_delta = (dist - ring_radius).abs();
            let ring = (1.0 - (ring_delta / ring_thickness)).clamp(0.0, 1.0);
            let center_glow = (1.0 - dist / (ring_radius * 1.4)).clamp(0.0, 1.0) * frame.glow;
            let intensity = (ring * 0.9 + center_glow * 0.45).clamp(0.0, 1.0);

            let idx = ((y * width + x) * 4) as usize;
            rgba[idx] = (base_r * intensity * 255.0) as u8;
            rgba[idx + 1] = (base_g * intensity * 255.0) as u8;
            rgba[idx + 2] = (base_b * intensity * 255.0) as u8;
            rgba[idx + 3] = (intensity * 255.0) as u8;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MusicReactiveEvent {
    Kick,
    Snare,
    Hat,
    BassNote(u8),
    PadNote(u8),
    LeadNote(u8),
}

#[derive(Debug, Clone, PartialEq)]
pub struct BeatEnergy {
    pub kick_energy: f32,
    pub snare_energy: f32,
    pub bass_energy: f32,
    pub pad_energy: f32,
}

impl Default for BeatEnergy {
    fn default() -> Self {
        Self {
            kick_energy: 0.0,
            snare_energy: 0.0,
            bass_energy: 0.0,
            pad_energy: 0.0,
        }
    }
}

impl BeatEnergy {
    pub fn apply_event(&mut self, event: MusicReactiveEvent) {
        match event {
            MusicReactiveEvent::Kick => self.kick_energy += 1.0,
            MusicReactiveEvent::Snare => self.snare_energy += 0.8,
            MusicReactiveEvent::BassNote(_) => self.bass_energy += 0.5,
            MusicReactiveEvent::PadNote(_) => self.pad_energy += 0.3,
            _ => {}
        }
    }

    pub fn decay_frame(&mut self) {
        self.kick_energy *= 0.92;
        self.snare_energy *= 0.92;
        self.bass_energy *= 0.94;
        self.pad_energy *= 0.96;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MusicReactiveVisualState {
    pub beat_energy: BeatEnergy,
    pub reactor_glow_boost: f32,
    pub particle_burst_energy: f32,
    pub spark_energy: f32,
    pub pulse_ring_scale: f32,
    pub ambient_glow_boost: f32,
    pub camera_wobble: f32,
}

impl Default for MusicReactiveVisualState {
    fn default() -> Self {
        Self {
            beat_energy: BeatEnergy::default(),
            reactor_glow_boost: 0.0,
            particle_burst_energy: 0.0,
            spark_energy: 0.0,
            pulse_ring_scale: 1.0,
            ambient_glow_boost: 0.0,
            camera_wobble: 0.0,
        }
    }
}

impl MusicReactiveVisualState {
    pub fn apply_event(&mut self, event: MusicReactiveEvent) {
        self.beat_energy.apply_event(event);

        match event {
            MusicReactiveEvent::Kick => {
                self.reactor_glow_boost =
                    (self.reactor_glow_boost + 0.22 + self.beat_energy.kick_energy * 0.08)
                        .clamp(0.0, 2.5);
                self.pulse_ring_scale =
                    (self.pulse_ring_scale + 0.03 + self.beat_energy.kick_energy * 0.012)
                        .clamp(1.0, 1.22);
            }
            MusicReactiveEvent::Snare => {
                self.particle_burst_energy =
                    (self.particle_burst_energy + 0.2 + self.beat_energy.snare_energy * 0.1)
                        .clamp(0.0, 2.5);
            }
            MusicReactiveEvent::Hat => {
                self.spark_energy = (self.spark_energy + 0.2).clamp(0.0, 1.0);
            }
            MusicReactiveEvent::BassNote(note) => {
                let note_bias = (note as f32 - 36.0).max(0.0) * 0.0008;
                self.pulse_ring_scale = (self.pulse_ring_scale
                    + 0.01
                    + self.beat_energy.bass_energy * 0.01
                    + note_bias)
                    .clamp(1.0, 1.22);
            }
            MusicReactiveEvent::PadNote(note) => {
                let note_bias = (note as f32 - 40.0).max(0.0) * 0.002;
                self.ambient_glow_boost = (self.ambient_glow_boost
                    + 0.07
                    + self.beat_energy.pad_energy * 0.06
                    + note_bias)
                    .clamp(0.0, 2.5);
            }
            MusicReactiveEvent::LeadNote(note) => {
                let note_bias = (note as f32 - 60.0).max(0.0) * 0.0015;
                self.camera_wobble = (self.camera_wobble + 0.06 + note_bias).clamp(0.0, 0.35);
            }
        }
    }

    pub fn apply_events(&mut self, events: &[MusicReactiveEvent]) {
        for event in events {
            self.apply_event(*event);
        }
    }

    pub fn advance_frame(&mut self) {
        self.beat_energy.decay_frame();

        self.pulse_ring_scale = (self.pulse_ring_scale
            + self.beat_energy.kick_energy * 0.004
            + self.beat_energy.bass_energy * 0.002)
            .clamp(1.0, 1.24);
        self.reactor_glow_boost =
            (self.reactor_glow_boost + self.beat_energy.kick_energy * 0.015).clamp(0.0, 2.5);
        self.particle_burst_energy =
            (self.particle_burst_energy + self.beat_energy.snare_energy * 0.02).clamp(0.0, 2.5);
        self.ambient_glow_boost =
            (self.ambient_glow_boost + self.beat_energy.pad_energy * 0.012).clamp(0.0, 2.5);

        self.reactor_glow_boost *= 0.86;
        self.particle_burst_energy *= 0.74;
        self.spark_energy *= 0.7;
        self.pulse_ring_scale = 1.0 + (self.pulse_ring_scale - 1.0) * 0.68;
        self.ambient_glow_boost *= 0.91;
        self.camera_wobble *= 0.82;
    }
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
    reactive_visuals: MusicReactiveVisualState,
    typography_style: TypographyStyle,
    typography_reactive: TypographyReactiveState,
    lyric_timeline: Vec<TimedLyricRenderEvent>,
    lyric_cursor: usize,
    active_lyric: Option<LyricRenderEvent>,
    lyric_bpm: u32,
    camera_state: CameraState,
    camera_time_seconds: f32,
}

impl MockRenderer {
    pub fn new(config: RenderBootstrapConfig) -> Self {
        let backend_ready = config.backend_mode == RenderBackendMode::Mock;
        Self {
            config,
            frames_rendered: 0,
            backend_ready,
            reactive_visuals: MusicReactiveVisualState::default(),
            typography_style: choose_typography_style(0),
            typography_reactive: TypographyReactiveState::default(),
            lyric_timeline: Vec::new(),
            lyric_cursor: 0,
            active_lyric: None,
            lyric_bpm: 120,
            camera_state: CameraState::default(),
            camera_time_seconds: 0.0,
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
        self.camera_time_seconds += 1.0 / 60.0;
        self.typography_reactive.advance_frame();
        let pulse = (self.reactive_visuals.beat_energy.kick_energy * 0.35
            + self.reactive_visuals.beat_energy.bass_energy * 0.2)
            .clamp(0.0, 1.0);
        self.camera_state
            .update_for_frame(self.camera_time_seconds, pulse);

        let beat_time = self.camera_time_seconds * self.lyric_bpm as f32 / 60.0;
        while self.lyric_cursor < self.lyric_timeline.len()
            && self.lyric_timeline[self.lyric_cursor].beat_time <= beat_time
        {
            let mut event = self.lyric_timeline[self.lyric_cursor].event.clone();
            event.scale = (event.scale + self.typography_reactive.scale_boost).clamp(0.4, 2.5);
            self.active_lyric = Some(event);
            self.lyric_cursor += 1;
        }

        self.reactive_visuals.advance_frame();
        RenderFrameStats {
            frame_id: self.frames_rendered,
            stages_executed: stages.len(),
        }
    }

    pub fn apply_music_events(&mut self, events: &[MusicReactiveEvent]) {
        for event in events {
            match event {
                MusicReactiveEvent::Kick => self.typography_reactive.scale_boost += 0.18,
                MusicReactiveEvent::Snare => self.typography_reactive.spark_intensity += 0.3,
                MusicReactiveEvent::BassNote(_) => self.typography_reactive.glow_boost += 0.22,
                MusicReactiveEvent::PadNote(_) => self.typography_reactive.ambient_boost += 0.16,
                MusicReactiveEvent::LeadNote(_) => self.typography_reactive.letter_motion += 0.08,
                MusicReactiveEvent::Hat => {}
            }
        }
        self.reactive_visuals.apply_events(events);
    }

    pub fn reactive_visuals(&self) -> &MusicReactiveVisualState {
        &self.reactive_visuals
    }

    pub fn set_camera_rig(&mut self, rig: CameraRig) {
        self.camera_state.rig = rig;
    }

    pub fn camera_state(&self) -> &CameraState {
        &self.camera_state
    }

    pub fn set_typography_seed(&mut self, seed: u64) {
        self.typography_style = choose_typography_style(seed);
    }

    pub fn typography_style(&self) -> TypographyStyle {
        self.typography_style
    }

    pub fn typography_reactive_state(&self) -> TypographyReactiveState {
        self.typography_reactive
    }

    pub fn set_lyric_timeline(&mut self, timeline: Vec<TimedLyricRenderEvent>, bpm: u32) {
        self.lyric_timeline = timeline;
        self.lyric_cursor = 0;
        self.active_lyric = None;
        self.lyric_bpm = bpm.max(1);
    }

    pub fn active_lyric(&self) -> Option<&LyricRenderEvent> {
        self.active_lyric.as_ref()
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

    #[test]
    fn music_reactive_visuals_respond_to_events() {
        let mut state = MusicReactiveVisualState::default();
        state.apply_events(&[
            MusicReactiveEvent::Kick,
            MusicReactiveEvent::Snare,
            MusicReactiveEvent::Hat,
            MusicReactiveEvent::BassNote(42),
            MusicReactiveEvent::PadNote(55),
            MusicReactiveEvent::LeadNote(79),
        ]);

        assert!(state.reactor_glow_boost > 0.0);
        assert!(state.particle_burst_energy > 0.0);
        assert!(state.spark_energy > 0.0);
        assert!(state.pulse_ring_scale > 1.0);
        assert!(state.ambient_glow_boost > 0.0);
        assert!(state.camera_wobble > 0.0);
    }

    #[test]
    fn music_reactive_visuals_decay_deterministically() {
        let mut state = MusicReactiveVisualState::default();
        state.apply_event(MusicReactiveEvent::Kick);
        state.apply_event(MusicReactiveEvent::PadNote(50));

        let first = state.clone();
        state.advance_frame();
        let second = state.clone();

        assert!(second.reactor_glow_boost < first.reactor_glow_boost);
        assert!(second.ambient_glow_boost < first.ambient_glow_boost);
        assert!(second.pulse_ring_scale <= first.pulse_ring_scale.max(1.0));
    }

    #[test]
    fn beat_energy_accumulates_with_expected_weights() {
        let mut energy = BeatEnergy::default();
        energy.apply_event(MusicReactiveEvent::Kick);
        energy.apply_event(MusicReactiveEvent::Snare);
        energy.apply_event(MusicReactiveEvent::BassNote(42));
        energy.apply_event(MusicReactiveEvent::PadNote(55));
        energy.apply_event(MusicReactiveEvent::Hat);

        assert_eq!(energy.kick_energy, 1.0);
        assert_eq!(energy.snare_energy, 0.8);
        assert_eq!(energy.bass_energy, 0.5);
        assert_eq!(energy.pad_energy, 0.3);
    }

    #[test]
    fn beat_energy_decays_with_expected_factors() {
        let mut energy = BeatEnergy {
            kick_energy: 1.0,
            snare_energy: 0.8,
            bass_energy: 0.5,
            pad_energy: 0.3,
        };

        energy.decay_frame();

        assert!((energy.kick_energy - 0.92).abs() < 1e-6);
        assert!((energy.snare_energy - 0.736).abs() < 1e-6);
        assert!((energy.bass_energy - 0.47).abs() < 1e-6);
        assert!((energy.pad_energy - 0.288).abs() < 1e-6);
    }

    #[test]
    fn camera_orbit_and_pulse_orbit_are_deterministic() {
        let mut orbit = CameraState {
            rig: CameraRig::Orbit,
            ..CameraState::default()
        };
        orbit.update_for_frame(2.0, 0.8);

        let mut pulse_orbit = CameraState {
            rig: CameraRig::PulseOrbit,
            ..CameraState::default()
        };
        pulse_orbit.update_for_frame(2.0, 0.8);

        let orbit_radius =
            (orbit.position[0] * orbit.position[0] + orbit.position[2] * orbit.position[2]).sqrt();
        let pulse_radius = (pulse_orbit.position[0] * pulse_orbit.position[0]
            + pulse_orbit.position[2] * pulse_orbit.position[2])
            .sqrt();

        assert!((orbit_radius - 4.0).abs() < 1e-5);
        assert!(pulse_radius > orbit_radius);
        assert!((pulse_orbit.position[1] - 1.5).abs() < 1e-6);
    }

    #[test]
    fn camera_flyby_and_reactor_dive_follow_expected_axes() {
        let mut flyby = CameraState {
            rig: CameraRig::Flyby,
            ..CameraState::default()
        };
        flyby.update_for_frame(3.0, 0.0);

        let mut dive = CameraState {
            rig: CameraRig::ReactorDive,
            ..CameraState::default()
        };
        dive.update_for_frame(3.0, 0.0);

        assert!((flyby.position[1] - 1.5).abs() < 1e-6);
        assert!((flyby.position[2] - (-2.4)).abs() < 1e-5);
        assert!((dive.position[0]).abs() < 1e-6);
        assert!((dive.position[2] - 5.7).abs() < 1e-5);
    }

    #[test]
    fn mock_renderer_updates_camera_each_frame() {
        let mut renderer = MockRenderer::new(RenderBootstrapConfig::default());
        renderer.set_camera_rig(CameraRig::Flyby);
        let before = renderer.camera_state().position;
        renderer.run_frame(&RENDER_MAIN_STAGES);
        let after = renderer.camera_state().position;
        assert_ne!(before, after);
    }

    #[test]
    fn typography_seed_choice_is_deterministic() {
        let mut renderer = MockRenderer::new(RenderBootstrapConfig::default());
        renderer.set_typography_seed(99);
        let a = renderer.typography_style();
        renderer.set_typography_seed(99);
        let b = renderer.typography_style();
        assert_eq!(a, b);
    }

    #[test]
    fn lyric_events_activate_on_timeline() {
        let mut renderer = MockRenderer::new(RenderBootstrapConfig::default());
        renderer.set_lyric_timeline(
            vec![TimedLyricRenderEvent {
                beat_time: 0.0,
                event: LyricRenderEvent {
                    text: "Pulse".to_string(),
                    position: [0.1, 0.8],
                    scale: 1.0,
                },
            }],
            120,
        );

        renderer.run_frame(&RENDER_MAIN_STAGES);
        let text = renderer
            .active_lyric()
            .map(|e| e.text.as_str())
            .unwrap_or("none");
        assert_eq!(text, "Pulse");
    }

    #[test]
    fn typography_reacts_to_music_events() {
        let mut renderer = MockRenderer::new(RenderBootstrapConfig::default());
        renderer.apply_music_events(&[
            MusicReactiveEvent::Kick,
            MusicReactiveEvent::Snare,
            MusicReactiveEvent::BassNote(40),
            MusicReactiveEvent::PadNote(55),
            MusicReactiveEvent::LeadNote(70),
        ]);
        let state = renderer.typography_reactive_state();
        assert!(state.scale_boost > 0.0);
        assert!(state.spark_intensity > 0.0);
        assert!(state.glow_boost > 0.0);
        assert!(state.ambient_boost > 0.0);
        assert!(state.letter_motion > 0.0);
    }
}
