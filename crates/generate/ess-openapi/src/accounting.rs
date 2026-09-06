//! Closed import accounting and replay admission, separate from the legacy structural DTO.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt::Write as _;

use crate::{import, project, Refusal, ServiceInterface, INTERFACE_FORMAT};

/// Persisted import result, including its source and complete adapter accounting.
pub const IMPORT_FORMAT: &str = "ess-openapi-import/1";
/// Fixed interpretation and accounting rules; not an executable normalization recipe.
pub const NORMALIZATION_PROFILE: &str = "ess-openapi-service-subset/1";
/// The only schema dialect admitted by this service-interface adapter.
pub const SCHEMA_DIALECT: &str = "https://spec.openapis.org/oas/3.1/dialect/base";

/// Original UTF-8 source and the bare lowercase SHA-256 of exactly those bytes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImportSource {
    /// Original input, including whitespace and comments.
    pub text: String,
    /// SHA-256 of `text.as_bytes()`, not compiled ESS identity.
    pub sha256: String,
}

/// Closed accounting vocabulary for the fixed normalization profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AccountingCode {
    /// A known annotation was omitted without changing the structural constraints.
    AnnotationOmitted,
    /// A schema keyword is not preserved by the selected variant.
    ConstraintUnpreserved,
    /// An interface or extension feature is not preserved.
    FeatureUnpreserved,
}

/// One omitted annotation or unpreserved semantic feature at its exact source site.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AccountingEntry {
    /// Escaped JSON Pointer into the retained source.
    pub pointer: String,
    /// Meaning fixed by the normalization profile.
    pub code: AccountingCode,
    /// Human-readable explanation, included in canonical accounting.
    pub detail: String,
}

/// An unresolved local reference, preserving each distinct referring site.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UnresolvedReference {
    /// Escaped source pointer to the `$ref` keyword.
    pub pointer: String,
    /// Decoded component-schema name.
    pub target: String,
}

/// Deterministic accounting, admitted only as part of a replay-checked import result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImportAccounting {
    /// Known omitted annotations, sorted and deduplicated by typed field order.
    pub normalizations: Vec<AccountingEntry>,
    /// Unpreserved semantic features, sorted and deduplicated by typed field order.
    pub coverage_gaps: Vec<AccountingEntry>,
    /// Missing targets, retaining distinct source sites in sorted order.
    pub unresolved_references: Vec<UnresolvedReference>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct WireImport {
    format: String,
    normalization: String,
    source: ImportSource,
    schema_dialect: String,
    interface: ServiceInterface,
    accounting: ImportAccounting,
}

/// A successful import or replay-checked persisted result. Construction and mutation are private.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportReport {
    wire: WireImport,
}

impl ImportReport {
    pub(crate) fn new(
        text: &str,
        interface: ServiceInterface,
        accounting: ImportAccounting,
    ) -> Self {
        let sha256 = Sha256::digest(text.as_bytes()).iter().fold(
            String::with_capacity(64),
            |mut output, byte| {
                write!(output, "{byte:02x}").expect("writing a String cannot fail");
                output
            },
        );
        Self {
            wire: WireImport {
                format: IMPORT_FORMAT.to_owned(),
                normalization: NORMALIZATION_PROFILE.to_owned(),
                source: ImportSource {
                    text: text.to_owned(),
                    sha256,
                },
                schema_dialect: SCHEMA_DIALECT.to_owned(),
                interface,
                accounting,
            },
        }
    }

    /// Read-only structural interface; extracting a clone does not transfer an accounting claim.
    pub fn interface(&self) -> &ServiceInterface {
        &self.wire.interface
    }

    /// Source retained by this exact import.
    pub fn source(&self) -> &ImportSource {
        &self.wire.source
    }

    /// Durable accounting derived from the retained source.
    pub fn accounting(&self) -> &ImportAccounting {
        &self.wire.accounting
    }

    /// Canonical typed pretty JSON with one final LF.
    pub fn to_canonical_json(&self) -> String {
        let mut output = serde_json::to_string_pretty(&self.wire).expect("typed import serializes");
        output.push('\n');
        output
    }
}

