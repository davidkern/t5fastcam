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

Transcode 120fps mjpeg stream to YUV 4:2:2
`ffmpeg -re -input_format mjpeg -framerate 120 -i /dev/video0 -pix_fmt yuv420p -f v4l2 /dev/video2`

Run against decoded stream
`cargo run -- /dev/video2`

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
