//! Known failures: declared, bound to one exact execution, and accounted apart from conformance.
//!
//! `docs/design/mutation-scope-and-known-failures.md` (beyond10x/ess#294, beyond10x/ess#296). A
//! retrofit describes the behaviour a system is meant to have, and the system does not always have
//! it yet. Three closed documents let an author say so without changing what conformance means:
//!
//! - [`Declaration`], `ess-known-failures/1`: an external list of exact scenario IDs that fail
//!   against one implementation build of one suite of one specification, each with the reason and
//!   the record that tracks its repair. It is never a marker on an authored outcome, and it never
//!   makes a run pass.
//! - [`ExecutionContext`], `ess-conformance-execution/1`: the host's statement of which public build
//!   produced one exact report of one exact suite. It carries digests, the report's own
//!   implementation label and the build, and nothing else: no values, no timestamps, no commands.
//! - [`Accounting`], `ess-known-failure-accounting/1`: the original report's `failed` scenarios,
//!   partitioned into the declared ones and the unexpected ones, beside the original terminal
//!   counts. It has no conformance field: the ordinary report keeps its verdict.
//!
//! Every identity is exact. A declaration whose suite bytes, specification, implementation label
//! or build differ from the run it is applied to is refused, as is a declared scenario that passed
//! (stale), ended any other way than `failed`, or is not in the suite. A failure the declaration
//! does not name is never absorbed by it.

use std::collections::BTreeSet;
use std::fmt;
use std::fmt::Write as _;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::counts::ScenarioCounts;
use crate::{AdmittedSuite, CountReport};

/// The declaration family this build reads.
pub const DECLARATION_FORMAT: &str = "ess-known-failures/1";
/// The execution-context family this build reads and writes.
pub const EXECUTION_FORMAT: &str = "ess-conformance-execution/1";
/// The accounting family this build reads and writes.
pub const ACCOUNTING_FORMAT: &str = "ess-known-failure-accounting/1";
/// The execution context's conventional name beside a `report.json`.
pub const EXECUTION_FILE: &str = "execution.json";

/// `sha256:` and the lowercase hexadecimal SHA-256 of `bytes`: the identity every digest field here
/// carries, the suite's `sha256-json-bytes/1` included.
pub fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .fold("sha256:".to_owned(), |mut text, byte| {
            write!(text, "{byte:02x}").expect("writing to a String");
            text
        })
}

/// Whether `text` is `sha256:` and exactly 64 lowercase hexadecimal digits.
pub fn is_sha256(text: &str) -> bool {
    text.strip_prefix("sha256:").is_some_and(|hex| {
        hex.len() == 64
            && hex
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    })
}

// ---- refusals -----------------------------------------------------------------------------------

/// What kind of refusal a [`Refusal`] is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RefusalKind {
    /// The document is not of its format: unreadable, an unknown or duplicate key, a wrong
    /// `format`, an invalid digest, an empty or unordered list, an empty string.
    Malformed,
    /// The specification, suite bytes, report bytes, implementation label or build identity is not
    /// the one the document is bound to.
    Identity,
    /// A declared scenario the suite does not hold.
    UnknownScenario,
    /// A declared scenario that passed: the declaration is stale.
    Stale,
    /// A declared scenario that ended `error`, `unsupported` or `skipped`: only a `failed` scenario
    /// can be a known failure, and an execution error is never exempted.
    NotFailed,
    /// A declaration supplied after emission that the emission did not bind, or one whose bytes
    /// differ from the bytes it bound.
    Unbound,
    /// An accounting document that does not partition its report as the declaration says.
    Accounting,
}

impl RefusalKind {
    /// Its stable name, the first word of every refusal message.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Malformed => "known-failures.malformed",
            Self::Identity => "known-failures.identity",
            Self::UnknownScenario => "known-failures.unknown-scenario",
            Self::Stale => "known-failures.stale",
            Self::NotFailed => "known-failures.not-failed",
            Self::Unbound => "known-failures.unbound",
            Self::Accounting => "known-failures.accounting",
        }
    }
}

/// Why a declaration, execution context or accounting document was refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refusal {
    /// What kind.
    pub kind: RefusalKind,
    /// What, exactly.
    pub detail: String,
}

impl Refusal {
    /// A refusal of `kind`.
    pub fn new(kind: RefusalKind, detail: impl Into<String>) -> Self {
        Self {
            kind,
            detail: detail.into(),
        }
    }
}

