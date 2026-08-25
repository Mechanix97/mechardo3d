use std::net::SocketAddr;

use axum::http::HeaderMap;

/// Identify the client behind the request.
///
/// In production the app sits behind Caddy, so the socket address is the
/// proxy's and every visitor would share a single rate-limit bucket. When
/// `trust_proxy_headers` is on we use the address the proxy recorded instead.
///
/// `X-Forwarded-For` is read from the **right**: the proxy appends the peer it
/// actually saw, while anything further left was supplied by the client and can
/// be forged.
pub fn client_ip(
    headers: &HeaderMap,
    socket_addr: SocketAddr,
    trust_proxy_headers: bool,
) -> String {
    if trust_proxy_headers {
        if let Some(ip) = forwarded_for(headers) {
            return ip;
        }
        if let Some(ip) = header_value(headers, "x-real-ip") {
            return ip;
        }
    }
    socket_addr.ip().to_string()
}

fn forwarded_for(headers: &HeaderMap) -> Option<String> {
    let raw = header_value(headers, "x-forwarded-for")?;
    raw.rsplit(',')
        .map(|part| part.trim())
        .find(|part| !part.is_empty())
        .map(|part| part.to_string())
}

fn header_value(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    fn socket() -> SocketAddr {
        SocketAddr::from(([10, 0, 0, 1], 51234))
    }

    fn headers_with(name: &'static str, value: &'static str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(name, HeaderValue::from_static(value));
        headers
    }

    #[test]
    fn falls_back_to_the_socket_address() {
        assert_eq!(client_ip(&HeaderMap::new(), socket(), true), "10.0.0.1");
    }

    #[test]
    fn uses_the_proxy_recorded_address() {
        let headers = headers_with("x-forwarded-for", "203.0.113.7");
        assert_eq!(client_ip(&headers, socket(), true), "203.0.113.7");
    }

    #[test]
    fn ignores_client_supplied_entries_on_the_left() {
        let headers = headers_with("x-forwarded-for", "1.2.3.4, 203.0.113.7");
        assert_eq!(client_ip(&headers, socket(), true), "203.0.113.7");
    }

    #[test]
    fn ignores_proxy_headers_when_not_trusted() {
        let headers = headers_with("x-forwarded-for", "203.0.113.7");
        assert_eq!(client_ip(&headers, socket(), false), "10.0.0.1");
    }

    #[test]
    fn falls_back_to_x_real_ip() {
        let headers = headers_with("x-real-ip", "198.51.100.9");
        assert_eq!(client_ip(&headers, socket(), true), "198.51.100.9");
    }

    #[test]
    fn ignores_empty_header_values() {
        let headers = headers_with("x-forwarded-for", "  ,  ");
        assert_eq!(client_ip(&headers, socket(), true), "10.0.0.1");
    }
}
