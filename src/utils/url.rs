pub fn remove_protocol(url: &str) -> String {
    let protocol_end = url.find("://");

    if let Some(protocol_end) = protocol_end {
        url[protocol_end + 3..].to_string()
    } else {
        url.to_string()
    }
}

pub fn get_host_from_url(url: &str) -> String {
    let url_without_protocol = remove_protocol(url);
    let path_start = url_without_protocol.find('/');

    if let Some(path_start) = path_start {
        url_without_protocol[..path_start].to_string()
    } else {
        url_without_protocol
    }
}
