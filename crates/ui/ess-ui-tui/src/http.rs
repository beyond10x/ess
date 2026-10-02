//! The adapter that reads and commands the HTTP surface ESS synthesizes for a served component.
//!
//! Routes come from a [`Binding`] (`ess_ui_check::binding`), never from a derivation of this
//! crate's own: a read is `GET <base><path>` with its parameters under their query keys, a
//! command `POST <base><path>` with its input as a JSON body, and every command answer is read by
//! [`ess_ui::binding::classify`]. The transport is HTTP/1.1 over `std::net`, `http://` only: an
//! `https://` base URL is refused by name. A request that has not finished within [`TIMEOUT`] is
//! given up, so a stalled server cannot hold the terminal.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::io::{Read as _, Write as _};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::{Duration, Instant};

use ess_ui::binding::{classify, Answer, Binding, CommandRoute, ServedComponent};
use serde_yaml::Value;

use crate::data::{DataAdapter, ReadRequest, ReadResult};
use crate::expr::display;
use crate::TuiError;

/// How long one request may take, connecting included, before it is given up.
pub const TIMEOUT: Duration = Duration::from_secs(15);

/// The largest answer read; a longer one is given up.
const MAX_ANSWER: usize = 16 * 1024 * 1024;

/// Where one served component is reached: `http://<host>[:<port>][<prefix>]`.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Base {
    /// `host:port`, as connected to and sent as `Host`.
    authority: String,
    /// The path every route is appended to, without a trailing `/`.
    prefix: String,
}

impl Base {
    fn parse(url: &str) -> Result<Self, String> {
        if url
            .get(..8)
            .is_some_and(|scheme| scheme.eq_ignore_ascii_case("https://"))
        {
            return Err(format!(
                "{url}: `https://` is refused: the terminal speaks plain HTTP/1.1 and has no TLS; \
                 give an `http://` base URL"
            ));
        }
        let rest = url
            .get(..7)
            .filter(|scheme| scheme.eq_ignore_ascii_case("http://"))
            .map(|_| &url[7..])
            .ok_or_else(|| format!("{url}: a base URL starts with `http://`"))?;
        if rest.contains(['?', '#']) {
            return Err(format!("{url}: a base URL carries no query or fragment"));
        }
        if rest.chars().any(|c| c.is_whitespace() || c.is_control()) {
            return Err(format!(
                "{url}: a base URL holds no space or control character"
            ));
        }
        let (authority, prefix) = rest.split_at(rest.find('/').unwrap_or(rest.len()));
        if authority.is_empty() || authority.contains('@') {
            return Err(format!("{url}: a base URL names a host (and no user)"));
        }
        Ok(Self {
            authority: authority_of(authority).map_err(|why| format!("{url}: {why}"))?,
            prefix: prefix.trim_end_matches('/').to_owned(),
        })
    }
}

/// `host[:port]` or `[v6][:port]` as `host:port` (port 80 when absent), or why it cannot be
/// connected to: no host, an unclosed `[`, a bare IPv6 address, or a port that is empty, not a
/// number, or outside 1 to 65535.
fn authority_of(authority: &str) -> Result<String, String> {
    let (host, port) = if let Some(inner) = authority.strip_prefix('[') {
        let (address, after) = inner
            .split_once(']')
            .ok_or("an IPv6 host opened with `[` is closed with `]`")?;
        if address.is_empty() {
            return Err("an IPv6 host names an address".to_owned());
        }
        match after {
            "" => (&authority[..address.len() + 2], None),
            _ => (
                &authority[..address.len() + 2],
                Some(
                    after
                        .strip_prefix(':')
                        .ok_or("an IPv6 host is followed by `:<port>`")?,
                ),
            ),
        }
    } else {
        match authority.split_once(':') {
            None => (authority, None),
            Some((_, port)) if port.contains(':') => {
                return Err("an IPv6 host is written in brackets, `[::1]:8080`".to_owned())
            }
            Some((host, port)) => (host, Some(port)),
        }
    };
    if host.is_empty() {
        return Err("a base URL names a host".to_owned());
    }
    let port = match port {
        None => 80,
        Some(port) => Some(port)
            .filter(|port| port.bytes().all(|byte| byte.is_ascii_digit()))
            .and_then(|port| port.parse::<u16>().ok())
            .filter(|port| *port != 0)
            .ok_or_else(|| format!("the port `{port}` is not a number from 1 to 65535"))?,
    };
    Ok(format!("{host}:{port}"))
}

