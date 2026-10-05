//! Explicit synthesis seeds: admission of the rows an operator selected (beyond10x/ess#413,
//! `docs/design/synthesis-seeds.md`).
//!
//! A seed is the initial `setup` of one arrangement of one authored document, selected by source
//! and instance. The whole document is compiled against the model by the authored compiler, so the
//! row obeys every rule authored setup does — typed fields, declared state, invariants, Optional
//! absence against present null — and nothing but that one setup row is taken from it: never its
//! timeline, never its assertions, never a state its timeline would reach.
//!
//! The admitted set is immutable and travels in the invocation context of the search
//! ([`super::caller::InvocationModels`]); there is no global seed catalogue.
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use ess_compiler::ir::EssIr;
use ess_primitives::evidence::SpecDigest;

use crate::authored::Source;
use crate::coverage::SourceIdentity;
use crate::scenario::{ConformanceSuite, InstanceName, ScenarioStep, SuiteProvenance};
use crate::synthesis_seeds::{SeedRecord, SynthesisSeeds, MAX_SELECTIONS};

/// One explicitly selected seed: an authored document, by its original text, and the arrangement
/// whose setup is the row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeedSelection {
    /// The document. Its `origin` must be a checked root-relative source identity; it is what the
    /// suite records, so it never carries an absolute host path.
    pub source: Source,
    /// The arrangement of that document whose `setup` is selected.
    pub instance: InstanceName,
}

/// The admitted seed rows of one request, bound to the model they were compiled against.
///
/// Built only by [`AdmittedSeeds::compile`]; [`AdmittedSeeds::empty`] is the seed-free set every
/// existing entry point delegates with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmittedSeeds {
    binding: Option<(SpecDigest, SpecDigest)>,
    rows: Vec<SeedRecord>,
    sources: BTreeMap<SourceIdentity, String>,
}

/// The seed-free set, shared by every unseeded invocation context.
pub(super) static EMPTY: AdmittedSeeds = AdmittedSeeds::empty();

/// Why a seed request was refused, before any output or target activity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SeedAdmissionError {
    /// No selection at all: an explicit request must name at least one.
    EmptySelection,
    /// More than [`MAX_SELECTIONS`] selections.
    TooManySelections {
        /// How many were requested.
        count: usize,
    },
    /// A source origin that is not a checked root-relative identity.
    InvalidSource {
        /// The origin as given.
        source: String,
        /// Why it is refused.
        detail: String,
    },
    /// Two different texts under one source identity.
    AmbiguousSource {
        /// The identity.
        source: String,
    },
    /// The same source and instance selected twice.
    DuplicateSelector {
        /// The source.
        source: String,
        /// The instance.
        instance: InstanceName,
    },
    /// The source document does not compile against the model.
    SourceRefused {
        /// The source.
        source: String,
        /// Every authored refusal, rendered.
        refusals: Vec<String>,
    },
    /// The source declares no arrangement of that name.
    UnknownInstance {
        /// The source.
        source: String,
        /// The instance asked for.
        instance: InstanceName,
    },
    /// The arrangement exists but carries no `setup`.
    InstanceWithoutSetup {
        /// The source.
        source: String,
        /// The instance.
        instance: InstanceName,
    },
    /// Two selections establish one qualified identity.
    DuplicateIdentity {
        /// The declared entity.
        entity: String,
        /// The first selection, as `source#instance`.
        first: String,
        /// The second selection, as `source#instance`.
        second: String,
    },
    /// The seeds were compiled against another model.
    WrongModel,
    /// The model's actors carry caller attributes, which seeded synthesis does not support.
    CallerModel,
    /// Authored-only coverage acquisition generates nothing a seed could serve.
    AuthoredOnly,
    /// The component asked for is not declared.
    UnknownComponent(super::UnknownComponent),
}

impl SeedAdmissionError {
    /// A stable code naming the refusal.
    pub fn code(&self) -> &'static str {
        match self {
            Self::EmptySelection => "empty-selection",
            Self::TooManySelections { .. } => "too-many-selections",
            Self::InvalidSource { .. } => "invalid-source",
            Self::AmbiguousSource { .. } => "ambiguous-source",
            Self::DuplicateSelector { .. } => "duplicate-selector",
            Self::SourceRefused { .. } => "source-refused",
            Self::UnknownInstance { .. } => "unknown-instance",
            Self::InstanceWithoutSetup { .. } => "instance-without-setup",
            Self::DuplicateIdentity { .. } => "duplicate-identity",
            Self::WrongModel => "wrong-model",
            Self::CallerModel => "caller-model",
            Self::AuthoredOnly => "authored-only",
            Self::UnknownComponent(_) => "unknown-component",
        }
    }
}

