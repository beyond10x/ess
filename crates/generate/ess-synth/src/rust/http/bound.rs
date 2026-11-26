//! The server's half of an `ess-http/1` binding (`ess generate synthesize --http`).
//!
//! Emitted only for a surface the binding routes something of; every other surface, and every
//! surface synthesized without a binding, keeps its bytes. What it adds is stage 2 of
//! `docs/design/http-presentation-binding.md` as a server reads it ("Precedence" and "What the
//! servers write (k5, k6)"):
//!
//! - template dispatch: a route is a template, several methods may share one, and a path every
//!   template misses is `404` while a path some template matches under another method is `405`;
//! - the request the binding declares: a `json` body less the fields the path carries, a `form`
//!   body or a query string, read into the object the input's decoder already reads, so a request
//!   is refused for the same reasons whatever it was spelled as;
//! - the answer the binding declares per outcome — status, body, headers, a challenge, a
//!   redirect — read off the envelope the transport-free `run_*` renders, so `handle` and the
//!   route still share one rendering of every outcome;
//! - the standard refusals the binding overrides, on the routes it names and no other.
//!
//! What is read from the binding is read through `ess_gen::http`, which the Go server reads too.

use std::fmt::Write as _;

use ess_compiler::ir::{ResolvedCommand, ResolvedField, ResolvedView};
use ess_gen::ess_http::{
    self, AnswerSource, Body, Encoding, Header, Param, RedirectMode, RefusalBody, RefusalKey,
    ResponseEncoding,
};
use ess_gen::http::{self, Served};

use super::{generic_list, handler_ident, runner_ident, table, where_clause, Server};
use crate::rust::layout::Layout;
use crate::rust::wire;

/// The `HTTP` module text as a bound surface needs it: an answer carries headers and may carry no body, and a
/// status a binding declares has its reason phrase.
pub(super) fn http_module(base: &str) -> String {
    let mut module = base.to_owned();
    for (from, to) in [
        (
            "    /// The body.\n    pub body: String,\n}",
            "    /// The body; empty, with an empty `content_type`, for an answer that has none.\n    \
             pub body: String,\n    /// Every header the binding declares for this answer, in the \
             order it is written.\n    pub headers: Vec<(String, String)>,\n}",
        ),
        (
            "            body: body.into(),\n        }",
            "            body: body.into(),\n            headers: Vec::new(),\n        }",
        ),
        (
            "    let head = format!(\n        \"HTTP/1.1 {} {}\\r\\nContent-Type: {}\\r\\nContent-Length: \
             {}\\r\\nConnection: close\\r\\n\\r\\n\",\n        answer.status,\n        \
             reason(answer.status),\n        answer.content_type,\n        answer.body.len()\n    \
             );",
            "    // RFC 9110 §8.6 and §15: a `1xx`, `204` or `304` has no content and no \
             `Content-Length`.\n    let bodiless = answer.status < 200 || answer.status == 204 || \
             answer.status == 304;\n    let mut head = format!(\"HTTP/1.1 {} {}\\r\\n\", \
             answer.status, reason(answer.status));\n    if !bodiless && \
             !answer.content_type.is_empty() {\n        head.push_str(&format!(\"Content-Type: \
             {}\\r\\n\", answer.content_type));\n    }\n    if !bodiless {\n        \
             head.push_str(&format!(\"Content-Length: {}\\r\\n\", answer.body.len()));\n    }\n    \
             for (name, value) in &answer.headers {\n        head.push_str(&format!(\"{name}: \
             {value}\\r\\n\"));\n    }\n    head.push_str(\"Connection: close\\r\\n\\r\\n\");",
        ),
        (
            "    stream.write_all(head.as_bytes())?;\n    \
             stream.write_all(answer.body.as_bytes())?;\n",
            "    stream.write_all(head.as_bytes())?;\n    if !bodiless {\n        \
             stream.write_all(answer.body.as_bytes())?;\n    }\n",
        ),
        ("        _ => \"Unknown\",\n", REASONS),
    ] {
        assert!(
            module.contains(from),
            "the HTTP module no longer holds what a bound surface rewrites: {from}"
        );
        module = module.replacen(from, to, 1);
    }
    module
}

/// [`http_module`] of `base`, read through each surface's `body_limit` when `capped`.
pub(super) fn served_http_module(base: &str, capped: bool) -> String {
    let module = http_module(base);
    if capped {
        capped_read(&module)
    } else {
        module
    }
}

/// The `HTTP` module text of a surface whose binding bounds a request (`limits`, or a `too_large`
/// refusal): `read` asks the surface's `body_limit` for the matched route's cap and refusal after
/// the head is parsed and before the body is allocated, and the default refusal is
/// `body_too_large`. Applied after [`http_module`], in capped mode only, so every other surface
/// keeps its bytes.
pub(super) fn capped_read(module: &str) -> String {
    let mut module = module.to_owned();
    for (from, to) in [
        (
            "pub fn read(reader: &mut std::io::BufReader<std::net::TcpStream>) -> Result<Request, \
             Response> {",
            "pub fn read(\n    reader: &mut std::io::BufReader<std::net::TcpStream>,\n    limit: \
             &dyn Fn(&str, &str) -> Option<(usize, fn(usize, usize) -> Response)>,\n) -> \
             Result<Request, Response> {",
        ),
        (
            "    if length > MAX_BODY {\n        return Err(Response::refusal(\n            413,\n            \
             &format!(\"the body is {length} bytes and this surface reads at most \
             {MAX_BODY}\"),\n        ));\n    }\n",
            "    // The matched route's cap and refusal, from the binding; the fixed bound and its \
             answer where the\n    // binding bounds nothing for this request.\n    let (cap, \
             too_large): (usize, fn(usize, usize) -> Response) =\n        limit(&method, \
             &path).unwrap_or((MAX_BODY, body_too_large));\n    if length > cap {\n        return \
             Err(too_large(length, cap));\n    }\n",
        ),
        (
            "/// One line of the request head, read within what is left of [`MAX_HEAD`]",
            "/// The default answer to a body larger than the cap its route reads: `413` with the \
             sizes.\npub fn body_too_large(length: usize, cap: usize) -> Response {\n    \
             Response::refusal(\n        413,\n        &format!(\"the body is {length} bytes and \
             this surface reads at most {cap}\"),\n    )\n}\n\n/// One line of the request head, \
             read within what is left of [`MAX_HEAD`]",
        ),
    ] {
        assert!(
            module.contains(from),
            "the HTTP module no longer holds what a capped read rewrites: {from}"
        );
        module = module.replacen(from, to, 1);
    }
    module
}

/// `body_limit`: for a request's method and path, the cap of the row that answers it and the
/// function that refuses a larger body, or `None` where the fixed bound stands. One block per row
/// in [`http::dispatch_order`], as [`dispatch`] walks them, so a literal row answers before a
/// variable row, and a row that bounds nothing (a document, a derived route, a declared route that
/// reads no body) answers `None` before a later, more general row can capture its request.
pub(super) fn body_limit(out: &mut String, server: &Server<'_>, routes: &[http::Route<'_>]) {
    let ir = server.ir;
    let Some(binding) = server.binding else {
        return;
    };
    out.push_str(
        "\n/// The most body bytes the route a request matches reads, and the refusal a larger one \
         gets (`limits`,\n/// `refusals.too_large`); `None` where the fixed bound of \
         [`http::MAX_BODY`] stands. Asked after the head is\n/// parsed and before the body is \
         read.\nfn body_limit(method: &str, path: &str) -> Option<(usize, fn(usize, usize) -> \
         http::Response)> {\n",
    );
    let rows = table(routes, ir);
    let keys: Vec<(&str, &str)> = rows
        .iter()
        .map(|(method, path, _, _)| (*method, path.as_str()))
        .collect();
    for (method, path, _, _) in http::dispatch_order(&keys)
        .into_iter()
        .map(|index| rows[index].clone())
    {
        let route = routes
            .iter()
            .find(|route| route.path == path && route.method.as_str() == method);
        let declared = route.and_then(|route| route.declared);
        let opening = if declared.is_some() {
            format!("    if http::matched({path:?}, path).is_some() {{\n")
        } else {
            format!("    if path == {path:?} {{\n")
        };
        out.push_str(&opening);
        let _ = writeln!(out, "        if method == {method:?} {{");
        let result = match (route, declared) {
            (Some(route), Some(declared)) if binding.ir().bounds_request(declared) => {
                let cap = binding
                    .ir()
                    .request_bytes(declared)
                    .unwrap_or(ess_http::FIXED_REQUEST_BYTES);
                format!(
                    "Some(({cap}, refusal_too_large{}))",
                    refusal_suffix(server, route)
                )
            }
            _ => "None".to_owned(),
        };
        let _ = writeln!(out, "            return {result};\n        }}\n    }}");
    }
    out.push_str("    None\n}\n");
}

/// The reason phrases a bound surface adds, for every status a binding may declare that the
/// derived surface never answers (RFC 9110 §15), and the arm for everything else.
const REASONS: &str = "        201 => \"Created\",
        203 => \"Non-Authoritative Information\",
        204 => \"No Content\",
        205 => \"Reset Content\",
        206 => \"Partial Content\",
        301 => \"Moved Permanently\",
        302 => \"Found\",
        303 => \"See Other\",
        307 => \"Temporary Redirect\",
        308 => \"Permanent Redirect\",
        401 => \"Unauthorized\",
        402 => \"Payment Required\",
        403 => \"Forbidden\",
        406 => \"Not Acceptable\",
        407 => \"Proxy Authentication Required\",
        408 => \"Request Timeout\",
        410 => \"Gone\",
        412 => \"Precondition Failed\",
        414 => \"URI Too Long\",
        415 => \"Unsupported Media Type\",
        416 => \"Range Not Satisfiable\",
        417 => \"Expectation Failed\",
        421 => \"Misdirected Request\",
        423 => \"Locked\",
        424 => \"Failed Dependency\",
        425 => \"Too Early\",
        426 => \"Upgrade Required\",
        428 => \"Precondition Required\",
        429 => \"Too Many Requests\",
        451 => \"Unavailable For Legal Reasons\",
        500 => \"Internal Server Error\",
        503 => \"Service Unavailable\",
        504 => \"Gateway Timeout\",
        505 => \"HTTP Version Not Supported\",
        506 => \"Variant Also Negotiates\",
        507 => \"Insufficient Storage\",
        508 => \"Loop Detected\",
        511 => \"Network Authentication Required\",
        _ => \"Unknown\",
";

/// What the `http` module of a bound surface adds after the query reader: template matching, the
/// `form` and `query` decoders, and the writers of a declared answer.
pub(super) const BOUND: &str = r#"
// ---- an `ess-http/1` binding, served -------------------------------------------------------------

/// How one flat value — a path segment, a `form` or `query` value — is read as the JSON value its
/// field's decoder reads. The same table `ess_gen::http::Flat` states, so the Rust and Go servers
/// refuse one request alike.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flat {
    /// `String`, or a newtype of it: any text, the empty text included.
    String,
    /// Any other scalar a decoder reads from JSON text. Empty text is malformed.
    Text,
    /// `Integer`: a whole number's spelling is a JSON number, anything else stays text for the
    /// decoder to refuse by name. Empty text is malformed.
    Integer,
    /// `Boolean`: `true` or `false`, anything else stays text. Empty text is malformed.
    Boolean,
}

/// How many values one `form` or `query` key carries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shape {
    /// Exactly one; given twice, the request is malformed (RFC 6749 §3.1).
    One,
    /// At most one; absent, the field is `None`.
    Optional,
    /// Any number: the key repeats once per element.
    List,
}

/// The media type of a `form` request body.
pub const FORM: &str = "application/x-www-form-urlencoded";

impl Response {
    /// An answer without a body: no `Content-Type`, and `Content-Length: 0` except on a `1xx`,
    /// `204` or `304`, which [`write`] gives none.
    pub fn empty(status: u16) -> Self {
        Self::new(status, "", "")
    }
}

/// A path's segments after its leading `/`; none for the root, and `None` for a path without one.
fn split(path: &str) -> Option<Vec<&str>> {
    let rest = path.strip_prefix('/')?;
    Some(if rest.is_empty() {
        Vec::new()
    } else {
        rest.split('/').collect()
    })
}

