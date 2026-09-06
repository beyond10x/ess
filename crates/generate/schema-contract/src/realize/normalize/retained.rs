//! Lexical capture and the explicit retained-document decoding boundary.

use super::{finding, NumberPath, Refused};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde::de::{self, MapAccess, Visitor};
use serde::Deserializer as _;
use serde_json::value::RawValue;

pub(super) fn require_text(paths: &[Vec<NumberPath>]) -> Result<(), Refused> {
    if paths.is_empty() {
        return Ok(());
    }
    Err(error(
        "/input",
        "input_capture_provenance",
        "raw JSON capture requires original token bytes; use a text input entrypoint",
    ))
}

pub(super) fn decode(encoded: &str) -> Result<String, Refused> {
    let bytes = STANDARD
        .decode(encoded)
        .ok()
        .filter(|bytes| STANDARD.encode(bytes) == encoded)
        .ok_or_else(|| {
            error(
                "/input",
                "input_base64",
                "expected canonical standard base64",
            )
        })?;
    String::from_utf8(bytes)
        .map_err(|_| error("/input", "input_utf8", "retained JSON bytes must be UTF-8"))
}

pub(super) fn capture(raw: &RawValue, at: &str, depth: usize) -> Result<String, Refused> {
    validate(
        raw,
        at,
        depth,
        "captured JSON token contains invalid Unicode",
    )?;
    Ok(STANDARD.encode(raw.get().as_bytes()))
}

// RawValue has already validated grammar with serde_json's iterative ignore_value
// scanner. This bounded walk checks strings and depth without parsing any number.
// Object visitation is deliberately sequential: a later invalid key cannot hide
// the first child's depth/Unicode failure. Duplicate keys are never collected.
pub(super) fn validate(
    raw: &RawValue,
    at: &str,
    depth: usize,
    detail: &str,
) -> Result<(), Refused> {
    if depth > 64 {
        return Err(error(at, "input_depth", "JSON input exceeds 64 levels"));
    }
    match raw.get().as_bytes()[0] {
        b'{' => {
            struct Object<'a> {
                at: &'a str,
                depth: usize,
                detail: &'a str,
                failure: &'a mut Option<Refused>,
            }
            impl<'de> Visitor<'de> for Object<'_> {
                type Value = ();
                fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    formatter.write_str("a captured JSON object")
                }
                fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<(), A::Error> {
                    while map.next_key::<String>()?.is_some() {
                        let raw = map.next_value::<Box<RawValue>>()?;
                        if let Err(error) = validate(&raw, self.at, self.depth + 1, self.detail) {
                            *self.failure = Some(error);
                            return Err(de::Error::custom("captured token refused"));
                        }
                    }
                    Ok(())
                }
            }
            let mut failure = None;
            let result = serde_json::Deserializer::from_str(raw.get()).deserialize_map(Object {
                at,
                depth,
                detail,
                failure: &mut failure,
            });
            result.map_err(|_| failure.unwrap_or_else(|| unicode(at, detail)))
        }
        b'[' => {
            let values: Vec<Box<RawValue>> =
                serde_json::from_str(raw.get()).map_err(|_| unicode(at, detail))?;
            for value in values {
                validate(&value, at, depth + 1, detail)?;
            }
            Ok(())
        }
        b'"' => serde_json::from_str::<String>(raw.get())
            .map(|_| ())
            .map_err(|_| unicode(at, detail)),
        _ => Ok(()),
    }
}

fn unicode(at: &str, detail: &str) -> Refused {
    error(at, "input_syntax", detail)
}

fn error(at: &str, rule: &str, detail: &str) -> Refused {
    Refused(vec![finding(at, rule, detail)])
}
