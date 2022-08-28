use anyhow::{Context, Result};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window
};
use std::borrow::Cow;
use std::sync::mpsc::{Receiver, TryRecvError};
use crate::video_frame::VideoFrame;


pub async fn run(event_loop: EventLoop<()>, window: Window, video_receiver: Receiver<VideoFrame>) -> Result<()> {
    let size = window.inner_size();
    let instance = wgpu::Instance::new(wgpu::Backends::all());
    let display_surface = unsafe { instance.create_surface(&window) };
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            compatible_surface: Some(&display_surface),
        })
        .await
        .with_context(|| "Failed to find an appropriate gpu adapter.")?;
    
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
        .with_context(|| "Failed to create gpu device.")?;
    
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[],
        push_constant_ranges: &[],
    });

    let swapchain_format = display_surface.get_supported_formats(&adapter)[0];

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[],
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

    let mut config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: swapchain_format,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Fifo,
    };

    display_surface.configure(&device, &config);

    event_loop.run(move |event, _, control_flow| {
        // take ownership of resources to ensure they are properly cleaned up on exit
        let _ = (&instance, &adapter, &shader, &pipeline_layout, &video_receiver);

        let mut video_frame = None;

        match video_receiver.try_recv() {
            Ok(frame) => {
                println!("frame {}", frame.sequence);
                video_frame = Some(frame);
            },
            Err(TryRecvError::Disconnected) => {
                *control_flow = ControlFlow::Exit;
                return;
            },
            Err(TryRecvError::Empty) => {},
        }

        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                // Reconfigure the display_surface to the new size
                config.width = size.width;
                config.height = size.height;
                display_surface.configure(&device, &config);
                window.request_redraw();
            }
            
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,

            Event::RedrawRequested(_) => {
                let frame = display_surface
                    .get_current_texture()
                    .expect("Failed to acquire next swap chain texture");
                
                let view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                
                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

                {
                    let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                                store: true,
                            },
                        })],
                        depth_stencil_attachment: None,
                    });
                    rpass.set_pipeline(&render_pipeline);
                    rpass.draw(0..3, 0..1);
                }

                queue.submit(Some(encoder.finish()));
                frame.present();
            }

            _ => {}
        }
    })
}
