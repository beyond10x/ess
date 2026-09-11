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
        accessor_preflight(original)?;
        let value = Json::parse(original, "$suite")?;
        validate_suite(&value)?;
        let suite: ConformanceSuite = serde_json::from_str(original)
            .map_err(|e| AdmissionError::new("InvalidSuite", "$suite", e.to_string()))?;
        self::suite(&suite)?;
        payload_agrees_with_its_shape(&suite)?;
        entity_setup(&suite)?;
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
    if !matches!(version.major(), 1..=9) {
        return Err(p["suite_version"].error(
            "UnsupportedSuiteVersion",
            "execution readers admit suite majors 1–9",
        ));
    }
    if matches!(version.major(), 5 | 7 | 9) != root.contains_key("coverage") {
        return Err(value.error(
            "InvalidCoverage",
            "coverage is required exactly for suite/5, suite/7 and suite/9",
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
fn values(value: &Json, major: u32, accessors: bool) -> Result<(), AdmissionError> {
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
            "observed_selection" => {
                if major < 6 || !accessors {
                    return Err(v.error(
                        "UnsupportedVocabulary",
                        "observed_selection requires suite/6 or /7 invocation input",
                    ));
                }
                let fields = v.closed(&["kind", "event", "selection"], &[])?;
                if fields["selection"].raw.len() > 2_097_152 {
                    return Err(v.error(
                        "SelectionResource",
                        "selection observation exceeds byte bound",
                    ));
                }
                let _: crate::selection::Observation =
                    serde_json::from_str(&fields["selection"].raw)
                        .map_err(|error| v.error("InvalidSelection", error.to_string()))?;
                crate::quoted_predicate_format::admit_selection(&fields["selection"].raw, major)?;
            }
            "observed_accessor" => {
                if major < 6 || !accessors {
                    return Err(v.error(
                        "UnsupportedVocabulary",
                        "observed_accessor requires suite/6 or /7 invocation input",
                    ));
                }
                let fields = v.closed(&["kind", "event", "accessor"], &[])?;
                let raw = fields["accessor"].closed(&["plan", "target", "types"], &[])?;
                if raw["plan"].raw.len() > ess_domain::accessor::MAX_BYTES * 2 {
                    return Err(v.error(
                        "AccessorResource",
                        "serialized accessor exceeds admission byte bound",
                    ));
                }
                let _: crate::accessor::Observation = serde_json::from_str(&fields["accessor"].raw)
                    .map_err(|e| v.error("InvalidAccessor", e.to_string()))?;
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
            values(&f["fields"], major, false)?;
        }
        "satisfies" => {
            let f = value.closed(&["expect", "predicate"], &[])?;
            f["predicate"].payload()?;
            crate::quoted_predicate_format::admit_predicate(&f["predicate"].raw, major)?;
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
                values(v, major, false)?;
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
        || (major < 6 && matches!(tag, "establish_entity" | "expect_reading_order"))
        || (major < 8 && tag == "expect_response_payload")
    {
        return Err(value.error("UnsupportedVocabulary", "step requires a newer suite major"));
    }
    let (required, optional): (&[&str], &[&str]) = match tag {
        "establish_entity" => (
            &["step", "instance", "entity", "identity", "fields", "state"],
            &[],
        ),
        "expect_reading_order" => (&["step", "left", "right", "order"], &[]),
        "expect_response_payload" => (&["step", "response"], &[]),
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
        "check_periodic" if major >= 6 => (&["step", "check"], &[]),
        "expect_quiet" => (&["step", "event", "instant", "elapsed"], &[]),
        "expect_halt" | "eventually_halt" => (&["step", "view", "after"], &["params"]),
        _ => return Err(value.error("UnsupportedStep", tag)),
    };
    for (key, field) in value.closed(required, optional)? {
        match key.as_str() {
            "response" if tag == "expect_response_payload" => {
                let _: crate::response::Observation = serde_json::from_str(&field.raw)
                    .map_err(|error| field.error("InvalidResponse", error.to_string()))?;
            }
            "left" | "right" if tag == "expect_reading_order" => {
                let reference: crate::reading::ReadingReference = serde_json::from_str(&field.raw)
                    .map_err(|error| field.error("InvalidReading", error.to_string()))?;
                reference
                    .validate()
                    .map_err(|error| field.error("InvalidReading", &error))?;
            }
            "force" | "outcome" => {
                field.closed(&["command", "outcome"], &[])?;
            }
            "input" | "params" => values(field, major, tag == "expect_invocation")?,
            "identity" => field.payload()?,
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

fn response_payloads(suite: &ConformanceSuite) -> Result<(), AdmissionError> {
    for step in suite
        .scenarios
        .values()
        .flat_map(|scenario| &scenario.steps)
    {
        if let ScenarioStep::ExpectResponsePayload { response } = step {
            if suite.provenance.suite_version.major() < 8 {
                return Err(AdmissionError::new(
                    "UnsupportedVocabulary",
                    "$suite",
                    "response payload observations require suite/8 or /9",
                ));
            }
            response
                .validate()
                .map_err(|error| AdmissionError::new("InvalidResponse", "$suite", error))?;
        }
    }
    Ok(())
}

/// Check directly constructed suites before artifact creation or target effects.
pub fn suite(suite: &ConformanceSuite) -> Result<(), AdmissionError> {
    crate::quoted_predicate_format::admit_suite(suite)?;
    response_payloads(suite)?;
    entity_setup(suite)?;
    crate::periodic::admit_suite(suite)?;
    for scenario in suite.scenarios.values() {
        for step in &scenario.steps {
            if let ScenarioStep::ExpectReadingOrder { left, right, .. } = step {
                if suite.provenance.suite_version.major() < 6 {
                    return Err(AdmissionError::new(
                        "UnsupportedVocabulary",
                        "$suite",
                        "clock reading requires suite/6 or /7",
                    ));
                }
                for reference in [left, right] {
                    reference
                        .validate()
                        .map_err(|error| AdmissionError::new("InvalidReading", "$suite", error))?;
                }
            }
        }
    }
    let mut accessor_bytes = 0_usize;
    for scenario in suite.scenarios.values() {
        for step in &scenario.steps {
            if let ScenarioStep::ExpectInvocation { input, .. } = step {
                for value in input.values() {
                    if let crate::ScenarioValue::ObservedSelection { selection, .. } = value {
                        if suite.provenance.suite_version.major() < 6 {
                            return Err(AdmissionError::new(
                                "UnsupportedVocabulary",
                                "$suite",
                                "observed_selection requires suite/6 or /7",
                            ));
                        }
                        selection.validate().map_err(|error| {
                            AdmissionError::new("InvalidSelection", "$suite", error)
                        })?;
                        accessor_bytes =
                            accessor_bytes.saturating_add(selection.bytes().map_err(|error| {
                                AdmissionError::new("SelectionResource", "$suite", error)
                            })?);
                        if accessor_bytes > 16 * 1024 * 1024 {
                            return Err(AdmissionError::new(
                                "SelectionResource",
                                "$suite",
                                "serialized projection plans exceed 16777216 bytes",
                            ));
                        }
                    }
                    if let crate::ScenarioValue::ObservedAccessor { accessor, .. } = value {
                        if suite.provenance.suite_version.major() < 6 {
                            return Err(AdmissionError::new(
                                "UnsupportedVocabulary",
                                "$suite",
                                "observed_accessor requires suite/6 or /7",
                            ));
                        }
                        accessor_bytes =
                            accessor_bytes.saturating_add(accessor.bytes().map_err(|e| {
                                AdmissionError::new("AccessorResource", "$suite", e)
                            })?);
                        if accessor_bytes > 16 * 1024 * 1024 {
                            return Err(AdmissionError::new(
                                "AccessorResource",
                                "$suite",
                                "serialized accessor plans exceed 16777216 bytes",
                            ));
                        }
                    }
                }
            }
        }
    }
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

/// Setup-specific structural admission; model constraints remain the target's obligation.
pub(crate) fn entity_setup(suite: &ConformanceSuite) -> Result<(), AdmissionError> {
    for (id, scenario) in &suite.scenarios {
        if !scenario
            .steps
            .iter()
            .any(|step| matches!(step, ScenarioStep::EstablishEntity { .. }))
        {
            continue;
        }
        if suite.provenance.suite_version.major() < 6 {
            return Err(AdmissionError::new(
                "UnsupportedVocabulary",
                "/provenance/suite_version",
                "entity setup requires suite/6 or suite/7",
            ));
        }
        let mut instances = std::collections::BTreeSet::new();
        let mut identities = Vec::new();
        let mut asserted = false;
        for (index, step) in scenario.steps.iter().enumerate() {
            let path = format!("/scenarios/{}/steps/{index}", escape(&id.to_string()));
            match step {
                ScenarioStep::EstablishEntity {
                    instance,
                    entity,
                    identity,
                    fields,
                    ..
                } => {
                    asserted = false;
                    setup_literal(identity, 0, &path)?;
                    for value in fields.values() {
                        setup_literal(value, 0, &path)?;
                    }
                    if !instances.insert(instance) || identities.contains(&(entity, identity)) {
                        return Err(AdmissionError::new(
                            "DuplicateEntitySetup",
                            path,
                            "duplicate instance or qualified identity",
                        ));
                    }
                    if matches!(identity, Node::Null) {
                        return Err(AdmissionError::new(
                            "InvalidEntitySetup",
                            path,
                            "identity cannot be null",
                        ));
                    }
                    if fields.keys().any(|key| {
                        !key.starts_with(|character: char| character.is_ascii_alphabetic())
                            || !key.chars().all(|character| {
                                character.is_ascii_alphanumeric() || character == '_'
                            })
                    }) {
                        return Err(AdmissionError::new(
                            "InvalidEntitySetup",
                            path,
                            "field names must be local identifiers",
                        ));
                    }
                    identities.push((entity, identity));
                }
                ScenarioStep::CaptureInstance { instance, .. } => {
                    if !instances.insert(instance) {
                        return Err(AdmissionError::new(
                            "DuplicateEntitySetup",
                            path,
                            "duplicate instance binding",
                        ));
                    }
                }
                ScenarioStep::ExpectView { .. }
                | ScenarioStep::EventuallyView { .. }
                | ScenarioStep::ExpectOutcome { .. }
                | ScenarioStep::ExpectError { .. }
                | ScenarioStep::ExpectEvent { .. }
                | ScenarioStep::ExpectNoEvent { .. }
                | ScenarioStep::EventuallyEvent { .. }
                | ScenarioStep::ExpectInvocation { .. }
                | ScenarioStep::ExpectNotBefore { .. }
                | ScenarioStep::ExpectWithin { .. }
                | ScenarioStep::ExpectQuiet { .. }
                | ScenarioStep::ExpectHalt { .. }
                | ScenarioStep::EventuallyHalt { .. } => asserted = true,
                _ => {}
            }
        }
        if !asserted {
            return Err(AdmissionError::new(
                "InvalidEntitySetup",
                format!("/scenarios/{}", escape(&id.to_string())),
                "setup must be followed by an assertion",
            ));
        }
    }
    Ok(())
}

fn setup_literal(value: &Node, depth: u16, path: &str) -> Result<(), AdmissionError> {
    if depth > 120 {
        return Err(AdmissionError::new(
            "InvalidEntitySetup",
            path,
            "setup literal nesting exceeds 120",
        ));
    }
    match value {
        Node::Number(number) if !number.get().is_finite() => Err(AdmissionError::new(
            "InvalidEntitySetup",
            path,
            "setup numbers must be finite",
        )),
        Node::Seq(values) => {
            for child in values {
                setup_literal(child, depth + 1, path)?;
            }
            Ok(())
        }
        Node::Map(fields) => {
            for child in fields.values() {
                setup_literal(child, depth + 1, path)?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
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

/// Borrow raw new plans before the structural JSON tree clones any of their contents.
fn accessor_preflight(original: &str) -> Result<(), AdmissionError> {
    use serde::Deserialize;
    use serde_json::value::RawValue;
    #[derive(Deserialize)]
    struct Document<'a> {
        #[serde(borrow)]
        scenarios: std::collections::BTreeMap<String, &'a RawValue>,
    }
    #[derive(Deserialize)]
    struct Scenario<'a> {
        #[serde(borrow)]
        steps: Vec<&'a RawValue>,
    }
    #[derive(Deserialize)]
    struct Step<'a> {
        #[serde(borrow)]
        input: Option<std::collections::BTreeMap<String, &'a RawValue>>,
    }
    #[derive(Deserialize)]
    struct Value<'a> {
        kind: String,
        #[serde(borrow)]
        accessor: Option<&'a RawValue>,
        #[serde(borrow)]
        selection: Option<&'a RawValue>,
    }
    // The normal admitter owns malformed-document diagnostics and all legacy semantics.
    let Ok(document) = serde_json::from_str::<Document<'_>>(original) else {
        return Ok(());
    };
    let mut bytes = 0_usize;
    for scenario in document.scenarios.values() {
        let Ok(scenario) = serde_json::from_str::<Scenario<'_>>(scenario.get()) else {
            continue;
        };
        for step in scenario.steps {
            let Ok(step) = serde_json::from_str::<Step<'_>>(step.get()) else {
                continue;
            };
            for value in step.input.iter().flat_map(|input| input.values()) {
                let Ok(value) = serde_json::from_str::<Value<'_>>(value.get()) else {
                    continue;
                };
                if value.kind != "observed_accessor" && value.kind != "observed_selection" {
                    continue;
                }
                if let Some(accessor) = value.accessor.or(value.selection) {
                    let size = accessor.get().len();
                    bytes = bytes.saturating_add(size);
                    if size > 2_097_152 || bytes > 16_777_216 {
                        return Err(AdmissionError::new(
                            "AccessorResource",
                            "$suite",
                            "serialized accessor input exceeds its byte budget",
                        ));
                    }
                }
            }
        }
    }
    Ok(())
}
