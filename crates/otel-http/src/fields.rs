//! HTTP request field extraction utilities.

pub use http::header::FORWARDED;
use http::{HeaderName, Request};

/// The `X-Request-Id` header used for request correlation.
pub const X_REQUEST_ID: HeaderName = HeaderName::from_static("x-request-id");
/// The `Request-Id` fallback header used for request correlation.
pub const REQUEST_ID: HeaderName = HeaderName::from_static("request-id");
/// The `X-Forwarded-Proto` header used to determine the original URL scheme.
pub const X_FORWARDED_PROTO: HeaderName = HeaderName::from_static("x-forwarded-proto");

/// Extract the http method from the request
pub fn extract_http_method<T>(request: &Request<T>) -> &str {
    request.method().as_str()
}

/// Extract the http route from the request
pub fn extract_http_route<T>(request: &Request<T>) -> &str {
    request.uri().path()
}

/// Extract the http version from the request
pub fn extract_http_version<T>(request: &Request<T>) -> http::Version {
    request.version()
}

/// Returns the OpenTelemetry [`network.protocol.name`](https://opentelemetry.io/docs/specs/semconv/attributes-registry/network/).
///
/// The registry describes this as the **application-layer** protocol (examples include `http`, not
/// `https`). Use [`extract_http_scheme`] for [`url.scheme`](https://opentelemetry.io/docs/specs/semconv/registry/attributes/url/)
/// (`http` vs `https`), and [`extract_network_protocol_version`] for
/// [`network.protocol.version`](https://opentelemetry.io/docs/specs/semconv/registry/attributes/network/)
/// (e.g. `1.1`, `2`).
pub fn extract_network_protocol_name<T>(_request: &Request<T>) -> &'static str {
    "http"
}

/// Extract the network protocol version from the request.
pub fn extract_network_protocol_version<T>(request: &Request<T>) -> Option<&'static str> {
    match request.version() {
        http::Version::HTTP_09 => Some("0.9"),
        http::Version::HTTP_10 => Some("1.0"),
        http::Version::HTTP_11 => Some("1.1"),
        http::Version::HTTP_2 => Some("2"),
        http::Version::HTTP_3 => Some("3"),
        _ => None,
    }
}

/// Extract the http scheme from the request
pub fn extract_http_scheme<T>(request: &Request<T>) -> Option<&str> {
    request.uri().scheme().map(|s| s.as_str())
}

/// Extract the URL scheme from the request URI or proxy headers.
pub fn extract_url_scheme<T>(request: &Request<T>) -> Option<&str> {
    extract_http_scheme(request)
        .or_else(|| extract_field_from_headers(request.headers(), &X_FORWARDED_PROTO))
        .or_else(|| extract_forwarded_proto(request.headers()))
}

/// Extract the http target from the request
pub fn extract_http_target<T>(request: &Request<T>) -> Option<&str> {
    request.uri().path_and_query().map(|s| s.as_str())
}

/// Extract the URL path from the request.
pub fn extract_url_path<T>(request: &Request<T>) -> &str {
    request.uri().path()
}

/// Extract the URL query from the request.
pub fn extract_url_query<T>(request: &Request<T>) -> Option<&str> {
    request.uri().query()
}

/// Extract the user agent from the request headers
pub fn extract_user_agent<T>(request: &Request<T>) -> Option<&str> {
    extract_field_from_headers(request.headers(), &http::header::USER_AGENT)
}

/// Extract the `Host` header value.
pub fn extract_host<T>(request: &Request<T>) -> Option<&str> {
    extract_field_from_headers(request.headers(), &http::header::HOST)
}

/// Extract the request id from the request headers
pub fn extract_request_id<T>(request: &Request<T>) -> Option<&str> {
    extract_request_id_from_headers(request.headers())
}

/// Extract the request id from the request headers
pub fn extract_request_id_from_headers(headers: &http::HeaderMap) -> Option<&str> {
    headers
        .get(X_REQUEST_ID)
        .or_else(|| headers.get(REQUEST_ID))
        .and_then(|value| value.to_str().ok())
}

/// Extract a field from the request headers
pub fn extract_field_from_headers<'a>(
    headers: &'a http::HeaderMap,
    field: &HeaderName,
) -> Option<&'a str> {
    headers.get(field).and_then(|value| value.to_str().ok())
}

fn extract_forwarded_proto(headers: &http::HeaderMap) -> Option<&str> {
    let forwarded = extract_field_from_headers(headers, &FORWARDED)?;

    forwarded.split(',').find_map(|entry| {
        entry.split(';').find_map(|part| {
            let (key, value) = part.trim().split_once('=')?;
            if key.eq_ignore_ascii_case("proto") {
                Some(value.trim_matches('"'))
            } else {
                None
            }
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use http::HeaderMap;

    #[test]
    fn test_extract_request_id_with_x_request_id() {
        let mut headers = HeaderMap::new();
        headers.insert(X_REQUEST_ID, "test-id-1".parse().unwrap());
        let request_id = extract_request_id_from_headers(&headers);
        assert_eq!(request_id, Some("test-id-1"));
    }

    #[test]
    fn test_extract_request_id_without_request_id() {
        let request = Request::builder().body(()).unwrap();
        let request_id = extract_request_id(&request);
        assert_eq!(request_id, None);
    }

    #[test]
    fn test_extract_user_agent() {
        let request = Request::builder()
            .header(http::header::USER_AGENT, "test-user-agent")
            .body(())
            .unwrap();
        let user_agent = extract_user_agent(&request);
        assert_eq!(user_agent, Some("test-user-agent"));
    }

    #[test]
    fn test_extract_host() {
        let request = Request::builder()
            .header(http::header::HOST, "test-host")
            .body(())
            .unwrap();
        let host = extract_host(&request);
        assert_eq!(host, Some("test-host"));
    }

    #[test]
    fn test_extract_network_protocol_name() {
        let request = Request::builder().body(()).unwrap();
        assert_eq!(extract_network_protocol_name(&request), "http");
    }

    /// Per OTel registry, application protocol name stays `http` even for `https://` URIs.
    #[test]
    fn test_extract_network_protocol_name_https_uri_still_http() {
        let request = Request::builder()
            .uri("https://example.com/path")
            .body(())
            .unwrap();
        assert_eq!(extract_network_protocol_name(&request), "http");
        assert_eq!(extract_url_scheme(&request), Some("https"));
    }

    #[test]
    fn test_extract_network_protocol_version() {
        let request = Request::builder()
            .version(http::Version::HTTP_2)
            .body(())
            .unwrap();
        let version = extract_network_protocol_version(&request);
        assert_eq!(version, Some("2"));
    }

    #[test]
    fn test_extract_url_parts() {
        let request = Request::builder().uri("/items?kind=test").body(()).unwrap();
        let path = extract_url_path(&request);
        let query = extract_url_query(&request);
        assert_eq!(path, "/items");
        assert_eq!(query, Some("kind=test"));
    }

    #[test]
    fn test_extract_url_scheme_from_x_forwarded_proto() {
        let request = Request::builder()
            .uri("/items")
            .header(X_FORWARDED_PROTO, "https")
            .body(())
            .unwrap();
        assert_eq!(extract_url_scheme(&request), Some("https"));
    }

    #[test]
    fn test_extract_url_scheme_from_forwarded_header() {
        let request = Request::builder()
            .uri("/items")
            .header(FORWARDED, "for=192.0.2.60;proto=https;by=203.0.113.43")
            .body(())
            .unwrap();
        assert_eq!(extract_url_scheme(&request), Some("https"));
    }
}
