//! A recorded command/response log, converted into `ess-history/1` through a declared adapter.
//!
//! A service that logs its calls — one JSON object per line, in whatever shape it chose — records
//! what the concurrent recorder in [`crate::record`] records: who called what on which subject, when
//! the call left and when its answer arrived. The [`Adapter`] is the operator's declaration of where
//! each [`Operation`](crate::history::Operation) field sits in a log line: a JSON pointer, or
//! `absent`. [`import_for`] reads the log through it and hands the result to [`history::read`], so
//! what it returns is exactly what `check-history` would admit. [`import`] is the same without a
//! compiled specification, for a caller that has only its digest.
//!
//! # Nothing is guessed
//!
//! A field the adapter declares `absent`, or a line does not carry (the pointer resolves to nothing
//! or to `null`), is never filled from another field, a neighbouring line or a default:
//!
//! * where the operation cannot be judged without it — `client`, `command`, `subject_key`,
//!   `invoked_at`, `completion`, and `returned_at` and `outcome` on a `Returned` call — the log is
//!   refused ([`ImportRefusal::MissingFields`]), naming every such field on every line, not the
//!   first;
//! * `operation_id` is not needed to judge a history: an operation without one is given a
//!   generated identity, and that is reported as a [`CoverageGap`] on its line;
//! * `rows` (optional in the adapter; `absent` when undeclared) is what a read of a view is judged
//!   from. A view read without rows is imported and reported as a `rows` gap on its line, because
//!   `check-history` will list it as not judged. [`import_for`] knows which lines are view reads:
//!   the command names a view of the specification, or the outcome is
//!   [`READ`]. [`import`] has no specification, so it reports the gap on
//!   every answered line without rows.
//!
//! The document's `seed` is not a property of a recorded log at all; it is written 0 and reported
//! as a gap with no line. The completion is read through the adapter's own `values` map from the
//! log's word to `Returned` or `Indeterminate`; a word it does not map is refused, not interpreted.
//!
//! **A history imported with gaps carries `seed` 0 and generated identities by construction**, and
//! the document itself cannot say which values were carried and which were not: the gaps
//! [`Imported::gaps`] returns (which `import-history` also writes beside the document) are the only
//! record of that.
//!
//! # Generated identities
//!
//! A generated identity is the first 16 bytes of SHA-256 over the log's own SHA-256 and the line
//! number, written as a lower-case UUID with version 8 and the RFC 9562 variant. No ESS writer
//! mints a version-8 identity (the recorders write version 4), so a generated identity cannot be
//! mistaken for one the recorder wrote. A carried identity that happens to equal a generated one,
//! or several lines carrying one identity, are refused, naming every line that carries it. A
//! carried identity must be a canonical UUID in lower case; any other spelling is refused at its
//! line as [`ImportRefusal::Field`].
//!
//! # What is derived, and how
//!
//! * `client`: a log names its clients by label (a string or an integer). Each distinct label is
//!   numbered in order of first appearance, and `clients` is how many there are. The numbering is a
//!   renaming, not an inference: it says nothing the log does not.
//! * `history_id`: the first 16 bytes of the SHA-256 of the log, written as a UUID, so one log
//!   always imports to one document.
//! * `spec_digest`: the digest of the specification the operator imports against. That is the
//!   operator's claim that the log was recorded from an implementation of it.
//!
//! Instants are taken as the log writes them and must be JSON unsigned integers: epoch
//! milliseconds serve, since the checker compares instants only by order. A string timestamp is
//! refused rather than parsed.
//!
//! Every refusal names the log line (or lines) and the field, including those the `ess-history/1`
//! reader makes after conversion ([`ImportRefusal::History`]).

use std::collections::BTreeMap;
use std::fmt::{self, Write as _};

use ess_compiler::ir::EssIr;
use ess_domain::name::QualifiedName as ModelName;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::history::{
    self, Completion, History, HistoryRefusal, QualifiedName, SpecDigest, Uuid, HISTORY_FORMAT,
};
use crate::scenario::SuiteProvenance;
use crate::sessions::READ;

