# t5fastcam

Records frames through the Tilt Five glasses with a 120fps monochrome camera and filters
the frames through a virtual color wheel to produce a color video stream without flicker
or color banding.

## Building

This is built with [Rust](https://www.rust-lang.org/) and tested on Ubuntu 22.04.1 LTS but should work on other Linux distros which have Video For Linux v2 (v4l2).

Ensure dependencies are available with:
`sudo apt install cmake libclang-dev libfontconfig-dev`

And then `cargo build`.

## Running

The camera I'm using is: "Arducam 120fps Global Shutter USB Camera Board, 1MP 720P OV9281 UVC Webcam Module with Low Distortion M12 Lens". Other monochrome high-speed global shutter cameras should also work.

Create a loopback video device
`sudo modprobe v4l2loopback devices=1 video_nr=2 card_label="Fake" exclusive_caps=1`

Extract luminance channel from 120fps mjpeg stream and provide raw stream on fake device
`ffmpeg -re -input_format mjpeg -framerate 120 -i /dev/video0 -vcodec rawvideo -pix_fmt gray -f v4l2 /dev/video2`

Run against raw stream
`cargo run -- /dev/video2`

If something goes wrong and ffmpeg gives the error "Could not write header for output file", then teardown the loopback with `sudo rmmod v4l2loopback` and recreate it.

## License

Copyright 2022 The t5fastcam Authors.

This project is licensed under either of

Apache License, Version 2.0, (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)
MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT) at your option.

"Tilt Five" and "T5" are trademarks owned by [Tilt Five, Inc](https://www.tiltfive.com/).

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
`t5fastcam` by you, as defined in the Apache-2.0 license, shall be dual licensed as above,
without any additional terms or conditions.