/// The raw text of each variable segment of `path`, in order, where `path` matches `template`:
/// as many segments, each literal equal, and each variable holding a segment that is not empty.
/// `None` where it does not match.
pub fn matched(template: &str, path: &str) -> Option<Vec<String>> {
    let template = split(template)?;
    let given = split(path)?;
    if template.len() != given.len() {
        return None;
    }
    let mut values = Vec::new();
    for (expected, text) in template.iter().zip(given) {
        if expected.starts_with('{') && expected.ends_with('}') {
            if text.is_empty() {
                return None;
            }
            values.push(text.to_owned());
        } else if *expected != literal_spelling(text) {
            return None;
        }
    }
    Some(values)
}

/// `text` with each `%XX` escape of an unreserved character decoded (RFC 3986 §6.2.2.2), the
/// spelling a segment is compared with a literal in: `%7Earchived` is `~archived`. Every other
/// escape stays as written, and a variable's text is decoded where it is read.
fn literal_spelling(text: &str) -> String {
    let bytes = text.as_bytes();
    let digit = |at: usize| bytes.get(at).and_then(|byte| char::from(*byte).to_digit(16));
    let mut out = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        let escaped = match (bytes[index], digit(index + 1), digit(index + 2)) {
            (b'%', Some(high), Some(low)) => u8::try_from(high * 16 + low)
                .ok()
                .filter(|byte| byte.is_ascii_alphanumeric() || b"-._~".contains(byte)),
            _ => None,
        };
        if let Some(byte) = escaped {
            out.push(byte);
            index += 3;
        } else {
            out.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8(out).unwrap_or_else(|_| text.to_owned())
}

/// `text` with every `%XX` escape decoded; `None` where an escape is not two hexadecimal digits or
/// the bytes are not UTF-8. A `+` stays a `+`: in a path it is itself.
fn percent_decoded(text: &str) -> Option<String> {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            let high = char::from(*bytes.get(index + 1)?).to_digit(16)?;
            let low = char::from(*bytes.get(index + 2)?).to_digit(16)?;
            out.push(u8::try_from(high * 16 + low).ok()?);
            index += 3;
        } else {
            out.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8(out).ok()
}

/// One flat text as the JSON value its field's decoder reads.
///
/// # Errors
///
/// A refusal naming `at` when the text is empty and the field is not a `String`.
pub fn flat_value(text: String, flat: Flat, at: &str) -> Result<crate::json::Value, String> {
    use crate::json::Value;
    Ok(match flat {
        Flat::String => Value::Text(text),
        _ if text.is_empty() => {
            return Err(format!("{at}: expected a value, found the empty text"));
        }
        Flat::Integer if whole(&text) => Value::Number(text),
        Flat::Boolean if text == "true" => Value::Bool(true),
        Flat::Boolean if text == "false" => Value::Bool(false),
        _ => Value::Text(text),
    })
}

/// One path variable's raw segment, percent-decoded, as the JSON value its field's decoder reads.
///
/// # Errors
///
/// A refusal naming `at` when the segment is not percent-encoded UTF-8, or is empty text for a
/// field that is not a `String`.
pub fn path_value(raw: &str, flat: Flat, at: &str) -> Result<crate::json::Value, String> {
    let text = percent_decoded(raw)
        .ok_or_else(|| format!("{at}: expected percent-encoded UTF-8 text, found `{raw}`"))?;
    flat_value(text, flat, at)
}

/// A `form` body or a query string as the object the input's decoder reads: one member per
/// declared `(wire name, flat, shape)` the text names, keyed by the wire name, and a `List` as an
/// array, empty when its key is absent.
///
/// Keys and values are form-decoded (`+` and `%XX`, UTF-8). A key the path carries, no field
/// declares, or that does not decode (every declared name is UTF-8), is unknown: ignored where
/// `ignore`, and malformed otherwise.
///
/// # Errors
///
/// A refusal naming the parameter under `at` for an unknown key under `refuse` (one that does not
/// decode included), a scalar key given twice, a known key's value that is not percent-encoded
/// UTF-8, or an empty value for a field that is not a `String`.
pub fn flat_object(
    text: &str,
    fields: &[(&str, Flat, Shape)],
    carried: &[&str],
    ignore: bool,
    at: &str,
) -> Result<crate::json::Value, String> {
    let mut found: Vec<Vec<String>> = vec![Vec::new(); fields.len()];
    for pair in text.split('&').filter(|pair| !pair.is_empty()) {
        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        // Every declared name is UTF-8, so a key that does not decode names no field: unknown.
        let Some(key) = form_decoded(key) else {
            if ignore {
                continue;
            }
            return Err(format!("{at}: a key is not percent-encoded UTF-8: `{key}`"));
        };
        let place = crate::json::nested(at, &key);
        let index = fields.iter().position(|(wire, _, _)| *wire == key);
        let Some(index) = index.filter(|_| !carried.contains(&key.as_str())) else {
            if ignore {
                continue;
            }
            return Err(if carried.contains(&key.as_str()) {
                format!("{place}: the path carries this field, so it is not a parameter here")
            } else {
                format!("{place}: no field of this request is named so")
            });
        };
        let value = form_decoded(value).ok_or_else(|| {
            format!("{place}: expected percent-encoded UTF-8 text, found `{value}`")
        })?;
        found[index].push(value);
    }
    let mut members = Vec::new();
    for ((wire, flat, shape), values) in fields.iter().zip(found) {
        let place = crate::json::nested(at, wire);
        match shape {
            Shape::List => {
                let items = values
                    .into_iter()
                    .map(|value| flat_value(value, *flat, &place))
                    .collect::<Result<Vec<_>, String>>()?;
                members.push(((*wire).to_owned(), crate::json::Value::Array(items)));
            }
            Shape::One | Shape::Optional => match values.len() {
                0 => {}
                1 => {
                    let value = values.into_iter().next().unwrap_or_default();
                    members.push(((*wire).to_owned(), flat_value(value, *flat, &place)?));
                }
                several => {
                    return Err(format!("{place}: expected one value, found {several}"));
                }
            },
        }
    }
    Ok(crate::json::Value::Object(members))
}

/// Whether the request's `Content-Type` is `expected`, its parameters ignored and its case too.
pub fn content_type_is(request: &Request, expected: &str) -> bool {
    request
        .headers
        .iter()
        .find(|(name, _)| name == "content-type")
        .is_some_and(|(_, value)| {
            value
                .split(';')
                .next()
                .unwrap_or_default()
                .trim()
                .eq_ignore_ascii_case(expected)
        })
}

/// The body as text.
///
/// # Errors
///
/// A refusal when it is not UTF-8.
pub fn body_text(body: &[u8]) -> Result<&str, String> {
    std::str::from_utf8(body).map_err(|error| format!("the body is not UTF-8: {error}"))
}

/// A `json` body as the value the input's decoder reads. Where the body is not `required`, no
/// body, or one of only whitespace, is the input `{}`.
///
/// # Errors
///
/// A refusal when the body is not UTF-8 or not JSON.
pub fn json_body(body: &[u8], required: bool) -> Result<crate::json::Value, String> {
    let text = body_text(body)?;
    if !required && text.trim().is_empty() {
        return Ok(crate::json::Value::Object(Vec::new()));
    }
    crate::json::parse(text).map_err(|error| format!("the body is not JSON: {error}"))
}

/// `value` with the members the path carries added. A path variable's value wins over the request
/// for its field, and the request naming that field is malformed: the path carries it.
///
/// # Errors
///
/// A refusal naming the member under `at` when `value` already holds one the path carries.
pub fn with_path(
    value: crate::json::Value,
    path: Vec<(String, crate::json::Value)>,
    at: &str,
) -> Result<crate::json::Value, String> {
    match value {
        crate::json::Value::Object(mut members) if !path.is_empty() => {
            for (wire, _) in &path {
                if members.iter().any(|(key, _)| key == wire) {
                    return Err(format!(
                        "{}: the path carries this field, so the request does not",
                        crate::json::nested(at, wire)
                    ));
                }
            }
            members.extend(path);
            Ok(crate::json::Value::Object(members))
        }
        // Not an object: the decoder refuses it by name.
        other => Ok(other),
    }
}

/// A scalar of an answer as text: a string as itself, a number as its spelling, a boolean as
/// `true` or `false`; `None` for an absent optional.
pub fn scalar_text(value: Option<&crate::json::Value>) -> Option<String> {
    match value? {
        crate::json::Value::Text(text) | crate::json::Value::Number(text) => Some(text.clone()),
        crate::json::Value::Bool(value) => Some(value.to_string()),
        _ => None,
    }
}

/// Adds the header `name` with `value` to `answer`; an absent value omits it.
///
/// # Errors
///
/// A refusal when the value is not a field value (RFC 9110 §5.5): it holds a control character
/// other than a horizontal tab, or starts or ends with a space or a tab. Such a value is never
/// written, truncated or re-encoded: a header value from input must not inject a header.
pub fn push_header(answer: &mut Response, name: &str, value: Option<String>) -> Result<(), String> {
    let Some(value) = value else {
        return Ok(());
    };
    if let Some(found) = value.chars().find(|c| c.is_control() && *c != '\t') {
        return Err(format!(
            "header `{name}` would hold {found:?}, which is not field-value text"
        ));
    }
    if value.starts_with([' ', '\t']) || value.ends_with([' ', '\t']) {
        return Err(format!(
            "header `{name}` would start or end with a space or a tab, which a reader strips"
        ));
    }
    answer.headers.push((name.to_owned(), value));
    Ok(())
}

/// One RFC 9110 §11.6.1 challenge: `scheme`, then each param present as `name="value"`, in the
/// order given, separated by `, `. Each value is a quoted string whose `"` and `\` are escaped, so
/// no value can leave it; an absent param is omitted.
///
/// # Errors
///
/// A refusal when a value holds a control character other than a horizontal tab, which a quoted
/// string cannot carry.
pub fn challenge(scheme: &str, params: &[(&str, Option<String>)]) -> Result<String, String> {
    let mut out = scheme.to_owned();
    let mut first = true;
    for (name, value) in params {
        let Some(value) = value else {
            continue;
        };
        if let Some(found) = value.chars().find(|c| c.is_control() && *c != '\t') {
            return Err(format!(
                "challenge param `{name}` would hold {found:?}, which a quoted string cannot carry"
            ));
        }
        out.push_str(if first { " " } else { ", " });
        first = false;
        out.push_str(name);
        out.push_str("=\"");
        for c in value.chars() {
            if c == '"' || c == '\\' {
                out.push('\\');
            }
            out.push(c);
        }
        out.push('"');
    }
    Ok(out)
}

/// `text` as `application/x-www-form-urlencoded` writes it: ASCII letters, digits and `*-._` as
/// themselves, a space as `+`, every other byte as `%XX`.
pub fn form_encoded(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for byte in text.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'*' | b'-' | b'.' | b'_' => {
                out.push(char::from(byte));
            }
            b' ' => out.push('+'),
            other => out.push_str(&format!("%{other:02X}")),
        }
    }
    out
}

/// Where a redirect sends the caller: `location`, with each parameter present appended to its
/// query (keeping a query it already has) or written as its fragment, form-encoded, in the order
/// given. An absent parameter is omitted.
///
/// # Errors
///
/// A refusal when `location` is absent or is not an absolute URI without a fragment: a scheme
/// (an ASCII letter, then letters, digits, `+`, `-` and `.`), a `:`, and visible ASCII only.
pub fn redirect(
    location: Option<String>,
    fragment: bool,
    parameters: &[(&str, Option<String>)],
) -> Result<String, String> {
    let location = location.ok_or_else(|| "the redirect has no location".to_owned())?;
    let absolute = location.split_once(':').is_some_and(|(scheme, _)| {
        let mut characters = scheme.chars();
        characters.next().is_some_and(|c| c.is_ascii_alphabetic())
            && characters.all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'))
    });
    if !absolute || location.contains('#') || !location.chars().all(|c| c.is_ascii_graphic()) {
        return Err(format!(
            "the redirect location {location:?} is not an absolute URI without a fragment"
        ));
    }
    let written: Vec<String> = parameters
        .iter()
        .filter_map(|(name, value)| {
            value
                .as_ref()
                .map(|value| format!("{}={}", form_encoded(name), form_encoded(value)))
        })
        .collect();
    if written.is_empty() {
        return Ok(location);
    }
    let joined = written.join("&");
    Ok(if fragment {
        format!("{location}#{joined}")
    } else {
        match location.split_once('?') {
            Some((_, query)) if query.is_empty() || query.ends_with('&') => {
                format!("{location}{joined}")
            }
            Some(_) => format!("{location}&{joined}"),
            None => format!("{location}?{joined}"),
        }
    })
}