/// The `format` value of every adapter this module reads.
pub const ADAPTER_FORMAT: &str = "ess-history-adapter/1";

/// The closed set of adapter formats this build reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum AdapterFormat {
    /// `ess-history-adapter/1`.
    #[serde(rename = "ess-history-adapter/1")]
    V1,
}

/// The word `absent`: the log does not carry this field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum Absent {
    /// `absent`.
    #[serde(rename = "absent")]
    Absent,
}

/// Where one field sits in a log line.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
pub enum FieldSource {
    /// The log does not carry it.
    Absent(Absent),
    /// It is at this JSON pointer.
    Pointer(Pointer),
}

fn absent() -> FieldSource {
    FieldSource::Absent(Absent::Absent)
}

/// `{ pointer: /a/b }`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pointer {
    /// An RFC 6901 JSON pointer into one log line.
    pub pointer: String,
}

/// Where the completion sits, and what each of the log's words for it means.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
pub enum CompletionSource {
    /// The log does not carry it.
    Absent(Absent),
    /// It is at this pointer, written as one of these words.
    Mapped(MappedCompletion),
}

/// `{ pointer: /a/b, values: { ok: Returned, timeout: Indeterminate } }`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MappedCompletion {
    /// An RFC 6901 JSON pointer into one log line.
    pub pointer: String,
    /// The log's word for each completion.
    pub values: BTreeMap<String, Completion>,
}

/// One source per [`Operation`](crate::history::Operation) field. Every field but `rows` must be
/// declared.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Fields {
    /// `operation_id`.
    pub operation_id: FieldSource,
    /// `client`.
    pub client: FieldSource,
    /// `command`.
    pub command: FieldSource,
    /// `subject_key`.
    pub subject_key: FieldSource,
    /// `invoked_at`.
    pub invoked_at: FieldSource,
    /// `returned_at`.
    pub returned_at: FieldSource,
    /// `outcome`.
    pub outcome: FieldSource,
    /// `completion`.
    pub completion: CompletionSource,
    /// `rows`: a JSON array of row identities, for a read of a view. `absent` when undeclared.
    #[serde(default = "absent")]
    pub rows: FieldSource,
}

/// An `ess-history-adapter/1` document: how a log's lines map onto `ess-history/1` operations.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Adapter {
    /// Always [`AdapterFormat::V1`].
    pub format: AdapterFormat,
    /// Where each operation field is.
    pub fields: Fields,
}

/// A field the log does not carry and the history does without.
///
/// Serializes as `{"line", "field", "written"}` in that order, `line` `null` for a document field.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CoverageGap {
    /// The log line (1-based), or `None` for a document field.
    pub line: Option<usize>,
    /// The `ess-history/1` field.
    pub field: &'static str,
    /// What was written instead, and why that is not a guess.
    pub written: String,
}

impl fmt::Display for CoverageGap {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.line {
            Some(line) => write!(
                formatter,
                "coverage-gap: line {line}: `{}` not carried; {}",
                self.field, self.written
            ),
            None => write!(
                formatter,
                "coverage-gap: `{}` not carried; {}",
                self.field, self.written
            ),
        }
    }
}

/// A field an operation cannot be judged without, missing from one log line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissingField {
    /// The log line, 1-based.
    pub line: usize,
    /// The operation's identity, where the line carries one.
    pub operation_id: Option<String>,
    /// The `ess-history/1` field.
    pub field: &'static str,
}

/// A log converted into a history, with what it did not carry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Imported {
    /// The history, admitted by [`history::read`].
    pub history: History,
    /// Every field not carried, in line order, the document's own last.
    pub gaps: Vec<CoverageGap>,
}

impl Imported {
    /// [`Self::gaps`] as pretty JSON with one trailing newline: an array, in the order held.
    ///
    /// # Panics
    ///
    /// It does not: every gap serializes.
    pub fn gaps_json(&self) -> String {
        let mut text = serde_json::to_string_pretty(&self.gaps)
            .unwrap_or_else(|error| panic!("coverage gaps serialize: {error}"));
        text.push('\n');
        text
    }
}

