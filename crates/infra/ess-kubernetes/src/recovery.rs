//! Credential and TLS boundary for the finite recovery API requests.
//!
//! This is the only place in the recovery path that holds a credential or opens a socket, and it
//! returns bounded sanitized data rather than a transport object. Nothing downstream of it selects
//! kubeconfig authority, follows a redirect, resolves a proxy or sees a bearer token: the engine
//! asks for a namespace identity, a self-subject identity or one direct object address, and gets a
//! value it can compare.
//!
//! It deliberately builds no shared `InfraIr`/`EssIr` envelope. The topology scanner next door
//! answers a different question — a reduced, deliberately partial `infra-observation/2` profile
//! over a whole namespace — and its output is not recovery observation authority.
//!
//! Three properties are the whole security surface here, and each has a test in
//! `crates/infra/ess-kubernetes/tests/recovery_adapter.rs`:
//!
//! * the trusted CA bundle is the only root, so a certificate outside it fails the handshake;
//! * the server name is checked, so a certificate for another host fails the handshake;
//! * the request set is finite and the response is bounded, so neither a redirect nor an
//!   unbounded body can move or exhaust the reader.

use std::io::{Read as _, Write as _};
use std::net::TcpStream;
use std::sync::Arc;
use std::time::Duration;

/// The largest admitted response body.
pub const RESPONSE_LIMIT: usize = 4 * 1024 * 1024;
/// The bound on one request's connect, handshake and read.
pub const REQUEST_TIMEOUT: Duration = Duration::from_secs(15);

/// Why a bounded recovery request could not be completed.
///
/// Every variant is deliberately coarse. A transport diagnostic can carry a URL, a header or a
/// response body, and each of those can carry a credential, so what crosses this boundary is the
/// operation label and the kind of failure — never the server's own words.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecoveryError {
    /// The endpoint is not a canonical HTTPS root this adapter admits.
    Endpoint(String),
    /// The trusted CA bundle did not parse, or is empty.
    TrustAnchor(String),
    /// The connection, handshake or certificate verification failed.
    Transport(String),
    /// The response was not a bounded, well-formed HTTP/1.1 response.
    Response(String),
    /// The server answered with a status this adapter does not admit.
    Status(u16),
    /// The response body was not the JSON shape the finite request expects.
    Decode(String),
}

impl std::fmt::Display for RecoveryError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Endpoint(detail) => write!(formatter, "endpoint: {detail}"),
            Self::TrustAnchor(detail) => write!(formatter, "trust anchor: {detail}"),
            Self::Transport(detail) => write!(formatter, "transport: {detail}"),
            Self::Response(detail) => write!(formatter, "response: {detail}"),
            Self::Status(code) => write!(formatter, "unadmitted response status {code}"),
            Self::Decode(detail) => write!(formatter, "decode: {detail}"),
        }
    }
}

impl std::error::Error for RecoveryError {}

/// The finite set of requests this adapter can make.
///
/// A closed enum rather than a path string. A caller cannot ask this adapter for an arbitrary URL,
/// so there is no path through it to a resource the recovery contract does not cover.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Request {
    /// One namespace's identity.
    Namespace {
        /// The namespace name.
        name: String,
    },
    /// The executing credential's self-subject review.
    SelfSubject,
    /// One `apps/v1` workload.
    Workload {
        /// The namespace.
        namespace: String,
        /// `deployments` or `statefulsets`.
        plural: String,
        /// The object name.
        name: String,
    },
    /// One `v1` Service.
    Service {
        /// The namespace.
        namespace: String,
        /// The object name.
        name: String,
    },
}

impl Request {
    /// The exact request line path, built here and nowhere else.
    fn path(&self) -> Result<String, RecoveryError> {
        fn segment(value: &str) -> Result<&str, RecoveryError> {
            if value.is_empty()
                || value.len() > 253
                || !value
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
            {
                return Err(RecoveryError::Endpoint(
                    "a request names a segment outside the admitted grammar".to_owned(),
                ));
            }
            Ok(value)
        }
        Ok(match self {
            Self::Namespace { name } => format!("/api/v1/namespaces/{}", segment(name)?),
            Self::SelfSubject => "/apis/authentication.k8s.io/v1/selfsubjectreviews".to_owned(),
            Self::Workload {
                namespace,
                plural,
                name,
            } => format!(
                "/apis/apps/v1/namespaces/{}/{}/{}",
                segment(namespace)?,
                segment(plural)?,
                segment(name)?
            ),
            Self::Service { namespace, name } => format!(
                "/api/v1/namespaces/{}/services/{}",
                segment(namespace)?,
                segment(name)?
            ),
        })
    }

