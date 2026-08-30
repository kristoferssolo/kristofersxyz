use crate::{
    configuration::PublicOrigin,
    security_events::{HostRejection, SecurityEvent},
};
use axum::{
    extract::{Request, State},
    http::{HeaderMap, HeaderValue, StatusCode, Uri, header, uri::Authority},
    middleware::Next,
    response::{IntoResponse, Response},
};

/// Rejects requests addressed to an authority other than the one in
/// `PUBLIC_ORIGIN`.
///
/// Only the request target and the `Host` header name the authority a client
/// asked for. `Forwarded`, `X-Forwarded-Host`, and their relatives are supplied
/// by whoever opened the connection, so they never contribute; the trusted
/// proxy has to preserve or set the canonical `Host` instead. This runs ahead
/// of CSRF checks, body buffering, authentication, and session lookup, so a
/// misdirected request never reaches Owner state.
pub async fn verify(
    State(public_origin): State<PublicOrigin>,
    request: Request,
    next: Next,
) -> Response {
    match resolve(request.uri(), request.headers()) {
        Ok(authority) if authority == *public_origin.authority() => next.run(request).await,
        Ok(_) => reject(HostRejection::Unexpected),
        Err(reason) => reject(reason),
    }
}

/// The authority the request is addressed to, taken from the request target and
/// the `Host` header. The two must agree when both are present.
fn resolve(uri: &Uri, headers: &HeaderMap) -> Result<Authority, HostRejection> {
    match (uri.authority(), host_header(headers)?) {
        (None, None) => Err(HostRejection::Missing),
        (Some(target), None) => Ok(target.clone()),
        (None, Some(host)) => Ok(host),
        (Some(target), Some(host)) if *target == host => Ok(host),
        (Some(_), Some(_)) => Err(HostRejection::Conflicting),
    }
}

/// The parsed `Host` header. Repeated headers disagree with each other by
/// construction, because a request has one target.
fn host_header(headers: &HeaderMap) -> Result<Option<Authority>, HostRejection> {
    let mut values = headers.get_all(header::HOST).into_iter();
    let Some(host) = values.next() else {
        return Ok(None);
    };
    if values.next().is_some() {
        return Err(HostRejection::Conflicting);
    }

    host.to_str()
        .ok()
        .and_then(|host| host.parse::<Authority>().ok())
        .map(Some)
        .ok_or(HostRejection::Malformed)
}

/// The rejected request never identified this application, so its response
/// carries no content worth storing anywhere.
fn reject(reason: HostRejection) -> Response {
    SecurityEvent::HostRejected { reason }.record();
    (
        StatusCode::MISDIRECTED_REQUEST,
        [(header::CACHE_CONTROL, HeaderValue::from_static("no-store"))],
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use claims::{assert_err_eq, assert_ok_eq};

    fn uri(value: &str) -> Uri {
        value.parse().expect("the test URI is valid")
    }

    fn headers(hosts: &[&str]) -> HeaderMap {
        let mut headers = HeaderMap::new();
        for host in hosts {
            headers.append(
                header::HOST,
                HeaderValue::from_str(host).expect("the test host is a header value"),
            );
        }
        headers
    }

    fn authority(value: &str) -> Authority {
        value.parse().expect("the test authority is valid")
    }

    #[test]
    fn either_the_target_or_the_host_header_names_the_authority() {
        assert_ok_eq!(
            resolve(&uri("/work/traxor"), &headers(&["kristofers.xyz"])),
            authority("kristofers.xyz")
        );
        assert_ok_eq!(
            resolve(&uri("https://kristofers.xyz/work/traxor"), &headers(&[])),
            authority("kristofers.xyz")
        );
        assert_ok_eq!(
            resolve(
                &uri("https://kristofers.xyz/"),
                &headers(&["KRISTOFERS.XYZ"])
            ),
            authority("kristofers.xyz")
        );
    }

    #[test]
    fn an_unusable_authority_names_its_reason() {
        assert_err_eq!(resolve(&uri("/"), &headers(&[])), HostRejection::Missing);
        assert_err_eq!(
            resolve(&uri("/"), &headers(&["kristofers.xyz/admin"])),
            HostRejection::Malformed
        );
        assert_err_eq!(
            resolve(&uri("/"), &headers(&[""])),
            HostRejection::Malformed
        );
        assert_err_eq!(
            resolve(&uri("/"), &headers(&["kristofers.xyz", "attacker.example"])),
            HostRejection::Conflicting
        );
        assert_err_eq!(
            resolve(
                &uri("https://kristofers.xyz/"),
                &headers(&["attacker.example"])
            ),
            HostRejection::Conflicting
        );
    }
}