/// Every way an adapter or a log is refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportRefusal {
    /// The adapter is not an `ess-history-adapter/1` document.
    Adapter {
        /// What was wrong.
        detail: String,
    },
    /// A log line is not a JSON object.
    Line {
        /// The line, 1-based.
        line: usize,
        /// What the parser reported.
        detail: String,
    },
    /// A field is present with a value outside its type, or a completion word the adapter does not
    /// map.
    Field {
        /// The line, 1-based.
        line: usize,
        /// The `ess-history/1` field.
        field: &'static str,
        /// What was wrong.
        detail: String,
    },
    /// Fields an operation cannot be judged without, each on its line.
    MissingFields {
        /// Every one, in line order.
        missing: Vec<MissingField>,
    },
    /// Two lines carry one operation identity, or a carried one equals a generated one.
    DuplicateOperation {
        /// The identity.
        operation_id: String,
        /// Every line it is on.
        lines: Vec<usize>,
    },
    /// The converted document was refused by the `ess-history/1` reader.
    History {
        /// The log lines the refused operation came from; empty for a document-level refusal.
        lines: Vec<usize>,
        /// The `ess-history/1` operation field refused, where it is one.
        field: Option<String>,
        /// What the reader said.
        refusal: HistoryRefusal,
    },
}

impl ImportRefusal {
    /// The refusal's stable name.
    pub fn code(&self) -> &'static str {
        match self {
            Self::Adapter { .. } => "import.adapter-malformed",
            Self::Line { .. } => "import.line-malformed",
            Self::Field { .. } => "import.field-malformed",
            Self::MissingFields { .. } => "import.missing-fields",
            Self::DuplicateOperation { .. } => "import.duplicate-operation",
            Self::History { refusal, .. } => refusal.code(),
        }
    }
}

/// `line 1, line 2`.
fn lines_text(lines: &[usize]) -> String {
    lines
        .iter()
        .map(|line| format!("line {line}"))
        .collect::<Vec<_>>()
        .join(", ")
}

impl fmt::Display for ImportRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let code = self.code();
        match self {
            Self::Adapter { detail } => write!(formatter, "{code}: {detail}"),
            Self::Line { line, detail } => write!(formatter, "{code}: line {line}: {detail}"),
            Self::Field {
                line,
                field,
                detail,
            } => write!(formatter, "{code}: line {line}: `{field}`: {detail}"),
            Self::MissingFields { missing } => {
                write!(
                    formatter,
                    "{code}: {} field(s) an operation cannot be judged without",
                    missing.len()
                )?;
                for gap in missing {
                    let operation = gap
                        .operation_id
                        .as_deref()
                        .map_or_else(String::new, |id| format!(" (operation {id})"));
                    write!(
                        formatter,
                        "\n  line {}{operation}: `{}` missing",
                        gap.line, gap.field
                    )?;
                }
                Ok(())
            }
            Self::DuplicateOperation {
                operation_id,
                lines,
            } => write!(
                formatter,
                "{code}: {}: `operation_id` {operation_id} names more than one call",
                lines_text(lines)
            ),
            Self::History {
                lines,
                field,
                refusal,
            } => {
                if !lines.is_empty() {
                    write!(formatter, "{}: ", lines_text(lines))?;
                }
                if let Some(field) = field {
                    write!(formatter, "`{field}`: ")?;
                }
                write!(formatter, "{refusal}")
            }
        }
    }
}

impl std::error::Error for ImportRefusal {}

