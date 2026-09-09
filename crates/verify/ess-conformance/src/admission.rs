//! Original-byte admission for the frozen suite/1–4 execution vocabulary.
use crate::count_json::Json;
use crate::{scenario::SuiteFormat, ConformanceSuite, Holds, ScenarioStep};
use ess_domain::Primitive;
use ess_primitives::node::Node;
use sha2::{Digest, Sha256};
use std::fmt;
use std::fmt::Write;

/// A precise refusal at a persisted document boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmissionError {
    /// All observed structural issues.
    pub issues: Vec<AdmissionIssue>,
}
/// One refused field or document invariant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmissionIssue {
    /// Stable refusal classification.
    pub reason: &'static str,
    /// Location in the input.
    pub path: String,
    /// Explanation of the refused value.
    pub detail: String,
}
impl AdmissionError {
    pub(crate) fn new(
        reason: &'static str,
        path: impl Into<String>,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            issues: vec![AdmissionIssue {
                reason,
                path: path.into(),
                detail: detail.into(),
            }],
        }
    }
}
impl fmt::Display for AdmissionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for issue in &self.issues {
            write!(f, "{}: {}: {}; ", issue.path, issue.reason, issue.detail)?;
        }
        Ok(())
    }
}
impl std::error::Error for AdmissionError {}

/// Refuses a step whose payload carries a value its own declared shape does not admit.
///
/// A step declares both — `payload` says what a field holds, `shape` says what its declared type
/// permits there — and until this ran nothing compared them, so a suite could assert an event
/// carries the `Uuid` `"x"` and be admitted by every reader (review finding F08,
/// `docs/design/review-primitive-semantics.md`).
///
/// **Only the fields the payload names.** A shape is a claim about the declaration and a payload is
/// a partial claim about values; requiring every declared leaf to be present would refuse the
/// suites this repository already writes, which is a different rule from the one being fixed. The
/// browser adapter's `primitiveAdmits` applies the identical rule to the identical document, so the
/// two admitters cannot disagree about one suite.
fn payload_agrees_with_its_shape(suite: &ConformanceSuite) -> Result<(), AdmissionError> {
    for (id, scenario) in &suite.scenarios {
        for (position, step) in scenario.steps.iter().enumerate() {
            let (ScenarioStep::ExpectEvent { payload, shape, .. }
            | ScenarioStep::EventuallyEvent { payload, shape, .. }) = step
            else {
                continue;
            };
            for (field, leaf) in shape.leaves() {
                let Some(value) = payload.get(field) else {
                    continue;
                };
                if matches!(value, Node::Null) || leaf.holds.admits(value) {
                    continue;
                }
                return Err(AdmissionError::new(
                    "InvalidSuite",
                    format!("$suite.scenarios.{id}.steps.{position}.payload.{field}"),
                    format!(
                        "the payload holds {} and the step's own shape declares {}",
                        crate::report::quote(value),
                        leaf.holds
                    ),
                ));
            }
        }
    }
    Ok(())
}

/// An immutable original suite and its admitted execution value.
#[derive(Debug, Clone)]
pub struct AdmittedSuite {
    original: String,
    suite: ConformanceSuite,
    digest: String,
    coverage: Option<crate::coverage::Inventory>,
}
impl AdmittedSuite {
    /// Admit the original UTF-8 document before any target activity.
    pub fn from_json(original: &str) -> Result<Self, AdmissionError> {
        let admitted = Self::parse(original)?;
        if admitted
            .coverage
            .as_ref()
            .is_some_and(|c| matches!(c.selection.filter, crate::coverage::Filter::Explicit { .. }))
        {
            return Err(AdmissionError::new(
                "MissingParent",
                "$suite.coverage.selection.filter",
                "explicit suite requires its complete admitted input carrier",
            ));
        }
        Ok(admitted)
    }
    pub(crate) fn parse(original: &str) -> Result<Self, AdmissionError> {
        let value = Json::parse(original, "$suite")?;
        validate_suite(&value)?;
        let suite: ConformanceSuite = serde_json::from_str(original)
            .map_err(|e| AdmissionError::new("InvalidSuite", "$suite", e.to_string()))?;
        payload_agrees_with_its_shape(&suite)?;
        let coverage = value
            .object()?
            .get("coverage")
            .map(|c| crate::coverage::parse_inventory(c, &suite))
            .transpose()?;
        let digest = Sha256::digest(original.as_bytes()).iter().fold(
            "sha256:".to_owned(),
            |mut text, byte| {
                write!(text, "{byte:02x}").expect("writing to String");
                text
            },
        );
        Ok(Self {
            original: original.into(),
            suite,
            digest,
            coverage,
        })
    }
    /// Serialize once, then execute the value admitted from those exact bytes.
    pub fn from_suite(suite: &ConformanceSuite) -> Result<Self, AdmissionError> {
        Self::from_json(&suite.to_canonical_json()?)
    }
    /// The entire unchanged document, including its final newline.
    pub fn original_json(&self) -> &str {
        &self.original
    }
    /// The admitted, immutable execution value.
    pub fn suite(&self) -> &ConformanceSuite {
        &self.suite
    }
    /// SHA-256 identity under sha256-json-bytes/1.
    pub fn digest(&self) -> &str {
        &self.digest
    }
    /// Immutable admitted inventory; historical suites have no coverage claim.
    pub fn coverage(&self) -> Option<&crate::coverage::Inventory> {
        self.coverage.as_ref()
    }
}

