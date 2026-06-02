#[cfg(test)]
mod tests {
    use crate::utils::path::{get_original_path, get_resize_width_from_path, object_key, parse_path};

    #[test]
    fn test_object_key() {
        // Remote (read-through cache): host segment disambiguates upstreams.
        assert_eq!(
            object_key("images", "example.com", "/photo.png", true),
            "images/example.com/photo.png"
        );
        assert_eq!(
            object_key("files", "example.com", "doc.pdf", true),
            "files/example.com/doc.pdf"
        );

        // Bucket-origin: no upstream, host segment dropped.
        assert_eq!(
            object_key("images", "example.com", "/photo.png", false),
            "images/photo.png"
        );
        assert_eq!(
            object_key("files", "example.com", "doc.pdf", false),
            "files/doc.pdf"
        );

        // Leading slash on path is normalized away in both modes.
        assert_eq!(
            object_key("images", "h", "///nested/a.png", true),
            "images/h/nested/a.png"
        );
    }

    #[test]
    fn test_get_resize_width_from_path() {
        assert_eq!(
            get_resize_width_from_path("/path/to/file.w100.jpg"),
            Some(100)
        );
        assert_eq!(
            get_resize_width_from_path("/path/to/file.with.dot.w200.jpg"),
            Some(200)
        );
        assert_eq!(
            get_resize_width_from_path("/path/to/file.with.dot.w200w.jpg"),
            None
        );
        assert_eq!(
            get_resize_width_from_path("/path/to/file.with.dot.w300.jpg.webp"),
            Some(300)
        );
        assert_eq!(
            get_resize_width_from_path("/path/to/file.with.dot.300.jpg.webp"),
            None
        );
        assert_eq!(get_resize_width_from_path("/path/to/file.jpg"), None);
        assert_eq!(
            get_resize_width_from_path("/path/to/file.with.dot.jpg"),
            None
        );
    }

    #[test]
    fn test_get_original_path() {
        let paths = vec![
            ["/path/to/file.w100.jpg", "/path/to/file.jpg"],
            ["/path/to/webp.w100.jpg.webp", "/path/to/webp.jpg"],
            ["/path/to/avif.w100.jpg.avif", "/path/to/avif.jpg"],
            [
                "/path/to/file.with.dot.w100.jpg",
                "/path/to/file.with.dot.jpg",
            ],
            [
                "/path/to/webp.with.dot.w100.jpg.webp",
                "/path/to/webp.with.dot.jpg",
            ],
            [
                "/path/to/avif.with.dot.w100.jpg.avif",
                "/path/to/avif.with.dot.jpg",
            ],
            ["/path/to/file.jpg", "/path/to/file.jpg"],
            ["/path/to/file.with.dot.jpg", "/path/to/file.with.dot.jpg"],
            [
                "/path/to/webp.with.dot.jpg.webp",
                "/path/to/webp.with.dot.jpg",
            ],
            [
                "/path/to/avif.with.dot.jpg.avif",
                "/path/to/avif.with.dot.jpg",
            ],
        ];

        for (_, path) in paths.iter().enumerate() {
            assert_eq!(
                get_original_path(path[0], get_resize_width_from_path(path[0]).is_some()),
                path[1]
            );
        }
    }

    #[test]
    fn test_parse_path() {
        assert_eq!(
            parse_path("https://example.com/images/w100/path/to/image.jpg"),
            (
                Some("https://example.com".to_string()),
                "/images/w100/path/to/image.jpg".to_string()
            )
        );
        assert_eq!(
            parse_path("https://example.com/"),
            (Some("https://example.com".to_string()), "/".to_string())
        );
        assert_eq!(
            parse_path("https://example.com"),
            (Some("https://example.com".to_string()), "/".to_string())
        );
        assert_eq!(
            parse_path("/images/w100/path/to/image.jpg"),
            (None, "/images/w100/path/to/image.jpg".to_string())
        );
        assert_eq!(
            parse_path("images/w100/path/to/image.jpg"),
            (None, "/images/w100/path/to/image.jpg".to_string())
        );
    }
}
