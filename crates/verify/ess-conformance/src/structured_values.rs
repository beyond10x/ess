//! Instance references inside a list, a map value or a struct member (suite/32 and /33,
//! beyond10x/ess#242).
//!
//! An authored step may write `{$instance: name}` wherever the declared type at that position is
//! the instance's identity type — `ring_sequence: [{$instance: a}, {$instance: b}]` for a
//! `List<ReleaseRingId>` — and not only as a whole input field. The identity is known only when
//! the scenario runs, so the value cannot be a [`Literal`](crate::ScenarioValue::Literal): it is a
//! [`List`](crate::ScenarioValue::List) or [`Members`](crate::ScenarioValue::Members) whose
//! elements are values of their own, and the runner resolves each one before it sends the whole.
//!
//! Only a structured value that holds a reference is written this way; one that holds none stays
//! the literal it always was, so a suite without such a reference keeps its bytes and its format.
//! Inside one, an element is a literal, an instance, or another list or members value — nothing
//! else, and no deeper than [`MAX_DEPTH`].
//!
//! Two new value kinds are words an older reader does not know: the Rust reader fails on an
//! unknown variant, and a reader that skipped the kind would send `{"kind": "list", …}` to the
//! target as a value. So a suite carrying one says so in its first line: suite/[`ORDINARY`], and
//! suite/[`COVERAGE`] where it also carries a coverage inventory. Each implies every major below
//! it. The generated Go and TypeScript runners read suites up to
//! `ess-conformance/27`, so generating either for such a suite is refused, naming the Rust runner.

use std::collections::BTreeMap;

use crate::admission::AdmissionError;
use crate::scenario::{ConformanceSuite, ScenarioValue};

/// First ordinary suite that carries structured values holding references.
pub const ORDINARY: u32 = 32;
/// The corresponding inventory-bearing suite.
pub const COVERAGE: u32 = 33;

/// What an older envelope reports for this vocabulary.
pub const REQUIRES: &str = "instance references inside a list or mapping require suite/32 or /33";

/// How deep one structured value may nest, counting the outermost as one.
pub const MAX_DEPTH: usize = 64;

/// Whether this value is one of the two structured kinds.
fn structured(value: &ScenarioValue) -> bool {
    matches!(
        value,
        ScenarioValue::List { .. } | ScenarioValue::Members { .. }
    )
}

/// Whether any step of the suite carries a structured value anywhere it carries scenario values.
pub fn used_by(suite: &ConformanceSuite) -> bool {
    suite.scenarios.values().any(|scenario| {
        scenario.steps.iter().any(|step| {
            let mut found = false;
            crate::now_offset::visit(step, &mut |value| found |= structured(value));
            found
        })
    })
}

/// Refuse an explicitly pinned older format carrying a structured value, and one whose elements
/// are anything but literals, instances and structured values, before serialization or target
/// effects.
pub(crate) fn admit(suite: &ConformanceSuite) -> Result<(), AdmissionError> {
    if !used_by(suite) {
        return Ok(());
    }
    if suite.provenance.suite_version.major() < ORDINARY {
        return Err(AdmissionError::new(
            "UnsupportedVocabulary",
            "$suite",
            REQUIRES,
        ));
    }
    let mut refused = None;
    for scenario in suite.scenarios.values() {
        for step in &scenario.steps {
            crate::now_offset::visit(step, &mut |value| {
                if refused.is_none() && structured(value) {
                    refused = elements(value, 1).err();
                }
            });
        }
    }
    refused.map_or(Ok(()), Err)
}

/// What a structured value's elements may be, and how deep it may nest.
fn elements(value: &ScenarioValue, depth: usize) -> Result<(), AdmissionError> {
    if depth > MAX_DEPTH {
        return Err(AdmissionError::new(
            "InvalidStructuredValue",
            "$suite",
            format!("a structured value nests deeper than {MAX_DEPTH}"),
        ));
    }
    let children: Vec<&ScenarioValue> = match value {
        ScenarioValue::List { items } => items.iter().collect(),
        ScenarioValue::Members { members } => members.values().collect(),
        _ => Vec::new(),
    };
    children.into_iter().try_for_each(|child| match child {
        ScenarioValue::Literal { .. } | ScenarioValue::Instance { .. } => Ok(()),
        ScenarioValue::List { .. } | ScenarioValue::Members { .. } => elements(child, depth + 1),
        _ => Err(AdmissionError::new(
            "InvalidStructuredValue",
            "$suite",
            "a structured value holds only literals, instances and structured values",
        )),
    })
}

/// Resolves a structured value by resolving each element with `leaf`, which answers every kind a
/// structured value's elements may be other than the two structured ones.
pub(crate) fn resolve(
    value: &ScenarioValue,
    leaf: &dyn Fn(&ScenarioValue) -> Result<ess_primitives::node::Node, String>,
) -> Result<ess_primitives::node::Node, String> {
    use ess_primitives::node::Node;
    match value {
        ScenarioValue::List { items } => items
            .iter()
            .map(|item| resolve(item, leaf))
            .collect::<Result<Vec<_>, _>>()
            .map(Node::Seq),
        ScenarioValue::Members { members } => members
            .iter()
            .map(|(key, member)| resolve(member, leaf).map(|node| (key.clone(), node)))
            .collect::<Result<BTreeMap<_, _>, _>>()
            .map(Node::Map),
        other => leaf(other),
    }
}