/// Reads an `ess-history-adapter/1` document, written as YAML or JSON.
///
/// # Errors
///
/// [`ImportRefusal::Adapter`] for an unknown or undeclared field, another format, or a pointer that
/// does not start with `/`.
pub fn adapter(text: &str) -> Result<Adapter, ImportRefusal> {
    let adapter: Adapter = serde_yaml::from_str(text).map_err(|error| ImportRefusal::Adapter {
        detail: format!(
            "{error}; each field is `absent` or `{{ pointer: /… }}`, and `completion` also \
             takes `values`"
        ),
    })?;
    let fields = &adapter.fields;
    let pointers = [
        ("operation_id", &fields.operation_id),
        ("client", &fields.client),
        ("command", &fields.command),
        ("subject_key", &fields.subject_key),
        ("invoked_at", &fields.invoked_at),
        ("returned_at", &fields.returned_at),
        ("outcome", &fields.outcome),
        ("rows", &fields.rows),
    ]
    .into_iter()
    .filter_map(|(field, source)| match source {
        FieldSource::Pointer(Pointer { pointer }) => Some((field, pointer)),
        FieldSource::Absent(_) => None,
    })
    .chain(match &fields.completion {
        CompletionSource::Mapped(mapped) => Some(("completion", &mapped.pointer)),
        CompletionSource::Absent(_) => None,
    });
    for (field, pointer) in pointers {
        if !pointer.starts_with('/') {
            return Err(ImportRefusal::Adapter {
                detail: format!("`{field}`: the pointer `{pointer}` does not start with `/`"),
            });
        }
    }
    Ok(adapter)
}

/// One line's value at `source`, or `None` where the adapter declares it absent, the pointer
/// resolves to nothing, or to `null`.
fn at<'v>(line: &'v Value, source: &FieldSource) -> Option<&'v Value> {
    match source {
        FieldSource::Absent(_) => None,
        FieldSource::Pointer(Pointer { pointer }) => {
            line.pointer(pointer).filter(|value| !value.is_null())
        }
    }
}

/// 16 bytes written as a UUID.
fn as_uuid(bytes: &[u8]) -> String {
    let mut hex = String::with_capacity(32);
    for byte in &bytes[..16] {
        let _ = write!(hex, "{byte:02x}");
    }
    format!(
        "{}-{}-{}-{}-{}",
        &hex[0..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..32]
    )
}

/// The identity generated for `line` of the log whose SHA-256 is `log_digest`: version 8, RFC 9562
/// variant, a namespace no ESS writer mints in.
fn generated(log_digest: &[u8], line: usize) -> String {
    let mut hasher = Sha256::new();
    hasher.update(log_digest);
    hasher.update((line as u64).to_be_bytes());
    let mut bytes = hasher.finalize();
    bytes[6] = (bytes[6] & 0x0f) | 0x80;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    as_uuid(&bytes)
}

/// What one log line carries of each operation field; `None` where it carries nothing.
struct Carried {
    operation_id: Option<String>,
    client: Option<Value>,
    command: Option<String>,
    subject_key: Option<String>,
    invoked_at: Option<u64>,
    returned_at: Option<u64>,
    outcome: Option<String>,
    completion: Option<Completion>,
    rows: Option<Vec<String>>,
}