/// Reads and commands the served surface a [`Binding`] describes.
#[derive(Debug, Clone)]
pub struct HttpAdapter {
    binding: Binding,
    bases: BTreeMap<String, Base>,
    authorization: Option<String>,
    state: BTreeMap<String, Value>,
}

impl HttpAdapter {
    /// An adapter for `binding`, each served component reached at its base URL in `bases`
    /// (component → `http://…`), every request carrying `authorization` as its `Authorization`
    /// header when given. Refused: a base URL that is not `http://` (`https://` by name), a
    /// component of the binding without a base URL or one the binding does not serve, and an
    /// authorization holding a line break or another control character.
    pub fn new(
        binding: Binding,
        bases: &BTreeMap<String, String>,
        authorization: Option<String>,
    ) -> Result<Self, TuiError> {
        let refused = |message: String| TuiError::Binding(message);
        let mut parsed = BTreeMap::new();
        for (component, url) in bases {
            if !binding.components.contains_key(component) {
                return Err(refused(format!(
                    "--base-url {component}=…: the document binds no served component \
                     `{component}` (it binds {})",
                    names(&binding)
                )));
            }
            let base = Base::parse(url).map_err(refused)?;
            parsed.insert(component.clone(), base);
        }
        if let Some(missing) = binding
            .components
            .keys()
            .find(|component| !parsed.contains_key(*component))
        {
            return Err(refused(format!(
                "no --base-url for `{missing}`: every served component the document binds needs \
                 one (it binds {})",
                names(&binding)
            )));
        }
        if authorization
            .as_deref()
            .is_some_and(|value| value.chars().any(char::is_control))
        {
            return Err(refused(
                "ESS_UI_AUTHORIZATION holds a line break or another control character, which \
                 no header may carry"
                    .to_owned(),
            ));
        }
        Ok(Self {
            binding,
            bases: parsed,
            authorization,
            state: BTreeMap::new(),
        })
    }

    /// The served component a view or command (`kind`) is reached at, and its route there.
    fn served<'a, T>(
        &'a self,
        name: &str,
        routes: impl Fn(&'a ServedComponent) -> &'a BTreeMap<String, T>,
    ) -> Option<(&'a Base, &'a T)> {
        let qualified = self.binding.names.get(name).map_or(name, String::as_str);
        self.binding
            .components
            .iter()
            .find_map(|(component, served)| {
                routes(served)
                    .get(qualified)
                    .map(|route| (&self.bases[component], route))
            })
    }

    /// Sends `command` with `input` and returns the raw answer: its status and body. `Err` when
    /// no answer came (the command is unbound, or the transport failed or timed out).
    pub fn post(
        &self,
        command: &str,
        input: &BTreeMap<String, Value>,
    ) -> Result<(u16, String), String> {
        let (base, route) = self
            .served(command, |served| &served.commands)
            .ok_or_else(|| format!("no served component accepts {command}"))?;
        let body = serde_json::Value::Object(
            input
                .iter()
                .map(|(name, value)| (name.clone(), to_json(value)))
                .collect(),
        )
        .to_string();
        self.exchange(
            base,
            "POST",
            &format!("{}{}", base.prefix, route.path),
            Some(&body),
        )
    }