/// Appends a JSON value exactly as it was read.
pub fn push_json(out: &mut String, value: &crate::json::Value) {
    use crate::json::Value;
    match value {
        Value::Null => out.push_str("null"),
        Value::Bool(value) => out.push_str(if *value { "true" } else { "false" }),
        Value::Number(spelling) => out.push_str(spelling),
        Value::Text(text) => crate::json::push_text(out, text),
        Value::Array(items) => {
            out.push('[');
            for (index, item) in items.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                push_json(out, item);
            }
            out.push(']');
        }
        Value::Object(members) => {
            out.push('{');
            for (index, (name, item)) in members.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                crate::json::push_text(out, name);
                out.push(':');
                push_json(out, item);
            }
            out.push('}');
        }
    }
}

/// A bare body: the value as JSON, `null` where there is none.
pub fn bare(value: Option<&crate::json::Value>) -> String {
    let mut out = String::new();
    push_json(&mut out, value.unwrap_or(&crate::json::Value::Null));
    out
}

/// `{<member>: <wire name>, …the error's own fields}`: the error object a binding declares.
pub fn error_object(member: &str, wire: &str, payload: Option<&crate::json::Value>) -> String {
    let mut out = String::from("{");
    crate::json::member(&mut out, member);
    crate::json::push_text(&mut out, wire);
    if let Some(crate::json::Value::Object(fields)) = payload {
        for (name, value) in fields {
            crate::json::member(&mut out, name);
            push_json(&mut out, value);
        }
    }
    out.push('}');
    out
}

/// `{<member>: <text>}`: a standard refusal's literal body.
pub fn literal_object(member: &str, text: &str) -> String {
    let mut out = String::from("{");
    crate::json::member(&mut out, member);
    crate::json::push_text(&mut out, text);
    out.push('}');
    out
}

/// A `single` view route's answer, from the `{"rows": [...]}` its shared path renders: the one row,
/// bare, at `200`, or `empty` without a body when the view holds none.
pub fn single(rows: &str, empty: u16) -> Response {
    let read = crate::json::parse(rows).unwrap_or(crate::json::Value::Null);
    match read.member("rows") {
        Some(crate::json::Value::Array(items)) if !items.is_empty() => {
            Response::new(200, JSON, bare(items.first()))
        }
        _ => Response::empty(empty),
    }
}
"#;

/// The readers of a route with `unknown: ignore` (beyond10x/ess#752), which follow [`BOUND`] only
/// in a surface whose binding declares one, so every other bound surface keeps its bytes.
pub(super) const IGNORING: &str = r#"
/// A `json` request read under `unknown: ignore`: `value` less every member no field of `fields`
/// names, at its top level only (a struct inside keeps its own closedness), and less every member
/// the path carries that holds the path's value.
///
/// # Errors
///
/// A refusal naming the member under `at` when a member the path carries holds another value.
pub fn ignoring(
    value: crate::json::Value,
    fields: &[&str],
    path: &[(String, crate::json::Value)],
    at: &str,
) -> Result<crate::json::Value, String> {
    let crate::json::Value::Object(members) = value else {
        // Not an object: the decoder refuses it by name.
        return Ok(value);
    };
    for (wire, carried) in path {
        if let Some((_, member)) = members.iter().find(|(name, _)| name == wire) {
            if !same_value(member, carried) {
                return Err(echoed(at, wire));
            }
        }
    }
    Ok(crate::json::Value::Object(
        members
            .into_iter()
            .filter(|(name, _)| fields.contains(&name.as_str()))
            .collect(),
    ))
}

/// Holds each key of a `form` body or a query string read under `unknown: ignore` that the path
/// carries, `(wire name, flat)`, to the path's value, read as its field reads it.
///
/// # Errors
///
/// A refusal naming the key under `at` when its value does not decode or is not the path's.
pub fn echoed_keys(
    text: &str,
    carried: &[(&str, Flat)],
    path: &[(String, crate::json::Value)],
    at: &str,
) -> Result<(), String> {
    for pair in text.split('&').filter(|pair| !pair.is_empty()) {
        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        let Some(key) = form_decoded(key) else {
            continue;
        };
        let Some(position) = carried.iter().position(|(wire, _)| *wire == key) else {
            continue;
        };
        let place = crate::json::nested(at, &key);
        let text = form_decoded(value).ok_or_else(|| {
            format!("{place}: expected percent-encoded UTF-8 text, found `{value}`")
        })?;
        let read = flat_value(text, carried[position].1, &place)?;
        if !path
            .iter()
            .find(|(wire, _)| *wire == key)
            .is_some_and(|(_, held)| same_value(&read, held))
        {
            return Err(echoed(at, &key));
        }
    }
    Ok(())
}

/// The refusal of a member or key the path carries with another value.
fn echoed(at: &str, wire: &str) -> String {
    format!(
        "{}: the path carries this field with another value",
        crate::json::nested(at, wire)
    )
}

/// Whether a request's `member` holds what the path's `carried` value does: the same JSON value,
/// or two numbers that denote one integer.
fn same_value(member: &crate::json::Value, carried: &crate::json::Value) -> bool {
    use crate::json::Value;
    match (member, carried) {
        (Value::Number(_), Value::Number(_)) if member != carried => matches!(
            (
                crate::json::integer_at(member, "", ""),
                crate::json::integer_at(carried, "", ""),
            ),
            (Ok(one), Ok(other)) if one == other
        ),
        _ => member == carried,
    }
}
"#;

/// What the `http` module of a surface adds where a route takes a `multipart` request
/// (beyond10x/ess#754): the RFC 7578 reader, which reads a text part as a `form` value and a file
/// part's bytes as the base64 text the input's `Bytes` decoder reads.
pub(super) const MULTIPART: &str = r#"
// ---- a `multipart/form-data` request (RFC 7578), served -----------------------------------------

/// The media type of a `multipart` request body.
pub const MULTIPART: &str = "multipart/form-data";

/// What a `multipart` request carries one input field as.
#[derive(Debug, Clone, Copy)]
pub enum Part {
    /// A text part, read as a `form` value is.
    Text(Flat),
    /// A file part: its bytes as sent, its filename and media type read into input fields.
    File {
        /// The input field its filename is read into, `(wire name, required)`.
        filename: Option<(&'static str, bool)>,
        /// The input field its media type is read into, `(wire name, required)`.
        media_type: Option<(&'static str, bool)>,
        /// The media types it may carry; any where empty.
        media_types: &'static [&'static str],
    },
}

/// One part of a `multipart/form-data` body, as sent.
#[derive(Debug, Clone)]
pub struct Sent {
    /// Its `Content-Disposition` `name`.
    pub name: String,
    /// Its `Content-Disposition` `filename`, if it has one.
    pub filename: Option<String>,
    /// Its media type without parameters; `text/plain` where it states none (RFC 7578 §4.4).
    pub media_type: String,
    /// Its content.
    pub bytes: Vec<u8>,
}

/// The first `needle` in `haystack` at or after `from`.
fn find(haystack: &[u8], needle: &[u8], from: usize) -> Option<usize> {
    haystack
        .get(from..)?
        .windows(needle.len())
        .position(|window| window == needle)
        .map(|at| at + from)
}

/// A header parameter's value: a quoted string unquoted, its `\` escapes undone, or the token.
fn unquoted(text: &str) -> String {
    let Some(inner) = text
        .strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
    else {
        return text.to_owned();
    };
    let mut out = String::new();
    let mut escaped = false;
    for character in inner.chars() {
        if escaped || character != '\\' {
            out.push(character);
            escaped = false;
        } else {
            escaped = true;
        }
    }
    out
}

/// A header value's first item and its `name=value` parameters, split at each `;` outside a
/// quoted string.
fn parameters(value: &str) -> (String, Vec<(String, String)>) {
    let mut items = Vec::new();
    let mut item = String::new();
    let mut quoted = false;
    let mut escaped = false;
    for character in value.chars() {
        match character {
            _ if escaped => escaped = false,
            '\\' if quoted => escaped = true,
            '"' => quoted = !quoted,
            ';' if !quoted => {
                items.push(std::mem::take(&mut item));
                continue;
            }
            _ => {}
        }
        item.push(character);
    }
    items.push(item);
    let first = items.remove(0).trim().to_owned();
    let pairs = items
        .iter()
        .filter_map(|item| item.split_once('='))
        .map(|(name, value)| (name.trim().to_ascii_lowercase(), unquoted(value.trim())))
        .collect();
    (first, pairs)
}

/// The `boundary` of a `multipart/form-data` request.
///
/// # Errors
///
/// A refusal when the request's `Content-Type` is not `multipart/form-data` or states no boundary
/// of 1 to 70 characters.
fn boundary(request: &Request) -> Result<String, String> {
    let value = request
        .headers
        .iter()
        .find(|(name, _)| name == "content-type")
        .map_or("", |(_, value)| value.as_str());
    let (essence, pairs) = parameters(value);
    if !essence.eq_ignore_ascii_case(MULTIPART) {
        return Err("the body is not `multipart/form-data`".to_owned());
    }
    pairs
        .into_iter()
        .find(|(name, value)| name == "boundary" && (1..=70).contains(&value.len()))
        .map(|(_, value)| value)
        .ok_or_else(|| "the body is `multipart/form-data` without a `boundary`".to_owned())
}

/// One part's header section and content.
fn sent(head: &[u8], bytes: Vec<u8>) -> Result<Sent, String> {
    let head = std::str::from_utf8(head).map_err(|_| "a part's headers are not UTF-8".to_owned())?;
    let (mut name, mut filename, mut media_type) = (None, None, None);
    for line in head.split("\r\n").filter(|line| !line.is_empty()) {
        let Some((key, value)) = line.split_once(':') else {
            return Err(format!("a part's header is not `name: value`: `{line}`"));
        };
        let key = key.trim();
        if key.eq_ignore_ascii_case("content-disposition") {
            let (kind, pairs) = parameters(value);
            if !kind.eq_ignore_ascii_case("form-data") {
                return Err(format!("a part's disposition is `{kind}`, not `form-data`"));
            }
            for (key, value) in pairs {
                match key.as_str() {
                    "name" => name = Some(value),
                    "filename" => filename = Some(value),
                    _ => {}
                }
            }
        } else if key.eq_ignore_ascii_case("content-type") {
            media_type = Some(parameters(value).0);
        }
    }
    let name = name.ok_or_else(|| "a part has no `Content-Disposition: form-data` name".to_owned())?;
    Ok(Sent {
        name,
        filename,
        media_type: media_type.unwrap_or_else(|| "text/plain".to_owned()),
        bytes,
    })
}

/// The parts of a `multipart/form-data` body, in the order sent (RFC 2046 §5.1.1): what precedes
/// the first boundary and follows the closing one is ignored.
///
/// # Errors
///
/// A refusal when the request is not `multipart/form-data` with a boundary, holds no boundary, is
/// not closed by its final boundary, or holds a part without a `form-data` name.
pub fn multipart_parts(request: &Request) -> Result<Vec<Sent>, String> {
    let boundary = boundary(request)?;
    let body = &request.body[..];
    let delimiter = format!("--{boundary}").into_bytes();
    let close = [b"\r\n".as_slice(), &delimiter].concat();
    let mut at = if body.starts_with(&delimiter) {
        delimiter.len()
    } else {
        find(body, &close, 0)
            .map(|start| start + close.len())
            .ok_or_else(|| "the body holds no `multipart` boundary".to_owned())?
    };
    let unclosed = || "the body is not closed by its final boundary".to_owned();
    let mut parts = Vec::new();
    loop {
        if body[at..].starts_with(b"--") {
            return Ok(parts);
        }
        while matches!(body.get(at), Some(b' ' | b'\t')) {
            at += 1;
        }
        if !body[at..].starts_with(b"\r\n") {
            return Err(unclosed());
        }
        at += 2;
        let (head, start) = if body[at..].starts_with(b"\r\n") {
            (&body[at..at], at + 2)
        } else {
            let end = find(body, b"\r\n\r\n", at).ok_or_else(unclosed)?;
            (&body[at..end], end + 4)
        };
        let end = find(body, &close, start).ok_or_else(unclosed)?;
        parts.push(sent(head, body[start..end].to_vec())?);
        at = end + close.len();
    }
}

/// A `multipart` body as the object the input's decoder reads: one member per declared
/// `(wire name, part, shape)` a part names, keyed by the wire name, a `List` as an array, empty
/// when no part names it, and a file part's filename and media type under the wire names it reads
/// them into. A part the path carries, or no field declares, is unknown: ignored where `ignore`,
/// and malformed otherwise.
///
/// # Errors
///
/// A refusal naming the part under `at` for an unknown part under `refuse`, a part that is not a
/// `List` given twice, a text part that is not UTF-8 or is empty for a field that is not a
/// `String`, a file part whose media type is not one it may carry, or one without the filename or
/// media type a required input field reads; and the refusals of [`multipart_parts`].
pub fn multipart_object(
    request: &Request,
    fields: &[(&str, Part, Shape)],
    carried: &[&str],
    ignore: bool,
    at: &str,
) -> Result<crate::json::Value, String> {
    let mut found: Vec<Vec<Sent>> = vec![Vec::new(); fields.len()];
    for part in multipart_parts(request)? {
        let place = crate::json::nested(at, &part.name);
        let index = fields.iter().position(|(wire, _, _)| *wire == part.name);
        let Some(index) = index.filter(|_| !carried.contains(&part.name.as_str())) else {
            if ignore {
                continue;
            }
            return Err(if carried.contains(&part.name.as_str()) {
                format!("{place}: the path carries this field, so it is not a part here")
            } else {
                format!("{place}: no field of this request is named so")
            });
        };
        found[index].push(part);
    }
    let mut members = Vec::new();
    for ((wire, part, shape), sent) in fields.iter().zip(found) {
        let place = crate::json::nested(at, wire);
        if *shape != Shape::List && sent.len() > 1 {
            return Err(format!("{place}: expected one part, found {}", sent.len()));
        }
        let mut values = Vec::new();
        for one in sent {
            values.push(part_value(one, *part, &place, &mut members)?);
        }
        if *shape == Shape::List {
            members.push(((*wire).to_owned(), crate::json::Value::Array(values)));
        } else if let Some(value) = values.pop() {
            members.push(((*wire).to_owned(), value));
        }
    }
    Ok(crate::json::Value::Object(members))
}

/// One part's value, a file part's filename and media type pushed onto `members`.
fn part_value(
    sent: Sent,
    part: Part,
    place: &str,
    members: &mut Vec<(String, crate::json::Value)>,
) -> Result<crate::json::Value, String> {
    use crate::json::Value;
    let (filename, media_type, media_types) = match part {
        Part::Text(flat) => {
            let text = String::from_utf8(sent.bytes)
                .map_err(|_| format!("{place}: expected UTF-8 text"))?;
            return flat_value(text, flat, place);
        }
        Part::File { filename, media_type, media_types } => (filename, media_type, media_types),
    };
    if !media_types.is_empty()
        && !media_types
            .iter()
            .any(|admitted| admitted.eq_ignore_ascii_case(&sent.media_type))
    {
        let admitted: Vec<String> = media_types.iter().map(|it| format!("`{it}`")).collect();
        return Err(format!(
            "{place}: the part's media type is `{}`, not one of {}",
            sent.media_type,
            admitted.join(", ")
        ));
    }
    for (read, value, what) in [
        (filename, sent.filename, "filename"),
        (media_type, Some(sent.media_type), "media type"),
    ] {
        let Some((wire, required)) = read else {
            continue;
        };
        match value {
            Some(text) => members.push((wire.to_owned(), Value::Text(text))),
            None if required => {
                return Err(format!(
                    "{place}: the part has no {what}, which input field `{wire}` requires"
                ));
            }
            None => {}
        }
    }
    let mut encoded = String::new();
    crate::json::push_base64(&mut encoded, &sent.bytes);
    Ok(Value::Text(encoded.trim_matches('"').to_owned()))
}
"#;

/// What the `http` module adds where a `multipart` route has a path variable and
/// `unknown: ignore`: a part the path carries is held to the path's value, as a `form` key is.
pub(super) const ECHOED_PARTS: &str = r#"
/// Holds each part of a `multipart` body read under `unknown: ignore` that the path carries,
/// `(wire name, flat)`, to the path's value, read as its field reads it.
///
/// # Errors
///
/// A refusal naming the part under `at` when its text does not decode or is not the path's, and
/// the refusals of [`multipart_parts`].
pub fn echoed_parts(
    request: &Request,
    carried: &[(&str, Flat)],
    path: &[(String, crate::json::Value)],
    at: &str,
) -> Result<(), String> {
    for part in multipart_parts(request)? {
        let Some(position) = carried.iter().position(|(wire, _)| *wire == part.name) else {
            continue;
        };
        let place = crate::json::nested(at, &part.name);
        let text = String::from_utf8(part.bytes)
            .map_err(|_| format!("{place}: expected UTF-8 text"))?;
        let read = flat_value(text, carried[position].1, &place)?;
        if !path
            .iter()
            .find(|(wire, _)| *wire == part.name)
            .is_some_and(|(_, held)| same_value(&read, held))
        {
            return Err(echoed(at, &part.name));
        }
    }
    Ok(())
}
"#;

/// What the `http` module adds where a route lists `request.query` (beyond10x/ess#754): the join
/// of the fields read from the query string with the body's.
pub(super) const QUERIED: &str = r#"
/// `value` with the members the query string carries added (beyond10x/ess#754): a field the
/// route's `request.query` lists, by wire name in `listed`, is read from the query string alone,
/// and the body naming it is malformed, whether the query carries it or not.
///
/// # Errors
///
/// A refusal naming the member under `at` when `value` holds one `listed` names, the first in
/// the order listed.
pub fn with_query(
    value: crate::json::Value,
    query: crate::json::Value,
    listed: &[&str],
    at: &str,
) -> Result<crate::json::Value, String> {
    let crate::json::Value::Object(read) = query else {
        return Ok(value);
    };
    match value {
        crate::json::Value::Object(mut members) => {
            for wire in listed {
                if members.iter().any(|(key, _)| key == wire) {
                    return Err(format!(
                        "{}: the query carries this field, so the body does not",
                        crate::json::nested(at, wire)
                    ));
                }
            }
            members.extend(read);
            Ok(crate::json::Value::Object(members))
        }
        // Not an object: the decoder refuses it by name.
        other => Ok(other),
    }
}
"#;

/// The credential checks of a route whose binding requires a scheme (beyond10x/ess#681), which
/// follow [`BOUND`] only in a surface whose binding declares `authentication`, so every other
/// bound surface keeps its bytes. Each says whether the request carries a credential where the
/// scheme puts one; none reads or keeps its value beyond the check, and none reads the query.
pub(super) const CREDENTIALS: &str = r#"
/// Whether the request carries exactly one `Authorization` header under the auth scheme `scheme`,
/// compared ignoring case, followed by one or more spaces and a token68 (RFC 9110 §11.4, §11.2).
pub fn authorization(request: &Request, scheme: &str) -> bool {
    let mut given = request.headers.iter().filter(|(name, _)| name == "authorization");
    let (Some((_, value)), None) = (given.next(), given.next()) else {
        return false;
    };
    let Some((named, credentials)) = value.split_once(' ') else {
        return false;
    };
    let token = credentials.trim_start_matches(' ');
    let body = token.trim_end_matches('=');
    named.eq_ignore_ascii_case(scheme)
        && !body.is_empty()
        && body
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"-._~+/".contains(&byte))
}