/// Reads one line's fields through `fields`, refusing a value outside its field's type.
fn carried(line: usize, value: &Value, fields: &Fields) -> Result<Carried, ImportRefusal> {
    let wrong = |field: &'static str, detail: String| ImportRefusal::Field {
        line,
        field,
        detail,
    };
    let text = |field: &'static str, source: &FieldSource| {
        at(value, source)
            .map(|found| {
                found
                    .as_str()
                    .map(ToOwned::to_owned)
                    .ok_or_else(|| wrong(field, format!("{found} is not a string")))
            })
            .transpose()
    };
    let name = |field: &'static str, source: &FieldSource| {
        text(field, source)?
            .map(|found| {
                QualifiedName::new(found.clone())
                    .map(|_| found)
                    .map_err(|error| wrong(field, error))
            })
            .transpose()
    };
    let instant = |field: &'static str, source: &FieldSource| {
        at(value, source)
            .map(|found| {
                found.as_u64().ok_or_else(|| {
                    wrong(field, format!("{found} is not an unsigned integer instant"))
                })
            })
            .transpose()
    };
    let operation_id = text("operation_id", &fields.operation_id)?
        .map(|found| match Uuid::new(found.clone()) {
            Ok(identity) if identity.is_lower_case() => Ok(found),
            Ok(_) => Err(wrong(
                "operation_id",
                format!("`{found}` is not written in lower-case hexadecimal"),
            )),
            Err(error) => Err(wrong("operation_id", error)),
        })
        .transpose()?;
    let client = at(value, &fields.client)
        .map(|label| match label {
            Value::String(_) | Value::Number(_) => Ok(label.clone()),
            other => Err(wrong(
                "client",
                format!("{other} is neither a string nor a number"),
            )),
        })
        .transpose()?;
    let completion = match &fields.completion {
        CompletionSource::Absent(_) => None,
        CompletionSource::Mapped(mapped) => {
            match value.pointer(&mapped.pointer).filter(|it| !it.is_null()) {
                None => None,
                Some(word) => {
                    let found = word.as_str().and_then(|word| mapped.values.get(word));
                    Some(*found.ok_or_else(|| {
                        wrong(
                            "completion",
                            format!("{word} is not a word the adapter's `values` maps"),
                        )
                    })?)
                }
            }
        }
    };
    let rows = at(value, &fields.rows)
        .map(|found| {
            found
                .as_array()
                .and_then(|rows| {
                    rows.iter()
                        .map(|row| row.as_str().map(ToOwned::to_owned))
                        .collect::<Option<Vec<_>>>()
                })
                .ok_or_else(|| wrong("rows", format!("{found} is not an array of strings")))
        })
        .transpose()?;
    Ok(Carried {
        operation_id,
        client,
        command: name("command", &fields.command)?,
        subject_key: text("subject_key", &fields.subject_key)?,
        invoked_at: instant("invoked_at", &fields.invoked_at)?,
        returned_at: instant("returned_at", &fields.returned_at)?,
        outcome: name("outcome", &fields.outcome)?,
        completion,
        rows,
    })
}

impl Carried {
    /// The `ess-history/1` operation object, with only the fields the line carries.
    fn operation(self, operation_id: String, client: Option<usize>) -> Value {
        let mut operation = serde_json::Map::new();
        operation.insert("operation_id".to_owned(), Value::from(operation_id));
        let mut put = |key: &str, field: Option<Value>| {
            if let Some(field) = field {
                operation.insert(key.to_owned(), field);
            }
        };
        put("client", client.map(Value::from));
        put("command", self.command.map(Value::from));
        put("subject_key", self.subject_key.map(Value::from));
        put("invoked_at", self.invoked_at.map(Value::from));
        put("returned_at", self.returned_at.map(Value::from));
        put("outcome", self.outcome.map(Value::from));
        put(
            "completion",
            self.completion.map(|completion| {
                Value::from(match completion {
                    Completion::Returned => "Returned",
                    Completion::Indeterminate => "Indeterminate",
                })
            }),
        );
        put("rows", self.rows.map(Value::from));
        Value::Object(operation)
    }

    /// Every field this line's operation cannot be judged without and does not carry.
    fn missing(&self, line: usize) -> Vec<MissingField> {
        let mut needed = vec![
            ("client", self.client.is_some()),
            ("command", self.command.is_some()),
            ("subject_key", self.subject_key.is_some()),
            ("invoked_at", self.invoked_at.is_some()),
            ("completion", self.completion.is_some()),
        ];
        if self.completion == Some(Completion::Returned) {
            needed.push(("returned_at", self.returned_at.is_some()));
            needed.push(("outcome", self.outcome.is_some()));
        }
        needed
            .into_iter()
            .filter(|(_, present)| !present)
            .map(|(field, _)| MissingField {
                line,
                operation_id: self.operation_id.clone(),
                field,
            })
            .collect()
    }
}

/// What the importer knows about which lines are reads of a view.
#[derive(Clone, Copy)]
enum Reads<'a> {
    /// Nothing: no specification was given.
    Unknown,
    /// The specification's views.
    Of(&'a EssIr),
}

