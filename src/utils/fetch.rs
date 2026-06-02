use bytes::Bytes;
use reqwest::Client;
use tracing::error;

pub async fn fetch_remote(host: &str, path: &str) -> Result<Bytes, reqwest::Error> {
    let encoded_path: String = path
        .split('/')
        .map(|segment| urlencoding::encode(segment).into_owned())
        .collect::<Vec<_>>()
        .join("/");
    let url = format!("{}{}", host, encoded_path);

    match Client::new().get(&url).send().await?.error_for_status() {
        Ok(response) => response.bytes().await,
        Err(err) => {
            error!("Failed to fetch {}", url);
            Err(err)
        }
    }
}