/// Whether the request carries exactly one header `name` (lower case) that is not empty.
pub fn key_header(request: &Request, name: &str) -> bool {
    let mut given = request.headers.iter().filter(|(given, _)| given == name);
    matches!((given.next(), given.next()), (Some((_, value)), None) if !value.trim().is_empty())
}

/// Whether a `Cookie` header carries the cookie `name` with a value that is not empty
/// (RFC 6265 §5.4).
pub fn key_cookie(request: &Request, name: &str) -> bool {
    request
        .headers
        .iter()
        .filter(|(given, _)| given == "cookie")
        .flat_map(|(_, value)| value.split(';'))
        .any(|pair| {
            pair.trim()
                .split_once('=')
                .is_some_and(|(key, value)| key == name && !value.is_empty())
        })
}
"#;

/// Whether a route of `binding` requires a scheme, which the surface checks with [`CREDENTIALS`].
pub(super) fn authenticates(binding: &http::Binding) -> bool {
    binding
        .ir()
        .routes()
        .iter()
        .any(|route| !binding.credentials_on(route).is_empty())
}

/// Whether a route of `binding` lists `request.query`, which the surface joins with [`QUERIED`].
pub(super) fn reads_query_beside_body(binding: &http::Binding) -> bool {
    binding.ir().routes().iter().any(|route| {
        route
            .request
            .as_ref()
            .is_some_and(|request| !request.query.is_empty())
    })
}

/// Whether a route of `binding` takes a `multipart` request, which the surface reads with
/// [`MULTIPART`].
pub(super) fn reads_multipart(binding: &http::Binding) -> bool {
    binding.ir().routes().iter().any(|route| {
        route
            .request
            .as_ref()
            .is_some_and(|request| request.encoding == ess_http::Encoding::Multipart)
    })
}

/// Whether a `multipart` route of `binding` has a path variable and `unknown: ignore`, which the
/// surface reads with [`ECHOED_PARTS`].
pub(super) fn echoes_parts(binding: &http::Binding) -> bool {
    binding.ir().routes().iter().any(|route| {
        !route.parameters.is_empty()
            && route.request.as_ref().is_some_and(|request| {
                request.encoding == ess_http::Encoding::Multipart
                    && request.unknown == ess_http::Unknown::Ignore
            })
    })
}

/// Whether a route of `binding` declares `unknown: ignore` beside a `json` body or a path variable,
/// which the surface reads with [`IGNORING`].
pub(super) fn ignores_unknown(binding: &http::Binding) -> bool {
    binding.ir().routes().iter().any(|route| {
        route.request.as_ref().is_some_and(|request| {
            request.unknown == ess_http::Unknown::Ignore
                && (request.encoding == ess_http::Encoding::Json || !route.parameters.is_empty())
        })
    })
}

/// The route match of a bound surface: every row of the table, tried in turn in
/// [`http::dispatch_order`], path order but for a literal segment taking precedence over a
/// variable at the first position two templates of one method differ.
///
/// A row the binding declares is a template ([`http::Route::declared`]); a derived row and the two
/// documents are exact paths. A path some row matches under the request's method is answered by
/// that row; one some row matches under another method is `405`, naming every method a matching
/// row answers, in its body and in `Allow`; one no row matches is `404`.
pub(super) fn dispatch(out: &mut String, server: &Server<'_>, routes: &[http::Route<'_>]) {
    let ir = server.ir;
    let grants = http::checks_grants(ir);

    out.push_str(
        "\n/// Answers one request.\n///\n/// The routes are templates an `ess-http/1` binding \
         declares, beside the routes it does not name.\n/// A path no route matches is a `404` \
         naming where the whole table is published; a path some\n/// route matches under another \
         method is a `405` naming every method its routes answer, in `Allow` too.\n///\n/// Public so a caller \
         can hand it a request it built itself: [`serve`] is this function behind a\n/// socket, \
         and nothing else.\n",
    );
    if grants {
        out.push_str(
            "///\n/// `caller` is who the realization authenticated the request as, or `None`. \
             Every command\n/// checks its grant before it runs.\n",
        );
    }
    super::dispatch_entries(out, server, routes);
    out.push_str("{\n    let mut allowed: Vec<&str> = Vec::new();\n");
    let rows = table(routes, ir);
    let keys: Vec<(&str, &str)> = rows
        .iter()
        .map(|(method, path, _, _)| (*method, path.as_str()))
        .collect();
    for (method, path, _, _) in http::dispatch_order(&keys)
        .into_iter()
        .map(|index| rows[index].clone())
    {
        let route = routes
            .iter()
            .find(|route| route.path == path && route.method.as_str() == method);
        let declared = route.filter(|route| route.declared.is_some());
        let variables = declared.is_some_and(|route| !http::path_fields(ir, route).is_empty());
        let opening = match (declared, variables) {
            (Some(_), true) => {
                format!("    if let Some(path) = http::matched({path:?}, &request.path) {{\n")
            }
            (Some(_), false) => {
                format!("    if http::matched({path:?}, &request.path).is_some() {{\n")
            }
            (None, _) => format!("    if request.path == {path:?} {{\n"),
        };
        out.push_str(&opening);
        let _ = writeln!(out, "        if request.method == {method:?} {{");
        let call = match route {
            None if path == http::OPENAPI => {
                "            return http::Response::new(200, http::JSON, OPENAPI);\n".to_owned()
            }
            None => {
                "            return http::Response::new(200, http::MARKDOWN, DOCS);\n".to_owned()
            }
            Some(route) => answer_call(server, route, declared.is_some(), variables),
        };
        out.push_str(&call);
        let _ = writeln!(out, "        }}\n        allowed.push({method:?});\n    }}");
    }
    out.push_str(
        "    if allowed.is_empty() {\n        return http::Response::refusal(\n            404,\n            \
         &format!(\"`{}` is not a path this surface declares; `GET /openapi.json` publishes \
         every one that is\", request.path),\n        );\n    }\n    \
         // RFC 9110 §15.5.6: a `405` names the methods the path answers in `Allow`.\n    \
         allowed.sort_unstable();\n    allowed.dedup();\n    let allow = allowed.join(\", \");\n    \
         let mut answer = http::method_not_allowed(&allow);\n    \
         answer.headers.push((\"Allow\".to_owned(), allow));\n    answer\n}\n",
    );
}