impl Reads<'_> {
    /// The `rows` gap for an answered line without rows, or `None` where the line is not a read.
    fn gap(self, carried: &Carried, declared: bool) -> Option<String> {
        if carried.completion != Some(Completion::Returned) || carried.rows.is_some() {
            return None;
        }
        let why = if declared {
            "the line carries no rows"
        } else {
            "the adapter declares no `rows`"
        };
        match self {
            Self::Unknown => Some(format!(
                "{why}, and no specification says whether this line is a view read; if it is, \
                 check-history will not judge it"
            )),
            Self::Of(ir) => {
                let command = carried.command.as_deref().unwrap_or_default();
                let read = carried.outcome.as_deref() == Some(READ)
                    || ModelName::new(command).is_ok_and(|name| ir.views().contains_key(&name));
                read.then(|| format!("{why}; check-history will not judge this read"))
            }
        }
    }
}

/// [`import`] against the compiled specification `ir`: the history carries its digest, and a
/// `rows` gap is reported exactly on the lines that read one of its views.
///
/// # Errors
///
/// As [`import`].
pub fn import_for(log: &[u8], adapter: &Adapter, ir: &EssIr) -> Result<Imported, ImportRefusal> {
    convert(
        log,
        adapter,
        &SuiteProvenance::of(ir).spec_digest,
        Reads::Of(ir),
    )
}

/// Reads `log`, one JSON object per non-blank line, through `adapter`, into an `ess-history/1`
/// document recorded against `spec_digest`, and admits it through [`history::read`].
///
/// With no specification to say which lines read a view, every answered line without rows is
/// reported as a `rows` gap; [`import_for`] reports only the reads.
///
/// # Errors
///
/// [`ImportRefusal`]: a line that is not a JSON object, a value outside its field's type, every
/// field an operation cannot be judged without, one identity on two lines, and whatever the
/// history reader refuses, each naming its log line.
pub fn import(
    log: &[u8],
    adapter: &Adapter,
    spec_digest: &SpecDigest,
) -> Result<Imported, ImportRefusal> {
    convert(log, adapter, spec_digest, Reads::Unknown)
}

fn convert(
    log: &[u8],
    adapter: &Adapter,
    spec_digest: &SpecDigest,
    reads: Reads<'_>,
) -> Result<Imported, ImportRefusal> {
    let log_digest = Sha256::digest(log);
    let rows_declared = matches!(adapter.fields.rows, FieldSource::Pointer(_));
    let mut gaps = Vec::new();
    let mut missing = Vec::new();
    let mut labels: Vec<Value> = Vec::new();
    let mut operations = Vec::new();
    // For each operation, in document order: the log line it came from and its identity.
    let mut origins: Vec<(usize, String)> = Vec::new();

    for (index, raw) in log.split(|byte| *byte == b'\n').enumerate() {
        let line = index + 1;
        if raw.iter().all(u8::is_ascii_whitespace) {
            continue;
        }
        let value: Value = serde_json::from_slice(raw).map_err(|error| ImportRefusal::Line {
            line,
            detail: error.to_string(),
        })?;
        if !value.is_object() {
            return Err(ImportRefusal::Line {
                line,
                detail: "not a JSON object".to_owned(),
            });
        }
        let mut carried = carried(line, &value, &adapter.fields)?;
        missing.extend(carried.missing(line));
        if let Some(written) = reads.gap(&carried, rows_declared) {
            gaps.push(CoverageGap {
                line: Some(line),
                field: "rows",
                written,
            });
        }

        let operation_id = carried.operation_id.take().unwrap_or_else(|| {
            let id = generated(&log_digest, line);
            gaps.push(CoverageGap {
                line: Some(line),
                field: "operation_id",
                written: format!("generated as {id}"),
            });
            id
        });
        origins.push((line, operation_id.clone()));
        let client = carried.client.take().map(|label| {
            labels
                .iter()
                .position(|known| *known == label)
                .unwrap_or_else(|| {
                    labels.push(label);
                    labels.len() - 1
                })
        });

        operations.push(carried.operation(operation_id, client));
    }

    if let Some(duplicate) = duplicate(&origins) {
        return Err(duplicate);
    }
    if !missing.is_empty() {
        return Err(ImportRefusal::MissingFields { missing });
    }
    gaps.push(CoverageGap {
        line: None,
        field: "seed",
        written: "a recorded log is drawn from no seed; written 0".to_owned(),
    });

    let document = serde_json::json!({
        "format": HISTORY_FORMAT,
        "history_id": as_uuid(&log_digest),
        "spec_digest": spec_digest,
        "seed": 0,
        "clients": labels.len(),
        "operations": operations,
    });
    let bytes = serde_json::to_vec(&document)
        .unwrap_or_else(|error| panic!("a JSON value serializes: {error}"));
    let history =
        history::read(&bytes, spec_digest).map_err(|refusal| located(refusal, &origins))?;
    Ok(Imported { history, gaps })
}

