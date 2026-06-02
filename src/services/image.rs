use axum::response::Response;
use bytes::Bytes;
use image::DynamicImage;
use reqwest::StatusCode;
use tracing::error;

use crate::{
    env::state::AppState,
    storage::{Storage, StorageError},
    utils::{
        fetch::fetch_remote,
        img::{encode_image, encode_image_to_avif, encode_image_to_webp, guess_image_format, resize_image},
        path::{get_original_path, get_resize_width_from_path},
        url::get_host_from_url,
    },
};

pub async fn process_image_request(
    state: &AppState,
    host: Option<String>,
    path: &str,
) -> Result<Response, StatusCode> {
    let target_host = host.unwrap_or(state.host.clone());
    let pure_host = get_host_from_url(&target_host);
    let target_key = image_key(&pure_host, path);

    match state.storage.exists(&target_key).await {
        Ok(true) => {
            return state
                .storage
                .serve(&target_key)
                .await
                .map_err(|err| map_storage_err(err, &target_key));
        }
        Ok(false) => {}
        Err(err) => return Err(map_storage_err(err, &target_key)),
    }

    let resize_width = get_resize_width_from_path(path);
    let convert_to_webp = path.ends_with(".webp");
    let convert_to_avif = path.ends_with(".avif");
    let original_path = get_original_path(path, resize_width.is_some());
    let original_key = image_key(&pure_host, &original_path);

    let original_bytes = load_or_fetch(state.storage.as_ref(), &original_key, &target_host, &original_path).await?;

    if resize_width.is_none() && !convert_to_webp && !convert_to_avif {
        return state
            .storage
            .serve(&target_key)
            .await
            .map_err(|err| map_storage_err(err, &target_key));
    }

    let image = decode_image(&original_bytes)?;

    let (intermediate_bytes, intermediate_image) = if convert_to_avif {
        let bytes =
            Bytes::from(encode_image_to_avif(&image, Some(80.0)).map_err(internal_error)?);
        let decoded = decode_image(&bytes)?;
        (bytes, decoded)
    } else if convert_to_webp {
        let bytes = Bytes::from(encode_image_to_webp(&image).map_err(internal_error)?);
        let decoded = decode_image(&bytes)?;
        (bytes, decoded)
    } else {
        (original_bytes.clone(), image)
    };

    let final_bytes = match resize_width {
        Some(width) if intermediate_image.width() > width => {
            let resized = resize_image(intermediate_image, width);
            let format = guess_image_format(&target_key);
            Bytes::from(encode_image(&resized, format).map_err(internal_error)?)
        }
        _ => intermediate_bytes,
    };

    if let Err(err) = state.storage.put_bytes(&target_key, final_bytes).await {
        error!("Failed to persist {}: {}", target_key, err);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    state
        .storage
        .serve(&target_key)
        .await
        .map_err(|err| map_storage_err(err, &target_key))
}

async fn load_or_fetch(
    storage: &dyn Storage,
    key: &str,
    host: &str,
    path: &str,
) -> Result<Bytes, StatusCode> {
    match storage.exists(key).await {
        Ok(true) => {
            return storage
                .get_bytes(key)
                .await
                .map_err(|err| map_storage_err(err, key));
        }
        Ok(false) => {}
        Err(err) => return Err(map_storage_err(err, key)),
    }

    let bytes = fetch_remote(host, path).await.map_err(|_| StatusCode::NOT_FOUND)?;
    storage
        .put_bytes(key, bytes.clone())
        .await
        .map_err(|err| map_storage_err(err, key))?;
    Ok(bytes)
}

fn image_key(host: &str, path: &str) -> String {
    format!("images/{}/{}", host, path.trim_start_matches('/'))
}

fn decode_image(bytes: &Bytes) -> Result<DynamicImage, StatusCode> {
    image::load_from_memory(bytes).map_err(|err| {
        error!("Failed to decode image: {}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

fn internal_error(err: String) -> StatusCode {
    error!("Image processing failed: {}", err);
    StatusCode::INTERNAL_SERVER_ERROR
}

fn map_storage_err(err: StorageError, key: &str) -> StatusCode {
    match err {
        StorageError::NotFound => StatusCode::NOT_FOUND,
        StorageError::Other(msg) => {
            error!("Storage error on {}: {}", key, msg);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
