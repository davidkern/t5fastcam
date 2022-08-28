use anyhow::{Context, Result};
use v4l::video::Capture;
use v4l::Device;

/// Show settings and capabilities of the video device
pub fn show_device_info(device: &Device) -> Result<()> {
    let device_format = device
        .format()
        .with_context(|| "Failed to get device format.")?;

    println!("Active format:\n{}", device_format);

    let device_params = device
        .params()
        .with_context(|| "Failed to get device params.")?;

    println!("Active parameters:\n{}", device_params);

    let device_formats = device
        .enum_formats()
        .with_context(|| "Failed to enumerate available device formats.")?;

    println!("Available formats:");
    for device_format in device_formats {
        println!("  {} ({})", device_format.fourcc, device_format.description);

        let framesizes = device
            .enum_framesizes(device_format.fourcc)
            .with_context(|| {
                format!(
                    "Failed to enumerate frame sizes for {}",
                    device_format.fourcc
                )
            })?;
        for framesize in framesizes {
            for discrete in framesize.size.to_discrete() {
                println!("   Size: {}", discrete);

                let frameintervals = device
                    .enum_frameintervals(framesize.fourcc, discrete.width, discrete.height)
                    .with_context(|| {
                        format!(
                            "Failed to enumerate frame interval for {}@{}",
                            framesize.fourcc, discrete
                        )
                    })?;
                for frameinterval in frameintervals {
                    println!("    Interval: {}", frameinterval);
                }
            }
        }

        println!();
    }

    Ok(())
}
