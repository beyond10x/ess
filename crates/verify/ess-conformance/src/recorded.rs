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
//! A word is a string or an integer ([`Words`]): a string word maps a log string, an integer word
//! (a YAML integer key such as `200`) maps a log integer, so HTTP statuses can be mapped, and
//! neither maps the other. A JSON object's keys are strings, so a JSON adapter maps strings only.
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

/// Where one field sits in a log line: the word `absent` or `{ pointer: /… }`, and no other
/// spelling. Read by `absent_or`, not `#[serde(untagged)]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldSource {
    /// The log does not carry it.
    Absent(Absent),
    /// It is at this JSON pointer.
    Pointer(Pointer),
}

fn absent() -> FieldSource {
    FieldSource::Absent(Absent::Absent)
}

/// A source as its author wrote it: the word `absent`, or a mapping.
enum Plain<T> {
    Absent(Absent),
    Given(T),
}

/// Reads a source from exactly the two spellings `ess-history-adapter/1` declares: a string, which
/// must be `absent`, or a mapping, which must be a `T`.
///
/// `#[serde(untagged)]` is not used because serde buffers an untagged value first, and a buffered
/// value is read more loosely than the document declares: a unit value also as the one-key
/// mapping `{absent: null}`, and a struct also as a list of its fields (`["/a"]`). This reads the
/// value in place instead, so every other shape is refused where it stands, and the refusal
/// carries its path in the document (`fields.client`).
fn absent_or<'de, D, T>(deserializer: D, shape: &'static str) -> Result<Plain<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    deserializer.deserialize_any(AbsentOr {
        shape,
        given: std::marker::PhantomData,
    })
}

struct AbsentOr<T> {
    shape: &'static str,
    given: std::marker::PhantomData<T>,
}

impl<'de, T: Deserialize<'de>> serde::de::Visitor<'de> for AbsentOr<T> {
    type Value = Plain<T>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "`absent` or {}", self.shape)
    }

    fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Self::Value, E> {
        Absent::deserialize(serde::de::value::StrDeserializer::<E>::new(value)).map(Plain::Absent)
    }

    fn visit_map<A: serde::de::MapAccess<'de>>(self, map: A) -> Result<Self::Value, A::Error> {
        T::deserialize(serde::de::value::MapAccessDeserializer::new(map)).map(Plain::Given)
    }
}

impl<'de> Deserialize<'de> for FieldSource {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(match absent_or(deserializer, "`{ pointer: /… }`")? {
            Plain::Absent(absent) => Self::Absent(absent),
            Plain::Given(pointer) => Self::Pointer(pointer),
        })
    }
}

/// `{ pointer: /a/b }`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pointer {
    /// An RFC 6901 JSON pointer into one log line.
    pub pointer: String,
}

/// Where the completion sits, and what each of the log's words for it means: the word `absent` or
/// `{ pointer: /…, values: {…} }`, and no other spelling. Read by `absent_or`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompletionSource {
    /// The log does not carry it.
    Absent(Absent),
    /// It is at this pointer, written as one of these words.
    Mapped(MappedCompletion),
}

impl<'de> Deserialize<'de> for CompletionSource {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(
            match absent_or(deserializer, "`{ pointer: /…, values: {…} }`")? {
                Plain::Absent(absent) => Self::Absent(absent),
                Plain::Given(mapped) => Self::Mapped(mapped),
            },
        )
    }
}

/// `{ pointer: /a/b, values: { ok: Returned, timeout: Indeterminate } }`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MappedCompletion {
    /// An RFC 6901 JSON pointer into one log line.
    pub pointer: String,
    /// The log's word for each completion.
    pub values: Words<Completion>,
}

/// What each of a log's words means, where a word is a string or an integer.
///
/// A string word maps a log value that is that string; an integer word (a YAML integer key, as in
/// `200: Returned`) maps a log value that is that integer, so an HTTP status can be mapped. Neither
/// maps the other: the string `"200"` is not the number 200. A word is written once, whichever
/// kind: an adapter mapping both `"200"` and `200` is refused. A JSON object's keys are strings,
/// so a JSON adapter maps string words only.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Words<T> {
    /// By spelling: the kind the adapter wrote the word as, and what it means.
    entries: BTreeMap<String, (WordKind, T)>,
}

/// How an adapter wrote a word.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WordKind {
    /// A string, mapping log strings.
    Text,
    /// An integer, mapping log integers.
    Integer,
}

impl<T> Words<T> {
    /// What the word spelled `word` means, whichever kind the adapter wrote it as.
    pub fn get(&self, word: &str) -> Option<&T> {
        self.entries.get(word).map(|(_, meaning)| meaning)
    }