/// What one matched row does under its own method: check the grant, then answer.
fn answer_call(
    server: &Server<'_>,
    route: &http::Route<'_>,
    declared: bool,
    variables: bool,
) -> String {
    let ir = server.ir;
    let layout = server.layout;
    let path = if variables { "&path" } else { "&[]" };
    let refuse = if declared {
        format!("refusal_not_granted{}", refusal_suffix(server, route))
    } else {
        "not_granted".to_owned()
    };
    let mut call = credential_check(server, route);
    match route.serves {
        Served::Command(handle) => {
            let command = ir.command(handle);
            if http::checks_grants(ir) {
                let _ = write!(
                    call,
                    "            if let Err(actor) = admit(caller, {:?}) {{\n                \
                     return {refuse}(actor);\n            }}\n",
                    command.name.to_string()
                );
            }
            if declared {
                let _ = writeln!(
                    call,
                    "            return {}(system, request, {path}, taken);",
                    bound_ident(layout, &command.name)
                );
            } else {
                let _ = writeln!(
                    call,
                    "            return {}(system, &request.body, taken);",
                    handler_ident(layout, &command.name)
                );
            }
        }
        Served::View(handle) => {
            let view = ir.view(handle);
            if http::read_checked(ir, handle.name()) {
                let _ = write!(
                    call,
                    "            if let Err(actor) = admit_read(caller, {:?}) {{\n                \
                     return {refuse}(actor);\n            }}\n",
                    view.name.to_string()
                );
            }
            if declared {
                let _ = writeln!(
                    call,
                    "            return {}(system, request, {path});",
                    bound_ident(layout, &view.name)
                );
            } else if view.params.is_empty() {
                let _ = writeln!(
                    call,
                    "            return http::answer({}(system));",
                    runner_ident(layout, &view.name)
                );
            } else {
                let _ = writeln!(
                    call,
                    "            return {}(system, &request.query);",
                    handler_ident(layout, &view.name)
                );
            }
        }
    }
    call
}

/// The handler function name of one construct's bound route.
fn bound_ident(layout: &Layout, declared: &ess_domain::name::QualifiedName) -> String {
    format!("serve_bound_{}", wire::ident(layout, declared))
}

/// The suffix of the standard-refusal functions a declared route answers through: none for the
/// binding's (`refusal_malformed`, …), `_<ident>` for a route that overrides one itself
/// (`routes[i].refusals`), whose functions answer its own overrides over the binding's.
fn refusal_suffix(server: &Server<'_>, route: &http::Route<'_>) -> String {
    if route
        .declared
        .is_none_or(|declared| declared.refusals.is_empty())
    {
        return String::new();
    }
    let name = match route.serves {
        Served::Command(handle) => &server.ir.command(handle).name,
        Served::View(handle) => &server.ir.view(handle).name,
    };
    format!("_{}", wire::ident(server.layout, name))
}

/// One bound route's handler: read the request the binding declares, run the shared path, answer
/// what the binding declares.
pub(super) fn handler(out: &mut String, server: &Server<'_>, route: &http::Route<'_>) {
    match route.serves {
        Served::Command(handle) => command(out, server, route, server.ir.command(handle)),
        Served::View(handle) => view(out, server, route, server.ir.view(handle)),
    }
}

/// The JSON values each path variable of `route` carries, as Rust source: one
/// `(wire name, http::path_value(…)?)` per variable, in path order.
fn path_members(server: &Server<'_>, route: &http::Route<'_>) -> String {
    let fields = http::path_fields(server.ir, route);
    let members: Vec<String> = fields
        .iter()
        .enumerate()
        .map(|(index, (variable, field))| {
            format!(
                "({:?}.to_owned(), http::path_value(&path[{index}], http::Flat::{}, {:?})?)",
                ess_gen::schema::wire_field_name(field),
                flat_of(server, field).as_str(),
                format!("path.{variable}")
            )
        })
        .collect();
    format!("vec![{}]", members.join(", "))
}

/// How one field the path or a flat request carries is read from text.
fn flat_of(server: &Server<'_>, field: &ResolvedField) -> http::Flat {
    // A compiled binding admits only fields with a flat spelling (`ESS-HTTP-009`, `ESS-HTTP-021`).
    http::flat(server.ir, &field.type_ref).map_or(http::Flat::Text, |(_, flat)| flat)
}

/// A flat request's `(wire name, flat, shape)` table, as Rust source, every field the path does not
/// carry, in declaration order.
fn flat_table(server: &Server<'_>, fields: &[ResolvedField], route: &http::Route<'_>) -> String {
    let carried = http::carried(server.ir, route);
    let rows: Vec<String> = fields
        .iter()
        .filter(|field| !carried.contains(field.name.as_str()))
        .map(|field| {
            let (shape, flat) = http::flat(server.ir, &field.type_ref)
                .unwrap_or((http::Shape::One, http::Flat::Text));
            format!(
                "({:?}, http::Flat::{}, http::Shape::{})",
                ess_gen::schema::wire_field_name(field),
                flat.as_str(),
                shape.as_str()
            )
        })
        .collect();
    format!("&[{}]", rows.join(", "))
}

/// How a bound command route joins the object it read with the members its path carries, as Rust
/// source reading `value` and `request`. Under `unknown: ignore` (beyond10x/ess#752) what no field
/// names is dropped and what the path carries is held to the path's value before the path's members
/// are added.
fn joined(
    server: &Server<'_>,
    route: &http::Route<'_>,
    command: &ResolvedCommand,
    at: &str,
) -> String {
    let request = http::request(route);
    let variables = !http::path_fields(server.ir, route).is_empty();
    let ignore = request.unknown == ess_http::Unknown::Ignore;
    let members = if variables {
        path_members(server, route)
    } else {
        "Vec::new()".to_owned()
    };
    match (
        ignore && (variables || request.encoding == Encoding::Json),
        request.encoding,
    ) {
        (false, _) => format!("http::with_path(value, {members}, {at:?})"),
        (true, Encoding::Multipart) => format!(
            "let carried = {members};\n        http::echoed_parts(request, {}, &carried, \
             {at:?})?;\n        http::with_path(value, carried, {at:?})",
            carried_flat_table(server, route)
        ),
        (true, Encoding::Json) => format!(
            "let carried = {members};\n        let value = http::ignoring(value, {}, &carried, \
             {at:?})?;\n        http::with_path(value, carried, {at:?})",
            wire_table(server, &command.input, route)
        ),
        (true, encoding) => format!(
            "let carried = {members};\n        http::echoed_keys({}, {}, &carried, {at:?})?;\n        \
             http::with_path(value, carried, {at:?})",
            if encoding == Encoding::Query {
                "&request.query"
            } else {
                "http::body_text(&request.body)?"
            },
            carried_flat_table(server, route)
        ),
    }
}

/// `joined`, the join of the body with the path's members, joined in turn with the fields the
/// route's `request.query` lists, read from the query string as `encoding: query` reads them
/// (beyond10x/ess#754). A route that lists none keeps `joined` and its bytes.
fn queried(
    server: &Server<'_>,
    route: &http::Route<'_>,
    command: &ResolvedCommand,
    joined: String,
) -> String {
    let request = http::request(route);
    if request.query.is_empty() {
        return joined;
    }
    let fields: Vec<&ResolvedField> = request
        .query_fields()
        .filter_map(|name| command.input.iter().find(|field| field.name == name))
        .collect();
    let rows: Vec<String> = fields
        .iter()
        .map(|field| {
            let (shape, flat) = http::flat(server.ir, &field.type_ref)
                .unwrap_or((http::Shape::One, http::Flat::Text));
            format!(
                "({:?}, http::Flat::{}, http::Shape::{})",
                ess_gen::schema::wire_field_name(field),
                flat.as_str(),
                shape.as_str()
            )
        })
        .collect();
    let listed: Vec<String> = fields
        .iter()
        .map(|field| format!("{:?}", ess_gen::schema::wire_field_name(field)))
        .collect();
    format!(
        "let value = {{\n        {joined}\n        }}?;\n        let query = \
         http::flat_object(&request.query, &[{}], &[], {}, \"query\")?;\n        \
         http::with_query(value, query, &[{}], \"body\")",
        rows.join(", "),
        request.unknown == ess_http::Unknown::Ignore,
        listed.join(", ")
    )
}

/// A `multipart` request's `(wire name, part, shape)` table, as Rust source (beyond10x/ess#754):
/// every input field neither the path nor a part attribute carries, in declaration order, a
/// `Bytes` field ([`http::file_part`]) a file part with the wire names its attributes read into.
fn part_table(server: &Server<'_>, command: &ResolvedCommand, route: &http::Route<'_>) -> String {
    let carried = http::carried(server.ir, route);
    let read = http::attribute_fields(route.declared);
    let request = http::request(route);
    // The input field an attribute reads, as `Some((wire name, required))`.
    let attribute = |source: Option<&ess_http::Source>| {
        source
            .and_then(|source| {
                command
                    .input
                    .iter()
                    .find(|field| field.name == source.field())
            })
            .map_or_else(
                || "None".to_owned(),
                |field| {
                    format!(
                        "Some(({:?}, {}))",
                        ess_gen::schema::wire_field_name(field),
                        !field.type_ref.is_optional()
                    )
                },
            )
    };
    let rows: Vec<String> = command
        .input
        .iter()
        .filter(|field| {
            !carried.contains(field.name.as_str()) && !read.contains(field.name.as_str())
        })
        .map(|field| {
            let (shape, flat) = http::flat(server.ir, &field.type_ref)
                .unwrap_or((http::Shape::One, http::Flat::Text));
            let part = if http::file_part(server.ir, &field.type_ref).is_some() {
                let declared = request.parts.get(&field.name);
                let types: Vec<String> = declared
                    .map(|part| {
                        part.media_types
                            .iter()
                            .map(|it| format!("{it:?}"))
                            .collect()
                    })
                    .unwrap_or_default();
                format!(
                    "http::Part::File {{ filename: {}, media_type: {}, media_types: &[{}] }}",
                    attribute(declared.and_then(|part| part.filename.as_ref())),
                    attribute(declared.and_then(|part| part.media_type.as_ref())),
                    types.join(", ")
                )
            } else {
                format!("http::Part::Text(http::Flat::{})", flat.as_str())
            };
            format!(
                "({:?}, {part}, http::Shape::{})",
                ess_gen::schema::wire_field_name(field),
                shape.as_str()
            )
        })
        .collect();
    format!("&[{}]", rows.join(", "))
}

/// The wire names of the input fields the path does not carry, as a Rust slice.
fn wire_table(server: &Server<'_>, fields: &[ResolvedField], route: &http::Route<'_>) -> String {
    let carried = http::carried(server.ir, route);
    let wires: Vec<String> = fields
        .iter()
        .filter(|field| !carried.contains(field.name.as_str()))
        .map(|field| format!("{:?}", ess_gen::schema::wire_field_name(field)))
        .collect();
    format!("&[{}]", wires.join(", "))
}

/// The `(wire name, flat)` of each field the path carries, as a Rust slice.
fn carried_flat_table(server: &Server<'_>, route: &http::Route<'_>) -> String {
    let rows: Vec<String> = http::path_fields(server.ir, route)
        .iter()
        .map(|(_, field)| {
            format!(
                "({:?}, http::Flat::{})",
                ess_gen::schema::wire_field_name(field),
                flat_of(server, field).as_str()
            )
        })
        .collect();
    format!("&[{}]", rows.join(", "))
}

/// The wire names of the fields the path carries, as a Rust slice.
fn carried_table(server: &Server<'_>, route: &http::Route<'_>) -> String {
    let wires: Vec<String> = http::path_fields(server.ir, route)
        .iter()
        .map(|(_, field)| format!("{:?}", ess_gen::schema::wire_field_name(field)))
        .collect();
    format!("&[{}]", wires.join(", "))
}