    /// One request and its answer.
    fn exchange(
        &self,
        base: &Base,
        method: &str,
        target: &str,
        body: Option<&str>,
    ) -> Result<(u16, String), String> {
        let deadline = Instant::now() + TIMEOUT;
        let mut stream = connect(&base.authority, deadline)?;
        let mut request = format!(
            "{method} {target} HTTP/1.1\r\nHost: {}\r\nAccept: application/json\r\nConnection: close\r\n",
            base.authority
        );
        if let Some(authorization) = &self.authorization {
            let _ = write!(request, "Authorization: {authorization}\r\n");
        }
        if let Some(body) = body {
            let _ = write!(
                request,
                "Content-Type: application/json\r\nContent-Length: {}\r\n",
                body.len()
            );
        }
        request.push_str("\r\n");
        request.push_str(body.unwrap_or_default());
        let io = |error: std::io::Error| format!("{}: {error}", base.authority);
        stream
            .set_write_timeout(Some(remaining(deadline)?))
            .map_err(io)?;
        stream.write_all(request.as_bytes()).map_err(io)?;
        let mut answer = Vec::new();
        let mut chunk = [0_u8; 8192];
        loop {
            stream
                .set_read_timeout(Some(remaining(deadline)?))
                .map_err(io)?;
            match stream.read(&mut chunk) {
                Ok(0) => break,
                Ok(read) => {
                    answer.extend_from_slice(&chunk[..read]);
                    // Whole by its `Content-Length` or its last chunk: done, whether or not the
                    // server closes the connection.
                    if let Some(whole) = parse_answer(&answer, false)
                        .map_err(|error| format!("{}: {error}", base.authority))?
                    {
                        return Ok(whole);
                    }
                }
                Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(error)
                    if matches!(
                        error.kind(),
                        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                    ) =>
                {
                    return Err(format!(
                        "{}: no answer within {} s",
                        base.authority,
                        TIMEOUT.as_secs()
                    ));
                }
                Err(error) => return Err(io(error)),
            }
            if answer.len() > MAX_ANSWER {
                return Err(format!(
                    "{}: the answer is longer than {MAX_ANSWER} bytes",
                    base.authority
                ));
            }
        }
        parse_answer(&answer, true)
            .and_then(|whole| whole.ok_or_else(|| "the answer is cut short".to_owned()))
            .map_err(|error| format!("{}: {error}", base.authority))
    }
}

impl DataAdapter for HttpAdapter {
    fn read(&self, request: &ReadRequest) -> Result<ReadResult, String> {
        let (base, route) = self
            .served(&request.view, |served| &served.views)
            .ok_or_else(|| format!("no served component answers {}", request.view))?;
        let mut query = Vec::new();
        for param in &route.params {
            let value = match request.params.get(&param.name) {
                None | Some(Value::Null) => continue,
                Some(Value::String(text)) if text.is_empty() => continue,
                Some(value @ (Value::Sequence(_) | Value::Mapping(_))) => {
                    to_json(value).to_string()
                }
                Some(value) => display(value),
            };
            query.push(format!("{}={}", encode(&param.wire), encode(&value)));
        }
        let mut target = format!("{}{}", base.prefix, route.path);
        if !query.is_empty() {
            target.push('?');
            target.push_str(&query.join("&"));
        }
        let (status, body) = self.exchange(base, "GET", &target, None)?;
        if !(200..300).contains(&status) {
            let refused = serde_json::from_str::<serde_json::Value>(&body)
                .ok()
                .and_then(|body| {
                    body.get("refused")
                        .and_then(|r| r.as_str().map(str::to_owned))
                });
            return Err(match refused {
                Some(refused) => format!("{status} {}: {refused}", request.view),
                None => format!("{status} {}", request.view),
            });
        }
        let body: serde_json::Value = serde_json::from_str(&body)
            .map_err(|error| format!("{}: the answer is not JSON: {error}", request.view))?;
        Ok(match body {
            serde_json::Value::Object(ref members)
                if members.get("rows").is_some_and(serde_json::Value::is_array) =>
            {
                let rows = members["rows"]
                    .as_array()
                    .map(|rows| rows.iter().map(to_yaml).collect())
                    .unwrap_or_default();
                ReadResult {
                    rows,
                    total: members.get("total").and_then(serde_json::Value::as_u64),
                }
            }
            serde_json::Value::Array(rows) => ReadResult {
                rows: rows.iter().map(to_yaml).collect(),
                total: None,
            },
            other => ReadResult {
                rows: vec![to_yaml(&other)],
                total: None,
            },
        })
    }

    fn run(&mut self, command: &str, input: &BTreeMap<String, Value>) -> Answer {
        match self.post(command, input) {
            Ok((status, body)) => classify(status, &body),
            Err(_) => Answer::Transport,
        }
    }

    // `ess_ui_check::binding` refuses state a document places on the server, so the served
    // surface is never asked for any; what reaches here stays in this process.
    fn load_state(&self, path: &str) -> Option<Value> {
        self.state.get(path).cloned()
    }

    fn store_state(&mut self, path: &str, value: Value) {
        self.state.insert(path.to_owned(), value);
    }
}

