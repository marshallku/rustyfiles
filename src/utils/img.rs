use axum::{http::StatusCode, response::Response};
use ravif::{Encoder as AvifEncoder, Img};
use std::{
    fs::{copy, write},
    path::PathBuf,
};
use webp::Encoder;

use super::http::{response_error, response_file};

pub fn save_image_to_webp(image: &image::DynamicImage, path: &PathBuf) -> Result<(), String> {
    let encoder = match Encoder::from_image(image) {
        Ok(e) => e,
        Err(e) => {
            return Err(e.to_string());
        }
    };
    let webp_memory = encoder.encode(100f32);

    match write(path, &*webp_memory) {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

pub fn save_image_to_avif(
    image: &image::DynamicImage,
    path: &PathBuf,
    quality: Option<f32>,
) -> Result<(), String> {
    use rgb::FromSlice;

    let rgba_image = image.to_rgba8();

    let width = rgba_image.width() as usize;
    let height = rgba_image.height() as usize;

    let pixels = rgba_image.as_raw().as_rgba();
    let img = Img::new(pixels, width, height);

    let encoder = AvifEncoder::new()
        .with_quality(quality.unwrap_or(80.0))
        .with_speed(6);

    match encoder.encode_rgba(img) {
        Ok(avif_data) => match write(path, &avif_data.avif_file) {
            Ok(_) => Ok(()),
            Err(e) => Err(e.to_string()),
        },
        Err(e) => Err(e.to_string()),
    }
}

pub async fn save_resized_image(
    image: image::DynamicImage,
    width: Option<u32>,
    original_path: &PathBuf,
    target_path: &PathBuf,
) -> Response {
    if width.is_none() {
        return response_file(target_path).await;
    }

    if image.width() <= width.unwrap() {
        copy(original_path, target_path).ok();
        return response_file(target_path).await;
    }

    let resize_height = width.unwrap() * image.height() / image.width();
    let resized_image = image.thumbnail(width.unwrap(), resize_height);

    match resized_image.save(target_path.clone()) {
        Ok(_) => response_file(target_path).await,
        Err(_) => response_error(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
