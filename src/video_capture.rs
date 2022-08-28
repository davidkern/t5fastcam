use crate::video_frame::VideoFrame;
use std::sync::mpsc::{SyncSender, TrySendError};
use v4l::buffer::Type as BufferType;
use v4l::io::traits::CaptureStream;
use v4l::prelude::*;
use v4l::Device;
use std::time::Instant;

/// Starts a thread to capture frames from the camera and push them in a queue to be
/// pushed to the gpu by the main loop.
pub fn run_capture_thread(device: Device, video_sender: SyncSender<VideoFrame>) {
    std::thread::spawn(move || {
        //let t0 = Instant::now();
        //let mut fake_timestamp: f32 = 0.0;
        let mut stream = MmapStream::with_buffers(&device, BufferType::VideoCapture, 1)
            .expect("Failed to construct mmap stream for video input.");

        loop {
            let (data, metadata) = stream.next().expect("Failed to get next frame.");
            //fake_timestamp += 1.0 / 120.0;
            let frame = VideoFrame {
                //timestamp: Instant::now().duration_since(t0).as_nanos() as f32 / 1_000_000_000.0,
                timestamp: metadata.timestamp.sec as f32 + metadata.timestamp.usec as f32 / 1_000_000.0,
                //timestamp: fake_timestamp,
                data: data.to_vec(),
            };
            if let Err(e) = video_sender.try_send(frame) {
                if let TrySendError::Disconnected(_) = e {
                    // receiver hung up => exit the thread
                    break;
                } else {
                    println!("queue full, dropped frame.")
                }
            }
        }
    });
}
