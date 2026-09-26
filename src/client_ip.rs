use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;

use axum::http::HeaderMap;

/// Identify the client behind the request.
///
/// In production the app sits behind Caddy, so the socket address is the
/// proxy's and every visitor would share a single rate-limit bucket. When
/// `trust_proxy_headers` is on we use the address the proxy recorded instead.
///
/// `X-Forwarded-For` is read from the **right**: the proxy appends the peer it
/// actually saw, while anything further left was supplied by the client and can
/// be forged. The value is parsed as an IP address, so a client can't hand us
/// an arbitrary string to use as a rate-limit key; anything that doesn't parse
/// (or a missing header) falls back to the socket address.
///
/// `X-Real-IP` is not consulted: Caddy never sets it, so honoring it would only
/// ever give a client a way to spoof its address.
pub fn client_ip(
    headers: &HeaderMap,
    socket_addr: SocketAddr,
    trust_proxy_headers: bool,
) -> IpAddr {
    if trust_proxy_headers && let Some(ip) = forwarded_for(headers) {
        return ip;
    }
    socket_addr.ip()
}

fn forwarded_for(headers: &HeaderMap) -> Option<IpAddr> {
    let raw = header_value(headers, "x-forwarded-for")?;
    raw.rsplit(',')
        .map(str::trim)
        .find(|part| !part.is_empty())
        .and_then(|part| IpAddr::from_str(part).ok())
}

fn header_value(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

/// Key used to bucket a client for rate limiting.
///
/// IPv6 addresses are commonly assigned by the /64, so a visitor who owns a
/// whole /64 could rotate the last four segments on every request and dodge a
/// per-address limit entirely. Bucket IPv6 by its /64 prefix instead; IPv4
/// addresses are scarce enough to key on directly.
pub fn rate_limit_key(ip: IpAddr) -> String {
    match ip {
        IpAddr::V4(_) => ip.to_string(),
        IpAddr::V6(v6) => {
            let s = v6.segments();
            format!("{:x}:{:x}:{:x}:{:x}::/64", s[0], s[1], s[2], s[3])
        }
    }
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
        assert_eq!(
            client_ip(&HeaderMap::new(), socket(), true).to_string(),
            "10.0.0.1"
        );
    }

    #[test]
    fn uses_the_proxy_recorded_address() {
        let headers = headers_with("x-forwarded-for", "203.0.113.7");
        assert_eq!(
            client_ip(&headers, socket(), true).to_string(),
            "203.0.113.7"
        );
    }

    #[test]
    fn ignores_client_supplied_entries_on_the_left() {
        let headers = headers_with("x-forwarded-for", "1.2.3.4, 203.0.113.7");
        assert_eq!(
            client_ip(&headers, socket(), true).to_string(),
            "203.0.113.7"
        );
    }

    #[test]
    fn ignores_proxy_headers_when_not_trusted() {
        let headers = headers_with("x-forwarded-for", "203.0.113.7");
        assert_eq!(client_ip(&headers, socket(), false).to_string(), "10.0.0.1");
    }

    #[test]
    fn ignores_x_real_ip_even_when_trusted() {
        let headers = headers_with("x-real-ip", "198.51.100.9");
        assert_eq!(client_ip(&headers, socket(), true).to_string(), "10.0.0.1");
    }

    #[test]
    fn falls_back_to_the_socket_address_on_an_unparseable_header() {
        let headers = headers_with("x-forwarded-for", "not-an-ip");
        assert_eq!(client_ip(&headers, socket(), true).to_string(), "10.0.0.1");
    }

    #[test]
    fn ignores_empty_header_values() {
        let headers = headers_with("x-forwarded-for", "  ,  ");
        assert_eq!(client_ip(&headers, socket(), true).to_string(), "10.0.0.1");
    }

    #[test]
    fn keys_ipv4_addresses_directly() {
        assert_eq!(
            rate_limit_key("203.0.113.7".parse().unwrap()),
            "203.0.113.7"
        );
    }

    #[test]
    fn buckets_ipv6_addresses_by_their_64_prefix() {
        let a: IpAddr = "2001:db8::1".parse().unwrap();
        let b: IpAddr = "2001:db8::ffff:ffff:ffff:ffff".parse().unwrap();
        assert_eq!(rate_limit_key(a), rate_limit_key(b));
    }

    #[test]
    fn keeps_different_ipv6_64_prefixes_apart() {
        let a: IpAddr = "2001:db8:0:1::1".parse().unwrap();
        let b: IpAddr = "2001:db8:0:2::1".parse().unwrap();
        assert_ne!(rate_limit_key(a), rate_limit_key(b));
    }
}
