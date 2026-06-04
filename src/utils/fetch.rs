use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    time::Duration,
};

use bytes::Bytes;
use reqwest::{redirect::Policy, Client};
use tokio::net::lookup_host;
use tracing::error;
use url::{Host, Url};

const FETCH_TIMEOUT: Duration = Duration::from_secs(15);

#[derive(Debug)]
pub enum FetchError {
    /// The target was refused before any request was made: a malformed URL, an
    /// unsupported scheme, or a host that resolves to a private/internal
    /// address (SSRF guard).
    Blocked,
    Request(reqwest::Error),
}

impl std::fmt::Display for FetchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FetchError::Blocked => write!(f, "blocked target"),
            FetchError::Request(err) => write!(f, "{}", err),
        }
    }
}

/// Fetches `host + path` over HTTP.
///
/// When `guard` is `true` the target is treated as untrusted (it came from the
/// request URL, not the operator config) and the SSRF guard applies: the host
/// is resolved up front, the request is refused if any resolved address is
/// private/internal, and reqwest is pinned to the validated addresses so it
/// cannot re-resolve to a different IP. The operator-configured default origin
/// is fetched with `guard = false`.
pub async fn fetch_remote(host: &str, path: &str, guard: bool) -> Result<Bytes, FetchError> {
    let encoded_path: String = path
        .split('/')
        .map(|segment| urlencoding::encode(segment).into_owned())
        .collect::<Vec<_>>()
        .join("/");
    let url = format!("{}{}", host, encoded_path);

    let client = if guard {
        build_guarded_client(&url).await?
    } else {
        build_client(None)?
    };

    match client.get(&url).send().await.and_then(|res| res.error_for_status()) {
        Ok(response) => response.bytes().await.map_err(FetchError::Request),
        Err(err) => {
            error!("Failed to fetch {}", url);
            Err(FetchError::Request(err))
        }
    }
}

/// A reqwest client hardened against SSRF pivots regardless of the target:
/// redirects are not followed (a 302 cannot bypass the address check) and
/// proxy environment variables are ignored (a proxy must not resolve/fetch the
/// target on our behalf). A request timeout bounds slow-upstream DoS.
fn build_client(resolve: Option<(&str, &[SocketAddr])>) -> Result<Client, FetchError> {
    let mut builder = Client::builder()
        .redirect(Policy::none())
        .no_proxy()
        .timeout(FETCH_TIMEOUT);

    if let Some((domain, addrs)) = resolve {
        builder = builder.resolve_to_addrs(domain, addrs);
    }

    builder.build().map_err(FetchError::Request)
}

/// Validates an untrusted target and returns a client pinned to safe addresses.
async fn build_guarded_client(url: &str) -> Result<Client, FetchError> {
    let parsed = Url::parse(url).map_err(|_| FetchError::Blocked)?;

    if !matches!(parsed.scheme(), "http" | "https") {
        return Err(FetchError::Blocked);
    }

    let port = parsed.port_or_known_default().ok_or(FetchError::Blocked)?;

    match parsed.host() {
        // IP literals are not resolved via DNS, so reqwest connects to exactly
        // this address — validate it directly, no pinning needed.
        Some(Host::Ipv4(ip)) if !is_blocked_ip(IpAddr::V4(ip)) => build_client(None),
        Some(Host::Ipv6(ip)) if !is_blocked_ip(IpAddr::V6(ip)) => build_client(None),
        Some(Host::Domain(domain)) => {
            let addrs: Vec<SocketAddr> = lookup_host((domain, port))
                .await
                .map_err(|_| FetchError::Blocked)?
                .collect();

            if addrs.is_empty() || addrs.iter().any(|addr| is_blocked_ip(addr.ip())) {
                return Err(FetchError::Blocked);
            }

            // Pin reqwest to the validated addresses so it cannot perform a
            // second DNS lookup that resolves to an internal IP (DNS rebinding).
            build_client(Some((domain, &addrs)))
        }
        _ => Err(FetchError::Blocked),
    }
}

/// Whether an address belongs to a range that must not be reachable through the
/// proxy (loopback, private, link-local, etc.). Exposed within the crate for
/// table-driven tests.
pub(crate) fn is_blocked_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => is_blocked_v4(v4),
        IpAddr::V6(v6) => {
            // An IPv4 address tunnelled in IPv6 (e.g. ::ffff:127.0.0.1) must be
            // judged by its IPv4 range, not the IPv6 rules.
            if let Some(v4) = v6.to_ipv4_mapped().or_else(|| v6.to_ipv4()) {
                return is_blocked_v4(v4);
            }
            is_blocked_v6(v6)
        }
    }
}

fn is_blocked_v4(ip: Ipv4Addr) -> bool {
    let [a, b, _, _] = ip.octets();

    ip.is_loopback()
        || ip.is_private()
        || ip.is_link_local()
        || ip.is_unspecified()
        || ip.is_broadcast()
        || ip.is_documentation()
        || ip.is_multicast()
        || a == 0
        // 100.64.0.0/10 — carrier-grade NAT / shared address space.
        || (a == 100 && (b & 0xc0) == 64)
}

fn is_blocked_v6(ip: Ipv6Addr) -> bool {
    let first = ip.segments()[0];

    ip.is_loopback()
        || ip.is_unspecified()
        || ip.is_multicast()
        // fc00::/7 — unique local addresses.
        || (first & 0xfe00) == 0xfc00
        // fe80::/10 — link-local unicast.
        || (first & 0xffc0) == 0xfe80
}