/// A bound command route.
fn command(
    out: &mut String,
    server: &Server<'_>,
    route: &http::Route<'_>,
    command: &ResolvedCommand,
) {
    let ir = server.ir;
    let layout = server.layout;
    let system_crate = Layout::crate_ident(layout.system_package());
    let angled = generic_list(server);
    let ident = wire::ident(layout, &command.name);
    let run = runner_ident(layout, &command.name);
    let request = http::request(route);
    let variables = !http::path_fields(ir, route).is_empty();
    let ignore = request.unknown == ess_http::Unknown::Ignore;
    let bare = http::bare_body(command, route.declared);
    // What the path or the query string (beyond10x/ess#754) carries is not the body's to hold.
    let mut outside = http::carried(ir, route);
    outside.extend(http::queried(route.declared));
    let read = match request.encoding {
        // One input field's value is the whole body: read as the object holding that field alone,
        // or, where the body is not required, no body as the input `{}`.
        Encoding::Json if bare.is_some() => {
            let wire = bare.map(ess_gen::schema::wire_field_name).unwrap_or_default();
            let held = format!(
                "json::Value::Object(vec![({wire:?}.to_owned(), http::json_body(&request.body, \
                 true)?)])"
            );
            if http::body_required_without(command, &outside) {
                held
            } else {
                format!(
                    "if http::body_text(&request.body)?.trim().is_empty() {{\n            \
                     json::Value::Object(Vec::new())\n        }} else {{\n            {held}\n        \
                     }}"
                )
            }
        }
        Encoding::Json => format!(
            "http::json_body(&request.body, {})?",
            http::body_required_without(command, &outside)
        ),
        Encoding::Multipart => format!(
            "http::multipart_object(request, {}, {}, {ignore}, \"body\")?",
            part_table(server, command, route),
            carried_table(server, route)
        ),
        Encoding::Form => format!(
            "{{\n            if !http::content_type_is(request, http::FORM) {{\n                \
             return Err(\"the body is not `application/x-www-form-urlencoded`\".to_owned());\n            \
             }}\n            http::flat_object(http::body_text(&request.body)?, {}, {}, {ignore}, \
             \"body\")?\n        }}",
            flat_table(server, &command.input, route),
            carried_table(server, route)
        ),
        Encoding::Query => format!(
            "http::flat_object(&request.query, {}, {}, {ignore}, \"query\")?",
            flat_table(server, &command.input, route),
            carried_table(server, route)
        ),
    };
    let at = if request.encoding == Encoding::Query {
        "query"
    } else {
        "body"
    };
    let path = if variables { "path" } else { "_path" };
    let joined = queried(server, route, command, joined(server, route, command, at));
    let suffix = refusal_suffix(server, route);
    let declares = http::declares_answers(route);
    let answer = if declares {
        format!("answer_bound_{ident}(status, envelope, &value)")
    } else {
        "http::Response::new(status, http::JSON, envelope)".to_owned()
    };
    let _ = write!(
        out,
        "\n/// `{}` `{}`: `{}`, read and answered as its `ess-http/1` binding declares.\nfn \
         {}{angled}(system: &mut {system_crate}::System{angled}, request: &http::Request, \
         {path}: &[String], taken: &mut Vec<{system_crate}::SystemEvent>) -> http::Response\n{}{{\n    let read = || -> Result<json::Value, \
         String> {{\n        let value = {read};\n        {joined}\n    \
         }};\n    let value = match read() {{\n        Ok(value) => value,\n        Err(detail) => \
         return refusal_malformed{suffix}(&detail),\n    }};\n    match {run}(system, &value, taken) {{\n        \
         Ok((status, envelope)) => {answer},\n        Err(refused) => refused_bound{suffix}(&refused),\n    \
         }}\n}}\n",
        route.method.as_str(),
        route.path,
        command.name,
        bound_ident(layout, &command.name),
        where_clause(server),
    );
    if declares {
        outcome_answer(out, server, route, command);
    }
}

/// `answer_bound_*`: the declared answer of each outcome the route lists, read off the envelope
/// the shared path rendered, and the envelope at its derived status for every other outcome.
fn outcome_answer(
    out: &mut String,
    server: &Server<'_>,
    route: &http::Route<'_>,
    command: &ResolvedCommand,
) {
    let ident = wire::ident(server.layout, &command.name);
    let suffix = refusal_suffix(server, route);
    let listed: Vec<(&ess_compiler::ir::ResolvedOutcome, &ess_http::Outcome)> = command
        .outcomes
        .iter()
        .filter_map(|outcome| http::listed(route, outcome).map(|declared| (outcome, declared)))
        .collect();
    let sources: Vec<&AnswerSource> = listed
        .iter()
        .flat_map(|(_, declared)| answer_sources(declared))
        .collect();
    let reads_input = sources
        .iter()
        .any(|source| matches!(source, AnswerSource::Input(_)));
    let reads_response = sources
        .iter()
        .any(|source| matches!(source, AnswerSource::Response(_)))
        || listed
            .iter()
            .any(|(_, declared)| declared.body.reads_response());
    let reads_payload = sources
        .iter()
        .any(|source| matches!(source, AnswerSource::Error(_)))
        || listed
            .iter()
            .any(|(_, declared)| matches!(declared.body, Body::Error { .. }));
    let _ = write!(
        out,
        "\n/// The answer `{}` gives as its binding declares it: each listed outcome's status, body, \
         headers\n/// and redirect, and the envelope at its derived status for every other \
         outcome.\nfn answer_bound_{ident}(status: u16, envelope: String, input: &json::Value) -> \
         http::Response {{\n    match declared_{ident}(&envelope, input) {{\n        \
         Ok(Some(answer)) => answer,\n        Ok(None) => http::Response::new(status, http::JSON, \
         envelope),\n        // The effect stands; only its answer could not be written.\n        \
         Err(detail) => refusal_unfinished{suffix}(&detail, true),\n    }}\n}}\n\n/// Each listed outcome \
         of `{}`, answered as declared; `None` for an outcome the binding does not \
         list.\nfn declared_{ident}(envelope: &str, {}: &json::Value) -> \
         Result<Option<http::Response>, String> {{\n    let read = \
         entry::read(envelope).map_err(|error| {{\n        format!(\"its answer is not JSON \
         this surface reads back: {{error}}\")\n    }})?;\n",
        command.name,
        command.name,
        if reads_input { "input" } else { "_input" },
    );
    if reads_response {
        out.push_str("    let response = read.member(\"response\");\n");
    }
    if reads_payload {
        out.push_str("    let payload = read.member(\"payload\");\n");
    }
    out.push_str(
        "    let Some(json::Value::Text(outcome)) = read.member(\"outcome\") else {\n        return \
         Ok(None);\n    };\n    match outcome.as_str() {\n",
    );
    for (outcome, declared) in &listed {
        let _ = write!(
            out,
            "        {:?} => {{\n{}            Ok(Some(answer))\n        }}\n",
            outcome.name.as_str(),
            outcome_arm(server, command, outcome, declared)
        );
    }
    out.push_str("        _ => Ok(None),\n    }\n}\n");
}

/// One listed outcome's arm of `declared_*`: its body, its headers and its redirect, as Rust
/// statements building `answer`.
fn outcome_arm(
    server: &Server<'_>,
    command: &ResolvedCommand,
    outcome: &ess_compiler::ir::ResolvedOutcome,
    declared: &ess_http::Outcome,
) -> String {
    let ir = server.ir;
    let error = outcome.error.as_ref().map(|handle| ir.error(handle));
    let wire_name = error.map(|error| error.naming.wire_or(&error.name).to_owned());
    let status = declared.status;
    let body = match &declared.body {
        Body::Envelope => {
            format!("http::Response::new({status}, http::JSON, envelope.to_owned())")
        }
        Body::Response => {
            format!("http::Response::new({status}, http::JSON, http::bare(response))")
        }
        // The bare form's JSON under its one member (beyond10x/ess#755).
        Body::Wrapped { member, field } => {
            let bare = match field {
                None => "http::bare(response)".to_owned(),
                Some(field) => format!(
                    "http::bare(response.and_then(|response| response.member({:?})))",
                    response_wire(command, field)
                ),
            };
            format!(
                "http::Response::new({status}, http::JSON, {})",
                wrapped_json(member, &bare)
            )
        }
        // A string field answered `encoding: text`: its value alone, not JSON. The media type is
        // written inline rather than as a constant beside `JSON`, so a server whose binding
        // answers no text keeps its bytes.
        Body::ResponseField(field) if declared.encoding == ResponseEncoding::Text => format!(
            "http::Response::new({status}, {:?}, match response.and_then(|response| \
             response.member({:?})) {{ Some(crate::json::Value::Text(text)) => text.clone(), _ => \
             String::new() }})",
            http::TEXT_CONTENT_TYPE,
            response_wire(command, field)
        ),
        Body::ResponseField(field) => format!(
            "http::Response::new({status}, http::JSON, http::bare(response.and_then(|response| \
             response.member({:?}))))",
            response_wire(command, field)
        ),
        Body::None => format!("http::Response::empty({status})"),
        Body::Error { member } => format!(
            "http::Response::new({status}, http::JSON, http::error_object({member:?}, {:?}, \
                 payload))",
            wire_name.clone().unwrap_or_default()
        ),
    };
    let mut arm = format!("            let mut answer = {body};\n");
    for (name, header) in &declared.headers {
        let line = match header {
            Header::Literal(text) => format!(
                "            answer.headers.push(({name:?}.to_owned(), {text:?}.to_owned()));\n"
            ),
            Header::Source(source) => format!(
                "            http::push_header(&mut answer, {name:?}, {})?;\n",
                source_text(server, command, error, wire_name.as_deref(), source)
            ),
            Header::Challenge(challenge) => format!(
                "            answer.headers.push(({name:?}.to_owned(), \
                     http::challenge({:?}, &[{}])?));\n",
                challenge.scheme,
                params(
                    server,
                    command,
                    error,
                    wire_name.as_deref(),
                    &challenge.params
                )
            ),
        };
        arm.push_str(&line);
    }
    if let Some(redirect) = &declared.redirect {
        let parameters: Vec<String> = redirect
            .parameters
            .iter()
            .map(|(name, source)| {
                format!(
                    "({name:?}, {})",
                    source_text(server, command, error, wire_name.as_deref(), source)
                )
            })
            .collect();
        let _ = writeln!(
            arm,
            "            let location = http::redirect({}, {}, &[{}])?;\n            \
                 answer.headers.push((\"Location\".to_owned(), location));",
            source_text(
                server,
                command,
                error,
                wire_name.as_deref(),
                &redirect.location
            ),
            redirect.mode == RedirectMode::Fragment,
            parameters.join(", ")
        );
    }
    let mutated = !declared.headers.is_empty() || declared.redirect.is_some();
    if !mutated {
        arm = arm.replacen("let mut answer", "let answer", 1);
    }
    arm
}

/// Every source one listed outcome reads: its headers', its challenge params' and its redirect's.
fn answer_sources(declared: &ess_http::Outcome) -> Vec<&AnswerSource> {
    let mut sources = Vec::new();
    for header in declared.headers.values() {
        match header {
            Header::Literal(_) => {}
            Header::Source(source) => sources.push(source),
            Header::Challenge(challenge) => {
                for param in challenge.params.values() {
                    if let Param::Source(source) = param {
                        sources.push(source);
                    }
                }
            }
        }
    }
    if let Some(redirect) = &declared.redirect {
        sources.push(&redirect.location);
        sources.extend(redirect.parameters.values());
    }
    sources
}