/// The first identity, in log order, that more than one line carries, with every line carrying it.
fn duplicate(origins: &[(usize, String)]) -> Option<ImportRefusal> {
    let mut by_id: BTreeMap<&str, Vec<usize>> = BTreeMap::new();
    for (line, id) in origins {
        by_id.entry(id.as_str()).or_default().push(*line);
    }
    origins.iter().find_map(|(_, id)| {
        let lines = &by_id[id.as_str()];
        (lines.len() > 1).then(|| ImportRefusal::DuplicateOperation {
            operation_id: id.clone(),
            lines: lines.clone(),
        })
    })
}

/// A reader refusal, with the log lines and the field it concerns.
fn located(refusal: HistoryRefusal, origins: &[(usize, String)]) -> ImportRefusal {
    let by_id = |id: &str| -> Vec<usize> {
        origins
            .iter()
            .filter(|(_, known)| known.eq_ignore_ascii_case(id))
            .map(|(line, _)| *line)
            .collect()
    };
    let (lines, field) = match &refusal {
        HistoryRefusal::DuplicateOperation { operation_id }
        | HistoryRefusal::NonCanonicalUuid {
            value: operation_id,
        } => (by_id(operation_id), Some("operation_id")),
        HistoryRefusal::ClientOutOfRange { operation_id, .. } => {
            (by_id(operation_id), Some("client"))
        }
        HistoryRefusal::ReturnedWithoutReturnInstant { operation_id }
        | HistoryRefusal::ReturnBeforeInvoke { operation_id, .. }
        | HistoryRefusal::IndeterminateWithReturnInstant { operation_id } => {
            (by_id(operation_id), Some("returned_at"))
        }
        HistoryRefusal::IndeterminateWithOutcome { operation_id }
        | HistoryRefusal::ReturnedWithoutOutcome { operation_id } => {
            (by_id(operation_id), Some("outcome"))
        }
        HistoryRefusal::IndeterminateWithRows { operation_id } => {
            (by_id(operation_id), Some("rows"))
        }
        HistoryRefusal::IntegerOutOfRange { field, .. } => {
            let indexed = field.strip_prefix("operations[").and_then(|rest| {
                let (index, name) = rest.split_once("].")?;
                Some((index.parse::<usize>().ok()?, name.to_owned()))
            });
            match indexed {
                Some((index, name)) => {
                    let lines = origins.get(index).map(|(line, _)| vec![*line]);
                    return ImportRefusal::History {
                        lines: lines.unwrap_or_default(),
                        field: Some(name),
                        refusal,
                    };
                }
                None => (Vec::new(), None),
            }
        }
        HistoryRefusal::Malformed { .. }
        | HistoryRefusal::UnsupportedFormat { .. }
        | HistoryRefusal::SpecDigestMismatch { .. }
        | HistoryRefusal::NoClients => (Vec::new(), None),
    };
    ImportRefusal::History {
        lines,
        field: field.map(ToOwned::to_owned),
        refusal,
    }
}