impl fmt::Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.kind.as_str(), self.detail)
    }
}

impl std::error::Error for Refusal {}

fn malformed(what: &str, detail: impl fmt::Display) -> Refusal {
    Refusal::new(RefusalKind::Malformed, format!("{what}: {detail}"))
}

/// Reads `text` as one JSON object of `format`, refusing duplicate keys, a different `format`
/// (named before any other field is read) and, through `T`, unknown fields.
fn closed<T: serde::de::DeserializeOwned>(
    text: &str,
    format: &str,
    what: &str,
) -> Result<T, Refusal> {
    crate::count_json::Json::parse(text, "$").map_err(|error| malformed(what, error))?;
    let value: serde_json::Value =
        serde_json::from_str(text).map_err(|error| malformed(what, error))?;
    match value.get("format").and_then(serde_json::Value::as_str) {
        Some(declared) if declared == format => {}
        Some(other) => {
            return Err(malformed(
                what,
                format!("`format` is `{other}`, not `{format}`"),
            ))
        }
        None => return Err(malformed(what, format!("no `format`; it is `{format}`"))),
    }
    serde_json::from_str(text).map_err(|error| malformed(what, error))
}

fn digest_field(what: &str, field: &str, value: &str) -> Result<(), Refusal> {
    if is_sha256(value) {
        Ok(())
    } else {
        Err(malformed(
            what,
            format!("`{field}` is `{value}`, not `sha256:` and 64 lowercase hexadecimal digits"),
        ))
    }
}

/// Two-space JSON, keys in declaration order (which is sorted), and one trailing LF.
fn canonical(value: &impl Serialize) -> String {
    let mut text = serde_json::to_string_pretty(value).expect("the document serializes");
    text.push('\n');
    text
}

// ---- the declaration ----------------------------------------------------------------------------

/// One declared known failure. Fields are declared in key order.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KnownFailure {
    /// Why it fails: the author's words, never fetched or interpreted.
    pub reason: String,
    /// The exact scenario ID: no wildcard, prefix, command-wide or outcome-wide match.
    pub scenario: String,
    /// The record that tracks its repair: an issue or equivalent, never fetched.
    pub tracking: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct WireDeclaration {
    failures: Vec<KnownFailure>,
    format: String,
    implementation: String,
    implementation_build: String,
    spec_digest: String,
    suite_digest: String,
}

/// An admitted `ess-known-failures/1` document and the exact bytes it was read from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Declaration {
    wire: WireDeclaration,
    original: String,
    digest: String,
}

impl Declaration {
    /// Reads a declaration from its original bytes, refusing a document that is not closed and
    /// well formed. Nothing is compared to a run here: see [`admit`](Self::admit) and
    /// [`bind`](Self::bind).
    pub fn from_json(text: &str) -> Result<Self, Refusal> {
        const WHAT: &str = "ess-known-failures/1";
        let wire: WireDeclaration = closed(text, DECLARATION_FORMAT, WHAT)?;
        ess_primitives::evidence::SpecDigest::new(wire.spec_digest.clone())
            .map_err(|error| malformed(WHAT, format!("`spec_digest`: {error}")))?;
        digest_field(WHAT, "suite_digest", &wire.suite_digest)?;
        digest_field(WHAT, "implementation_build", &wire.implementation_build)?;
        if wire.implementation.is_empty() {
            return Err(malformed(WHAT, "`implementation` is empty"));
        }
        if wire.failures.is_empty() {
            return Err(malformed(
                WHAT,
                "`failures` is empty; a declaration names at least one failing scenario",
            ));
        }
        for failure in &wire.failures {
            for (field, value) in [
                ("scenario", &failure.scenario),
                ("reason", &failure.reason),
                ("tracking", &failure.tracking),
            ] {
                if value.trim().is_empty() {
                    return Err(malformed(
                        WHAT,
                        format!("a failure's `{field}` is empty: `{}`", failure.scenario),
                    ));
                }
            }
        }
        if let Some(pair) = wire
            .failures
            .windows(2)
            .find(|pair| pair[0].scenario >= pair[1].scenario)
        {
            return Err(malformed(
                WHAT,
                format!(
                    "`failures` must be in scenario ID order with no ID twice: `{}` then `{}`",
                    pair[0].scenario, pair[1].scenario
                ),
            ));
        }
        Ok(Self {
            wire,
            original: text.to_owned(),
            digest: sha256(text.as_bytes()),
        })
    }

