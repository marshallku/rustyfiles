#[cfg(test)]
mod tests {
    use crate::utils::url::{get_host_from_url, remove_protocol};

    #[test]
    fn test_remove_protocol() {
        assert_eq!(remove_protocol("https://example.com"), "example.com");
        assert_eq!(remove_protocol("http://example.com"), "example.com");
        assert_eq!(remove_protocol("example.com"), "example.com");
        assert_eq!(
            remove_protocol("https://example.com/path"),
            "example.com/path"
        );
        assert_eq!(
            remove_protocol("http://example.com/path"),
            "example.com/path"
        );
        assert_eq!(remove_protocol("example.com/path"), "example.com/path");
        assert_eq!(
            remove_protocol("https://example.com/path?query=value"),
            "example.com/path?query=value"
        );
        assert_eq!(
            remove_protocol("http://example.com/path?query=value"),
            "example.com/path?query=value"
        );
        assert_eq!(
            remove_protocol("example.com/path?query=value"),
            "example.com/path?query=value"
        );
    }

    #[test]
    fn test_get_host_from_url() {
        assert_eq!(get_host_from_url("https://example.com"), "example.com");
        assert_eq!(get_host_from_url("http://example.com"), "example.com");
        assert_eq!(get_host_from_url("example.com"), "example.com");
        assert_eq!(get_host_from_url("https://example.com/path"), "example.com");
        assert_eq!(get_host_from_url("http://example.com/path"), "example.com");
        assert_eq!(get_host_from_url("example.com/path"), "example.com");
        assert_eq!(
            get_host_from_url("https://example.com/path?query=value"),
            "example.com"
        );
    }
}
