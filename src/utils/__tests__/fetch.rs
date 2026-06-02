#[cfg(test)]
mod tests {
    use crate::utils::fetch::fetch_remote;

    #[tokio::test]
    async fn should_fetch_file() {
        let host = "https://marshallku.com";
        let path = "/favicon.ico";

        let result = fetch_remote(host, path).await;

        assert!(result.is_ok());
        assert!(!result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn should_error_on_missing_file() {
        let host = "https://marshallku.com";
        let path = "/must-be-404.ico";

        let result = fetch_remote(host, path).await;

        assert!(result.is_err());
    }
}
