use image::{codecs::jpeg::JpegEncoder, DynamicImage, ImageFormat};
use ravif::{Encoder as AvifEncoder, Img};
use std::io::Cursor;
use webp::Encoder;

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
        .unwrap_or("10".to_string())
        .parse::<u32>()
        .unwrap();
    let blur_sigma = std::env::var("RESIZED_IMAGE_BLUR_SIGMA")
        .unwrap_or("1.5".to_string())
        .parse::<f32>()
        .unwrap();

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