/// A challenge's params, in name order, as `(name, Option<String>)` Rust source.
fn params(
    server: &Server<'_>,
    command: &ResolvedCommand,
    error: Option<&ess_compiler::ir::ResolvedError>,
    wire_name: Option<&str>,
    params: &std::collections::BTreeMap<String, Param>,
) -> String {
    params
        .iter()
        .map(|(name, param)| match param {
            Param::Literal(text) => format!("({name:?}, Some({text:?}.to_owned()))"),
            Param::Source(source) => format!(
                "({name:?}, {})",
                source_text(server, command, error, wire_name, source)
            ),
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// The wire name of the command's response field `field`, which a `response.<field>` body
/// answers bare.
fn response_wire(command: &ResolvedCommand, field: &str) -> String {
    command
        .response
        .iter()
        .find(|declared| declared.name == field)
        .map_or_else(
            || field.to_owned(),
            |declared| ess_gen::schema::wire_field_name(declared).to_owned(),
        )
}

/// One answer source as an `Option<String>` Rust expression over `input`, `response` and
/// `payload`, each field by its wire name.
fn source_text(
    server: &Server<'_>,
    command: &ResolvedCommand,
    error: Option<&ess_compiler::ir::ResolvedError>,
    wire_name: Option<&str>,
    source: &AnswerSource,
) -> String {
    let _ = server;
    let wire_of = |fields: &[ResolvedField], name: &str| {
        fields.iter().find(|field| field.name == name).map_or_else(
            || name.to_owned(),
            |field| ess_gen::schema::wire_field_name(field).to_owned(),
        )
    };
    match source {
        AnswerSource::Input(field) => format!(
            "http::scalar_text(input.member({:?}))",
            wire_of(&command.input, field)
        ),
        AnswerSource::Response(field) => format!(
            "http::scalar_text(response.and_then(|response| response.member({:?})))",
            wire_of(&command.response, field)
        ),
        AnswerSource::Error(field) => format!(
            "http::scalar_text(payload.and_then(|payload| payload.member({:?})))",
            wire_of(error.map_or(&[][..], |error| &error.fields[..]), field)
        ),
        AnswerSource::ErrorWire => {
            format!("Some({:?}.to_owned())", wire_name.unwrap_or_default())
        }
    }
}

/// A bound view route.
fn view(out: &mut String, server: &Server<'_>, route: &http::Route<'_>, view: &ResolvedView) {
    let ir = server.ir;
    let layout = server.layout;
    let system_crate = Layout::crate_ident(layout.system_package());
    let angled = generic_list(server);
    let run = runner_ident(layout, &view.name);
    let variables = !http::path_fields(ir, route).is_empty();
    let path = if variables { "path" } else { "_path" };
    let representations = route
        .declared
        .map_or(&[][..], |declared| &declared.representations[..]);
    let request = if view.params.is_empty() && representations.is_empty() {
        "_request"
    } else {
        "request"
    };
    let suffix = refusal_suffix(server, route);
    let _ = write!(
        out,
        "\n/// `{}` `{}`: `{}`, read and answered as its `ess-http/1` binding declares.\nfn \
         {}{angled}(system: &{system_crate}::System{angled}, {request}: &http::Request, \
         {path}: &[String]) -> http::Response\n{}{{\n",
        route.method.as_str(),
        route.path,
        view.name,
        bound_ident(layout, &view.name),
        where_clause(server),
    );
    out.push_str(&selection(representations, &suffix));
    if view.params.is_empty() {
        let _ = writeln!(out, "    let result = {run}(system);");
    } else {
        let carried = http::carried(ir, route);
        let unmapped: Vec<&ResolvedField> = view
            .params
            .iter()
            .filter(|param| !carried.contains(param.name.as_str()))
            .collect();
        let _ = write!(
            out,
            "    let read = || -> Result<json::Value, String> {{\n        let value = \
             http::query_object(&request.query, {}, \"query\")?;\n        http::with_path(value, \
             {}, \"query\")\n    }};\n    let value = match read() {{\n        Ok(value) => value,\n        \
             Err(detail) => return refusal_malformed{suffix}(&detail),\n    }};\n    let result = \
             {run}(system, &value);\n",
            super::query_params(ir, &unmapped),
            if variables {
                path_members(server, route)
            } else {
                "Vec::new()".to_owned()
            },
        );
    }
    let body = route
        .declared
        .map_or(ess_http::ViewBody::Rows, |declared| declared.body.form);
    let empty = route
        .declared
        .filter(|_| body == ess_http::ViewBody::Single)
        .and_then(|declared| declared.empty.as_ref());
    // The member a wrapped body names (beyond10x/ess#755): the bare answer goes under it.
    let member = route
        .declared
        .and_then(|declared| declared.body.member.as_deref());
    let (status, answer) = match (body, empty, member) {
        (_, Some(empty), None) => ("_status", single_answer(empty)),
        // The one row under its member; the empty answer as declared.
        (_, Some(empty), Some(member)) => (
            "_status",
            format!(
                "{{\n            let mut answer = {};\n            if answer.status == 200 {{\n                \
                 answer.body = {};\n            }}\n            answer\n        }}",
                single_answer(empty),
                wrapped_json(member, "answer.body")
            ),
        ),
        // The rows of `{"rows": [...]}`, bare.
        (ess_http::ViewBody::Array, None, None) => (
            "status",
            "{\n            let read = json::parse(&body).unwrap_or(json::Value::Null);\n            \
             http::Response::new(status, http::JSON, http::bare(read.member(\"rows\")))\n        \
             }"
            .to_owned(),
        ),
        // The rows of `{"rows": [...]}`, under the member.
        (ess_http::ViewBody::Array, None, Some(member)) => (
            "status",
            format!(
                "{{\n            let read = json::parse(&body).unwrap_or(json::Value::Null);\n            \
                 http::Response::new(status, http::JSON, {})\n        }}",
                wrapped_json(member, "http::bare(read.member(\"rows\"))")
            ),
        ),
        _ => (
            "status",
            "http::Response::new(status, http::JSON, body)".to_owned(),
        ),
    };
    let answer = represented(answer, server, view, representations);
    let _ = write!(
        out,
        "    match result {{\n        Ok(({status}, body)) => {answer},\n        Err(refused) => \
         refused_bound{suffix}(&refused),\n    }}\n}}\n"
    );
}

/// Rust code for the JSON text `inner`, an expression of type `String`, under the one `member` of
/// an object: a wrapped body (beyond10x/ess#755). Written inline, so a server whose binding wraps
/// nothing keeps its bytes.
fn wrapped_json(member: &str, inner: &str) -> String {
    format!(
        "{{ let mut wrapped = String::from(\"{{\"); crate::json::member(&mut wrapped, \
         {member:?}); wrapped.push_str(&{inner}); wrapped.push('}}'); wrapped }}"
    )
}

/// Rust code binding `selected`, the representation the query selects (beyond10x/ess#753): its
/// separator, whether a header line comes first, and its headers, or `None` for the JSON answer.
/// A value no representation names, and the parameter given twice, are `malformed`. Written only
/// on a route that declares representations, so every other server keeps its bytes.
pub(super) fn selection(representations: &[ess_http::Representation], suffix: &str) -> String {
    let Some(first) = representations.first() else {
        return String::new();
    };
    let parameter = &first.query.parameter;
    let values: Vec<String> = representations
        .iter()
        .map(|representation| format!("`{}`", representation.query.value))
        .collect();
    let expected = match &values[..] {
        [only] => only.clone(),
        several => format!("one of {}", several.join(", ")),
    };
    let mut arms = String::new();
    for representation in representations {
        let headers: Vec<String> = representation
            .headers
            .iter()
            .filter_map(|(name, header)| match header {
                Header::Literal(text) => Some(format!("({name:?}, {text:?})")),
                Header::Source(_) | Header::Challenge(_) => None,
            })
            .collect();
        let _ = writeln!(
            arms,
            "            Some(json::Value::Text(value)) if value == {:?} => Some(({:?}, {}, &[{}])),",
            representation.query.value,
            representation.encoding.csv.separator.as_char(),
            representation.encoding.csv.header,
            headers.join(", ")
        );
    }
    format!(
        "    // The representation the query selects: its separator, whether a header line comes \
         first, and\n    // its headers; `None` answers the JSON body.\n    let selected: \
         Option<(char, bool, &[(&str, &str)])> =\n        match http::query_object(&request.query, \
         &[({parameter:?}, http::Scalar::Text)], \"query\") {{\n            Err(detail) => return \
         refusal_malformed{suffix}(&detail),\n            Ok(found) => match \
         found.member({parameter:?}) {{\n            None => None,\n{arms}            Some(_) => {{\n                \
         return refusal_malformed{suffix}({:?})\n            }}\n        }},\n    }};\n",
        format!("query.{parameter}: expected {expected}")
    )
}

/// `answer`, the JSON answer, as it stands, or beside the CSV one when the route declares
/// `representations` (beyond10x/ess#753): `selected` chooses between them.
fn represented(
    answer: String,
    server: &Server<'_>,
    view: &ResolvedView,
    representations: &[ess_http::Representation],
) -> String {
    if representations.is_empty() {
        return answer;
    }
    format!(
        "match selected {{\n            None => {answer},\n            Some((separator, header, headers)) => {}\n        }}",
        csv_answer(server, view)
    )
}

/// Whether each field of `view`, in declaration order, is a text cell of its CSV: a `String`, an
/// `Optional<String>` or a newtype of one. Such a cell opening with `=`, `+`, `-`, `@`, a tab or a
/// CR is written behind `'`, so a spreadsheet does not read it as a formula; a number, an enum or
/// a timestamp is written as it is. The Go server's writer reads the same rule.
fn text_cells(ir: &ess_compiler::ir::EssIr, view: &ResolvedView) -> Vec<&'static str> {
    view.fields
        .iter()
        .map(|field| match http::flat(ir, &field.type_ref) {
            Some((http::Shape::One | http::Shape::Optional, http::Flat::String)) => "true",
            _ => "false",
        })
        .collect()
}

/// Rust code answering the rows of `{"rows": [...]}` (`body`, at `status`) as CSV, for the
/// `separator`, `header` and `headers` [`selection`] bound: the view's fields in declaration order
/// under their wire names, CRLF line ends, an absent value an empty field, each value in its flat
/// text spelling, a text cell ([`text_cells`]) opening with `=`, `+`, `-`, `@`, a tab or a CR
/// behind `'`, and a value holding the separator, `"`, CR or LF quoted with `"` doubled.
fn csv_answer(server: &Server<'_>, view: &ResolvedView) -> String {
    let columns: Vec<String> = view
        .fields
        .iter()
        .map(|field| format!("{:?}", ess_gen::schema::wire_field_name(field)))
        .collect();
    let guarded = text_cells(server.ir, view);
    format!(
        "{{\n                let read = json::parse(&body).unwrap_or(json::Value::Null);\n                \
         let columns = [{}];\n                // A text cell opening with a character a spreadsheet reads as a formula is\n                // written behind an apostrophe.\n                let guarded = [{}];\n                let field = |out: &mut String, text: &str| {{\n                    \
         if text.contains([separator, '\"', '\\r', '\\n']) {{\n                        \
         out.push('\"');\n                        out.push_str(&text.replace('\"', \"\\\"\\\"\"));\n                        \
         out.push('\"');\n                    }} else {{\n                        \
         out.push_str(text);\n                    }}\n                }};\n                let mut csv \
         = String::new();\n                if header {{\n                    for (index, column) in \
         columns.iter().enumerate() {{\n                        if index > 0 {{\n                            \
         csv.push(separator);\n                        }}\n                        field(&mut csv, \
         column);\n                    }}\n                    csv.push_str(\"\\r\\n\");\n                \
         }}\n                if let Some(json::Value::Array(rows)) = read.member(\"rows\") {{\n                    \
         for row in rows {{\n                        for (index, column) in \
         columns.iter().enumerate() {{\n                            if index > 0 {{\n                                \
         csv.push(separator);\n                            }}\n                            let text = \
         match row.member(column) {{\n                                None | \
         Some(json::Value::Null) => String::new(),\n                                \
         Some(json::Value::Text(text)) => text.clone(),\n                                \
         Some(value) => http::bare(Some(value)),\n                            }};\n                            \
         let text = if guarded[index]\n                                && text.starts_with(['=', '+', '-', '@', '\\t', '\\r'])\n                            {{\n                                format!(\"'{{text}}\")\n                            }} else {{\n                                text\n                            }};\n                            \
         field(&mut csv, &text);\n                        }}\n                        \
         csv.push_str(\"\\r\\n\");\n                    }}\n                }}\n                let mut \
         answer = http::Response::new(status, \"text/csv; charset=utf-8\", csv);\n                \
         answer.headers.extend(\n                    headers\n                        .iter()\n                        \
         .map(|(name, value)| ((*name).to_owned(), (*value).to_owned())),\n                );\n                \
         answer\n            }}",
        columns.join(", "),
        guarded.join(", ")
    )
}

/// A `single` view's answer: its one row, or `empty` when it holds none, with no body or the
/// one literal member `empty.body` declares.
fn single_answer(empty: &ess_http::Empty) -> String {
    match &empty.body {
        RefusalBody::None => format!("http::single(&body, {})", empty.status),
        RefusalBody::Literal { member, text } => format!(
            "{{\n            let answer = http::single(&body, {status});\n            if \
             answer.status == {status} {{\n                http::Response::new({status}, \
             http::JSON, http::literal_object({member:?}, {text:?}))\n            }} else {{\n                \
             answer\n            }}\n        }}",
            status = empty.status
        ),
    }
}

/// The standard refusals of a bound surface's declared routes, as its binding answers them
/// (`refusals:`), and today's answer for each the binding does not override. A route that
/// overrides one itself (`routes[i].refusals`) answers through its own set of functions, named with
/// [`refusal_suffix`], so every other route keeps its bytes; the shared set is written only where a
/// route answers through it, so the generated server builds with warnings denied.
pub(super) fn refusals(out: &mut String, server: &Server<'_>, routes: &[http::Route<'_>]) {
    let ir = server.ir;
    let Some(binding) = server.binding else {
        return;
    };
    let declared: Vec<&http::Route<'_>> = routes
        .iter()
        .filter(|route| route.declared.is_some())
        .collect();
    let own = |route: &http::Route<'_>| {
        route
            .declared
            .is_some_and(|declared| !declared.refusals.is_empty())
    };
    let checked = |route: &http::Route<'_>| match route.serves {
        Served::Command(_) => http::checks_grants(ir),
        Served::View(handle) => http::read_checked(ir, handle.name()),
    };
    let bounded = |route: &http::Route<'_>| {
        route
            .declared
            .is_some_and(|declared| binding.ir().bounds_request(declared))
    };
    let guarded = |route: &http::Route<'_>| {
        route
            .declared
            .is_some_and(|declared| !binding.credentials_on(declared).is_empty())
    };
    let shared: Vec<&http::Route<'_>> = declared
        .iter()
        .copied()
        .filter(|route| !own(route))
        .collect();
    if declared.is_empty() || !shared.is_empty() {
        refusal_set(
            out,
            &RefusalSet {
                suffix: String::new(),
                on: "a bound route".to_owned(),
                from: "",
                checked: shared.iter().any(|route| checked(route)),
                too_large: shared.iter().any(|route| bounded(route)),
                authenticated: shared.iter().any(|route| guarded(route)),
            },
            |key| binding.refusal(key),
        );
    }
    for route in declared.into_iter().filter(|route| own(route)) {
        let Some(own) = route.declared else {
            continue;
        };
        refusal_set(
            out,
            &RefusalSet {
                suffix: refusal_suffix(server, route),
                on: format!("`{} {}`", route.method.as_str(), route.path),
                from: "routes[].",
                checked: checked(route),
                too_large: bounded(route),
                authenticated: guarded(route),
            },
            |key| binding.refusal_on(own, key),
        );
    }
}