impl fmt::Display for SeedAdmissionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "synthesis seed refused [{}]: ", self.code())?;
        match self {
            Self::EmptySelection => write!(f, "an explicit seed request selects nothing"),
            Self::TooManySelections { count } => write!(
                f,
                "{count} selections exceed the bound of {MAX_SELECTIONS} one request admits"
            ),
            Self::InvalidSource { source, detail } => write!(f, "source `{source}`: {detail}"),
            Self::AmbiguousSource { source } => write!(
                f,
                "two different documents share the source identity `{source}`"
            ),
            Self::DuplicateSelector { source, instance } => {
                write!(f, "`{source}` instance `{instance}` is selected twice")
            }
            Self::SourceRefused { source, refusals } => write!(
                f,
                "`{source}` does not compile against the model: {}",
                refusals.join("; ")
            ),
            Self::UnknownInstance { source, instance } => {
                write!(f, "`{source}` declares no arrangement `{instance}`")
            }
            Self::InstanceWithoutSetup { source, instance } => write!(
                f,
                "`{source}` arrangement `{instance}` has no setup; only an explicit setup row is a \
                 seed"
            ),
            Self::DuplicateIdentity {
                entity,
                first,
                second,
            } => write!(
                f,
                "`{first}` and `{second}` both establish one `{entity}` identity"
            ),
            Self::WrongModel => write!(f, "the seeds were compiled against another model"),
            Self::CallerModel => write!(
                f,
                "the model's actors carry caller attributes; seeded synthesis of a caller \
                 assignment is not supported"
            ),
            Self::AuthoredOnly => write!(
                f,
                "authored-only coverage generates no obligation a seed could serve"
            ),
            Self::UnknownComponent(unknown) => write!(f, "{unknown}"),
        }
    }
}

impl std::error::Error for SeedAdmissionError {}

impl From<super::UnknownComponent> for SeedAdmissionError {
    fn from(unknown: super::UnknownComponent) -> Self {
        Self::UnknownComponent(unknown)
    }
}

impl Default for AdmittedSeeds {
    fn default() -> Self {
        Self::empty()
    }
}

impl AdmittedSeeds {
    /// The seed-free set.
    pub const fn empty() -> Self {
        Self {
            binding: None,
            rows: Vec::new(),
            sources: BTreeMap::new(),
        }
    }

    /// Whether no row was admitted.
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// How many rows were admitted.
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    /// Admit explicit selections against `ir`, in the order the design fixes: every source read
    /// once and checked, compiled whole, the nominated setup taken, and the set sorted and checked
    /// for duplicate identities.
    ///
    /// # Errors
    ///
    /// A [`SeedAdmissionError`] naming the first refused selection or source.
    pub fn compile(ir: &EssIr, selections: &[SeedSelection]) -> Result<Self, SeedAdmissionError> {
        if selections.is_empty() {
            return Err(SeedAdmissionError::EmptySelection);
        }
        if selections.len() > MAX_SELECTIONS {
            return Err(SeedAdmissionError::TooManySelections {
                count: selections.len(),
            });
        }
        let (texts, selectors) = read_selections(selections)?;
        let rows = ordered(nominated(ir, &texts, &selectors)?)?;
        let provenance = SuiteProvenance::of(ir);
        Ok(Self {
            binding: Some((provenance.spec_digest, provenance.contract_digest)),
            rows,
            sources: texts
                .iter()
                .map(|(identity, text)| (identity.clone(), crate::coverage::digest(text)))
                .collect(),
        })
    }

    /// The admitted rows, in search order.
    pub(crate) fn rows(&self) -> &[SeedRecord] {
        &self.rows
    }

    /// Refuse a nonempty set compiled against another model, or for a model this capability does
    /// not synthesize.
    pub(crate) fn bound_to(&self, ir: &EssIr) -> Result<(), SeedAdmissionError> {
        if self.is_empty() {
            return Ok(());
        }
        let provenance = SuiteProvenance::of(ir);
        if self.binding.as_ref() != Some(&(provenance.spec_digest, provenance.contract_digest)) {
            return Err(SeedAdmissionError::WrongModel);
        }
        if super::caller::uses(ir) {
            return Err(SeedAdmissionError::CallerModel);
        }
        Ok(())
    }

    /// Record the seeds on a finished suite: every source and selection, and every use the
    /// emitted generated scenarios hold. Selects the seed-bearing ordinary format.
    pub(crate) fn attach(&self, ir: &EssIr, suite: &mut ConformanceSuite) {
        if self.is_empty() {
            return;
        }
        let mut selections = self.rows.clone();
        selections.sort_by(|left, right| {
            (&left.source, &left.instance).cmp(&(&right.source, &right.instance))
        });
        let applications = crate::synthesis_seeds::applications(suite, &selections);
        suite.provenance.synthesis_seeds = Some(SynthesisSeeds {
            sources: self.sources.clone(),
            selections,
            applications,
        });
        suite.select_fresh_format_for(ir);
    }
}