fn validate_suite(value: &Json) -> Result<(), AdmissionError> {
    let root = value.closed(&["provenance", "scenarios"], &["coverage"])?;
    let p = root["provenance"].closed(
        &[
            "suite_version",
            "system",
            "specification_version",
            "spec_digest",
            "contract_digest",
        ],
        &["component"],
    )?;
    let version = SuiteFormat::parse(p["suite_version"].text()?)
        .map_err(|e| p["suite_version"].error("UnsupportedSuiteVersion", e.to_string()))?;
    if !matches!(version.major(), 1..=5) {
        return Err(p["suite_version"].error(
            "UnsupportedSuiteVersion",
            "execution readers admit suite majors 1–5",
        ));
    }
    if (version.major() == 5) != root.contains_key("coverage") {
        return Err(value.error(
            "InvalidCoverage",
            "coverage is required exactly for suite/5",
        ));
    }
    for scenario in root["scenarios"].object()?.values() {
        let s = scenario.closed(&["purpose", "steps", "source"], &[])?;
        for step in s["steps"].array()? {
            step_value(step, version.major())?;
        }
        for source in s["source"].array()? {
            semantic_reference(source)?;
        }
    }
    Ok(())
}
pub(crate) fn semantic_reference(value: &Json) -> Result<(), AdmissionError> {
    let fields = value.closed(&["kind", "name"], &[])?;
    match fields["kind"].text()? {
        "outcome" => {
            fields["name"].closed(&["command", "outcome"], &[])?;
        }
        "transition" => {
            fields["name"].closed(&["entity", "transition"], &[])?;
        }
        _ => {}
    }
    Ok(())
}
fn values(value: &Json) -> Result<(), AdmissionError> {
    for v in value.object()?.values() {
        let object = v.object()?;
        let tag = object
            .get("kind")
            .ok_or_else(|| v.error("MissingField", "kind"))?
            .text()?;
        match tag {
            "literal" => {
                let f = v.closed(&["kind", "value"], &[])?;
                f["value"].payload()?;
            }
            "instance" => {
                v.closed(&["kind", "instance"], &[])?;
            }
            "observed" => {
                v.closed(&["kind", "event", "field"], &[])?;
            }
            _ => return Err(v.error("UnsupportedScenarioValue", tag)),
        }
    }
    Ok(())
}
fn shape(value: &Json) -> Result<(), AdmissionError> {
    for v in value.object()?.values() {
        let object = v.object()?;
        let tag = object
            .get("holds")
            .ok_or_else(|| v.error("MissingField", "holds"))?
            .text()?;
        let required: &[&str] = match tag {
            "primitive" => &["holds", "kind"],
            "enum" => &["holds", "variants"],
            "list" | "map" | "union" => &["holds"],
            _ => return Err(v.error("UnsupportedHolds", tag)),
        };
        let fields = v.closed(required, &["optional"])?;
        if let Some(optional) = fields.get("optional") {
            optional.boolean()?;
        }
    }
    Ok(())
}
fn expectation(value: &Json, major: u32) -> Result<(), AdmissionError> {
    let object = value.object()?;
    let tag = object
        .get("expect")
        .ok_or_else(|| value.error("MissingField", "expect"))?
        .text()?;
    if major < 2 && matches!(tag, "counts" | "at") {
        return Err(value.error("UnsupportedVocabulary", "expectation requires suite/2"));
    }
    match tag {
        "contains" | "excludes" => {
            let f = value.closed(&["expect", "fields"], &[])?;
            values(&f["fields"])?;
        }
        "satisfies" => {
            let f = value.closed(&["expect", "predicate"], &[])?;
            f["predicate"].payload()?;
        }
        "counts" => {
            value.closed(&["expect"], &["at_least", "at_most"])?;
        }
        "ranked" => {
            value.closed(&["expect", "order_by"], &[])?;
        }
        "at" => {
            let f = value.closed(&["expect", "order_by", "position"], &["fields"])?;
            let p = f["position"].object()?;
            let tag = p
                .get("row")
                .ok_or_else(|| value.error("MissingField", "row"))?
                .text()?;
            f["position"].closed(
                if tag == "nth" {
                    &["row", "index"]
                } else {
                    &["row"]
                },
                &[],
            )?;
            if let Some(v) = f.get("fields") {
                values(v)?;
            }
        }
        _ => return Err(value.error("UnsupportedViewExpectation", tag)),
    }
    Ok(())
}
fn step_value(value: &Json, major: u32) -> Result<(), AdmissionError> {
    let object = value.object()?;
    let tag = object
        .get("step")
        .ok_or_else(|| value.error("MissingField", "step"))?
        .text()?;
    if (major < 3
        && matches!(
            tag,
            "mark_instant" | "expect_not_before" | "expect_within" | "expect_quiet"
        ))
        || (major < 4 && matches!(tag, "expect_halt" | "eventually_halt"))
    {
        return Err(value.error("UnsupportedVocabulary", "step requires a newer suite major"));
    }
    let (required, optional): (&[&str], &[&str]) = match tag {
        "configure_external_outcome" => (&["step", "force"], &[]),
        "execute_command" => (&["step", "command"], &["actor", "input"]),
        "expect_outcome" => (&["step", "outcome"], &[]),
        "expect_error" => (&["step", "error"], &["fields"]),
        "expect_event" | "eventually_event" => (&["step", "event"], &["payload", "shape"]),
        "expect_no_event" | "redeliver_event" => (&["step", "event"], &[]),
        "capture_instance" => (&["step", "instance", "entity", "event", "field"], &[]),
        "expect_invocation" => (&["step", "binding", "command"], &["input"]),
        "query_view" => (&["step", "view"], &["params"]),
        "expect_view" => (&["step", "view", "expectation"], &[]),
        "eventually_view" => (&["step", "view", "expectation"], &["params"]),
        "mark_instant" => (&["step", "instant"], &[]),
        "expect_not_before" | "expect_within" => (&["step", "instant", "elapsed"], &[]),
        "expect_quiet" => (&["step", "event", "instant", "elapsed"], &[]),
        "expect_halt" | "eventually_halt" => (&["step", "view", "after"], &["params"]),
        _ => return Err(value.error("UnsupportedStep", tag)),
    };
    for (key, field) in value.closed(required, optional)? {
        match key.as_str() {
            "force" | "outcome" => {
                field.closed(&["command", "outcome"], &[])?;
            }
            "input" | "params" => values(field)?,
            "fields" | "payload" => {
                field.object()?;
                field.payload()?;
            }
            "shape" => shape(field)?,
            "expectation" => expectation(field, major)?,
            _ => {}
        }
    }
    Ok(())
}

