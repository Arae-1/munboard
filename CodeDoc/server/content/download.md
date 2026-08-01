# Content Download (`content/download.rs`)

## Summary

This module downloads content from a remote URL. It validates the URL to prevent server-side request forgery (SSRF) by blocking private IP addresses. It saves the downloaded content to the temporary uploads directory.

## Imports

- `api::error::{ApiError, ApiResult}`: API error types.
- `config::Config`: Application configuration.
- `content::upload::UploadToken`: Upload token.
- `filesystem::{self, MAX_URL_DOWNLOAD_SIZE}`: File system utilities and download size limit.
- `model::enums::MimeType`: MIME type enum.
- `reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderValue, REFERER}`: HTTP headers.
- `reqwest::{Client, StatusCode}`: HTTP client.
- `std::net::IpAddr`: IP address type.
- `std::str::FromStr`: String parsing.
- `tokio::net::lookup_host`: Async DNS resolution.
- `url::Url`: URL parsing.

## Function: `is_private_ip(addr: &IpAddr) -> bool`

Checks if an IP address is in a private/non-routable range. Used for SSRF protection.

**IPv4 checks:**
- `127.0.0.0/8` (loopback): octets[0] == 127
- `10.0.0.0/8`: octets[0] == 10
- `172.16.0.0/12`: octets[0] == 172 and 16 <= octets[1] <= 31
- `192.168.0.0/16`: octets[0] == 192 and octets[1] == 168
- `169.254.0.0/16` (link-local): octets[0] == 169 and octets[1] == 254

**IPv6 checks:**
- Loopback: `ip.is_loopback()`
- Unique local address (`fc00::/7`): `(ip.segments()[0] & 0xfe00) == 0xfc00`

Returns `true` if any check matches (private address), `false` otherwise.

## Function: `validate_url(url: &Url) -> ApiResult<()>`

Validates a URL before downloading. Returns `ApiError::BlockedUrl` for invalid or private URLs.

**Algorithm:**
1. Check URL scheme: Must be `"http"` or `"https"`. Return `BlockedUrl` if not.
2. Extract host from URL. Return `BlockedUrl` if absent.
3. Get port: `url.port_or_known_default().unwrap_or(80)`.
4. Construct address string: `"{host}:{port}"`.
5. Async DNS lookup: `lookup_host(addr_string).await`.
6. For each resolved address: call `is_private_ip(&addr.ip())`. If any resolves to a private IP, return `BlockedUrl`.
7. Return `Ok(())`.

## Function: `from_url(config, url) -> ApiResult<UploadToken>`

Downloads a file from a URL and saves it to the temporary uploads directory.

**Algorithm:**
1. Call `validate_url(&url).await` (SSRF check).
2. Define a fake user agent string: `"Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:135.0) Gecko/20100101 Firefox/135.0"`.
3. Create an HTTP client with the user agent.
4. Send GET request to the URL.
5. If response status is **403 Forbidden**:
   - Some websites block requests without a `Referer` header.
   - Build a second client that includes a `Referer` header set to the URL itself.
   - Retry the GET request.
6. Call `response.error_for_status()` to convert HTTP errors to Rust errors.
7. Extract `Content-Type` header:
   - Get the header value, convert to string slice.
   - Default to `""` if absent.
8. Parse the content type to `MimeType` using `MimeType::from_str()`. Box the error on failure.
9. Save the file: `filesystem::save_uploaded_file(config, response.bytes_stream(), mime_type, Some(MAX_URL_DOWNLOAD_SIZE)).await`.
10. Return the resulting `UploadToken`.

The `MAX_URL_DOWNLOAD_SIZE` limit prevents unbounded downloads.
