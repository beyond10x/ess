//! Lossless typed display data. Never used to reconstruct target inputs.
use super::{
    bundle::{BlobRef, Execution},
    Error, Result,
};
use serde::Serialize;
use serde_json::value::RawValue;
use std::collections::BTreeMap;

/// Display schema identity.
pub const FORMAT: &str = "ess-conformance-browser-presentation/1";
/// Bounded rendering envelope, independent of native suite admission.
pub const MAX_BYTES: usize = 32 * 1024 * 1024;
const MAX_NODES: usize = 1_000_000;
const MAX_DEPTH: usize = 1024;

/// Semantic numbers are text, never JavaScript Number values.
#[derive(Debug, Serialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum DisplayValue {
    /// Explicit null.
    Null,
    /// Boolean scalar.
    Bool(bool),
    /// Text scalar.
    Text(String),
    /// Exact numeric token.
    NumberText(String),
    /// Ordered values.
    List(Vec<Self>),
    /// Deterministically ordered named values.
    Object(Vec<(String, Self)>),
}
#[derive(Serialize)]
struct Card {
    id: String,
    declaration: DisplayValue,
}
#[derive(Serialize)]
struct Document<'a> {
    format: &'static str,
    selected_digest: &'a str,
    parent_digests: Vec<&'a str>,
    sources: &'a [BlobRef],
    model: DisplayValue,
    suite: DisplayValue,
    scenarios: Vec<Card>,
}
struct Budget {
    nodes: usize,
}
impl Budget {
    fn value(&mut self, raw: &str, depth: usize) -> Result<DisplayValue> {
        // Traversing admitted nesting on the call stack overflowed before the logical bound.
        // Finish tasks assemble children in order; RawValue skips each nested body iteratively.
        enum Task {
            Value(String, usize),
            List(usize),
            Object(Vec<String>),
        }
        let mut tasks = vec![Task::Value(raw.to_owned(), depth)];
        let mut complete = Vec::new();
        while let Some(task) = tasks.pop() {
            match task {
                Task::List(count) => {
                    let children = complete.split_off(complete.len() - count);
                    complete.push(DisplayValue::List(children));
                }
                Task::Object(keys) => {
                    let children = complete.split_off(complete.len() - keys.len());
                    complete.push(DisplayValue::Object(
                        keys.into_iter().zip(children).collect(),
                    ));
                }
                Task::Value(raw, depth) => {
                    if depth > MAX_DEPTH || self.nodes >= MAX_NODES {
                        return Err(Error::ResourceLimit);
                    }
                    self.nodes += 1;
                    let raw = raw.trim();
                    match raw.as_bytes().first() {
                        Some(b'{') => {
                            let fields: BTreeMap<String, Box<RawValue>> =
                                serde_json::from_str(raw).map_err(|_| Error::InternalFailure)?;
                            let (keys, values): (Vec<_>, Vec<_>) = fields.into_iter().unzip();
                            tasks.push(Task::Object(keys));
                            tasks.extend(
                                values
                                    .into_iter()
                                    .rev()
                                    .map(|value| Task::Value(value.get().to_owned(), depth + 1)),
                            );
                        }
                        Some(b'[') => {
                            let values: Vec<Box<RawValue>> =
                                serde_json::from_str(raw).map_err(|_| Error::InternalFailure)?;
                            tasks.push(Task::List(values.len()));
                            tasks.extend(
                                values
                                    .into_iter()
                                    .rev()
                                    .map(|value| Task::Value(value.get().to_owned(), depth + 1)),
                            );
                        }
                        Some(b'"') => complete.push(DisplayValue::Text(
                            serde_json::from_str(raw).map_err(|_| Error::InternalFailure)?,
                        )),
                        Some(b't' | b'f') => complete.push(DisplayValue::Bool(
                            serde_json::from_str(raw).map_err(|_| Error::InternalFailure)?,
                        )),
                        Some(b'n') if raw == "null" => complete.push(DisplayValue::Null),
                        Some(b'-' | b'0'..=b'9') => {
                            complete.push(DisplayValue::NumberText(raw.into()));
                        }
                        _ => return Err(Error::InternalFailure),
                    }
                }
            }
        }
        complete.pop().ok_or(Error::InternalFailure)
    }
}
fn encode(value: &impl Serialize) -> Result<String> {
    let text = format!(
        "{}\n",
        serde_json::to_string_pretty(value).map_err(|_| Error::InternalFailure)?
    );
    if text.len() > MAX_BYTES {
        return Err(Error::ResourceLimit);
    }
    Ok(text)
}
/// Present the complete compiled model and admitted scenario declarations once.
pub(super) fn document(
    ir: &ess_compiler::EssIr,
    execution: &Execution,
    sources: &[BlobRef],
) -> Result<String> {
    let mut budget = Budget { nodes: 0 };
    let selected = execution.selected();
    let model = budget.value(&ir.to_canonical_json(), 0)?;
    let mut header: BTreeMap<String, Box<RawValue>> =
        serde_json::from_str(selected.original_json()).map_err(|_| Error::InternalFailure)?;
    let scenarios = header.remove("scenarios").ok_or(Error::InternalFailure)?;
    let scenarios: BTreeMap<String, Box<RawValue>> =
        serde_json::from_str(scenarios.get()).map_err(|_| Error::InternalFailure)?;
    let suite = budget.value(
        &serde_json::to_string(&header).map_err(|_| Error::InternalFailure)?,
        0,
    )?;
    let cards = scenarios
        .into_iter()
        .map(|(id, raw)| {
            if id.len() > 4096 {
                return Err(Error::ResourceLimit);
            }
            Ok(Card {
                id,
                declaration: budget.value(raw.get(), 0)?,
            })
        })
        .collect::<Result<_>>()?;
    encode(&Document {
        format: FORMAT,
        selected_digest: selected.digest(),
        parent_digests: execution
            .parents()
            .iter()
            .map(crate::AdmittedSuite::digest)
            .collect(),
        sources,
        model,
        suite,
        scenarios: cards,
    })
}
/// Display only the Runner's already sanitized canonical report.
pub(super) fn report(raw: &str) -> Result<String> {
    encode(&Budget { nodes: 0 }.value(raw, 0)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn producer_counts_logical_values_before_serde_display_wrappers() {
        // RawValue skips nested bodies iteratively; no global serde recursion override is needed.
        // Execute on the ordinary test thread: increasing its stack would conceal a product risk.
        for object in [false, true] {
            for depth in [1024, 1025] {
                let mut raw = "null".to_owned();
                for _ in 0..depth {
                    raw = if object {
                        format!("{{\"entry\":{raw}}}")
                    } else {
                        format!("[{raw}]")
                    };
                }
                let display = Budget { nodes: 0 }.value(&raw, 0);
                if depth == 1024 {
                    let value = display.unwrap();
                    let encoded = encode(&value).unwrap();
                    assert!(encoded.len() <= MAX_BYTES);
                    assert_eq!(encoded.len(), if object { 25_229_333 } else { 10_527_765 });
                    drop(value);
                } else {
                    assert!(matches!(display, Err(Error::ResourceLimit)));
                }
            }
        }
    }
}
