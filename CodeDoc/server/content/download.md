# Content Download (`content/download.rs`)

## Summary

This module downloads content from a remote URL. It validates URLs against a private-address blocklist to prevent server-side request forgery (SSRF), follows redirects manually with a fixed limit, and pins each redirect target to its pre-resolved addresses. It saves the downloaded content to the temporary uploads directory.

## Imports

- `api::error::{ApiError, ApiResult}`: API error types.
- `config::Config`: Application configuration.
- `content::upload::UploadToken`: Upload token.
- `filesystem`: File system utilities (download size limit comes from `config.max_url_download_size`, not a constant).
- `model::enums::MimeType`: MIME type enum.
- `reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderValue, LOCATION, REFERER}`: HTTP headers.
- `reqwest::redirect`: Redirect policy control.
- `reqwest::{Client, StatusCode}`: HTTP client.
- `std::net::{IpAddr, Ipv4Addr, SocketAddr}`: IP address types.
- `std::str::FromStr`: String parsing.
- `std::time::Duration`: Timeout durations.
- `tokio::net::lookup_host`: Async DNS resolution.
- `url::Url`: URL parsing.

## Function: `is_private_ipv4(ip: &Ipv4Addr) -> bool`

Checks if an IPv4 address is in a blocked, non-public range. Used for SSRF protection. The check inspects the first octets directly.

| Blocklist Entry | Range | Reason |
|-----------------|-------|--------|
| `octets[0] == 0` | `0.0.0.0/8` | Unspecified/broadcast addressing |
| `octets[0] == 10` | `10.0.0.0/8` | Private network |
| `octets[0] == 127` | `127.0.0.0/8` | Loopback |
| `octets[0] == 169 && octets[1] == 254` | `169.254.0.0/16` | Link-local |
| `octets[0] == 172 && 16 <= octets[1] <= 31` | `172.16.0.0/12` | Private network |
| `octets[0] == 192 && octets[1] == 168` | `192.168.0.0/16` | Private network |
| `octets[0] == 100 && 64 <= octets[1] <= 127` | `100.64.0.0/10` | Carrier-grade NAT (CGNAT) |
| `octets[0] == 192 && octets[1] == 0 && octets[2] == 0` | `192.0.0.0/24` | IETF protocol assignments |
| `octets[0] == 198 && 18 <= octets[1] <= 19` | `198.18.0.0/15` | Benchmarking |
| `224 <= octets[0] <= 239` | `224.0.0.0/4` | Multicast |

Returns `true` if the address matches any entry, `false` otherwise.

## Function: `is_private_ip(addr: &IpAddr) -> bool`

Checks if an IP address (IPv4 or IPv6) is in a blocked, non-public range. Used for SSRF protection.

**IPv4:** Delegates to `is_private_ipv4`.

**IPv6 checks:**
- Loopback (`::1`): `ip.is_loopback()`.
- Unspecified (`::`): `ip.is_unspecified()`.
- Unique local address (`fc00::/7`): `(ip.segments()[0] & 0xfe00) == 0xfc00`.
- Link-local (`fe80::/10`): `(ip.segments()[0] & 0xffc0) == 0xfe80`.
- IPv4-mapped IPv6 (`::ffff:a.b.c.d`): `ip.to_ipv4_mapped()`, then checked against the IPv4 blocklist with `is_private_ipv4`. This stops a v4-mapped address from bypassing the v4 rules.

Returns `true` if any check matches, `false` otherwise.

## Function: `validate_url(url: &Url) -> ApiResult<Vec<SocketAddr>>`

Validates a URL before downloading and resolves its host. Returns the resolved socket addresses, or `ApiError::BlockedUrl` for invalid, unresolved, or blocked URLs.

**Algorithm:**
1. Check URL scheme: Must be `"http"` or `"https"`. Return `BlockedUrl` if not.
2. Extract host from URL. Return `BlockedUrl` if absent.
3. Get port: `url.port_or_known_default().unwrap_or(80)`.
4. Construct address string: `"{host}:{port}"`.
5. Async DNS lookup: `lookup_host(addr_string).await`.
6. If no address resolves: return `BlockedUrl`.
7. For each resolved address: call `is_private_ip(&addr.ip())`. If any address is private, return `BlockedUrl`.
8. Return the resolved addresses (`Vec<SocketAddr>`).

## Function: `from_url(config, url) -> ApiResult<UploadToken>`

Downloads a file from a URL and saves it to the temporary uploads directory.

**Constants:**
- `MAX_REDIRECTS: usize = 10`: Maximum number of redirects to follow.
- `FAKE_USER_AGENT`: `"Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:135.0) Gecko/20100101 Firefox/135.0"`. Some websites refuse requests without a browser user-agent.

**Algorithm:**
1. Initialize `referer = None` and the working `url`.
2. Loop at most `MAX_REDIRECTS + 1` times:
   - Call `validate_url(&url)` (SSRF check) to get the resolved addresses.
   - Build an HTTP client:
     - `user_agent(FAKE_USER_AGENT)`.
     - `redirect(redirect::Policy::none())`: automatic redirects are disabled so each hop is validated separately.
     - `timeout(Duration::from_secs(30))`: 30-second wall-clock limit for each request.
     - `resolve_to_addrs(&host, &addrs)`: pins the host to the addresses already validated by `validate_url`. The client cannot connect anywhere else.
     - If a referer is stored, add it as a default `Referer` header.
   - Send a GET request to the URL.
   - If the response status is **403 Forbidden** and no referer is set:
     - Store `referer = url` and retry the loop. Some websites return 403 without a `Referer` header.
   - If the response status is a redirection (3xx):
     - Read the `Location` header. Missing or invalid values return `BlockedUrl`.
     - Resolve the new URL against the current one with `url.join(location)`.
     - Continue the loop. The next iteration re-validates the new URL against the blocklist.
   - Call `response.error_for_status()` to convert HTTP errors to Rust errors.
   - Extract the `Content-Type` header:
     - Get the header value, convert to string slice.
     - Default to `""` if absent.
   - Parse the content type to `MimeType` using `MimeType::from_str()`. Box the error on failure.
   - Save the file: `filesystem::save_uploaded_file(config, response.bytes_stream(), mime_type, Some(config.max_url_download_size)).await`.
   - Return the resulting `UploadToken`.
3. If the redirect limit is reached without a final response: return `Err(ApiError::BlockedUrl)`.

**Security notes:**
- Every redirect target goes through `validate_url` before its addresses are pinned with `resolve_to_addrs`. A redirect cannot reach a private address.
- The request timeout of 30 seconds prevents slow-or-drip downloads from tying up the server.
- The size cap for URL downloads comes from configuration (`config.max_url_download_size`), enforced while streaming to disk.