    /// Refuses a declaration of another specification or other suite bytes than `admitted`, or one
    /// naming a scenario `admitted` does not hold.
    pub fn admit(&self, admitted: &AdmittedSuite) -> Result<(), Refusal> {
        let provenance = &admitted.suite().provenance;
        if self.wire.spec_digest != provenance.spec_digest.as_str() {
            return Err(Refusal::new(
                RefusalKind::Identity,
                format!(
                    "the declaration is bound to specification `{}`, and the suite is of `{}`",
                    self.wire.spec_digest, provenance.spec_digest
                ),
            ));
        }
        if self.wire.suite_digest != admitted.digest() {
            return Err(Refusal::new(
                RefusalKind::Identity,
                format!(
                    "the declaration is bound to suite bytes `{}`, and the suite is `{}`; a \
                     still-failing scenario of a changed suite does not inherit its declaration",
                    self.wire.suite_digest,
                    admitted.digest()
                ),
            ));
        }
        let held: BTreeSet<String> = admitted
            .suite()
            .scenarios
            .keys()
            .map(ToString::to_string)
            .collect();
        if let Some(unknown) = self
            .wire
            .failures
            .iter()
            .find(|failure| !held.contains(&failure.scenario))
        {
            return Err(Refusal::new(
                RefusalKind::UnknownScenario,
                format!(
                    "`{}` is not a scenario of the suite `{}`",
                    unknown.scenario,
                    admitted.digest()
                ),
            ));
        }
        Ok(())
    }

    /// Refuses a declaration of another implementation label or another public build than the
    /// ones that produced the run it is applied to. `build` is the host's own identity of the
    /// build, fixed before the target ran, and never this declaration's.
    pub fn bind(&self, implementation: &str, build: &str) -> Result<(), Refusal> {
        if self.wire.implementation != implementation {
            return Err(Refusal::new(
                RefusalKind::Identity,
                format!(
                    "the declaration is bound to implementation `{}`, and the run was answered \
                     by `{implementation}`",
                    self.wire.implementation
                ),
            ));
        }
        self.bind_build(build)
    }

    /// Refuses a declaration of another public build than `build`, the host's own identity of the
    /// build, fixed before any target ran.
    pub fn bind_build(&self, build: &str) -> Result<(), Refusal> {
        if self.wire.implementation_build != build {
            return Err(Refusal::new(
                RefusalKind::Identity,
                format!(
                    "the declaration is bound to implementation build `{}`, and the run's build \
                     is `{build}`",
                    self.wire.implementation_build
                ),
            ));
        }
        Ok(())
    }

    /// Checks each declared scenario against what a run reported for it: `status(id)` answers
    /// `passed`, `failed`, `error`, `unsupported` or `skipped`, or `None` where the run has no
    /// such scenario. Only `failed` is admitted.
    pub fn check_statuses(
        &self,
        status: impl Fn(&str) -> Option<&'static str>,
    ) -> Result<(), Refusal> {
        for failure in &self.wire.failures {
            match status(&failure.scenario) {
                Some("failed") => {}
                Some("passed") => {
                    return Err(Refusal::new(
                        RefusalKind::Stale,
                        format!(
                            "`{}` passed, so its declaration ({}) is stale; remove it",
                            failure.scenario, failure.tracking
                        ),
                    ))
                }
                Some(other) => {
                    return Err(Refusal::new(
                        RefusalKind::NotFailed,
                        format!(
                            "`{}` ended `{other}`, and only a `failed` scenario can be a known \
                             failure",
                            failure.scenario
                        ),
                    ))
                }
                None => {
                    return Err(Refusal::new(
                        RefusalKind::UnknownScenario,
                        format!("`{}` has no result in the run", failure.scenario),
                    ))
                }
            }
        }
        Ok(())
    }

    /// The declared failures, in scenario ID order.
    pub fn failures(&self) -> &[KnownFailure] {
        &self.wire.failures
    }

    /// The declared failure of `scenario`, if there is one.
    pub fn get(&self, scenario: &str) -> Option<&KnownFailure> {
        self.wire
            .failures
            .binary_search_by(|failure| failure.scenario.as_str().cmp(scenario))
            .ok()
            .map(|index| &self.wire.failures[index])
    }

    /// The implementation label it is bound to.
    pub fn implementation(&self) -> &str {
        &self.wire.implementation
    }

    /// The public build it is bound to.
    pub fn implementation_build(&self) -> &str {
        &self.wire.implementation_build
    }

