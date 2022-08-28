mod device_info;
mod main_loop;
mod video_capture;
mod video_frame;

use anyhow::{Context, Result};
use clap::Parser;
use device_info::show_device_info;
use std::sync::mpsc::sync_channel;
use v4l::{
    fraction::Fraction,
    parameters::Capabilities,
    video::capture::parameters::{Modes, Parameters},
    video::Capture,
    Device,
};
use video_capture::run_capture_thread;
use winit::event_loop::EventLoop;

/// Records video from a high speed monochrome camera, processing
/// frames through a virtual color wheel to allow through-the-glasses
/// recording of the TiltFive experience without flicker or banding.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Camera device to open (for example: /dev/video0)
    #[clap(value_parser)]
    device: String,
}

fn main() -> Result<()> {
    env_logger::init();

    let args = Args::parse();

    println!("Opening device {}...", args.device);

    let device = Device::with_path(&args.device)
        .with_context(|| format!("Failed to open device {}", &args.device))?;

    device
        .set_params(&Parameters {
            capabilities: Capabilities::TIME_PER_FRAME,
            modes: Modes::empty(),
            interval: Fraction::new(1, 120),
        })
        .with_context(|| "Failt to set parameters")?;

    let (width, height) = show_device_info(&device)?;

    let (video_sender, video_receiver) = sync_channel(1200);

    run_capture_thread(device, video_sender);

    let event_loop = EventLoop::new();
    let window =
        winit::window::Window::new(&event_loop).with_context(|| "Failed to create window.")?;

    pollster::block_on(main_loop::run(
        event_loop,
        window,
        video_receiver,
        (width, height),
    ))?;

    Ok(())
}
