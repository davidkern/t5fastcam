use crate::video_frame::VideoFrame;
use anyhow::{Context, Result};
use std::borrow::Cow;
use std::sync::mpsc::{Receiver, TryRecvError};
use winit::{
    event::{Event, StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

pub async fn run(
    event_loop: EventLoop<()>,
    window: Window,
    video_receiver: Receiver<VideoFrame>,
    video_size: (u32, u32),
) -> Result<()> {
    let mut render_enabled = true;

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

    let swapchain_format = display_surface.get_supported_formats(&adapter)[0];

    let mut display_config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: swapchain_format,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Fifo,
    };

    display_surface.configure(&device, &display_config);

    let video_texture_size = wgpu::Extent3d {
        width: video_size.0,
        height: video_size.1,
        depth_or_array_layers: 1,
    };

    let video_texture = device.create_texture(&wgpu::TextureDescriptor {
        size: video_texture_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::R8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        label: Some("video_texture"),
    });

    let video_texture_view = video_texture.create_view(&wgpu::TextureViewDescriptor::default());
    let video_texture_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    });

    let video_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
            label: Some("video_bind_group_layout"),
        });

    let video_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &video_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&video_texture_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&video_texture_sampler),
            },
        ],
        label: Some("video_bind_group"),
    });

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&video_bind_group_layout],
        push_constant_ranges: &[],
    });

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

    event_loop.run(move |event, _, control_flow| {
        // take ownership of resources to ensure they are properly cleaned up on exit
        let _ = (
            &instance,
            &adapter,
            &shader,
            &pipeline_layout,
            &video_receiver,
        );

        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                // Reconfigure the display_surface to the new size
                render_enabled = false;
                display_config.width = size.width;
                display_config.height = size.height;
                display_surface.configure(&device, &display_config);
                window.request_redraw();
            }

            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                println!("close requested");
                render_enabled = false;
                *control_flow = ControlFlow::Exit;
                return;
            }

            Event::RedrawRequested(_) => {
                render_enabled = true;
            }

            Event::NewEvents(StartCause::Poll) => {
                match video_receiver.try_recv() {
                    Ok(frame) => {
                        if render_enabled {
                            queue.write_texture(
                                wgpu::ImageCopyTexture {
                                    texture: &video_texture,
                                    mip_level: 0,
                                    origin: wgpu::Origin3d::ZERO,
                                    aspect: wgpu::TextureAspect::All,
                                },
                                &frame.data,
                                wgpu::ImageDataLayout {
                                    offset: 0,
                                    bytes_per_row: std::num::NonZeroU32::new(video_size.0),
                                    rows_per_image: std::num::NonZeroU32::new(video_size.1),
                                },
                                video_texture_size,
                            );
                        }
                    }
                    Err(TryRecvError::Disconnected) => {
                        *control_flow = ControlFlow::Exit;
                        return;
                    }
                    Err(TryRecvError::Empty) => {}
                }

                if render_enabled {
                    if let Ok(frame) = display_surface.get_current_texture() {
                        let view = frame
                            .texture
                            .create_view(&wgpu::TextureViewDescriptor::default());

                        let mut encoder =
                            device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                                label: None,
                            });

                        {
                            let mut rpass =
                                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
                            rpass.set_bind_group(0, &video_bind_group, &[]);
                            rpass.draw(0..3, 0..1);
                        }

                        queue.submit(Some(encoder.finish()));
                        frame.present();
                    } else {
                        // lost the surface, stop rendering until it has been recreated
                        render_enabled = false;
                    }
                }
            }

            _ => {}
        }

        // if event != Event::MainEventsCleared && event != Event::RedrawEventsCleared && event != Event::NewEvents(StartCause::Poll) {
        //     println!("{:?},", event);
        // }
    })
}