    /// The specification digest it is bound to.
    pub fn spec_digest(&self) -> &str {
        &self.wire.spec_digest
    }

    /// The suite-byte digest it is bound to.
    pub fn suite_digest(&self) -> &str {
        &self.wire.suite_digest
    }

    /// The exact bytes it was read from.
    pub fn original(&self) -> &str {
        &self.original
    }

    /// The SHA-256 of [`original`](Self::original).
    pub fn digest(&self) -> &str {
        &self.digest
    }
}

// ---- the execution context ----------------------------------------------------------------------

/// `ess-conformance-execution/1`: which public build produced one exact report of one exact suite.
///
/// Declared host provenance, not remote attestation: a consumer trusts the host that produced the
/// result, as it trusts the report. Fields are declared in key order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionContext {
    format: String,
    implementation: String,
    implementation_build: String,
    report_digest: String,
    suite_digest: String,
}

impl ExecutionContext {
    /// The context of `report_text`, a report of `admitted` that names `implementation`, produced
    /// by the build `build` the host fixed before the run.
    pub fn new(
        report_text: &str,
        admitted: &AdmittedSuite,
        implementation: &str,
        build: &str,
    ) -> Result<Self, Refusal> {
        digest_field(EXECUTION_FORMAT, "implementation_build", build)?;
        Ok(Self {
            format: EXECUTION_FORMAT.to_owned(),
            implementation: implementation.to_owned(),
            implementation_build: build.to_owned(),
            report_digest: sha256(report_text.as_bytes()),
            suite_digest: admitted.digest().to_owned(),
        })
    }

    /// Reads a context, refusing a document that is not closed and well formed.
    pub fn from_json(text: &str) -> Result<Self, Refusal> {
        let context: Self = closed(text, EXECUTION_FORMAT, EXECUTION_FORMAT)?;
        digest_field(
            EXECUTION_FORMAT,
            "implementation_build",
            &context.implementation_build,
        )?;
        digest_field(EXECUTION_FORMAT, "report_digest", &context.report_digest)?;
        digest_field(EXECUTION_FORMAT, "suite_digest", &context.suite_digest)?;
        Ok(context)
    }

    /// Refuses a context of other report bytes, other suite bytes, or another implementation label
    /// than `report_text`, a report/2 of `admitted`, names.
    pub fn admit(&self, report_text: &str, admitted: &AdmittedSuite) -> Result<(), Refusal> {
        let report = CountReport::from_json(report_text, admitted)
            .map_err(|error| malformed("ess-conformance-report/2", error))?;
        let digest = sha256(report_text.as_bytes());
        if self.report_digest != digest {
            return Err(Refusal::new(
                RefusalKind::Identity,
                format!(
                    "the execution context is of report bytes `{}`, and the report is `{digest}`",
                    self.report_digest
                ),
            ));
        }
        if self.suite_digest != admitted.digest() {
            return Err(Refusal::new(
                RefusalKind::Identity,
                format!(
                    "the execution context is of suite bytes `{}`, and the suite is `{}`",
                    self.suite_digest,
                    admitted.digest()
                ),
            ));
        }
        if self.implementation != report.implementation() {
            return Err(Refusal::new(
                RefusalKind::Identity,
                format!(
                    "the execution context names implementation `{}`, and the report names `{}`",
                    self.implementation,
                    report.implementation()
                ),
            ));
        }
        Ok(())
    }

    /// The public build the host fixed before the run.
    pub fn implementation_build(&self) -> &str {
        &self.implementation_build
    }

    /// The report's implementation label.
    pub fn implementation(&self) -> &str {
        &self.implementation
    }

    /// Two-space JSON with sorted keys and one trailing LF.
    pub fn to_canonical_json(&self) -> String {
        canonical(self)
    }
}

// ---- the accounting -----------------------------------------------------------------------------

/// `ess-known-failure-accounting/1`: a report's failures, partitioned by a declaration.
///
/// `known_failed` and `unexpected_failed` together are exactly the report's `failed` scenarios;
/// `counts` are the report's own, unchanged, so a known failure is never a pass and never leaves
/// the total. There is no conformance field. Fields are declared in key order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Accounting {
    /// The report's original terminal counts.
    pub counts: ScenarioCounts,
    /// The SHA-256 of the declaration's original bytes.
    pub declaration_digest: String,
    /// [`ACCOUNTING_FORMAT`].
    pub format: String,
    /// The report's implementation label.
    pub implementation: String,
    /// The public build the run was produced by.
    pub implementation_build: String,
    /// Each declared failure the report has as `failed`, in scenario ID order.
    pub known_failed: Vec<KnownFailure>,
    /// The SHA-256 of the report's original bytes.
    pub report_digest: String,
    /// The specification digest.
    pub spec_digest: String,
    /// The SHA-256 of the suite's original bytes.
    pub suite_digest: String,
    /// Each `failed` scenario the declaration does not name, in scenario ID order.
    pub unexpected_failed: Vec<String>,
}