    /// Whether this request is issued as a POST with an empty typed body.
    fn posts(&self) -> bool {
        matches!(self, Self::SelfSubject)
    }
}

/// One bounded authenticated response: a status and a decoded JSON body.
#[derive(Debug, Clone)]
pub struct Response {
    /// The HTTP status.
    pub status: u16,
    /// The decoded body.
    pub body: serde_json::Value,
}

/// The credential and TLS boundary for one pinned API server.
///
/// The bearer token lives here and is written only into an `Authorization` header. It has no
/// accessor, it is not in the `Debug` rendering, and no error this module produces contains it.
pub struct RecoveryClient {
    host: String,
    port: u16,
    config: Arc<rustls::ClientConfig>,
    token: String,
}

impl std::fmt::Debug for RecoveryClient {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RecoveryClient")
            .field("host", &self.host)
            .field("port", &self.port)
            .finish_non_exhaustive()
    }
}

impl RecoveryClient {
    /// Binds a client to one canonical HTTPS root and exactly one trusted CA bundle.
    ///
    /// The bundle is the *only* root: no platform verifier, no bundled root set and no fallback.
    /// A server whose chain does not reach it fails the handshake, which is the property the
    /// adapter test exercises against a certificate signed by a different CA.
    pub fn connect(api_server: &str, ca_pem: &[u8], token: &str) -> Result<Self, RecoveryError> {
        let rest = api_server.strip_prefix("https://").ok_or_else(|| {
            RecoveryError::Endpoint("the API server is not an https:// root".to_owned())
        })?;
        if rest.contains('/') || rest.contains('@') || rest.contains('?') || rest.contains('#') {
            return Err(RecoveryError::Endpoint(
                "the API server is not a bare host[:port] root".to_owned(),
            ));
        }
        let (host, port) = match rest.rsplit_once(':') {
            Some((host, port)) if !host.is_empty() && !host.contains(':') => (
                host.to_owned(),
                port.parse::<u16>().map_err(|_| {
                    RecoveryError::Endpoint("the API server names no decimal port".to_owned())
                })?,
            ),
            Some(_) | None => (rest.to_owned(), 443),
        };

        let mut roots = rustls::RootCertStore::empty();
        let mut reader = std::io::BufReader::new(ca_pem);
        let mut added = 0usize;
        for certificate in rustls_pemfile::certs(&mut reader) {
            let certificate =
                certificate.map_err(|error| RecoveryError::TrustAnchor(format!("{error}")))?;
            roots
                .add(certificate)
                .map_err(|error| RecoveryError::TrustAnchor(format!("{error}")))?;
            added += 1;
        }
        if added == 0 {
            return Err(RecoveryError::TrustAnchor(
                "the trusted bundle carries no certificate".to_owned(),
            ));
        }

        let provider = Arc::new(rustls::crypto::ring::default_provider());
        let config = rustls::ClientConfig::builder_with_provider(provider)
            .with_safe_default_protocol_versions()
            .map_err(|error| RecoveryError::Transport(format!("{error}")))?
            .with_root_certificates(roots)
            .with_no_client_auth();

        Ok(Self {
            host,
            port,
            config: Arc::new(config),
            token: token.to_owned(),
        })
    }