    /// The kind the adapter wrote the word spelled `word` as.
    pub fn kind(&self, word: &str) -> Option<WordKind> {
        self.entries.get(word).map(|(kind, _)| *kind)
    }

    /// What a log value means: a string through a string word, an integer through an integer
    /// word, and nothing else.
    pub fn meaning(&self, value: &Value) -> Option<&T> {
        let (spelling, kind) = match value {
            Value::String(text) => (text.clone(), WordKind::Text),
            Value::Number(number) => (
                number
                    .as_u64()
                    .map(|integer| integer.to_string())
                    .or_else(|| number.as_i64().map(|integer| integer.to_string()))?,
                WordKind::Integer,
            ),
            _ => return None,
        };
        self.entries
            .get(&spelling)
            .filter(|(written, _)| *written == kind)
            .map(|(_, meaning)| meaning)
    }
}

/// One key of `values`, as written.
struct WordKey(String, WordKind);

impl<'de> Deserialize<'de> for WordKey {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Key;
        impl serde::de::Visitor<'_> for Key {
            type Value = WordKey;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a word: a string or an integer")
            }

            fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<WordKey, E> {
                Ok(WordKey(value.to_owned(), WordKind::Text))
            }

            fn visit_u64<E: serde::de::Error>(self, value: u64) -> Result<WordKey, E> {
                Ok(WordKey(value.to_string(), WordKind::Integer))
            }

            fn visit_i64<E: serde::de::Error>(self, value: i64) -> Result<WordKey, E> {
                Ok(WordKey(value.to_string(), WordKind::Integer))
            }
        }
        deserializer.deserialize_any(Key)
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Words<T> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Map<T>(std::marker::PhantomData<T>);
        impl<'de, T: Deserialize<'de>> serde::de::Visitor<'de> for Map<T> {
            type Value = Words<T>;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a mapping from each of the log's words to a completion")
            }

            fn visit_map<A: serde::de::MapAccess<'de>>(
                self,
                mut map: A,
            ) -> Result<Words<T>, A::Error> {
                let mut entries = BTreeMap::new();
                while let Some(WordKey(spelling, kind)) = map.next_key()? {
                    let meaning = map.next_value()?;
                    if entries.contains_key(&spelling) {
                        return Err(serde::de::Error::custom(format!(
                            "the word `{spelling}` is mapped twice (a string and an integer \
                             spelled alike are one word)"
                        )));
                    }
                    entries.insert(spelling, (kind, meaning));
                }
                Ok(Words { entries })
            }
        }
        deserializer.deserialize_map(Map(std::marker::PhantomData))
    }
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
    /// The model requires private one-time observations that legacy histories do not represent.
    UnsupportedOneTimeDisclosure,
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
            Self::UnsupportedOneTimeDisclosure => "import.one-time-disclosure-unsupported",
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
            Self::UnsupportedOneTimeDisclosure => write!(
                formatter,
                "{code}: one-time response observation is unsupported by recorded histories"
            ),
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
/// A document whose first non-whitespace character is `{` is first read as JSON, straight into the
/// typed [`Adapter`], so it means what it means to every JSON reader: a surrogate-pair escape is
/// its character, and a field or a word written twice is refused. If it is not JSON at all (a
/// syntax error, such as the unquoted keys of YAML flow style), it is read as YAML, like every
/// other document.
///
/// YAML's core-schema tags (`!!str`, `!!int`) are standard YAML and resolve to the plain value they
/// name. Any other tag (`!x`, `!Returned`, `!<x>`) is refused with its path: the format declares
/// none, and a tag is how YAML spells an enum value serde would otherwise accept.
///
/// # Errors
///
/// [`ImportRefusal::Adapter`] for a document that does not parse, a local YAML tag, an unknown,
/// undeclared or repeated field, another format, a word mapped twice, or a pointer that does not
/// start with `/`.
pub fn adapter(text: &str) -> Result<Adapter, ImportRefusal> {
    let malformed = |detail: String| ImportRefusal::Adapter { detail };
    let shape = "each field is `absent` or `{ pointer: /… }`, and `completion` also takes `values`";
    let adapter = match json_adapter(text) {
        Some(Ok(adapter)) => {
            // Read straight into the typed adapter, the JSON reader also takes serde's other
            // spellings: a struct as a list of its fields, a unit value as `{"Returned": null}`.
            // No place in the format holds a list or a null, so neither is admitted anywhere.
            let document: Value = serde_json::from_str(text).map_err(|error| {
                malformed(format!("{error}; a document the JSON reader read is JSON"))
            })?;
            if let Some((path, what)) = first_list_or_null(&document, "") {
                return Err(malformed(format!(
                    "`{path}`: {what} is not part of {ADAPTER_FORMAT}; {shape}"
                )));
            }
            adapter
        }
        Some(Err(error)) => {
            // The JSON reader decides, but names no path; the same text read as YAML names one
            // where the YAML reader refuses it too, so both are given.
            let detail = match yaml_adapter(text).and_then(checked) {
                Err(yaml) => format!("{error} (read as YAML: {yaml})"),
                Ok(_) => error.to_string(),
            };
            return Err(malformed(format!("{detail}; {shape}")));
        }
        None => yaml_adapter(text).map_err(|detail| malformed(format!("{detail}; {shape}")))?,
    };
    checked(adapter).map_err(malformed)
}

