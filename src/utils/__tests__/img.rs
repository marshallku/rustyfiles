#[cfg(test)]
mod tests {
    use crate::utils::{
        img::{save_image_to_avif, save_image_to_webp, save_resized_image},
        path,
    };

    use image::{DynamicImage, ImageReader, RgbImage};
    use reqwest::StatusCode;
    use std::{
        fs::{self, read, set_permissions},
        os::unix::fs::PermissionsExt,
    };
    use tempfile::tempdir;
    use webp::Decoder;

    const IMAGE_PATH: &str = "src/utils/__tests__/fixtures";
    const IMAGE_NAME: &str = "image";
    const IMAGE_EXT: &str = "jpg";

    #[tokio::test]
    async fn test_save_image_to_webp() {
        const IMAGE_SIZE: u32 = 100;

        let dir = tempdir().unwrap();
        let output_path = dir.path().join("test_image.webp");

        let image = DynamicImage::ImageRgb8(image::RgbImage::new(IMAGE_SIZE, IMAGE_SIZE));

        match save_image_to_webp(&image, &output_path) {
            Ok(_) => {}
            Err(e) => {
                panic!("Failed to save image as WebP: {}", e);
            }
        };

        assert!(
            output_path.exists(),
            "Output file does not exist after saving image as WebP"
        );

        let webp_data = read(&output_path).unwrap();
        let decoded_webp = Decoder::new(&webp_data).decode().unwrap();

        assert_eq!(
            decoded_webp.width(),
            IMAGE_SIZE,
            "Decoded WebP width does not match original image width"
        );
        assert_eq!(
            decoded_webp.height(),
            IMAGE_SIZE,
            "Decoded WebP height does not match original image height"
        );
    }

    #[tokio::test]
    async fn test_save_image_to_webp_large_image() {
        let image = DynamicImage::ImageRgb8(RgbImage::new(10000, 10000));
        let dir = tempdir().unwrap();
        let output_path = dir.path().join("large_image.webp");

        match save_image_to_webp(&image, &output_path) {
            Ok(_) => {}
            Err(e) => {
                panic!("Failed to save large image as WebP: {}", e);
            }
        };

        assert!(
            output_path.exists(),
            "Output file does not exist after saving large image as WebP"
        );
    }

    #[tokio::test]
    async fn test_save_image_to_webp_read_only_file_system() {
        let image = DynamicImage::ImageRgb8(RgbImage::new(100, 100));
        let dir = tempdir().unwrap();
        let output_path = dir.path().join("read_only_image.webp");

        set_permissions(dir.path(), std::fs::Permissions::from_mode(0o444)).unwrap();

        let result = save_image_to_webp(&image, &output_path);

        assert!(
            result.is_err(),
            "Expected error when saving to a read-only file system, but got success"
        );
        assert!(
            !output_path.exists(),
            "Output file should not exist after attempting to save to a read-only file system"
        );
    }

    #[tokio::test]
    async fn test_save_image_to_webp_invalid_dimension() {
        let dir = tempdir().unwrap();
        let output_path = dir.path().join("test_image.webp");

        let invalid_cases = [
            DynamicImage::ImageLuma8(image::GrayImage::new(0, 0)),
            DynamicImage::ImageLuma8(image::GrayImage::new(0, 100)),
            DynamicImage::ImageLuma8(image::GrayImage::new(100, 0)),
        ];

        for invalid_image in invalid_cases.iter() {
            let result = save_image_to_webp(&invalid_image, &output_path);
            assert!(
                result.is_err(),
                "Expected error for invalid image dimensions, but got success"
            );
        }
    }

    #[tokio::test]
    async fn test_save_resized_image() {
        const TARGET_WIDTH: u32 = 100;

        let dir = tempdir().unwrap();

        let image_path = format!("{}.{}", IMAGE_NAME, IMAGE_EXT);
        let path = format!("{}.w{}.{}", IMAGE_NAME, TARGET_WIDTH, IMAGE_EXT);

        let file_path = dir.path().join(path.clone());
        let original_path = path::get_original_path(&path, true);
        let original_file_path = dir.path().join(format!("image{}", original_path.clone()));

        match fs::copy(
            format!("{}/{}", IMAGE_PATH, image_path.clone()),
            file_path.clone(),
        ) {
            Ok(_) => {}
            Err(_) => {
                panic!("Failed to copy image file");
            }
        };

        let image = ImageReader::open(file_path.clone())
            .unwrap()
            .decode()
            .unwrap();

        let response =
            save_resized_image(image, Some(TARGET_WIDTH), &original_file_path, &file_path).await;

        assert_eq!(
            response.status(),
            StatusCode::OK,
            "Expected OK status for resized image response, but got {}",
            response.status()
        );

        let resized_image = ImageReader::open(file_path).unwrap().decode().unwrap();

        assert_eq!(
            resized_image.width(),
            TARGET_WIDTH,
            "Resized image width does not match target width"
        );
    }

