#[cfg(test)]
mod tests {
    use crate::utils::img::{
        encode_image, encode_image_to_avif, encode_image_to_webp, guess_image_format, resize_image,
    };

    use image::{DynamicImage, ImageFormat, RgbImage};
    use webp::Decoder;

    #[test]
    fn test_encode_image_to_webp() {
        const IMAGE_SIZE: u32 = 100;
        let image = DynamicImage::ImageRgb8(RgbImage::new(IMAGE_SIZE, IMAGE_SIZE));

        let bytes = encode_image_to_webp(&image).expect("encode webp");
        assert!(!bytes.is_empty());

        let decoded = Decoder::new(&bytes).decode().unwrap();
        assert_eq!(decoded.width(), IMAGE_SIZE);
        assert_eq!(decoded.height(), IMAGE_SIZE);
    }

    #[test]
    fn test_encode_image_to_webp_large_image() {
        let image = DynamicImage::ImageRgb8(RgbImage::new(10000, 10000));
        let bytes = encode_image_to_webp(&image).expect("encode webp");
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_encode_image_to_webp_invalid_dimension() {
        let invalid_cases = [
            DynamicImage::ImageLuma8(image::GrayImage::new(0, 0)),
            DynamicImage::ImageLuma8(image::GrayImage::new(0, 100)),
            DynamicImage::ImageLuma8(image::GrayImage::new(100, 0)),
        ];

        for img in invalid_cases.iter() {
            assert!(encode_image_to_webp(img).is_err());
        }
    }

    #[test]
    fn test_encode_image_to_avif() {
        const IMAGE_SIZE: u32 = 100;
        let image = DynamicImage::ImageRgb8(RgbImage::new(IMAGE_SIZE, IMAGE_SIZE));

        let bytes = encode_image_to_avif(&image, None).expect("encode avif");
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_encode_image_to_avif_large_image() {
        let image = DynamicImage::ImageRgb8(RgbImage::new(2000, 2000));
        let bytes = encode_image_to_avif(&image, None).expect("encode avif");
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_resize_image_shrinks() {
        const TARGET_WIDTH: u32 = 100;
        let image = DynamicImage::ImageRgb8(RgbImage::new(400, 400));
        let resized = resize_image(image, TARGET_WIDTH);
        assert_eq!(resized.width(), TARGET_WIDTH);
    }

    #[test]
    fn test_encode_image_jpeg_roundtrip() {
        let image = DynamicImage::ImageRgb8(RgbImage::new(80, 80));
        let bytes = encode_image(&image, ImageFormat::Jpeg).expect("encode jpeg");
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_guess_image_format() {
        assert_eq!(guess_image_format("foo/bar.png"), ImageFormat::Png);
        assert_eq!(guess_image_format("foo/bar.jpg"), ImageFormat::Jpeg);
        assert_eq!(guess_image_format("foo/bar.webp"), ImageFormat::WebP);
    }
}
