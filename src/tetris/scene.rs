use std::borrow::Cow;

pub(crate) struct Scene {
    pub(crate) scene_pipeline: wgpu::RenderPipeline,
}

impl<'a> Scene {
    pub(crate) fn new(base: &'a super::Base) -> Self {
        Scene {
            scene_pipeline: Scene::build_scene_pipeline(&base),
        }
    }

    fn build_scene_pipeline(base: &'a super::Base) -> wgpu::RenderPipeline {
        let shader = base
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("game_area.wgsl"))),
            });

        let pipeline_layout = base
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let swapchain_format = base.surface.get_supported_formats(&base.adapter)[0];

        let vertex_size = std::mem::size_of::<super::Vertex>();
        let vertex_buffers_descriptor = [wgpu::VertexBufferLayout {
            array_stride: vertex_size as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 0,
                shader_location: 0,
            }],
        }];

        base.device
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
            })
    }
}