/// What an answer that is not [`Answer::Accepted`] means to the user who sent `command`: a
/// declared error's display text from the binding, else what the surface's own refusals mean.
pub fn shown(binding: Option<&Binding>, command: &str, answer: &Answer) -> String {
    match answer {
        Answer::Accepted => format!("{command} accepted"),
        Answer::Refused { error, .. } => binding
            .and_then(|binding| route(binding, command))
            .and_then(|route| route.errors.get(error))
            .map_or_else(|| error.clone(), |error| error.display.clone()),
        Answer::NotGranted { actor: None } => {
            "Not signed in as anyone this may be done by".to_owned()
        }
        Answer::NotGranted { actor: Some(actor) } => format!("Not permitted for {actor}"),
        Answer::Malformed { refused } => format!("Not accepted: {refused}"),
        Answer::Unfinished { committed: true } => {
            "Done, but not every effect was delivered: do not send it again".to_owned()
        }
        Answer::Unfinished { committed: false } => "Not available yet: nothing was done".to_owned(),
        Answer::Transport => "No answer from the server".to_owned(),
    }
}

/// The status `command`'s declared `error` answers with, from the binding.
pub(crate) fn error_status(binding: &Binding, command: &str, error: &str) -> Option<u16> {
    route(binding, command)
        .and_then(|route| route.errors.get(error))
        .map(|error| error.status)
}

/// The route `command` (as the document writes it) is sent on.
fn route<'a>(binding: &'a Binding, command: &str) -> Option<&'a CommandRoute> {
    let qualified = binding.names.get(command).map_or(command, String::as_str);
    binding
        .components
        .values()
        .find_map(|served| served.commands.get(qualified))
}

/// The base URL each served component of `binding` is reached at, from `--base-url` values: one
/// `<url>` when the document binds one component, else `<component>=<url>` once per component.
pub fn base_urls(
    binding: &Binding,
    given: &[String],
) -> Result<BTreeMap<String, String>, TuiError> {
    let refused = |message: String| TuiError::Binding(message);
    let mut bases = BTreeMap::new();
    for value in given {
        let named = value
            .split_once('=')
            .filter(|(component, _)| !component.contains("://") && !component.is_empty());
        let (component, url) = if let Some((component, url)) = named {
            (component.to_owned(), url.to_owned())
        } else {
            let only = binding
                .components
                .keys()
                .next()
                .filter(|_| binding.components.len() == 1 && given.len() == 1);
            let Some(only) = only else {
                return Err(refused(format!(
                    "--base-url {value}: the document binds {} served component(s) ({}), and an \
                     unnamed base URL is taken only as the one of one: name each as \
                     --base-url <component>=<url>",
                    binding.components.len(),
                    names(binding)
                )));
            };
            (only.clone(), value.clone())
        };
        if bases.insert(component.clone(), url).is_some() {
            return Err(refused(format!("--base-url names `{component}` twice")));
        }
    }
    Ok(bases)
}

