use crate::api::error::{ApiError, ApiResult};
use crate::config::Config;
use crate::content::upload::UploadToken;
use crate::filesystem;
use crate::model::enums::MimeType;
use reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderValue, LOCATION, REFERER};
use reqwest::redirect;
use reqwest::{Client, StatusCode};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::str::FromStr;
use std::time::Duration;
use tokio::net::lookup_host;
use url::Url;

fn is_private_ipv4(ip: &Ipv4Addr) -> bool {
    let octets = ip.octets();
    match (octets[0], octets[1]) {
        (0, _) => true,             // 0.0.0.0/8, including the unspecified/broadcast address
        (10, _) => true,            // 10.0.0.0/8
        (127, _) => true,           // 127.0.0.0/8 loopback
        (169, 254) => true,         // 169.254.0.0/16 link-local
        (172, 16..=31) => true,     // 172.16.0.0/12
        (192, 168) => true,         // 192.168.0.0/16
        (100, 64..=127) => true,    // 100.64.0.0/10 CGNAT
        (192, 0) => true,           // 192.0.0.0/24
        (198, 18..=19) => true,     // 198.18.0.0/15 benchmarking
        (224..=239, _) => true,     // 224.0.0.0/4 multicast
        _ => false,
    }
}

fn is_private_ip(addr: &IpAddr) -> bool {
    match addr {
        IpAddr::V4(ip) => is_private_ipv4(ip),
        IpAddr::V6(ip) => {
            ip.is_loopback()
                || ip.is_unspecified()
                || (ip.segments()[0] & 0xfe00) == 0xfc00 // fc00::/7 ULA
                || (ip.segments()[0] & 0xffc0) == 0xfe80 // fe80::/10 link-local
                || ip.to_ipv4_mapped().is_some_and(|ip| is_private_ipv4(&ip))
        }
    }
}

async fn validate_url(url: &Url) -> ApiResult<Vec<SocketAddr>> {
    if url.scheme() != "http" && url.scheme() != "https" {
        return Err(ApiError::BlockedUrl);
    }

    let host = url.host_str().ok_or(ApiError::BlockedUrl)?;
    let port = url.port_or_known_default().unwrap_or(80);
    let addr_string = format!("{host}:{port}");

    let addrs = lookup_host(addr_string).await?.collect::<Vec<SocketAddr>>();
    if addrs.is_empty() {
        return Err(ApiError::BlockedUrl);
    }
    for addr in &addrs {
        if is_private_ip(&addr.ip()) {
            return Err(ApiError::BlockedUrl);
        }
    }

    Ok(addrs)
}

/// Attempts to download file at the specified `url`.
/// If successful, the file is saved in the temporary uploads directory
/// and a content token is returned.
pub async fn from_url(config: &Config, url: Url) -> ApiResult<UploadToken> {
    const MAX_REDIRECTS: usize = 10;
    // Some websites expect a user-agent
    const FAKE_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:135.0) Gecko/20100101 Firefox/135.0";

    let mut referer: Option<HeaderValue> = None;
    let mut url = url;

    for _ in 0..=MAX_REDIRECTS {
        let addrs = validate_url(&url).await?;
        let host = url.host_str().ok_or(ApiError::BlockedUrl)?.to_owned();

        let mut client_builder = Client::builder()
            .user_agent(FAKE_USER_AGENT)
            .redirect(redirect::Policy::none())
            .timeout(Duration::from_secs(30))
            .resolve_to_addrs(&host, &addrs);
        if let Some(referer) = &referer {
            let mut headers = HeaderMap::new();
            headers.insert(REFERER, referer.clone());
            client_builder = client_builder.default_headers(headers);
        }
        let client = client_builder.build()?;

        let response = client.get(url.clone()).send().await?;
        if response.status() == StatusCode::FORBIDDEN && referer.is_none() {
            // Add referer, as some websites will 403 without it
            referer = Some(HeaderValue::from_str(url.as_str())?);
            continue;
        }
        if response.status().is_redirection() {
            let location = response.headers().get(LOCATION).ok_or(ApiError::BlockedUrl)?.to_str()?;
            url = url.join(location).map_err(|_| ApiError::BlockedUrl)?;
            continue;
        }
        let response = response.error_for_status()?;

        let content_type = response
            .headers()
            .get(CONTENT_TYPE)
            .map(|header_value| header_value.to_str())
            .transpose()?;
        let mime_type = MimeType::from_str(content_type.unwrap_or("")).map_err(Box::from)?;

        return filesystem::save_uploaded_file(
            config,
            response.bytes_stream(),
            mime_type,
            Some(config.max_url_download_size),
        )
        .await;
    }

    Err(ApiError::BlockedUrl)
}
