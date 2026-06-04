use image::{codecs::jpeg::JpegEncoder, DynamicImage, ImageFormat, ImageReader, Limits};
use ravif::{Encoder as AvifEncoder, Img};
use std::io::Cursor;
use webp::Encoder;

/// Decodes an image, optionally rejecting ones whose dimensions exceed the
/// configured limits. This guards against decompression bombs (a small encoded
/// file that expands to an enormous bitmap). With both limits `None` the
/// behavior matches `image::load_from_memory` (the default 512 MiB allocation
/// cap still applies); a configured width/height adds a hard dimension cap.
pub fn decode_image_with_limits(
    bytes: &[u8],
    max_width: Option<u32>,
    max_height: Option<u32>,
) -> Result<DynamicImage, String> {
    let mut reader = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|err| err.to_string())?;

    let mut limits = Limits::default();
    limits.max_image_width = max_width;
    limits.max_image_height = max_height;
    reader.limits(limits);

    reader.decode().map_err(|err| err.to_string())
}

pub fn encode_image_to_webp(image: &DynamicImage) -> Result<Vec<u8>, String> {
    let encoder = Encoder::from_image(image).map_err(|e| e.to_string())?;
    let webp_memory = encoder.encode(100f32);
    Ok(webp_memory.to_vec())
}

pub fn encode_image_to_avif(image: &DynamicImage, quality: Option<f32>) -> Result<Vec<u8>, String> {
    use rgb::FromSlice;

    let rgba_image = image.to_rgba8();
    let width = rgba_image.width() as usize;
    let height = rgba_image.height() as usize;

    let pixels = rgba_image.as_raw().as_rgba();
    let img = Img::new(pixels, width, height);

    let encoder = AvifEncoder::new()
        .with_quality(quality.unwrap_or(80.0))
        .with_speed(6);

    encoder
        .encode_rgba(img)
        .map(|data| data.avif_file)
        .map_err(|e| e.to_string())
}

pub fn resize_image(image: DynamicImage, width: u32) -> DynamicImage {
    let resize_height = width * image.height() / image.width();
    let mut resized = image.thumbnail(width, resize_height);

    let blur_threshold = std::env::var("RESIZED_IMAGE_BLUR_THRESHOLD")
        .ok()
        .and_then(|value| value.parse::<u32>().ok())
        .unwrap_or(10);
    let blur_sigma = std::env::var("RESIZED_IMAGE_BLUR_SIGMA")
        .ok()
        .and_then(|value| value.parse::<f32>().ok())
        .unwrap_or(1.5);

    if width < blur_threshold {
        resized = resized.fast_blur(blur_sigma);
    }

    resized
}

pub fn encode_image(image: &DynamicImage, format: ImageFormat) -> Result<Vec<u8>, String> {
    let mut buffer = Cursor::new(Vec::new());

    match format {
        ImageFormat::Jpeg => {
            let encoder = JpegEncoder::new(&mut buffer);
            image
                .write_with_encoder(encoder)
                .map_err(|e| e.to_string())?;
        }
        _ => {
            image
                .write_to(&mut buffer, format)
                .map_err(|e| e.to_string())?;
        }
    }

    Ok(buffer.into_inner())
}

pub fn guess_image_format(path: &str) -> ImageFormat {
    ImageFormat::from_path(path).unwrap_or(ImageFormat::Jpeg)
}
