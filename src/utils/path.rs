pub fn get_resize_width_from_path(path: &str) -> Option<u32> {
    path.split('.').find_map(|part| {
        if part.starts_with('w') && part[1..].chars().all(char::is_numeric) {
            part[1..].parse::<u32>().ok()
        } else {
            None
        }
    })
}

pub fn get_original_path(path: &str, has_resize: bool) -> String {
    let (dir, filename) = match path.rfind('/') {
        Some(index) => (&path[..=index], &path[index + 1..]),
        None => ("", path),
    };

    let mut parts: Vec<&str> = filename.split('.').collect();

    if parts.last() == Some(&"webp") || parts.last() == Some(&"avif") {
        parts.pop();
    }

    if has_resize && parts.len() >= 2 {
        parts.remove(parts.len() - 2);
    }

    format!("{}{}", dir, parts.join("."))
}

/// Builds the storage object key for a request.
///
/// `include_host` controls the namespace: in read-through cache mode the host
/// segment disambiguates objects fetched from different upstreams
/// (`<prefix>/<host>/<path>`); in bucket-origin mode there is no upstream, so
/// the host is dropped and the key is simply `<prefix>/<path>`.
pub fn object_key(prefix: &str, host: &str, path: &str, include_host: bool) -> String {
    let path = path.trim_start_matches('/');

    if include_host {
        format!("{}/{}/{}", prefix, host, path)
    } else {
        format!("{}/{}", prefix, path)
    }
}

pub fn parse_path(full_path: &str) -> (Option<String>, String) {
    if full_path.contains("://") {
        // has protocol
        if let Some(protocol_end) = full_path.find("://") {
            let protocol = &full_path[..=protocol_end + 2];
            let after_protocol = &full_path[protocol_end + 3..];

            if let Some(path_start) = after_protocol.find('/') {
                let host = after_protocol[..path_start].to_string();
                let path = after_protocol[path_start..].to_string();
                return (Some(format!("{}{}", protocol, host)), path);
            } else {
                let host = after_protocol.to_string();
                return (Some(format!("{}{}", protocol, host)), "/".to_string());
            }
        }
    }

    if full_path.starts_with('/') {
        return (None, full_path.to_string());
    }

    (None, format!("/{}", full_path))
}
