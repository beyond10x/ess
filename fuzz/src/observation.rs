//! Bounded typed frames and an independent sequence/identity admission state machine.
use crate::{carrier::Bundle, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};
pub const MAX_RECORD: usize = 16_384;
pub const MAX_BLOB: usize = 1_048_576;
const MAX_ATTEMPTS: u64 = 1_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Entry {
    ByteCarrier,
    Structured,
}
impl Entry {
    pub fn name(self) -> &'static str {
        match self {
            Self::ByteCarrier => "byte-carrier",
            Self::Structured => "structured",
        }
    }
    pub fn decode(self, input: &[u8]) -> Result<Bundle> {
        match self {
            Self::ByteCarrier => crate::carrier::decode(input),
            Self::Structured => crate::structured::render(input),
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Target {
    Rust,
    Go,
    Web,
    Clap,
}
impl Target {
    pub const ALL: [Self; 4] = [Self::Rust, Self::Go, Self::Web, Self::Clap];
    pub fn production(self) -> ess_synth::Target {
        match self {
            Self::Rust => ess_synth::Target::Rust,
            Self::Go => ess_synth::Target::Go,
            Self::Web => ess_synth::Target::Web,
            Self::Clap => ess_synth::Target::Clap,
        }
    }
    pub fn from_production(target: ess_synth::Target) -> Self {
        match target {
            ess_synth::Target::Rust => Self::Rust,
            ess_synth::Target::Go => Self::Go,
            ess_synth::Target::Web => Self::Web,
            ess_synth::Target::Clap => Self::Clap,
        }
    }
    pub fn name(self) -> &'static str {
        self.production().name()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "stage", rename_all = "kebab-case", deny_unknown_fields)]
pub enum Stage {
    Decode,
    Parse { index: usize, label: String },
    AssembleValidate,
    Compile,
    Generator { name: String },
    DocsIr,
    Synthesis { target: Target },
}
impl Stage {
    pub fn key(&self) -> String {
        match self {
            Self::Decode => "decode".into(),
            Self::Parse { index, label } => format!("parse/{index}:{label}"),
            Self::AssembleValidate => "assemble-validate".into(),
            Self::Compile => "compile".into(),
            Self::Generator { name } => format!("generator/{name}"),
            Self::DocsIr => "docs-ir".into(),
            Self::Synthesis { target } => format!("synthesis/{}", target.name()),
        }
    }
}
pub const GENERATORS: [&str; 5] = ["docs", "site", "schema", "openapi", "asyncapi"];
pub fn downstream() -> Vec<Stage> {
    let mut stages = GENERATORS
        .into_iter()
        .map(|name| Stage::Generator { name: name.into() })
        .collect::<Vec<_>>();
    stages.push(Stage::DocsIr);
    stages.extend(Target::ALL.map(|target| Stage::Synthesis { target }));
    stages
}
pub fn stages(bundle: Option<&Bundle>) -> Vec<Stage> {
    let mut out = vec![Stage::Decode];
    if let Some(bundle) = bundle {
        out.extend(
            bundle
                .documents
                .iter()
                .enumerate()
                .map(|(index, doc)| Stage::Parse {
                    index,
                    label: doc.label.clone(),
                }),
        );
    }
    out.extend([Stage::AssembleValidate, Stage::Compile]);
    out.extend(downstream());
    out
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Blob {
    pub sha256: String,
    pub bytes: usize,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "returned", rename_all = "kebab-case", deny_unknown_fields)]
pub enum Returned {
    Success,
    Decoded { bundle: Blob },
    Assembled { major: u32 },
    Refused { detail: Blob },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "outcome", rename_all = "kebab-case", deny_unknown_fields)]
pub enum Outcome {
    Compiled,
    InputRefused,
    ParseRefused,
    ValidationRefused,
    CompileRefused,
    OutsideDomain { major: u32 },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "record", rename_all = "kebab-case", deny_unknown_fields)]