/// One set of standard-refusal functions: their name suffix, the route they answer on, as the
/// docs name it, where the overrides are declared, and whether the route checks a grant.
struct RefusalSet {
    suffix: String,
    on: String,
    from: &'static str,
    checked: bool,
    too_large: bool,
    authenticated: bool,
}

/// The functions answering each standard refusal for `set`: `refusal_malformed`,
/// `refusal_unfinished`, `refusal_not_granted` where a grant is checked, and `refused_bound`, each
/// with `set.suffix`. `lookup` gives the override standing for a key.
fn refusal_set<'a>(
    out: &mut String,
    set: &RefusalSet,
    lookup: impl Fn(RefusalKey) -> Option<&'a ess_http::Refusal>,
) {
    let RefusalSet {
        suffix, on, from, ..
    } = set;
    let _ = write!(
        out,
        "\n/// A request {on} cannot read as its binding declares: `{from}refusals.malformed`, \
         else `400`.\n"
    );
    match lookup(RefusalKey::Malformed) {
        Some(refusal) => {
            let _ = write!(
                out,
                "fn refusal_malformed{suffix}(_detail: &str) -> http::Response {{\n{}}}\n",
                overridden(refusal)
            );
        }
        None => {
            let _ = write!(
                out,
                "fn refusal_malformed{suffix}(detail: &str) -> http::Response {{\n    \
                 http::Response::refusal(400, detail)\n}}\n"
            );
        }
    }
    let _ = write!(
        out,
        "\n/// An unfinished realization on {on}: `{from}refusals.unfinished`, else `501` with \
         `committed`.\n"
    );
    match lookup(RefusalKey::Unfinished) {
        Some(refusal) => {
            let _ = write!(
                out,
                "fn refusal_unfinished{suffix}(_detail: &str, _committed: bool) -> \
                 http::Response {{\n{}}}\n",
                overridden(refusal)
            );
        }
        None => {
            let _ = write!(
                out,
                "fn refusal_unfinished{suffix}(detail: &str, committed: bool) -> http::Response \
                 {{\n    http::Response::unfinished(detail, committed)\n}}\n"
            );
        }
    }
    if set.checked {
        let _ = write!(
            out,
            "\n/// A caller no grant admits, on {on}: `{from}refusals.not_granted`, else the \
             standard `403`.\n"
        );
        match lookup(RefusalKey::NotGranted) {
            Some(refusal) => {
                let _ = write!(
                    out,
                    "fn refusal_not_granted{suffix}(_actor: Option<&str>) -> http::Response \
                     {{\n{}}}\n",
                    overridden(refusal)
                );
            }
            None => {
                let _ = write!(
                    out,
                    "fn refusal_not_granted{suffix}(actor: Option<&str>) -> http::Response {{\n    \
                     not_granted(actor)\n}}\n"
                );
            }
        }
    }
    if set.too_large {
        refusal_too_large(out, set, lookup(RefusalKey::TooLarge));
    }
    if set.authenticated {
        refusal_unauthenticated(out, set, lookup(RefusalKey::Unauthenticated));
    }
    let _ = write!(
        out,
        "\n/// What {on}'s shared path refused, as its binding answers it.\nfn \
         refused_bound{suffix}(refused: &entry::Refused) -> http::Response {{\n    match refused \
         {{\n        entry::Refused::Input(detail) => refusal_malformed{suffix}(detail),\n        \
         entry::Refused::Unmet(_) | entry::Refused::Undelivered(_) => {{\n            \
         refusal_unfinished{suffix}(&refused.to_string(), refused.committed())\n        }}\n        \
         other => http::Response::from(other),\n    }}\n}}\n"
    );
}

/// `refusal_too_large`, with `set.suffix`: the override for a body larger than the cap a route
/// reads, else the default `413` of `http::body_too_large`. Written only where a bounded route
/// answers through `set`.
fn refusal_too_large(out: &mut String, set: &RefusalSet, refusal: Option<&ess_http::Refusal>) {
    let RefusalSet {
        suffix, on, from, ..
    } = set;
    let _ = write!(
        out,
        "\n/// A body larger than the cap {on} reads: `{from}refusals.too_large`, else `413`.\n"
    );
    match refusal {
        Some(refusal) => {
            let _ = write!(
                out,
                "fn refusal_too_large{suffix}(_length: usize, _cap: usize) -> http::Response \
                 {{\n{}}}\n",
                overridden(refusal)
            );
        }
        None => {
            let _ = write!(
                out,
                "fn refusal_too_large{suffix}(length: usize, cap: usize) -> http::Response \
                 {{\n    http::body_too_large(length, cap)\n}}\n"
            );
        }
    }
}

/// `refusal_unauthenticated`, with `set.suffix`: the override for a call without the credential
/// its route requires, else `401` with `{"refused": "unauthenticated"}` and one `WWW-Authenticate`
/// per challenge the route passes (beyond10x/ess#681). Written only where a guarded route answers
/// through `set`.
fn refusal_unauthenticated(
    out: &mut String,
    set: &RefusalSet,
    refusal: Option<&ess_http::Refusal>,
) {
    let RefusalSet {
        suffix, on, from, ..
    } = set;
    let _ = write!(
        out,
        "\n/// A call without the credential {on} requires: `{from}refusals.unauthenticated`, else \
         `401` with\n/// the route's challenges.\n"
    );
    match refusal {
        Some(refusal) => {
            let _ = write!(
                out,
                "fn refusal_unauthenticated{suffix}(_challenges: &[&str]) -> http::Response \
                 {{\n{}}}\n",
                overridden(refusal)
            );
        }
        None => {
            let _ = write!(
                out,
                "fn refusal_unauthenticated{suffix}(challenges: &[&str]) -> http::Response {{\n    \
                 let mut answer = http::Response::refusal({}, {:?});\n    for challenge in \
                 challenges {{\n        answer\n            .headers\n            \
                 .push((\"{}\".to_owned(), (*challenge).to_owned()));\n    }}\n    answer\n}}\n",
                http::UNAUTHENTICATED,
                http::UNAUTHENTICATED_REFUSED,
                ess_http::CHALLENGE_HEADER
            );
        }
    }
}

/// The check a guarded route runs before its grant and before it decodes anything
/// (beyond10x/ess#681): a call authenticated as no actor, or carrying no credential where any one
/// of the route's schemes puts it, is answered `unauthenticated` and runs nothing. Empty for a
/// route the binding leaves open.
fn credential_check(server: &Server<'_>, route: &http::Route<'_>) -> String {
    let (Some(binding), Some(declared)) = (server.binding, route.declared) else {
        return String::new();
    };
    let credentials = binding.credentials_on(declared);
    if credentials.is_empty() {
        return String::new();
    }
    let mut alternatives = Vec::new();
    let mut handshake = false;
    for credential in credentials {
        match credential {
            http::Credential::Authorization(scheme) => {
                alternatives.push(format!("http::authorization(request, {scheme:?})"));
            }
            http::Credential::Header(name) => alternatives.push(format!(
                "http::key_header(request, {:?})",
                name.to_ascii_lowercase()
            )),
            http::Credential::Cookie(name) => {
                alternatives.push(format!("http::key_cookie(request, {name:?})"));
            }
            http::Credential::Handshake => handshake = true,
            http::Credential::Unread => {}
        }
    }
    let condition = match alternatives.len() {
        _ if handshake => "caller.is_none()".to_owned(),
        1 => format!("caller.is_none() || !{}", alternatives[0]),
        _ => format!("caller.is_none() || !({})", alternatives.join(" || ")),
    };
    let challenges: Vec<String> = binding
        .challenges_on(declared)
        .iter()
        .map(|challenge| format!("{challenge:?}"))
        .collect();
    format!(
        "            if {condition} {{\n                return refusal_unauthenticated{}(&[{}]);\n            }}\n",
        refusal_suffix(server, route),
        challenges.join(", ")
    )
}

/// Whether a bound surface answers the standard `403` (`not_granted`) anywhere: on a route the
/// binding does not name that checks a grant, or on a declared one that checks a grant where
/// neither the binding nor the route overrides `not_granted`. Where it does not, the function is
/// not emitted, so the generated server builds with warnings denied.
pub(super) fn answers_standard_not_granted(
    server: &Server<'_>,
    routes: &[http::Route<'_>],
) -> bool {
    let ir = server.ir;
    routes.iter().any(|route| {
        let checked = match route.serves {
            Served::Command(_) => http::checks_grants(ir),
            Served::View(handle) => http::read_checked(ir, handle.name()),
        };
        let overridden = server
            .binding
            .zip(route.declared)
            .is_some_and(|(binding, declared)| {
                binding
                    .refusal_on(declared, RefusalKey::NotGranted)
                    .is_some()
            });
        checked && (route.declared.is_none() || !overridden)
    })
}

/// The body of a function answering one overridden standard refusal: its status, its literal
/// body or none, and its literal headers, every one checked when the binding compiled.
fn overridden(refusal: &ess_http::Refusal) -> String {
    let body = match &refusal.body {
        RefusalBody::None => format!("http::Response::empty({})", refusal.status),
        RefusalBody::Literal { member, text } => format!(
            "http::Response::new({}, http::JSON, http::literal_object({member:?}, {text:?}))",
            refusal.status
        ),
    };
    if refusal.headers.is_empty() {
        return format!("    {body}\n");
    }
    let mut out = format!("    let mut answer = {body};\n");
    for (name, header) in &refusal.headers {
        let value = match header {
            Header::Literal(text) => text.clone(),
            Header::Challenge(challenge) => challenge_text(challenge),
            // A refusal's header never reads a source (`ESS-HTTP-026`).
            Header::Source(_) => continue,
        };
        let _ = writeln!(
            out,
            "    answer.headers.push(({name:?}.to_owned(), {value:?}.to_owned()));"
        );
    }
    out.push_str("    answer\n");
    out
}

/// A challenge whose params are all literals, written as the emitted `http::challenge` writes
/// one: the scheme, then each param as a quoted string with `"` and `\` escaped.
fn challenge_text(challenge: &ess_http::Challenge) -> String {
    let mut out = challenge.scheme.clone();
    let mut first = true;
    for (name, param) in &challenge.params {
        let Param::Literal(text) = param else {
            continue;
        };
        out.push_str(if first { " " } else { ", " });
        first = false;
        let _ = write!(out, "{name}=\"");
        for c in text.chars() {
            if c == '"' || c == '\\' {
                out.push('\\');
            }
            out.push(c);
        }
        out.push('"');
    }
    out
}