/// Accounts `report_text`, the original bytes of a report/2 of `admitted`, against `declaration`,
/// for a run the host says build `build` produced.
///
/// Refuses, before anything is written: a report that is not report/2 of exactly these suite
/// bytes; a declaration of another specification, suite, implementation label or build; a declared
/// scenario that did not fail.
pub fn account(
    report_text: &str,
    admitted: &AdmittedSuite,
    declaration: &Declaration,
    build: &str,
) -> Result<Accounting, Refusal> {
    let report = CountReport::from_json(report_text, admitted)
        .map_err(|error| malformed("ess-conformance-report/2", error))?;
    declaration.admit(admitted)?;
    declaration.bind(report.implementation(), build)?;
    let statuses = report.statuses();
    declaration.check_statuses(|id| statuses.get(id).copied())?;
    let unexpected_failed: Vec<String> = statuses
        .iter()
        .filter(|(id, status)| **status == "failed" && declaration.get(id).is_none())
        .map(|(id, _)| id.clone())
        .collect();
    Ok(Accounting {
        counts: report.counts().clone(),
        declaration_digest: declaration.digest().to_owned(),
        format: ACCOUNTING_FORMAT.to_owned(),
        implementation: report.implementation().to_owned(),
        implementation_build: build.to_owned(),
        known_failed: declaration.failures().to_vec(),
        report_digest: sha256(report_text.as_bytes()),
        spec_digest: admitted.suite().provenance.spec_digest.to_string(),
        suite_digest: admitted.digest().to_owned(),
        unexpected_failed,
    })
}

impl Accounting {
    /// Reads an accounting document, refusing one that is not closed and well formed.
    pub fn from_json(text: &str) -> Result<Self, Refusal> {
        let accounting: Self = closed(text, ACCOUNTING_FORMAT, ACCOUNTING_FORMAT)?;
        for (field, value) in [
            ("declaration_digest", &accounting.declaration_digest),
            ("implementation_build", &accounting.implementation_build),
            ("report_digest", &accounting.report_digest),
            ("suite_digest", &accounting.suite_digest),
        ] {
            digest_field(ACCOUNTING_FORMAT, field, value)?;
        }
        Ok(accounting)
    }

    /// Refuses this accounting unless it is exactly what [`account`] makes of `report_text`,
    /// `admitted` and `declaration_text` for its own build: every identity, the partition and the
    /// counts are recomputed from the original documents, never read from this one.
    pub fn validate(
        &self,
        report_text: &str,
        admitted: &AdmittedSuite,
        declaration_text: &str,
    ) -> Result<(), Refusal> {
        let declaration = Declaration::from_json(declaration_text)?;
        let expected = account(
            report_text,
            admitted,
            &declaration,
            &self.implementation_build,
        )?;
        if *self != expected {
            return Err(Refusal::new(
                RefusalKind::Accounting,
                "the accounting is not the partition of its report by its declaration",
            ));
        }
        Ok(())
    }

    /// Two-space JSON with sorted keys and one trailing LF.
    pub fn to_canonical_json(&self) -> String {
        canonical(self)
    }

    /// One line per declared failure, then one line per unexpected one, then the counts.
    pub fn render_text(&self) -> String {
        let mut out = format!(
            "known-failure accounting of {} (build {}): {} failed, {} declared known, {} \
             unexpected, of {} scenario(s); conformance is unchanged\n",
            self.implementation,
            self.implementation_build,
            self.counts.failed,
            self.known_failed.len(),
            self.unexpected_failed.len(),
            self.counts.total,
        );
        for known in &self.known_failed {
            let _ = writeln!(
                out,
                "known failure {}: {} (tracking {})",
                known.scenario, known.reason, known.tracking
            );
        }
        for unexpected in &self.unexpected_failed {
            let _ = writeln!(out, "unexpected failure {unexpected}");
        }
        out
    }
}
