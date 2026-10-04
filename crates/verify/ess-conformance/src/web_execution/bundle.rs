//! Closed byte-bound browser bundle and full Rust source/lineage admission.
use super::{presentation, Error, Result, ABI, MAX_FRAME};
use crate::{coverage::AdmittedInput, AdmittedSuite, ScenarioId, SuiteProvenance};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, fmt::Write as _, sync::Arc};

/// Closed product manifest identity.
pub const FORMAT: &str = "ess-conformance-browser/1";
/// One original acquired source; path is a stable bundle-relative label.
#[derive(Debug, Clone)]
pub struct SourceDocument {
    /// Relative archive label, not a local filesystem path.
    pub path: String,
    /// Exact original UTF-8 source.
    pub text: String,
}
/// Exact reference to one artifact's bytes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlobRef {
    /// Relative resource path.
    pub path: String,
    /// Bare lowercase SHA-256 over original bytes.
    pub sha256: String,
    /// Exact byte length, bounded to the product profile.
    pub byte_length: u32,
}
/// A blob received through the byte ABI; no JS JSON round trip.
#[derive(Debug, Clone)]
pub struct Blob {
    /// Exact matching manifest path.
    pub path: String,
    /// Original bytes, including any final LF.
    pub bytes: Vec<u8>,
}
/// Which original-byte admission entry point applies.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ExecutionRef {
    /// An ordinary suite, with no coverage carrier invented for it.
    OrdinarySuite {
        /// Exact original ordinary suite bytes.
        file: BlobRef,
    },
    /// A complete original input/1 with every parent string.
    CoverageInput {
        /// Exact complete input carrier, including original ancestor strings.
        file: BlobRef,
    },
}
impl ExecutionRef {
    fn file(&self) -> &BlobRef {
        match self {
            Self::OrdinarySuite { file } | Self::CoverageInput { file } => file,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Generator {
    package: String,
    version: String,
    semantic_revision: u32,
}
/// Closed manifest. A parsed manifest alone is not admission authority.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    format: String,
    abi: String,
    sources: Vec<BlobRef>,
    execution: ExecutionRef,
    presentation: BlobRef,
    generator: Generator,
}
/// An already admitted ordinary or coverage execution document.
#[derive(Debug, Clone)]
pub enum Execution {
    /// Ordinary original suite bytes.
    Ordinary(AdmittedSuite),
    /// Complete coverage input and lineage.
    Coverage(AdmittedInput),
}
impl Execution {
    /// Selected original suite capability.
    pub fn selected(&self) -> &AdmittedSuite {
        match self {
            Self::Ordinary(suite) => suite,
            Self::Coverage(input) => input.selected(),
        }
    }
    pub(super) fn parents(&self) -> &[AdmittedSuite] {
        match self {
            Self::Ordinary(_) => &[],
            Self::Coverage(input) => input.parents(),
        }
    }
    fn original(&self) -> Result<String> {
        match self {
            Self::Ordinary(suite) => Ok(suite.original_json().into()),
            Self::Coverage(input) => input
                .document()
                .to_canonical_json()
                .map_err(|_| Error::InternalFailure),
        }
    }
}
/// Complete admitted source and suite capability; no target exists here.
pub struct Loaded {
    ir: Arc<ess_compiler::EssIr>,
    sources: Vec<BlobRef>,
    execution: Execution,
    original_input: String,
    presentation: String,
}
impl Loaded {
    /// Validate every original file before exposing an execution capability.
    pub fn admit(manifest: &str, mut blobs: Vec<Blob>) -> Result<Self> {
        if manifest.len() > 256 * 1024 {
            return Err(Error::ResourceLimit);
        }
        let manifest_bytes = manifest.len();
        crate::count_json::Json::parse(manifest, "$browser").map_err(|_| Error::InvalidBundle)?;
        let manifest: Manifest =
            serde_json::from_str(manifest).map_err(|_| Error::InvalidBundle)?;
        if manifest.format != FORMAT
            || manifest.abi != ABI
            || manifest.generator.semantic_revision != 1
        {
            return Err(Error::IncompatibleAbi);
        }
        if manifest.generator.package != "ess-conformance" || manifest.generator.version.is_empty()
        {
            return Err(Error::InvalidBundle);
        }
        if manifest.sources.is_empty() || manifest.sources.len() > 1024 {
            return Err(Error::ResourceLimit);
        }
        let refs: Vec<&BlobRef> = manifest
            .sources
            .iter()
            .chain([manifest.execution.file(), &manifest.presentation])
            .collect();
        if refs.len() != blobs.len() {
            return Err(Error::InvalidBundle);
        }
        let mut seen = BTreeSet::new();
        let mut bytes = 32_usize
            .checked_add(manifest_bytes)
            .ok_or(Error::ResourceLimit)?;
        for (reference, blob) in refs.iter().zip(&blobs) {
            validate_path(&reference.path)?;
            if !seen.insert(&reference.path)
                || blob.path != reference.path
                || reference.byte_length as usize != blob.bytes.len()
                || reference.sha256 != hash(&blob.bytes)
            {
                return Err(Error::InvalidBundle);
            }
            bytes = bytes
                .checked_add(blob.path.len())
                .and_then(|n| n.checked_add(blob.bytes.len()))
                .and_then(|n| n.checked_add(8))
                .ok_or(Error::ResourceLimit)?;
            if bytes > MAX_FRAME {
                return Err(Error::ResourceLimit);
            }
        }
        // The checked references fix the layout: sources, then execution input, then presentation.
        let presentation_bytes = blobs.pop().ok_or(Error::InvalidBundle)?.bytes;
        let input = blobs.pop().ok_or(Error::InvalidBundle)?;
        let original_input = utf8(input.bytes)?;
        let execution = match &manifest.execution {
            ExecutionRef::OrdinarySuite { .. } => {
                let suite = AdmittedSuite::from_json(&original_input)
                    .map_err(|_| Error::AdmissionRefused)?;
                if suite.coverage().is_some() {
                    return Err(Error::InvalidBundle);
                }
                Execution::Ordinary(suite)
            }
            ExecutionRef::CoverageInput { .. } => Execution::Coverage(
                AdmittedInput::from_json(&original_input).map_err(|_| Error::AdmissionRefused)?,
            ),
        };
        let sources = blobs
            .into_iter()
            .map(|blob| {
                Ok(SourceDocument {
                    path: blob.path,
                    text: utf8(blob.bytes)?,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let ir = compile(&sources)?;
        check_provenance(&ir, &execution)?;
        let presentation = presentation::document(&ir, &execution, &manifest.sources)?;
        if presentation.as_bytes() != presentation_bytes {
            return Err(Error::InvalidBundle);
        }
        Ok(Self {
            ir: Arc::new(ir),
            sources: manifest.sources,
            execution,
            original_input,
            presentation,
        })
    }
    /// Exact selected input, retained throughout report production.
    pub fn selected(&self) -> &AdmittedSuite {
        self.execution.selected()
    }
    /// Original received or explicitly derived input bytes.
    pub fn original_input(&self) -> &str {
        &self.original_input
    }
    /// Full lossless declaration document.
    pub fn presentation(&self) -> &str {
        &self.presentation
    }
    /// Explicit coverage narrowing, never ordinary subset fabrication.
    pub(super) fn select(&self, ids: &[ScenarioId]) -> Result<Self> {
        if ids.windows(2).any(|pair| pair[0] >= pair[1]) {
            return Err(Error::InvalidFrame);
        }
        let Execution::Coverage(input) = &self.execution else {
            return Err(Error::SelectionNotAvailable);
        };
        let selected = input.select(ids).map_err(|_| Error::AdmissionRefused)?;
        let original_input = selected
            .document()
            .to_canonical_json()
            .map_err(|_| Error::InternalFailure)?;
        if original_input.len() > MAX_FRAME {
            return Err(Error::ResourceLimit);
        }
        let execution = Execution::Coverage(
            AdmittedInput::from_json(&original_input).map_err(|_| Error::AdmissionRefused)?,
        );
        let presentation = presentation::document(&self.ir, &execution, &self.sources)?;
        let source_bytes: usize = self
            .sources
            .iter()
            .map(|source| source.byte_length as usize)
            .sum();
        if original_input
            .len()
            .checked_add(presentation.len())
            .and_then(|n| n.checked_add(source_bytes))
            .is_none_or(|n| n > MAX_FRAME)
        {
            return Err(Error::ResourceLimit);
        }
        Ok(Self {
            ir: Arc::clone(&self.ir),
            sources: self.sources.clone(),
            execution,
            original_input,
            presentation,
        })
    }
}
fn utf8(bytes: Vec<u8>) -> Result<String> {
    String::from_utf8(bytes).map_err(|_| Error::InvalidBundle)
}
/// Check relative paths before any browser fetch.
pub fn validate_path(path: &str) -> Result<()> {
    if path.len() > 1024
        || path.is_empty()
        || path
            .chars()
            .any(|c| c.is_control() || matches!(c, '\\' | ':' | '?' | '#' | '%'))
        || path
            .split('/')
            .any(|p| p.is_empty() || matches!(p, "." | ".."))
    {
        return Err(Error::InvalidBundle);
    }
    Ok(())
}
/// Lowercase exact-byte SHA-256, not an authentication claim.
pub fn hash(bytes: &[u8]) -> String {
    hex(&Sha256::digest(bytes))
}
pub(super) fn hex(bytes: &[u8]) -> String {
    let mut result = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(result, "{byte:02x}");
    }
    result
}
fn reference(path: &str, bytes: &[u8]) -> Result<BlobRef> {
    validate_path(path)?;
    Ok(BlobRef {
        path: path.into(),
        sha256: hash(bytes),
        byte_length: u32::try_from(bytes.len()).map_err(|_| Error::ResourceLimit)?,
    })
}
fn compile(sources: &[SourceDocument]) -> Result<ess_compiler::EssIr> {
    let texts: Vec<&str> = sources.iter().map(|s| s.text.as_str()).collect();
    let parsed = ess_domain::spec::RawSpecFile::parse_all(&texts);
    let mut source_map = ess_compiler::source::SourceMap::new();
    let mut documents = Vec::new();
    for (source, parsed) in sources.iter().zip(parsed) {
        validate_path(&source.path)?;
        source_map.insert(&source.path, &source.text);
        documents.push((
            ess_domain::system::Source::new(source.path.clone()),
            parsed.map_err(|_| Error::AdmissionRefused)?,
        ));
    }
    let specification = ess_domain::spec::Specification::assemble(documents)
        .map_err(|_| Error::AdmissionRefused)?;
    let ir =
        ess_compiler::compile(&specification, &source_map).map_err(|_| Error::AdmissionRefused)?;
    crate::admission::model(&ir).map_err(|_| Error::AdmissionRefused)?;
    Ok(ir)
}
fn check_provenance(ir: &ess_compiler::EssIr, execution: &Execution) -> Result<()> {
    let expected = SuiteProvenance::of(ir);
    for suite in std::iter::once(execution.selected()).chain(execution.parents()) {
        let actual = &suite.suite().provenance;
        if actual.system != expected.system
            || actual.specification_version != expected.specification_version
            || actual.spec_digest != expected.spec_digest
            || actual.contract_digest != expected.contract_digest
        {
            return Err(Error::AdmissionRefused);
        }
        if actual
            .component
            .as_ref()
            .is_some_and(|name| !ir.components().keys().any(|key| key.to_string() == *name))
        {
            return Err(Error::AdmissionRefused);
        }
    }
    Ok(())
}
/// Generate a manifest and raw blobs from checked source/ordinary-or-coverage authority.
/// This emits no target implementation and does not invoke an installation.
pub fn create(sources: &[SourceDocument], execution: &Execution) -> Result<(String, Vec<Blob>)> {
    if sources.is_empty() || sources.len() > 1024 {
        return Err(Error::ResourceLimit);
    }
    let text = execution.original()?;
    let mut original_bytes = text.len();
    for source in sources {
        original_bytes = original_bytes
            .checked_add(source.text.len())
            .ok_or(Error::ResourceLimit)?;
        if original_bytes > MAX_FRAME {
            return Err(Error::ResourceLimit);
        }
    }
    let ir = compile(sources)?;
    check_provenance(&ir, execution)?;
    let mut blobs: Vec<Blob> = sources
        .iter()
        .map(|source| Blob {
            path: source.path.clone(),
            bytes: source.text.as_bytes().to_vec(),
        })
        .collect();
    let refs = blobs
        .iter()
        .map(|blob| reference(&blob.path, &blob.bytes))
        .collect::<Result<Vec<_>>>()?;
    let path = match execution {
        Execution::Ordinary(_) => "suite.json",
        Execution::Coverage(_) => "input.json",
    };
    let file = reference(path, text.as_bytes())?;
    let execution_ref = match execution {
        Execution::Ordinary(_) => ExecutionRef::OrdinarySuite { file },
        Execution::Coverage(_) => ExecutionRef::CoverageInput { file },
    };
    blobs.push(Blob {
        path: path.into(),
        bytes: text.into_bytes(),
    });
    let display = presentation::document(&ir, execution, &refs)?;
    let presentation_ref = reference("declarations.json", display.as_bytes())?;
    blobs.push(Blob {
        path: "declarations.json".into(),
        bytes: display.into_bytes(),
    });
    let manifest = Manifest {
        format: FORMAT.into(),
        abi: ABI.into(),
        sources: refs,
        execution: execution_ref,
        presentation: presentation_ref,
        generator: Generator {
            package: "ess-conformance".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            semantic_revision: 1,
        },
    };
    let text = format!(
        "{}\n",
        serde_json::to_string_pretty(&manifest).map_err(|_| Error::InternalFailure)?
    );
    // Exercise the exact reader over emitted originals, including all archive and byte budgets.
    Loaded::admit(&text, blobs.clone())?;
    Ok((text, blobs))
}
