#[cfg(test)]
mod tests {
    use std::net::IpAddr;

    use crate::utils::fetch::{fetch_remote, is_blocked_ip};

    #[tokio::test]
    #[ignore = "hits a live external host; run with `cargo test -- --ignored`"]
    async fn should_fetch_file() {
        let host = "https://marshallku.com";
        let path = "/favicon.ico";

        let result = fetch_remote(host, path, true).await;

        assert!(result.is_ok());
        assert!(!result.unwrap().is_empty());
    }

    #[tokio::test]
    #[ignore = "hits a live external host; run with `cargo test -- --ignored`"]
    async fn should_error_on_missing_file() {
        let host = "https://marshallku.com";
        let path = "/must-be-404.ico";

        let result = fetch_remote(host, path, true).await;

        assert!(result.is_err());
    }

    /// SSRF guard: untrusted targets resolving to internal addresses are
    /// refused before any connection. IP-literal hosts are classified without a
    /// DNS lookup, so these assertions are hermetic (no network).
    #[tokio::test]
    async fn guard_blocks_internal_literal_hosts() {
        for host in [
            "http://127.0.0.1",
            "http://10.0.0.1",
            "http://192.168.1.1",
            "http://169.254.169.254", // cloud metadata endpoint
            "http://[::1]",
            "http://[::ffff:127.0.0.1]",
            "http://0.0.0.0",
        ] {
            let result = fetch_remote(host, "/latest/meta-data", true).await;
            assert!(
                result.is_err(),
                "guard must block internal host {host:?}"
            );
        }
    }

    /// An unsupported scheme is refused by the guard.
    #[tokio::test]
    async fn guard_blocks_non_http_scheme() {
        let result = fetch_remote("file://", "/etc/passwd", true).await;
        assert!(result.is_err());
    }

    #[test]
    fn classifies_blocked_addresses() {
        let blocked = [
            "127.0.0.1",
            "10.1.2.3",
            "172.16.0.1",
            "192.168.0.1",
            "169.254.169.254",
            "100.64.0.1",
            "0.0.0.0",
            "::1",
            "fc00::1",
            "fe80::1",
            "::ffff:127.0.0.1",
        ];
        for ip in blocked {
            let parsed: IpAddr = ip.parse().unwrap();
            assert!(is_blocked_ip(parsed), "{ip} should be blocked");
        }
    }

    #[test]
    fn allows_public_addresses() {
        let allowed = ["8.8.8.8", "1.1.1.1", "93.184.216.34", "2606:4700:4700::1111"];
        for ip in allowed {
            let parsed: IpAddr = ip.parse().unwrap();
            assert!(!is_blocked_ip(parsed), "{ip} should be allowed");
        }
    }
}