    #[tokio::test]
    async fn test_save_resized_image_without_width() {
        let dir = tempdir().unwrap();

        let image_path = format!("{}.{}", IMAGE_NAME, IMAGE_EXT);
        let file_path = dir.path().join(image_path.clone());

        match fs::copy(format!("{}/{}", IMAGE_PATH, image_path), &file_path) {
            Ok(_) => {}
            Err(_) => {
                panic!("Failed to copy image file");
            }
        };

        let image = ImageReader::open(&file_path).unwrap().decode().unwrap();

        let response = save_resized_image(image, None, &file_path, &file_path).await;

        assert_eq!(
            response.status(),
            StatusCode::OK,
            "Expected OK status for image response without width, but got {}",
            response.status()
        );
    }

    #[tokio::test]
    async fn test_save_image_to_avif() {
        const IMAGE_SIZE: u32 = 100;

        let dir = tempdir().unwrap();
        let output_path = dir.path().join("test_image.avif");

        let image = DynamicImage::ImageRgb8(image::RgbImage::new(IMAGE_SIZE, IMAGE_SIZE));

        match save_image_to_avif(&image, &output_path, None) {
            Ok(_) => {}
            Err(e) => {
                panic!("Failed to save image as AVIF: {}", e);
            }
        };

        assert!(
            output_path.exists(),
            "Output file does not exist after saving image as AVIF"
        );

        // Check that the file is not empty
        let avif_data = read(&output_path).unwrap();
        assert!(!avif_data.is_empty(), "AVIF file should not be empty");
    }

    #[tokio::test]
    async fn test_save_image_to_avif_large_image() {
        let image = DynamicImage::ImageRgb8(RgbImage::new(2000, 2000));
        let dir = tempdir().unwrap();
        let output_path = dir.path().join("large_image.avif");

        match save_image_to_avif(&image, &output_path, None) {
            Ok(_) => {}
            Err(e) => {
                panic!("Failed to save large image as AVIF: {}", e);
            }
        };

        assert!(
            output_path.exists(),
            "Output file does not exist after saving large image as AVIF"
        );
    }

    #[tokio::test]
    async fn test_save_image_to_avif_read_only_file_system() {
        let image = DynamicImage::ImageRgb8(RgbImage::new(100, 100));
        let dir = tempdir().unwrap();
        let output_path = dir.path().join("read_only_image.avif");

        set_permissions(dir.path(), std::fs::Permissions::from_mode(0o444)).unwrap();

        let result = save_image_to_avif(&image, &output_path, None);

        assert!(
            result.is_err(),
            "Expected error when saving to a read-only file system, but got success"
        );
        assert!(
            !output_path.exists(),
            "Output file should not exist after attempting to save to a read-only file system"
        );
    }

    #[tokio::test]
    async fn test_save_resized_image_with_larger_width() {
        const ORIGINAL_WIDTH: u32 = 460;
        const TARGET_WIDTH: u32 = 600;

        let dir = tempdir().unwrap();

        let image_path = format!("{}.{}", IMAGE_NAME, IMAGE_EXT);
        let path = format!("{}.w{}.{}", IMAGE_NAME, TARGET_WIDTH, IMAGE_EXT);

        let file_path = dir.path().join(path.clone());
        let original_path = path::get_original_path(&path, true);
        let original_file_path = dir.path().join(format!("image{}", original_path.clone()));

        match fs::copy(
            format!("{}/{}", IMAGE_PATH, image_path.clone()),
            file_path.clone(),
        ) {
            Ok(_) => {}
            Err(_) => {
                panic!("Failed to copy image file");
            }
        };

        let image = ImageReader::open(file_path.clone())
            .unwrap()
            .decode()
            .unwrap();

        let response =
            save_resized_image(image, Some(TARGET_WIDTH), &original_file_path, &file_path).await;

        assert_eq!(
            response.status(),
            StatusCode::OK,
            "Expected OK status for resized image response, but got {}",
            response.status()
        );

        assert_eq!(
            response.status(),
            StatusCode::OK,
            "Expected OK status for image response with larger target width, but got {}",
            response.status()
        );

        let resized_image = ImageReader::open(&file_path).unwrap().decode().unwrap();

        assert_eq!(
            resized_image.width(),
            ORIGINAL_WIDTH,
            "Image width should not change when target width is larger than original"
        );
    }
}
