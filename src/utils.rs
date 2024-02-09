pub fn get_resize_width_from_path(path: &str) -> Option<u32> {
    return path
        .split('.')
        .find(|s| s.starts_with("w"))
        .and_then(|s| s.strip_prefix("w"))
        .and_then(|s| s.parse::<u32>().ok());
}

pub fn get_original_path(path: &str, has_resize: bool) -> String {
    let extension = path.split('.').last().unwrap_or("");
    let original_path = if has_resize {
        path.to_string()
    } else {
        path.split('.').collect::<Vec<&str>>()[..path.split('.').count() - 2].join(".")
            + "."
            + extension
    };

    return original_path;
}
