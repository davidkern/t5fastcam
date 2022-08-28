use crate::video_frame::VideoFrame;
use std::sync::mpsc::{SyncSender, TrySendError};
use v4l::buffer::Type as BufferType;
use v4l::io::traits::CaptureStream;
use v4l::prelude::*;
use v4l::Device;

/// Starts a thread to capture frames from the camera and push them in a queue to be
/// pushed to the gpu by the main loop.
pub fn run_capture_thread(device: Device, video_sender: SyncSender<VideoFrame>) {
    std::thread::spawn(move || {
        let mut stream = MmapStream::with_buffers(&device, BufferType::VideoCapture, 1)
            .expect("Failed to construct mmap stream for video input.");

        loop {
            let (data, metadata) = stream.next().expect("Failed to get next frame.");
            let frame = VideoFrame {
                sequence: metadata.sequence,
                data: data.to_vec(),
            };
            if let Err(TrySendError::Disconnected(_)) = video_sender.try_send(frame) {
                // receiver hung up => exit the thread
                break;
            }
        }
    });
}
