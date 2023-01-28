mod game_state;
mod vertex;

use std::borrow::Cow;

use anyhow::Context;
use wgpu::util::DeviceExt;
use winit::{
    event::{Event, WindowEvent},
    event_loop::ControlFlow,
};

use game_state::GameState;
use vertex::Vertex;

#[allow(dead_code)]
struct Inner {
    instance: wgpu::Instance,
    surface: wgpu::Surface,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    window_size: winit::dpi::PhysicalSize<u32>,
    surface_config: wgpu::SurfaceConfiguration,
    window: winit::window::Window,
    event_loop: winit::event_loop::EventLoop<()>,
}

impl Inner {
    async fn new() -> anyhow::Result<Inner> {
        let event_loop = winit::event_loop::EventLoop::new();
        let window =
            winit::window::Window::new(&event_loop).context("Couldn't initialise the window")?;
        let window_size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(&window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .context("Couldn't obtain an adapter")?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::downlevel_webgl2_defaults()
                        .using_resolution(adapter.limits()),
                },
                None,
            )
            .await
            .context("Couldn't create logical device and job queue")?;

        let swapchain_format = surface.get_supported_formats(&adapter)[0];

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: swapchain_format,
            width: window_size.width,
            height: window_size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface.get_supported_alpha_modes(&adapter)[0],
        };

        surface.configure(&device, &config);

        Ok(Inner {
            event_loop,
            window,
            window_size,
            instance,
            surface,
            surface_config: config,
            adapter,
            device,
            queue,
        })
    }
}

pub struct Tetris {
    inner: Inner,
    render_pipeline: wgpu::RenderPipeline,
    game_state: GameState,
}

impl Tetris {
    pub async fn new() -> anyhow::Result<Tetris> {
        let inner = Inner::new().await.context("Couldn't initialize inner")?;

        let game_state = GameState::default();

        let shader = inner
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("game_area.wgsl"))),
            });

        let pipeline_layout =
            inner
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &[],
                    push_constant_ranges: &[],
                });

        let swapchain_format = inner.surface.get_supported_formats(&inner.adapter)[0];

        let vertex_size = std::mem::size_of::<Vertex>();
        let vertex_buffers_descriptor = [wgpu::VertexBufferLayout {
            array_stride: vertex_size as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 0,
                shader_location: 0,
            }],
        }];

        let render_pipeline =
            inner
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: None,
                    layout: Some(&pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: "vs_main",
                        buffers: &vertex_buffers_descriptor,
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: "fs_main",
                        targets: &[Some(swapchain_format.into())],
                    }),
                    primitive: wgpu::PrimitiveState::default(),
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState::default(),
                    multiview: None,
                });

        Ok(Tetris {
            inner,
            render_pipeline,
            game_state,
        })
    }

    pub async fn run(mut self) {
        self.inner.event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;

            match event {
                Event::WindowEvent {
                    event: WindowEvent::Resized(size),
                    ..
                } => {
                    self.inner.surface_config.width = size.width;
                    self.inner.surface_config.height = size.height;
                    self.inner
                        .surface
                        .configure(&self.inner.device, &self.inner.surface_config);
                    self.inner.window.request_redraw();
                }
                Event::RedrawRequested(_) => {
                    let frame = self
                        .inner
                        .surface
                        .get_current_texture()
                        .expect("Caouldn't get next swapchain texture");
                    let view = frame
                        .texture
                        .create_view(&wgpu::TextureViewDescriptor::default());

                    let game_area_vertex_buffer =
                        self.inner
                            .device
                            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                label: None,
                                contents: bytemuck::cast_slice(&self.game_state.game_area()),
                                usage: wgpu::BufferUsages::VERTEX,
                            });

                    let mut encoder = self
                        .inner
                        .device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

                    {
                        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: None,
                            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                view: &view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                                    store: true,
                                },
                            })],
                            depth_stencil_attachment: None,
                        });

                        rpass.set_pipeline(&self.render_pipeline);
                        rpass.set_vertex_buffer(0, game_area_vertex_buffer.slice(..));
                        rpass.draw(0..3, 0..1);
                    }
                    self.inner.queue.submit(Some(encoder.finish()));
                    frame.present();
                }
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => *control_flow = ControlFlow::Exit,
                _ => {}
            }
        });
    }
}
