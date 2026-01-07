use std::sync::Arc;
use bytemuck::NoUninit;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

use anyhow::Result;

#[derive(Default)]
struct App {
    gpu: Option<Gpu>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let size = PhysicalSize::new(640, 480);

        let attrs = Window::default_attributes()
            .with_inner_size(size.clone())
            .with_resizable(false);

        let window = event_loop.create_window(attrs).unwrap();
        self.gpu = Some(pollster::block_on(Gpu::new(window, size)).unwrap());
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        if let Some(gpu) = &mut self.gpu {
            match event {
                WindowEvent::CloseRequested => {
                    println!("Closing window.");
                    event_loop.exit();
                }
                WindowEvent::Resized(size) => {
                    gpu.resize(size);
                }
                WindowEvent::RedrawRequested => {
                    gpu.render().unwrap()
                }
                _ => (),
            }
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, NoUninit)]
pub struct Vertex {
    pub pos: [f32; 3],
    pub color: [f32; 3]
}

struct Gpu {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer
}

impl Gpu {
    const VERTICES: &[Vertex] = &[
        Vertex { pos: [-0.5, -0.5, 0.0], color: [1.0, 0.0, 0.0] },
        Vertex { pos: [-0.5,  0.5, 0.0], color: [0.0, 1.0, 0.0] },
        Vertex { pos: [ 0.5, -0.5, 0.0], color: [0.0, 0.0, 1.0] },
        Vertex { pos: [ 0.5,  0.5, 0.0], color: [1.0, 1.0, 1.0] },
    ];

    const INDICES: &[u16] = &[
        0, 1, 2,
        1, 2, 3
    ]; 

    fn make_vertex_buffer(device: &wgpu::Device, vtx: &[Vertex]) -> wgpu::Buffer {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: "Vertex buffer".into(),
            size: (vtx.len() * size_of::<Vertex>()) as u64,
            mapped_at_creation: false,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX
        })
    }

    fn make_index_buffer(device: &wgpu::Device, idx: &[u16]) -> wgpu::Buffer {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: "Index buffer".into(),
            size: (idx.len() * size_of::<u16>()) as u64,
            mapped_at_creation: false,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::INDEX
        })
    }

    pub async fn new(window: Window, size: PhysicalSize<u32>) -> Result<Self> {
        let window = Arc::new(window);
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let surface = instance.create_surface(window.clone())?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await?;
        
        let (device, queue) = Self::get_device(&adapter).await?;
 
        let vertex_buffer = Self::make_vertex_buffer(&device, Gpu::VERTICES);
        queue.write_buffer(&vertex_buffer, 0, &bytemuck::cast_slice(Gpu::VERTICES));

        let index_buffer = Self::make_index_buffer(&device, Gpu::INDICES);
        queue.write_buffer(&index_buffer, 0, &bytemuck::cast_slice(Gpu::INDICES));
        
        let config = Self::get_config(&adapter, &surface, size);
        surface.configure(&device, &config);

        let bind_group_layout = Self::get_bind_group_layout(&device);
        let pipeline_layout = Self::get_pipeline_layout(&device, &bind_group_layout);
        let pipeline = Self::get_pipeline(&device, &config, &pipeline_layout);

        Ok(Self {
            window,
            surface,
            device,
            queue,
            config,
            pipeline,
            vertex_buffer,
            index_buffer
        })
    }

    async fn get_device(adapter: &wgpu::Adapter) -> Result<(wgpu::Device, wgpu::Queue)> {
        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor::default()).await?;

        device.set_device_lost_callback(|reason, message| {
            eprintln!("{:?}", reason);
            eprintln!("{message}");
        });

        queue.on_submitted_work_done(|| println!("Finished!"));

        Ok((device, queue))
    }

    fn get_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        return device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: "Bind group layout".into(),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None
                },
                count: None
            }]
        })
    }

    fn get_pipeline_layout(device: &wgpu::Device, layout: &wgpu::BindGroupLayout) -> wgpu::PipelineLayout {
        return device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: "Uniform buffer layout".into(),
            bind_group_layouts: &[layout],
            push_constant_ranges: &[]
        })
    }

    fn get_config(
        adapter: &wgpu::Adapter,
        surface: &wgpu::Surface<'static>,
        size: PhysicalSize<u32>,
    ) -> wgpu::SurfaceConfiguration {
        let capabilities = surface.get_capabilities(&adapter);

        let surface_format = capabilities
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(capabilities.formats[0]);

        wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: capabilities.present_modes[0],
            alpha_mode: capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        }
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.config.width = size.width;
        self.config.height = size.height;
    }

    fn get_pipeline(device: &wgpu::Device,
                    config: &wgpu::SurfaceConfiguration,
                    pipeline_layout: &wgpu::PipelineLayout)
                    -> wgpu::RenderPipeline {
        let shader_module = device.create_shader_module(
            wgpu::include_wgsl!("shader.wgsl")
        );

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Triangle render"),
            layout: Some(pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[
                    wgpu::VertexBufferLayout {
                        array_stride: size_of::<Vertex>() as u64,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &wgpu::vertex_attr_array![
                            0 => Float32x3,
                            1 => Float32x3
                        ]
                    }
                ]
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &vec![Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL
                })],
            }),
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0u64,
                alpha_to_coverage_enabled: false
            },
            multiview: None,
            cache: None
        })
    }

    pub fn render(&self) -> Result<()> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 0.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..Gpu::INDICES.len() as u32, 0, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        self.window.request_redraw();
        Ok(())
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop.run_app(&mut App::default()).unwrap();
}