/// The selections' sources, each read once under its checked identity with one text, and the
/// distinct selectors: admission step 1 of `docs/design/synthesis-seeds.md`.
fn read_selections(
    selections: &[SeedSelection],
) -> Result<(Texts<'_>, BTreeSet<(SourceIdentity, InstanceName)>), SeedAdmissionError> {
    let mut texts: Texts<'_> = BTreeMap::new();
    let mut selectors: BTreeSet<(SourceIdentity, InstanceName)> = BTreeSet::new();
    for selection in selections {
        let identity = SourceIdentity::new(selection.source.origin.clone()).map_err(|error| {
            SeedAdmissionError::InvalidSource {
                source: selection.source.origin.clone(),
                detail: error
                    .issues
                    .first()
                    .map_or_else(String::new, |issue| issue.detail.clone()),
            }
        })?;
        match texts.get(&identity) {
            Some(text) if *text != selection.source.text => {
                return Err(SeedAdmissionError::AmbiguousSource {
                    source: identity.as_str().to_owned(),
                })
            }
            Some(_) => {}
            None => {
                texts.insert(identity.clone(), &selection.source.text);
            }
        }
        if !selectors.insert((identity.clone(), selection.instance.clone())) {
            return Err(SeedAdmissionError::DuplicateSelector {
                source: identity.as_str().to_owned(),
                instance: selection.instance.clone(),
            });
        }
    }
    Ok((texts, selectors))
}

/// The source texts of one request, by checked identity.
type Texts<'a> = BTreeMap<SourceIdentity, &'a str>;

/// Each source compiled whole against the model, and only the nominated setup row of each
/// selection taken from it: admission steps 2 and 3.
fn nominated(
    ir: &EssIr,
    texts: &Texts<'_>,
    selectors: &BTreeSet<(SourceIdentity, InstanceName)>,
) -> Result<Vec<SeedRecord>, SeedAdmissionError> {
    let mut compiled = BTreeMap::new();
    for (identity, text) in texts {
        let source = Source::new(identity.as_str(), *text);
        let scenario = crate::authored::compile_one(ir, &source, &BTreeMap::new())
            .map_err(|refusals| SeedAdmissionError::SourceRefused {
                source: identity.as_str().to_owned(),
                refusals: refusals.iter().map(ToString::to_string).collect(),
            })?
            .1;
        let arrangements = crate::authored::arrangements(ir, &source).map_err(|detail| {
            SeedAdmissionError::SourceRefused {
                source: identity.as_str().to_owned(),
                refusals: vec![detail],
            }
        })?;
        compiled.insert(identity.clone(), (scenario, arrangements));
    }
    // 3. Only the nominated setup row of each selection.
    let mut rows = Vec::new();
    for (identity, instance) in selectors {
        let (scenario, arrangements) = &compiled[identity];
        let refused = |setup: bool| {
            let (source, instance) = (identity.as_str().to_owned(), instance.clone());
            if setup {
                SeedAdmissionError::InstanceWithoutSetup { source, instance }
            } else {
                SeedAdmissionError::UnknownInstance { source, instance }
            }
        };
        match arrangements.iter().find(|(name, _)| name == instance) {
            None => return Err(refused(false)),
            Some((_, false)) => return Err(refused(true)),
            Some((_, true)) => {}
        }
        let row = scenario.steps.iter().find_map(|step| match step {
            ScenarioStep::EstablishEntity {
                instance: established,
                entity,
                identity: literal,
                fields,
                state,
            } if established == instance => Some(SeedRecord {
                source: identity.clone(),
                instance: instance.clone(),
                entity: entity.clone(),
                identity: literal.clone(),
                fields: fields.clone(),
                state: state.clone(),
            }),
            _ => None,
        });
        rows.push(row.ok_or_else(|| refused(true))?);
    }
    Ok(rows)
}

/// The rows sorted independently of argument order, with one row per qualified identity:
/// admission step 4.
fn ordered(mut rows: Vec<SeedRecord>) -> Result<Vec<SeedRecord>, SeedAdmissionError> {
    let key = |row: &SeedRecord| {
        (
            row.entity.to_string(),
            serde_json::to_string(&(&row.identity, &row.fields, &row.state))
                .expect("an admitted setup row serializes"),
            row.source.clone(),
            row.instance.clone(),
        )
    };
    rows.sort_by_key(key);
    let mut seen: BTreeMap<(String, String), String> = BTreeMap::new();
    for row in &rows {
        let identity = (
            row.entity.to_string(),
            serde_json::to_string(&row.identity).expect("an admitted identity serializes"),
        );
        let name = format!("{}#{}", row.source.as_str(), row.instance);
        if let Some(first) = seen.insert(identity, name.clone()) {
            return Err(SeedAdmissionError::DuplicateIdentity {
                entity: row.entity.to_string(),
                first,
                second: name,
            });
        }
    }
    Ok(rows)
}