pub enum Record {
    Start {
        attempt: u64,
        entry: Entry,
        input: Blob,
    },
    StageStart {
        attempt: u64,
        identity: Stage,
    },
    StageResult {
        attempt: u64,
        identity: Stage,
        result: Returned,
    },
    Finish {
        attempt: u64,
        outcome: Outcome,
        not_reached: Vec<Stage>,
    },
}
pub fn write_frame(writer: &mut impl Write, record: &Record) -> Result<()> {
    let bytes = serde_json::to_vec(record)?;
    if bytes.len() > MAX_RECORD {
        return Err("observation record exceeds bound".into());
    }
    writer.write_all(&(bytes.len() as u32).to_be_bytes())?;
    writer.write_all(&bytes)?;
    writer.flush()?;
    Ok(())
}
pub fn read_frame(reader: &mut impl Read) -> Result<Option<Record>> {
    let mut header = [0u8; 4];
    let n = reader.read(&mut header[..1])?;
    if n == 0 {
        return Ok(None);
    }
    reader
        .read_exact(&mut header[1..])
        .map_err(|e| format!("truncated frame header: {e}"))?;
    let n = u32::from_be_bytes(header) as usize;
    if n == 0 || n > MAX_RECORD {
        return Err("observation frame length outside bound".into());
    }
    let mut bytes = vec![0; n];
    reader
        .read_exact(&mut bytes)
        .map_err(|e| format!("truncated observation frame: {e}"))?;
    Ok(Some(serde_json::from_slice(&bytes)?))
}
pub struct Writer {
    root: PathBuf,
    stream: File,
    next: u64,
}
impl Writer {
    pub fn create(root: &Path) -> Result<Self> {
        if let Some(parent) = root.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::create_dir(root)?;
        fs::create_dir(root.join("blobs"))?;
        let stream = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(root.join("observations.frames"))?;
        Ok(Self {
            root: root.into(),
            stream,
            next: 1,
        })
    }
    pub fn blob(&self, bytes: &[u8]) -> Result<Blob> {
        if bytes.len() > MAX_BLOB {
            return Err("observation blob exceeds bound".into());
        }
        let blob = Blob {
            sha256: crate::digest(bytes),
            bytes: bytes.len(),
        };
        let path = self.root.join("blobs").join(&blob.sha256);
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(mut file) => {
                file.write_all(bytes)?;
                file.sync_data()?;
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                if fs::read(&path)? != bytes {
                    return Err("existing observation blob changed".into());
                }
            }
            Err(e) => return Err(e.into()),
        }
        Ok(blob)
    }
    pub fn begin(&mut self, entry: Entry, input: &[u8]) -> Result<u64> {
        let attempt = self.next;
        self.next = self
            .next
            .checked_add(1)
            .ok_or("attempt identity overflow")?;
        let input = self.blob(input)?;
        self.emit(&Record::Start {
            attempt,
            entry,
            input,
        })?;
        Ok(attempt)
    }
    pub fn emit(&mut self, record: &Record) -> Result<()> {
        write_frame(&mut self.stream, record)
    }
}
pub fn read_blob(root: &Path, blob: &Blob) -> Result<Vec<u8>> {
    if blob.bytes > MAX_BLOB
        || blob.sha256.len() != 64
        || !blob
            .sha256
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err("invalid observation blob reference".into());
    }
    let mut file = File::open(root.join("blobs").join(&blob.sha256))?;
    let mut bytes = Vec::new();
    (&mut file)
        .take((MAX_BLOB + 1) as u64)
        .read_to_end(&mut bytes)?;
    if bytes.len() != blob.bytes || crate::digest(&bytes) != blob.sha256 {
        return Err("observation blob identity mismatch".into());
    }
    Ok(bytes)
}
#[derive(Debug, Clone, Default, Serialize)]
pub struct Counts {
    pub started: u64,
    pub success: u64,
    pub refused: u64,
}
#[derive(Debug, Clone, Default, Serialize)]
pub struct Summary {
    pub attempts: u64,
    pub compiled: u64,
    pub frames: u64,
    pub entries: BTreeMap<String, u64>,
    pub outcomes: BTreeMap<String, u64>,
    pub stages: BTreeMap<String, Counts>,
    pub compiled_source_identities: BTreeSet<String>,
}
struct Active {
    id: u64,
    entry: Entry,
    input: Vec<u8>,
    expected: Vec<Stage>,
    cursor: usize,
    pending: Option<Stage>,
    stopped: Option<Outcome>,
    compiled: bool,
    bundle_hash: Option<String>,
}
fn failure(message: impl Into<String>) -> crate::Error {
    message.into().into()
}
pub fn admit(root: &Path) -> Result<Summary> {
    let mut reader = File::open(root.join("observations.frames"))?;
    let mut summary = Summary::default();
    let mut active: Option<Active> = None;
    while let Some(record) = read_frame(&mut reader)? {
        summary.frames += 1;
        match record {
            Record::Start {
                attempt,
                entry,
                input,
            } => {
                if active.is_some() || attempt != summary.attempts + 1 || attempt > MAX_ATTEMPTS {
                    return Err("overlapping, duplicate, or non-monotonic attempt".into());
                }
                let input = read_blob(root, &input)?;
                active = Some(Active {
                    id: attempt,
                    entry,
                    input,
                    expected: stages(None),
                    cursor: 0,
                    pending: None,
                    stopped: None,
                    compiled: false,
                    bundle_hash: None,
                });
            }
            Record::StageStart { attempt, identity } => {
                let a = active.as_mut().ok_or("stage outside attempt")?;
                if a.id != attempt
                    || a.pending.is_some()
                    || a.stopped.is_some()
                    || a.expected.get(a.cursor) != Some(&identity)
                {
                    return Err(failure(format!(
                        "missing/overlapping/out-of-order stage {identity:?}; expected {:?}",
                        a.expected.get(a.cursor)
                    )));
                }
                summary.stages.entry(identity.key()).or_default().started += 1;
                a.pending = Some(identity);
            }
            Record::StageResult {
                attempt,
                identity,
                result,
            } => {
                let a = active.as_mut().ok_or("result outside attempt")?;
                if a.id != attempt || a.pending.as_ref() != Some(&identity) {
                    return Err("result lacks matching pending stage".into());
                }
                if let Returned::Refused { detail } = &result {
                    read_blob(root, detail)?;
                }
                match (&identity, &result) {
                    (Stage::Decode, Returned::Decoded { bundle }) => {
                        let bytes = read_blob(root, bundle)?;
                        let decoded = crate::carrier::decode(&bytes)?;
                        let expected = a.entry.decode(&a.input)?;
                        if decoded != expected || decoded.encode()? != bytes {
                            return Err("decoded/rendered source translation changed".into());
                        }
                        a.expected = stages(Some(&decoded));
                        a.bundle_hash = Some(bundle.sha256.clone());
                    }
                    (Stage::Decode, Returned::Refused { .. }) => {
                        if a.entry.decode(&a.input).is_ok() {
                            return Err(
                                "successful input was falsely classified as decode refusal".into(),
                            );
                        }
                        a.stopped = Some(Outcome::InputRefused);
                    }
                    (Stage::AssembleValidate, Returned::Assembled { major }) => {
                        if *major != 1 {
                            a.stopped = Some(Outcome::OutsideDomain { major: *major });
                        }
                    }
                    (Stage::AssembleValidate, Returned::Refused { .. }) => {
                        a.stopped = Some(Outcome::ValidationRefused)
                    }
                    (Stage::Parse { .. }, Returned::Success) => {}
                    (Stage::Parse { .. }, Returned::Refused { .. }) => {
                        a.stopped = Some(Outcome::ParseRefused)
                    }
                    (Stage::Compile, Returned::Success) => a.compiled = true,
                    (Stage::Compile, Returned::Refused { .. }) => {
                        a.stopped = Some(Outcome::CompileRefused)
                    }
                    (
                        Stage::Generator { .. } | Stage::DocsIr | Stage::Synthesis { .. },
                        Returned::Success | Returned::Refused { .. },
                    ) => {}
                    _ => return Err("returned kind does not belong to stage".into()),
                }
                let count = summary.stages.entry(identity.key()).or_default();
                if matches!(result, Returned::Refused { .. }) {
                    count.refused += 1
                } else {
                    count.success += 1
                }
                a.pending = None;
                a.cursor += 1;
            }
            Record::Finish {
                attempt,
                outcome,
                not_reached,
            } => {
                let a = active.take().ok_or("finish outside attempt")?;
                let expected = if let Some(stopped) = a.stopped {
                    stopped
                } else if a.compiled && a.cursor == a.expected.len() {
                    Outcome::Compiled
                } else {
                    return Err("terminal before complete pipeline".into());
                };
                if a.id != attempt
                    || a.pending.is_some()
                    || outcome != expected
                    || not_reached != a.expected[a.cursor..]
                {
                    return Err("incorrect terminal outcome/not-reached stages".into());
                }
                summary.attempts += 1;
                *summary.entries.entry(a.entry.name().into()).or_default() += 1;
                let outcome_key = match outcome {
                    Outcome::Compiled => "compiled",
                    Outcome::InputRefused => "input-refused",
                    Outcome::ParseRefused => "parse-refused",
                    Outcome::ValidationRefused => "validation-refused",
                    Outcome::CompileRefused => "compile-refused",
                    Outcome::OutsideDomain { .. } => "outside-domain",
                };
                *summary.outcomes.entry(outcome_key.into()).or_default() += 1;
                if a.compiled {
                    summary.compiled += 1;
                    summary
                        .compiled_source_identities
                        .insert(a.bundle_hash.ok_or("compiled without source identity")?);
                }
            }
        }
    }
    if let Some(a) = active {
        return Err(failure(format!(
            "missing terminal observation for attempt {}; pending={:?}; next={:?}",
            a.id,
            a.pending,
            a.expected.get(a.cursor)
        )));
    }
    for stage in downstream() {
        let count = summary
            .stages
            .get(&stage.key())
            .cloned()
            .unwrap_or_default();
        if count.started != summary.compiled || count.success + count.refused != summary.compiled {
            return Err(failure(format!(
                "downstream count mismatch {}",
                stage.key()
            )));
        }
    }
    Ok(summary)
}
pub fn qualify_live(summary: &Summary, entry: Entry) -> Result<()> {
    if summary.compiled == 0
        || summary.attempts == 0
        || summary.entries.len() != 1
        || summary.entries.get(entry.name()) != Some(&summary.attempts)
    {
        return Err(
            "live lane requires actual in-domain compiled callbacks from its bound entry".into(),
        );
    }
    Ok(())
}
