use crate::api::error::{ApiError, ApiResult};
use crate::config::Config;
use crate::content::upload::UploadToken;
use crate::filesystem::{self, MAX_URL_DOWNLOAD_SIZE};
use crate::model::enums::MimeType;
use reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderValue, REFERER};
use reqwest::{Client, StatusCode};
use std::net::IpAddr;
use std::str::FromStr;
use tokio::net::lookup_host;
use url::Url;

fn is_private_ip(addr: &IpAddr) -> bool {
    match addr {
        IpAddr::V4(ip) => {
            let octets = ip.octets();
            octets[0] == 127
                || octets[0] == 10
                || (octets[0] == 172 && octets[1] >= 16 && octets[1] <= 31)
                || (octets[0] == 192 && octets[1] == 168)
                || (octets[0] == 169 && octets[1] == 254)
        }
        IpAddr::V6(ip) => {
            ip.is_loopback() || (ip.segments()[0] & 0xfe00) == 0xfc00
        }
    }
}

async fn validate_url(url: &Url) -> ApiResult<()> {
    if url.scheme() != "http" && url.scheme() != "https" {
        return Err(ApiError::BlockedUrl);
    }

    let host = url.host().ok_or(ApiError::BlockedUrl)?;
    let port = url.port_or_known_default().unwrap_or(80);
    let addr_string = format!("{host}:{port}");

    let addrs = lookup_host(addr_string).await?;
    for addr in addrs {
        if is_private_ip(&addr.ip()) {
            return Err(ApiError::BlockedUrl);
        }
    }

    Ok(())
}

/// Attempts to download file at the specified `url`.
/// If successful, the file is saved in the temporary uploads directory
/// and a content token is returned.
pub async fn from_url(config: &Config, url: Url) -> ApiResult<UploadToken> {
    validate_url(&url).await?;
    // Some websites expect a user-agent
    const FAKE_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:135.0) Gecko/20100101 Firefox/135.0";

    let client = Client::builder().user_agent(FAKE_USER_AGENT).build()?;
    let mut response = client.get(url.clone()).send().await?;
    if response.status() == StatusCode::FORBIDDEN {
        // Add referer, as some websites will 403 without it
        let mut headers = HeaderMap::new();
        headers.insert(REFERER, HeaderValue::from_str(url.as_str())?);

        let client = Client::builder()
            .user_agent(FAKE_USER_AGENT)
            .default_headers(headers)
            .build()?;
        response = client.get(url).send().await?;
    }
    let response = response.error_for_status()?;

    let content_type = response
        .headers()
        .get(CONTENT_TYPE)
        .map(|header_value| header_value.to_str())
        .transpose()?;
    let mime_type = MimeType::from_str(content_type.unwrap_or("")).map_err(Box::from)?;

    filesystem::save_uploaded_file(config, response.bytes_stream(), mime_type, Some(MAX_URL_DOWNLOAD_SIZE)).await
}