/// `adapter` if every pointer it declares starts with `/`.
fn checked(adapter: Adapter) -> Result<Adapter, String> {
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
            return Err(format!(
                "`fields.{field}`: the pointer `{pointer}` does not start with `/`"
            ));
        }
    }
    Ok(adapter)
}

/// The adapter a `{`-leading document means as JSON, or `None` where it is not one: another first
/// character, or a JSON syntax error (YAML flow style is not JSON). A JSON data error — an unknown,
/// missing or repeated field, a word mapped twice — is the JSON reader's refusal.
fn json_adapter(text: &str) -> Option<Result<Adapter, serde_json::Error>> {
    if !text.trim_start().starts_with('{') {
        return None;
    }
    match serde_json::from_str(text) {
        Err(error)
            if matches!(
                error.classify(),
                serde_json::error::Category::Syntax | serde_json::error::Category::Eof
            ) =>
        {
            None
        }
        read => Some(read),
    }
}

/// The path of the first list or `null` in a JSON document, and which it is.
fn first_list_or_null(value: &Value, path: &str) -> Option<(String, &'static str)> {
    let at = || {
        if path.is_empty() {
            "the document".to_owned()
        } else {
            path.to_owned()
        }
    };
    match value {
        Value::Array(_) => Some((at(), "a list")),
        Value::Null => Some((at(), "`null`")),
        Value::Object(members) => members.iter().find_map(|(key, member)| {
            let here = if path.is_empty() {
                key.clone()
            } else {
                format!("{path}.{key}")
            };
            first_list_or_null(member, &here)
        }),
        _ => None,
    }
}

/// The adapter a YAML document means, refusing any tag the YAML reader keeps (every tag but YAML's
/// own core-schema tags, which it resolves to the plain value) with its path.
fn yaml_adapter(text: &str) -> Result<Adapter, String> {
    let document: serde_yaml::Value =
        serde_yaml::from_str(text).map_err(|error| error.to_string())?;
    if let Some((path, tag)) = first_tag(&document, "") {
        return Err(format!(
            "`{path}`: the YAML tag `{tag}` is not part of {ADAPTER_FORMAT}"
        ));
    }
    serde_yaml::from_str(text).map_err(|error| error.to_string())
}

/// The path and tag of the first YAML tag in `value`, on a key or a value.
fn first_tag(value: &serde_yaml::Value, path: &str) -> Option<(String, String)> {
    let here = |segment: &str| {
        if path.is_empty() {
            segment.to_owned()
        } else {
            format!("{path}.{segment}")
        }
    };
    match value {
        serde_yaml::Value::Tagged(tagged) => Some((
            if path.is_empty() {
                "the document".to_owned()
            } else {
                path.to_owned()
            },
            tagged.tag.to_string(),
        )),
        serde_yaml::Value::Mapping(members) => members.iter().find_map(|(key, member)| {
            let segment = match key {
                serde_yaml::Value::String(text) => text.clone(),
                serde_yaml::Value::Number(number) => number.to_string(),
                serde_yaml::Value::Tagged(tagged) => {
                    return Some((here("<key>"), tagged.tag.to_string()));
                }
                other => format!("{other:?}"),
            };
            first_tag(member, &here(&segment))
        }),
        serde_yaml::Value::Sequence(items) => items
            .iter()
            .enumerate()
            .find_map(|(index, item)| first_tag(item, &format!("{path}[{index}]"))),
        _ => None,
    }
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
                    let found = mapped.values.meaning(word);
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
    crate::record::refuse_one_time(ir).map_err(|_| ImportRefusal::UnsupportedOneTimeDisclosure)?;
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
        HistoryRefusal::RetryOfUnknown { operation_id, .. } => {
            (by_id(operation_id), Some("retry_of"))
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
