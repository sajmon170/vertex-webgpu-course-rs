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

struct Gpu {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub pipeline: wgpu::RenderPipeline
}

impl Gpu {
    pub async fn new(window: Window, size: PhysicalSize<u32>) -> Result<Self> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let surface = instance.create_surface(window)?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await?;
        
        let (device, queue) = Self::get_device(&adapter).await?;
        let config = Self::get_config(&adapter, &surface, size);
        surface.configure(&device, &config);

        let pipeline = Self::get_pipeline(&device, &config);

        Ok(Self {
            surface,
            device,
            queue,
            config,
            pipeline
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

    fn get_pipeline(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration)
                    -> wgpu::RenderPipeline {
        let shader_module = device.create_shader_module(
            wgpu::include_wgsl!("shader.wgsl")
        );

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Triangle render"),
            layout: None,
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[]
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
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop.run_app(&mut App::default()).unwrap();
}
