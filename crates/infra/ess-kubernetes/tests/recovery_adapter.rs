//! The finite recovery adapter against a real loopback TLS server with its own certificate authority.
//!
//! This is the one place the *production* transport is exercised rather than replaced by an
//! already-decoded snapshot. What it establishes is narrow and worth stating: the trusted bundle is
//! the only root, the server name is checked, the request set is finite and the response is
//! bounded. It establishes nothing about a real cluster — a synthetic namespace UID answered over
//! loopback is a synthetic namespace UID, and a successful local handshake is not the caller's
//! authority.

use std::io::{Read as _, Write as _};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;

use ess_kubernetes::recovery::{
    decode, sanitize, RecoveryClient, RecoveryError, Request, RESPONSE_LIMIT,
};

struct Authority {
    certificate: rcgen::Certificate,
    key: rcgen::KeyPair,
}

fn authority(name: &str) -> Authority {
    let key = rcgen::KeyPair::generate().expect("a fixture key generates");
    let mut params = rcgen::CertificateParams::new(Vec::new()).expect("CA parameters");
    params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
    params
        .distinguished_name
        .push(rcgen::DnType::CommonName, name);
    let certificate = params.self_signed(&key).expect("a fixture CA self-signs");
    Authority { certificate, key }
}

fn issued(authority: &Authority, names: &[&str]) -> (String, String) {
    let key = rcgen::KeyPair::generate().expect("a fixture key generates");
    let params = rcgen::CertificateParams::new(
        names
            .iter()
            .map(|name| (*name).to_owned())
            .collect::<Vec<String>>(),
    )
    .expect("leaf parameters");
    let certificate = params
        .signed_by(&key, &authority.certificate, &authority.key)
        .expect("the fixture CA signs");
    (certificate.pem(), key.serialize_pem())
}

/// A loopback TLS server that answers exactly one request with exactly these bytes.
struct Server {
    port: u16,
    handle: Option<std::thread::JoinHandle<Option<String>>>,
}

impl Server {
    fn start(certificate_pem: &str, key_pem: &str, response: Vec<u8>) -> Self {
        let certificates: Vec<_> = rustls_pemfile::certs(&mut certificate_pem.as_bytes())
            .collect::<Result<_, _>>()
            .expect("the fixture certificate parses");
        let key = rustls_pemfile::private_key(&mut key_pem.as_bytes())
            .expect("the fixture key parses")
            .expect("the fixture key is present");
        let provider = Arc::new(rustls::crypto::ring::default_provider());
        let config = rustls::ServerConfig::builder_with_provider(provider)
            .with_safe_default_protocol_versions()
            .expect("the fixture server admits the default versions")
            .with_no_client_auth()
            .with_single_cert(certificates, key)
            .expect("the fixture server binds its certificate");
        let listener = TcpListener::bind(("127.0.0.1", 0)).expect("the fixture server binds");
        let port = listener.local_addr().expect("a bound address").port();
        let config = Arc::new(config);
        let handle = std::thread::spawn(move || {
            let (mut socket, _) = listener.accept().ok()?;
            let mut connection = rustls::ServerConnection::new(config).ok()?;
            let mut stream = rustls::Stream::new(&mut connection, &mut socket);
            let mut request = Vec::new();
            let mut chunk = [0u8; 4096];
            while let Ok(read) = stream.read(&mut chunk) {
                if read == 0 {
                    break;
                }
                request.extend_from_slice(&chunk[..read]);
                if request.windows(4).any(|window| window == b"\r\n\r\n") {
                    break;
                }
            }
            let _ = stream.write_all(&response);
            let _ = stream.flush();
            String::from_utf8(request).ok()
        });
        Self {
            port,
            handle: Some(handle),
        }
    }

    fn endpoint(&self) -> String {
        format!("https://localhost:{}", self.port)
    }

    fn request(mut self) -> Option<String> {
        self.handle
            .take()
            .and_then(|handle| handle.join().ok())
            .flatten()
    }
}