    /// Issues one finite request and returns its bounded decoded response.
    ///
    /// A redirect is refused rather than followed: `3xx` is an unadmitted status, so a server
    /// cannot move this reader to another endpoint. The body is read under [`RESPONSE_LIMIT`]
    /// before decoding, so a large or endless body cannot exhaust the reader.
    pub fn request(&self, request: &Request) -> Result<Response, RecoveryError> {
        let path = request.path()?;
        let server_name = rustls::pki_types::ServerName::try_from(self.host.clone())
            .map_err(|error| RecoveryError::Transport(format!("{error}")))?;
        let mut connection = rustls::ClientConnection::new(Arc::clone(&self.config), server_name)
            .map_err(|error| RecoveryError::Transport(format!("{error}")))?;
        let mut socket = TcpStream::connect((self.host.as_str(), self.port))
            .map_err(|error| RecoveryError::Transport(format!("{error}")))?;
        socket
            .set_read_timeout(Some(REQUEST_TIMEOUT))
            .and_then(|()| socket.set_write_timeout(Some(REQUEST_TIMEOUT)))
            .map_err(|error| RecoveryError::Transport(format!("{error}")))?;
        let mut stream = rustls::Stream::new(&mut connection, &mut socket);

        let method = if request.posts() { "POST" } else { "GET" };
        let body = if request.posts() {
            r#"{"apiVersion":"authentication.k8s.io/v1","kind":"SelfSubjectReview"}"#
        } else {
            ""
        };
        let head = format!(
            "{method} {path} HTTP/1.1\r\nHost: {}\r\nAuthorization: Bearer {}\r\nAccept: \
             application/json\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\
             Connection: close\r\n\r\n",
            self.host,
            self.token,
            body.len()
        );
        stream
            .write_all(head.as_bytes())
            .and_then(|()| stream.write_all(body.as_bytes()))
            .and_then(|()| stream.flush())
            .map_err(|error| RecoveryError::Transport(format!("{error}")))?;

        let mut raw = Vec::new();
        let mut chunk = [0u8; 8192];
        loop {
            match stream.read(&mut chunk) {
                Ok(0) => break,
                Ok(read) => {
                    if raw.len() + read > RESPONSE_LIMIT {
                        return Err(RecoveryError::Response(
                            "the response is past the admitted bound".to_owned(),
                        ));
                    }
                    raw.extend_from_slice(&chunk[..read]);
                }
                Err(error) if error.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(error) => return Err(RecoveryError::Transport(format!("{error}"))),
            }
        }
        decode(&raw)
    }
}

/// Parses one bounded HTTP/1.1 response into a status and a decoded JSON body.
///
/// Exported so the adapter's own tests can exercise the malformed, truncated and oversized shapes
/// without a socket. A `3xx` never becomes a second request here, because this function has no way
/// to make one.
pub fn decode(raw: &[u8]) -> Result<Response, RecoveryError> {
    let split = raw
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .ok_or_else(|| {
            RecoveryError::Response("the response has no header terminator".to_owned())
        })?;
    let head = std::str::from_utf8(&raw[..split])
        .map_err(|_| RecoveryError::Response("the response head is not UTF-8".to_owned()))?;
    let mut lines = head.lines();
    let status_line = lines
        .next()
        .ok_or_else(|| RecoveryError::Response("the response has no status line".to_owned()))?;
    let mut parts = status_line.split(' ');
    let version = parts.next().unwrap_or_default();
    if version != "HTTP/1.1" && version != "HTTP/1.0" {
        return Err(RecoveryError::Response(
            "the response is not HTTP/1.x".to_owned(),
        ));
    }
    let status: u16 = parts
        .next()
        .and_then(|code| code.parse().ok())
        .ok_or_else(|| RecoveryError::Response("the response has no status code".to_owned()))?;
    if (300..400).contains(&status) {
        return Err(RecoveryError::Status(status));
    }
    let body = &raw[split + 4..];
    if body.len() > RESPONSE_LIMIT {
        return Err(RecoveryError::Response(
            "the response is past the admitted bound".to_owned(),
        ));
    }
    if status == 404 {
        return Ok(Response {
            status,
            body: serde_json::Value::Null,
        });
    }
    if !(200..300).contains(&status) {
        return Err(RecoveryError::Status(status));
    }
    let decoded: serde_json::Value =
        serde_json::from_slice(body).map_err(|error| RecoveryError::Decode(format!("{error}")))?;
    Ok(Response {
        status,
        body: sanitize(decoded),
    })
}

/// Strips every Secret value shape from a decoded response before it can be retained.
///
/// The recovery reads cover `Deployment`, `StatefulSet` and `Service` objects and namespace
/// identities, none of which carries Secret material — and that is why this runs anyway. A response
/// whatever the server sent, the redaction is a property of the boundary rather than of the
/// request, and `crates/infra/ess-kubernetes/tests/recovery_adapter.rs` verifies it by mutation.
pub fn sanitize(value: serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(map) => serde_json::Value::Object(
            map.into_iter()
                .map(|(key, nested)| {
                    let redacted = matches!(key.as_str(), "data" | "stringData")
                        || key == "kubectl.kubernetes.io/last-applied-configuration";
                    if redacted {
                        (key, serde_json::Value::String("[redacted]".to_owned()))
                    } else {
                        (key, sanitize(nested))
                    }
                })
                .collect(),
        ),
        serde_json::Value::Array(items) => {
            serde_json::Value::Array(items.into_iter().map(sanitize).collect())
        }
        other => other,
    }
}