fn names(binding: &Binding) -> String {
    if binding.components.is_empty() {
        return "none".to_owned();
    }
    binding
        .components
        .keys()
        .map(|component| format!("`{component}`"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn remaining(deadline: Instant) -> Result<Duration, String> {
    deadline
        .checked_duration_since(Instant::now())
        .filter(|left| !left.is_zero())
        .ok_or_else(|| format!("no answer within {} s", TIMEOUT.as_secs()))
}

fn connect(authority: &str, deadline: Instant) -> Result<TcpStream, String> {
    let addresses = authority
        .to_socket_addrs()
        .map_err(|error| format!("{authority}: {error}"))?;
    let mut last = format!("{authority}: no address");
    for address in addresses {
        match TcpStream::connect_timeout(&address, remaining(deadline)?) {
            Ok(stream) => return Ok(stream),
            Err(error) => last = format!("{authority}: {error}"),
        }
    }
    Err(last)
}

/// The status and body of a whole HTTP/1.1 answer, its body framed by `Content-Length`,
/// `Transfer-Encoding: chunked` or the end of the connection.
///
/// `Ok(None)`: not whole yet. `eof` says the connection has closed, so a body framed by neither
/// is whatever came, and anything still missing is an error. Interim `1xx` answers before the
/// final one are read past (RFC 9110 §15.2); a `101` switches protocols and is refused.
fn parse_answer(mut answer: &[u8], eof: bool) -> Result<Option<(u16, String)>, String> {
    let incomplete = |why: &str| if eof { Err(why.to_owned()) } else { Ok(None) };
    let (status, lines, rest) = loop {
        let Some(end) = answer.windows(4).position(|window| window == b"\r\n\r\n") else {
            return incomplete("the answer has no header end");
        };
        let head = std::str::from_utf8(&answer[..end]).map_err(|_| "the header is not UTF-8")?;
        let mut lines = head.split("\r\n");
        let status = lines
            .next()
            .and_then(|line| line.strip_prefix("HTTP/1."))
            .and_then(|line| line.get(2..5))
            .and_then(|code| code.parse::<u16>().ok())
            .ok_or("the answer has no HTTP/1.x status line")?;
        let rest = &answer[end + 4..];
        match status {
            101 => return Err("the server switched protocols (101)".to_owned()),
            100..=199 => answer = rest,
            _ => break (status, lines, rest),
        }
    };
    let mut length = None;
    let mut chunked = false;
    for line in lines {
        let Some((name, value)) = line.split_once(':') else {
            continue;
        };
        let (name, value) = (name.trim(), value.trim());
        if name.eq_ignore_ascii_case("content-length") {
            length = Some(value.parse::<usize>().map_err(|_| "a bad Content-Length")?);
        } else if name.eq_ignore_ascii_case("transfer-encoding") {
            chunked = value.to_ascii_lowercase().contains("chunked");
        }
    }
    let body = if chunked {
        match unchunk(rest)? {
            Some(body) => body,
            None => return incomplete("a chunk is cut short"),
        }
    } else if let Some(length) = length {
        match rest.get(..length) {
            Some(body) => body.to_vec(),
            None => return incomplete("the body is shorter than its Content-Length"),
        }
    } else if eof {
        rest.to_vec()
    } else {
        return Ok(None);
    };
    let body = String::from_utf8(body).map_err(|_| "the body is not UTF-8")?;
    Ok(Some((status, body)))
}

/// A chunked body, or `None` while its last chunk has not arrived.
fn unchunk(mut rest: &[u8]) -> Result<Option<Vec<u8>>, String> {
    let mut body = Vec::new();
    loop {
        let Some(line_end) = rest.windows(2).position(|window| window == b"\r\n") else {
            return Ok(None);
        };
        let size = std::str::from_utf8(&rest[..line_end])
            .ok()
            .and_then(|line| usize::from_str_radix(line.split(';').next()?.trim(), 16).ok())
            .ok_or("a chunk size is not hexadecimal")?;
        rest = &rest[line_end + 2..];
        if size == 0 {
            return Ok(Some(body));
        }
        let (Some(chunk), Some(next)) = (rest.get(..size), rest.get(size + 2..)) else {
            return Ok(None);
        };
        body.extend_from_slice(chunk);
        rest = next;
    }
}

/// `text` percent-encoded for a query: every byte but the unreserved ones.
fn encode(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for byte in text.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            out.push(char::from(byte));
        } else {
            let _ = write!(out, "%{byte:02X}");
        }
    }
    out
}

/// A YAML value as JSON, numbers kept as numbers whatever `serde_json`'s number features are.
fn to_json(value: &Value) -> serde_json::Value {
    match value {
        Value::Null => serde_json::Value::Null,
        Value::Bool(flag) => serde_json::Value::Bool(*flag),
        Value::Number(number) => {
            if let Some(integer) = number.as_i64() {
                integer.into()
            } else if let Some(integer) = number.as_u64() {
                integer.into()
            } else {
                number
                    .as_f64()
                    .and_then(serde_json::Number::from_f64)
                    .map_or(serde_json::Value::Null, serde_json::Value::Number)
            }
        }
        Value::String(text) => serde_json::Value::String(text.clone()),
        Value::Sequence(items) => serde_json::Value::Array(items.iter().map(to_json).collect()),
        Value::Mapping(entries) => serde_json::Value::Object(
            entries
                .iter()
                .map(|(key, value)| (display(key), to_json(value)))
                .collect(),
        ),
        Value::Tagged(tagged) => to_json(&tagged.value),
    }
}

/// A JSON value as YAML, numbers kept as numbers whatever `serde_json`'s number features are.
pub(crate) fn to_yaml(value: &serde_json::Value) -> Value {
    match value {
        serde_json::Value::Null => Value::Null,
        serde_json::Value::Bool(flag) => Value::Bool(*flag),
        serde_json::Value::Number(number) => {
            if let Some(integer) = number.as_i64() {
                Value::Number(integer.into())
            } else if let Some(integer) = number.as_u64() {
                Value::Number(integer.into())
            } else {
                number.as_f64().map_or_else(
                    || Value::String(number.to_string()),
                    |float| Value::Number(float.into()),
                )
            }
        }
        serde_json::Value::String(text) => Value::String(text.clone()),
        serde_json::Value::Array(items) => Value::Sequence(items.iter().map(to_yaml).collect()),
        serde_json::Value::Object(members) => Value::Mapping(
            members
                .iter()
                .map(|(name, value)| (Value::String(name.clone()), to_yaml(value)))
                .collect(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_base_url_is_plain_http_and_names_its_host() {
        assert_eq!(
            Base::parse("http://127.0.0.1:8080/api/"),
            Ok(Base {
                authority: "127.0.0.1:8080".into(),
                prefix: "/api".into()
            })
        );
        assert_eq!(
            Base::parse("HTTP://desk").map(|base| base.authority),
            Ok("desk:80".into())
        );
        assert!(Base::parse("https://desk").is_err_and(|error| error.contains("https://")));
        assert!(Base::parse("HTTPS://desk").is_err_and(|error| error.contains("https://")));
        for refused in [
            "ftp://desk",
            "desk:80",
            "http://",
            "http://u@desk",
            "http://desk/?a=1",
            "http://desk:",
            "http://desk:0",
            "http://desk:+80",
            "http://desk:65536",
            "http://[::1",
            "http://[]:80",
            "http://::1:80",
            "http://[::1]80",
        ] {
            assert!(Base::parse(refused).is_err(), "{refused}");
        }
        assert_eq!(
            Base::parse("http://[::1]:8080").map(|base| base.authority),
            Ok("[::1]:8080".into())
        );
        assert_eq!(
            Base::parse("http://[::1]").map(|base| base.authority),
            Ok("[::1]:80".into())
        );
    }

    #[test]
    fn an_answer_is_framed_by_its_length_or_its_chunks() {
        let plain = b"HTTP/1.1 422 Unprocessable\r\nContent-Length: 2\r\n\r\n{}trailing";
        let whole = Ok(Some((422, "{}".to_owned())));
        assert_eq!(parse_answer(plain, false), whole, "whole before the close");
        assert_eq!(parse_answer(plain, true), whole);
        let chunked =
            b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n3\r\n[1,\r\n2\r\n2]\r\n0\r\n\r\n";
        assert_eq!(
            parse_answer(chunked, false),
            Ok(Some((200, "[1,2]".to_owned())))
        );
        // Not whole yet: more is read; whole only once the connection closes.
        let cut = &chunked[..chunked.len() - 7];
        assert_eq!(parse_answer(cut, false), Ok(None));
        assert!(parse_answer(cut, true).is_err());
        let unframed = b"HTTP/1.1 200 OK\r\n\r\n[]";
        assert_eq!(parse_answer(unframed, false), Ok(None));
        assert_eq!(
            parse_answer(unframed, true),
            Ok(Some((200, "[]".to_owned())))
        );
        // Interim answers are read past; a protocol switch is not an answer.
        let interim = b"HTTP/1.1 100 Continue\r\n\r\nHTTP/1.1 103 Early Hints\r\nLink: x\r\n\r\n\
                        HTTP/1.1 202 Accepted\r\nContent-Length: 2\r\n\r\n{}";
        assert_eq!(
            parse_answer(interim, false),
            Ok(Some((202, "{}".to_owned())))
        );
        assert!(parse_answer(b"HTTP/1.1 101 Switching\r\n\r\n", true).is_err());
        assert!(parse_answer(b"garbage", true).is_err());
        assert_eq!(parse_answer(b"garbage", false), Ok(None));
    }

    #[test]
    fn a_query_value_is_percent_encoded() {
        assert_eq!(encode("a b&c=d/é"), "a%20b%26c%3Dd%2F%C3%A9");
    }
}