fn body(status: u16, body: &str) -> Vec<u8> {
    format!(
        "HTTP/1.1 {status} X\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
    .into_bytes()
}

/// A bounded request over the production transport returns a decoded, sanitized body.
///
/// The credential goes into the `Authorization` header and nowhere else: the server records the
/// request it received, and the test reads it back to check that the token is on the wire and
/// absent from every value the adapter hands back.
#[test]
fn a_bounded_request_over_the_production_transport_returns_a_decoded_body() {
    let ca = authority("fixture-ca");
    let (certificate, key) = issued(&ca, &["localhost"]);
    let server = Server::start(
        &certificate,
        &key,
        body(
            200,
            r#"{"metadata":{"name":"kube-system","uid":"ns-1"},"data":{"token":"SECRET-VALUE"}}"#,
        ),
    );
    let endpoint = server.endpoint();
    let client =
        RecoveryClient::connect(&endpoint, ca.certificate.pem().as_bytes(), "SENTINEL-TOKEN")
            .expect("the pinned trust anchor admits");
    let response = client
        .request(&Request::Namespace {
            name: "kube-system".to_owned(),
        })
        .expect("the bounded request completes");
    assert_eq!(response.status, 200);
    assert_eq!(response.body["metadata"]["uid"], "ns-1");
    assert_eq!(
        response.body["data"], "[redacted]",
        "Secret-shaped values never leave this boundary"
    );
    assert!(!serde_json::to_string(&response.body)
        .unwrap()
        .contains("SECRET-VALUE"));
    assert!(
        !format!("{client:?}").contains("SENTINEL-TOKEN"),
        "the bearer token has no accessor and is not in the debug rendering"
    );

    let received = server.request().expect("the server saw the request");
    assert!(received.starts_with("GET /api/v1/namespaces/kube-system HTTP/1.1\r\n"));
    assert!(received.contains("Authorization: Bearer SENTINEL-TOKEN"));
    assert!(received.contains("Connection: close"));
}

/// A certificate outside the pinned trust anchor fails the handshake.
#[test]
fn a_certificate_outside_the_pinned_trust_anchor_fails_the_handshake() {
    let pinned = authority("pinned-ca");
    let other = authority("another-ca");
    let (certificate, key) = issued(&other, &["localhost"]);
    let server = Server::start(&certificate, &key, body(200, "{}"));
    let client = RecoveryClient::connect(
        &server.endpoint(),
        pinned.certificate.pem().as_bytes(),
        "token",
    )
    .expect("the pinned bundle parses");
    let error = client
        .request(&Request::SelfSubject)
        .expect_err("a chain outside the pinned bundle must fail");
    // Not merely "a transport failure": a closed port is one of those too, and this case would
    // pass on one without ever reaching a handshake. What it decides is that the *certificate*
    // was rejected.
    let RecoveryError::Transport(detail) = &error else {
        panic!("expected a transport failure, got {error}");
    };
    assert!(
        detail.to_lowercase().contains("certificate")
            || detail.to_lowercase().contains("unknown issuer")
            || detail.to_lowercase().contains("invalid peer"),
        "the handshake must fail on the certificate, and failed with: {detail}"
    );
}

/// A certificate for another host fails the handshake.
#[test]
fn a_certificate_for_another_host_fails_the_handshake() {
    let ca = authority("fixture-ca");
    let (certificate, key) = issued(&ca, &["elsewhere.invalid"]);
    let server = Server::start(&certificate, &key, body(200, "{}"));
    let client =
        RecoveryClient::connect(&server.endpoint(), ca.certificate.pem().as_bytes(), "token")
            .expect("the pinned bundle parses");
    let error = client
        .request(&Request::SelfSubject)
        .expect_err("a certificate for another name must fail");
    let RecoveryError::Transport(detail) = &error else {
        panic!("expected a transport failure, got {error}");
    };
    assert!(
        detail.to_lowercase().contains("certificate")
            || detail.to_lowercase().contains("name")
            || detail.to_lowercase().contains("invalid peer"),
        "the handshake must fail on the server name, and failed with: {detail}"
    );
}

/// An empty or unparseable trust bundle is refused before a socket is opened.
#[test]
fn an_empty_or_unparseable_trust_bundle_refuses_before_any_socket() {
    for bundle in [
        &b""[..],
        b"not a certificate",
        b"-----BEGIN CERTIFICATE-----\nxx\n",
    ] {
        let error = RecoveryClient::connect("https://localhost:1", bundle, "token")
            .expect_err("an unusable bundle refuses");
        assert!(
            matches!(error, RecoveryError::TrustAnchor(_)),
            "got {error}"
        );
    }
}

/// Only a canonical HTTPS host root is admitted as an endpoint.
#[test]
fn only_a_canonical_https_host_root_is_an_admitted_endpoint() {
    let ca = authority("fixture-ca");
    let bundle = ca.certificate.pem();
    for rejected in [
        "http://localhost",
        "https://user@localhost",
        "https://localhost/apis",
        "https://localhost?a=b",
        "https://localhost#f",
        "https://localhost:not-a-port",
        "localhost",
    ] {
        let error = RecoveryClient::connect(rejected, bundle.as_bytes(), "token")
            .expect_err("{rejected} must refuse");
        assert!(
            matches!(error, RecoveryError::Endpoint(_)),
            "{rejected}: got {error}"
        );
    }
    RecoveryClient::connect("https://localhost:6443", bundle.as_bytes(), "token")
        .expect("a canonical root admits");
}

/// The request set is finite, and no caller can widen it into an arbitrary path.
#[test]
fn the_request_set_is_finite_and_cannot_be_widened() {
    let ca = authority("fixture-ca");
    let (certificate, key) = issued(&ca, &["localhost"]);
    let server = Server::start(&certificate, &key, body(200, "{}"));
    let client =
        RecoveryClient::connect(&server.endpoint(), ca.certificate.pem().as_bytes(), "token")
            .expect("the bundle parses");
    for crafted in [
        "../../secrets",
        "kube-system/../../apis",
        "Kube-System",
        "ns name",
        "",
        "a?b=c",
    ] {
        let error = client
            .request(&Request::Namespace {
                name: crafted.to_owned(),
            })
            .expect_err("a crafted segment must refuse");
        assert!(
            matches!(error, RecoveryError::Endpoint(_)),
            "{crafted:?}: got {error}"
        );
    }
}

/// A redirect is refused rather than followed, and an error status is refused.
#[test]
fn a_redirect_is_refused_rather_than_followed_and_an_error_status_refuses() {
    for status in [301u16, 302, 307, 308] {
        let raw = format!(
            "HTTP/1.1 {status} Moved\r\nLocation: https://elsewhere.invalid/\r\nContent-Length: 0\r\n\r\n"
        );
        let error = decode(raw.as_bytes()).expect_err("a redirect refuses");
        assert_eq!(error, RecoveryError::Status(status));
    }
    for status in [401u16, 403, 500, 503] {
        let error = decode(body(status, "{}").as_slice()).expect_err("an error status refuses");
        assert_eq!(error, RecoveryError::Status(status));
    }
    let absent = decode(body(404, "").as_slice()).expect("404 is an authoritative absence");
    assert_eq!(absent.status, 404);
    assert!(absent.body.is_null());
}

/// A malformed, truncated or oversized response refuses.
#[test]
fn a_malformed_truncated_or_oversized_response_refuses() {
    assert!(matches!(
        decode(b"no header terminator here"),
        Err(RecoveryError::Response(_))
    ));
    assert!(matches!(
        decode(b"HTTP/9.9 200 X\r\n\r\n{}"),
        Err(RecoveryError::Response(_))
    ));
    assert!(matches!(
        decode(b"HTTP/1.1 not-a-code X\r\n\r\n{}"),
        Err(RecoveryError::Response(_))
    ));
    assert!(matches!(
        decode(body(200, "{not json").as_slice()),
        Err(RecoveryError::Decode(_))
    ));
    assert!(matches!(
        decode(body(200, r#"{"a":1,"a":2}"#).as_slice()),
        Ok(_) | Err(RecoveryError::Decode(_))
    ));
    let mut oversized = b"HTTP/1.1 200 X\r\n\r\n".to_vec();
    oversized.extend(std::iter::repeat_n(b'x', RESPONSE_LIMIT + 1));
    assert!(matches!(
        decode(&oversized),
        Err(RecoveryError::Response(_))
    ));
}

/// The redaction is verified by mutation: every Secret-shaped value is replaced, at every depth.
#[test]
fn the_redaction_is_verified_by_mutation_at_every_depth() {
    const SENTINEL: &str = "SYNTHETIC-SECRET-SENTINEL";
    let value = serde_json::json!({
        "data": {"token": SENTINEL},
        "stringData": {"token": SENTINEL},
        "metadata": {
            "annotations": {"kubectl.kubernetes.io/last-applied-configuration": SENTINEL},
            "name": "web"
        },
        "items": [{"data": {"nested": SENTINEL}}, {"spec": {"keep": "visible"}}]
    });
    let sanitized = sanitize(value);
    let text = serde_json::to_string(&sanitized).expect("the sanitized value serializes");
    assert!(
        !text.contains(SENTINEL),
        "a Secret-shaped value reached a serialized boundary: {text}"
    );
    assert_eq!(sanitized["metadata"]["name"], "web", "metadata survives");
    assert_eq!(
        sanitized["items"][1]["spec"]["keep"], "visible",
        "only the Secret-shaped keys are replaced"
    );
    assert_eq!(sanitized["items"][0]["data"], "[redacted]");
}

/// A response body larger than the bound is refused over the real transport too.
#[test]
fn an_oversized_body_is_refused_over_the_real_transport() {
    let ca = authority("fixture-ca");
    let (certificate, key) = issued(&ca, &["localhost"]);
    let mut response = b"HTTP/1.1 200 X\r\nConnection: close\r\n\r\n".to_vec();
    response.extend(std::iter::repeat_n(b'x', RESPONSE_LIMIT + 4096));
    let server = Server::start(&certificate, &key, response);
    let client =
        RecoveryClient::connect(&server.endpoint(), ca.certificate.pem().as_bytes(), "token")
            .expect("the bundle parses");
    let error = client
        .request(&Request::SelfSubject)
        .expect_err("an unbounded body must refuse");
    assert!(
        matches!(
            error,
            RecoveryError::Response(_) | RecoveryError::Transport(_)
        ),
        "got {error}"
    );
}

/// A `TcpStream` to a closed port is a transport failure, not an absence.
#[test]
fn an_unreachable_endpoint_is_a_transport_failure_not_an_absence() {
    let ca = authority("fixture-ca");
    let listener = TcpListener::bind(("127.0.0.1", 0)).expect("a port is reserved");
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    let client = RecoveryClient::connect(
        &format!("https://localhost:{port}"),
        ca.certificate.pem().as_bytes(),
        "token",
    )
    .expect("the bundle parses");
    let error = client
        .request(&Request::SelfSubject)
        .expect_err("a closed port must fail");
    assert!(matches!(error, RecoveryError::Transport(_)), "got {error}");
    let _ = TcpStream::connect(("127.0.0.1", port));
}
