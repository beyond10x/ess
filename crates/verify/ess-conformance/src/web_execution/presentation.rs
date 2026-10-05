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
/// The identity a completed run is displayed under (design section 8): which admitted bytes it
/// executed, in which namespace, under which nonce and presentation generation. None of it is a
/// target observation, so none of it can carry a protected value.
pub(super) struct Receipt<'a> {
    /// Fresh namespace given to `Ids` and to the installation.
    pub namespace: &'a str,
    /// Run nonce, lowercase hexadecimal.
    pub nonce: &'a str,
    /// Presentation generation the run was requested under.
    pub generation: u32,
    /// Exact admitted selected suite digest.
    pub selected_digest: &'a str,
    /// Every retained original parent digest, nearest first.
    pub parent_digests: Vec<&'a str>,
    /// Bare SHA-256 of the exact execution input bytes held.
    pub input_digest: &'a str,
    /// The manifest's bare SHA-256 of every original source document, in declared order.
    pub source_digests: Vec<&'a str>,
    /// The installed implementation's identity, as its report names it.
    pub implementation: &'a str,
    /// The runtime build that executed it.
    pub runtime: &'a str,
}
impl Budget {
    /// Count one display node at `depth` exactly as [`Budget::value`] counts each value.
    fn node(&mut self, depth: usize) -> Result<()> {
        if depth > MAX_DEPTH || self.nodes >= MAX_NODES {
            return Err(Error::ResourceLimit);
        }
        self.nodes += 1;
        Ok(())
    }
}
/// Display the Runner's already sanitized canonical run under its receipt. The wrapper, the
/// receipt and the run share one node and depth budget, the one the player checks the whole
/// completed display against.
pub(super) fn completed(receipt: &Receipt<'_>, raw: &str) -> Result<String> {
    encode(&completed_value(receipt, raw)?)
}
fn completed_value(receipt: &Receipt<'_>, raw: &str) -> Result<DisplayValue> {
    let mut budget = Budget { nodes: 0 };
    budget.node(0)?;
    budget.node(1)?;
    let mut text = |value: &str| -> Result<DisplayValue> {
        budget.node(2)?;
        Ok(DisplayValue::Text(value.to_owned()))
    };
    let state = text("completed")?;
    let namespace = text(receipt.namespace)?;
    let nonce = text(receipt.nonce)?;
    let selected = text(receipt.selected_digest)?;
    let input = text(receipt.input_digest)?;
    let implementation = text(receipt.implementation)?;
    let runtime = text(receipt.runtime)?;
    budget.node(2)?;
    let generation = DisplayValue::NumberText(receipt.generation.to_string());
    let mut list = |values: &[&str]| -> Result<DisplayValue> {
        budget.node(2)?;
        values
            .iter()
            .map(|value| {
                budget.node(3)?;
                Ok(DisplayValue::Text((*value).to_owned()))
            })
            .collect::<Result<_>>()
            .map(DisplayValue::List)
    };
    let parents = list(&receipt.parent_digests)?;
    let sources = list(&receipt.source_digests)?;
    let receipt = DisplayValue::Object(vec![
        ("state".into(), state),
        ("namespace".into(), namespace),
        ("nonce".into(), nonce),
        ("generation".into(), generation),
        ("selected_digest".into(), selected),
        ("parent_digests".into(), parents),
        ("input_digest".into(), input),
        ("source_digests".into(), sources),
        ("implementation".into(), implementation),
        ("runtime".into(), runtime),
    ]);
    let run = budget.value(raw, 1)?;
    Ok(DisplayValue::Object(vec![
        ("receipt".into(), receipt),
        ("run".into(), run),
    ]))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The completed display is checked by the player as one value, so its receipt wrapper counts
    /// against the same depth and node budget as the run inside it.
    #[test]
    fn completed_display_counts_its_receipt_wrapper_in_the_same_budget() {
        let receipt = Receipt {
            namespace: "browser-ns",
            nonce: "00",
            generation: 0,
            selected_digest: "sha256:00",
            parent_digests: vec!["sha256:01"],
            input_digest: "02",
            source_digests: vec!["03", "04"],
            implementation: "target 1",
            runtime: "runtime",
        };
        let nested = |depth: usize| format!("{}null{}", "[".repeat(depth), "]".repeat(depth));
        // The run sits at depth 1 under the wrapper: 1,023 more levels fit, 1,024 do not.
        assert!(completed(&receipt, &nested(MAX_DEPTH - 1)).is_ok());
        assert!(matches!(
            completed(&receipt, &nested(MAX_DEPTH)),
            Err(Error::ResourceLimit)
        ));
        // Wrapper 1, receipt 1, eight scalar fields, two lists and their three entries: 15 nodes.
        let entries = MAX_NODES - 15 - 1;
        let fits = format!("[{}]", vec!["null"; entries].join(","));
        assert!(completed_value(&receipt, &fits).is_ok());
        let over = format!("[{}]", vec!["null"; entries + 1].join(","));
        assert!(matches!(
            completed_value(&receipt, &over),
            Err(Error::ResourceLimit)
        ));
    }

    /// Section 4: exactly 1,000,000 display nodes are presented; one more is a resource limit.
    #[test]
    fn display_node_budget_admits_its_exact_bound_and_refuses_one_over() {
        for (entries, admitted) in [(MAX_NODES - 1, true), (MAX_NODES, false)] {
            let raw = format!("[{}]", vec!["null"; entries].join(","));
            let display = Budget { nodes: 0 }.value(&raw, 0);
            if admitted {
                let DisplayValue::List(values) = display.unwrap() else {
                    panic!("a list")
                };
                assert_eq!(values.len() + 1, MAX_NODES);
            } else {
                assert!(matches!(display, Err(Error::ResourceLimit)));
            }
        }
    }

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