/// Reads a closed import envelope and reimports its retained source to check all derived facts.
/// Legacy interface-only documents have unavailable accounting and require original-source reimport.
pub fn read_import(text: &str) -> Result<ImportReport, Vec<Refusal>> {
    // YAML's Value/Mapping visitor refuses duplicate keys recursively, including JSON input.
    // Do this before any typed BTreeMap or JSON Value could erase a repeated map entry.
    let value = strict_value(text)?;
    if value.get("format").and_then(serde_yaml::Value::as_str) == Some(INTERFACE_FORMAT) {
        return Err(vec![Refusal::new(
            "/format",
            "legacy service-interface accounting unavailable; reimport the original OpenAPI source",
        )]);
    }
    let wire: WireImport = serde_yaml::from_value(value).map_err(|error| {
        vec![Refusal::new(
            "/",
            format!("malformed import envelope: {error}"),
        )]
    })?;
    for (pointer, actual, expected) in [
        ("/format", wire.format.as_str(), IMPORT_FORMAT),
        (
            "/normalization",
            wire.normalization.as_str(),
            NORMALIZATION_PROFILE,
        ),
        (
            "/schema_dialect",
            wire.schema_dialect.as_str(),
            SCHEMA_DIALECT,
        ),
    ] {
        if actual != expected {
            return Err(vec![Refusal::new(
                pointer,
                format!("expected `{expected}`, found `{actual}`"),
            )]);
        }
    }
    let replay = import(&wire.source.text)?;
    if replay.wire != wire {
        return Err(vec![Refusal::new(
            "/",
            "import envelope differs from replay of its retained source, identity or accounting",
        )]);
    }
    Ok(replay)
}

/// Projects a checked import only when every semantic feature and reference is accounted for.
/// Known annotation omissions alone do not block structural projection.
pub fn project_import(report: &ImportReport) -> Result<String, Vec<Refusal>> {
    let accounting = report.accounting();
    let mut refusals: Vec<_> = accounting
        .coverage_gaps
        .iter()
        .map(|gap| {
            Refusal::new(
                &gap.pointer,
                format!(
                    "unpreserved semantics prevent checked projection: {}",
                    gap.detail
                ),
            )
        })
        .collect();
    refusals.extend(accounting.unresolved_references.iter().map(|reference| {
        Refusal::new(
            &reference.pointer,
            format!(
                "unresolved local interface type `{}` prevents checked projection",
                reference.target
            ),
        )
    }));
    if !refusals.is_empty() {
        return Err(refusals);
    }
    project(report.interface())
}

pub(crate) fn strict_value(text: &str) -> Result<serde_yaml::Value, Vec<Refusal>> {
    let value: serde_yaml::Value = serde_yaml::from_str(text)
        .map_err(|error| vec![Refusal::new("/", format!("malformed document: {error}"))])?;
    check_json_shape(&value, "")?;
    Ok(value)
}

fn check_json_shape(value: &serde_yaml::Value, pointer: &str) -> Result<(), Vec<Refusal>> {
    match value {
        serde_yaml::Value::Mapping(mapping) => {
            for (key, child) in mapping {
                let key = key
                    .as_str()
                    .ok_or_else(|| vec![Refusal::new(pointer, "object keys must be strings")])?;
                check_json_shape(child, &format!("{pointer}/{}", crate::pointer_escape(key)))?;
            }
        }
        serde_yaml::Value::Sequence(values) => {
            for (index, child) in values.iter().enumerate() {
                check_json_shape(child, &format!("{pointer}/{index}"))?;
            }
        }
        serde_yaml::Value::Number(number)
            if number.as_f64().is_some_and(|value| !value.is_finite()) =>
        {
            return Err(vec![Refusal::new(
                pointer,
                "non-finite numbers are outside JSON",
            )]);
        }
        serde_yaml::Value::Tagged(_) => {
            return Err(vec![Refusal::new(
                pointer,
                "YAML tags are outside the JSON document shape",
            )])
        }
        _ => {}
    }
    Ok(())
}