/// Check the model even when synthesis would omit unsupported fields or whole scenarios.
pub fn model(ir: &ess_compiler::EssIr) -> Result<(), AdmissionError> {
    checked(ess_compiler::binary64::locations(ir).into_iter().collect())
}

/// Check directly constructed suites before artifact creation or target effects.
pub fn suite(suite: &ConformanceSuite) -> Result<(), AdmissionError> {
    let mut found = Vec::new();
    for (id, scenario) in &suite.scenarios {
        for (index, step) in scenario.steps.iter().enumerate() {
            if let ScenarioStep::ExpectEvent { shape, .. }
            | ScenarioStep::EventuallyEvent { shape, .. } = step
            {
                for (name, leaf) in shape.leaves() {
                    if matches!(
                        leaf.holds,
                        Holds::Primitive {
                            kind: Primitive::Binary64
                        }
                    ) {
                        found.push(format!(
                            "/scenarios/{}/steps/{index}/shape/{}",
                            escape(&id.to_string()),
                            escape(name)
                        ));
                    }
                }
            }
        }
    }
    checked(found)
}

fn escape(value: &str) -> String {
    value.replace('~', "~0").replace('/', "~1")
}
fn checked(locations: Vec<String>) -> Result<(), AdmissionError> {
    if locations.is_empty() {
        Ok(())
    } else {
        Err(AdmissionError {
            issues: locations.into_iter().map(|path| AdmissionIssue {
                reason: "UnsupportedPrimitive",
                path,
                detail: "finite Binary64 is not admitted by the current conformance suite and codecs".into(),
            }).collect(),
        })
    }
}

#[allow(clippy::trivially_copy_pass_by_ref)] // Serde serialize_with requires a borrowed field.
pub(crate) fn serialize_primitive<S: serde::Serializer>(
    primitive: &Primitive,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    if *primitive == Primitive::Binary64 {
        return Err(serde::ser::Error::custom(
            "Binary64 is not admitted by this conformance suite format",
        ));
    }
    serde::Serialize::serialize(primitive, serializer)
}
pub(crate) fn deserialize_primitive<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Primitive, D::Error> {
    let primitive = <Primitive as serde::Deserialize>::deserialize(deserializer)?;
    if primitive == Primitive::Binary64 {
        return Err(serde::de::Error::custom(
            "Binary64 is not admitted by this conformance suite format",
        ));
    }
    Ok(primitive)
}
