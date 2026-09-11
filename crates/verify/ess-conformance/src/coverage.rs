//! Inventory claims and exact-parent selection, separate from the historical suite DTO.
use crate::{
    admission::AdmissionError, count_json::Json, AdmittedSuite, ConformanceSuite, EssSemanticRef,
    ScenarioId,
};
use ess_domain::component::ComponentName;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Write,
};

/// Coverage suite version. Legacy construction still defaults to suite/4.
pub const COVERAGE_SUITE_FORMAT: &str = "ess-conformance/5";
/// Carrier retaining exact selected and parent documents.
pub const SUITE_INPUT_FORMAT: &str = "ess-conformance-input/1";

pub(crate) fn error(detail: impl Into<String>) -> AdmissionError {
    AdmissionError::new("InvalidCoverage", "$coverage", detail)
}
pub(crate) fn digest(text: &str) -> String {
    Sha256::digest(text.as_bytes())
        .iter()
        .fold("sha256:".into(), |mut out, byte| {
            write!(out, "{byte:02x}").expect("writing to String");
            out
        })
}
pub(crate) fn valid_digest(value: &str) -> bool {
    value.strip_prefix("sha256:").is_some_and(|v| {
        v.len() == 64
            && v.bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    })
}
pub(crate) fn canonical(value: &impl Serialize) -> Result<String, AdmissionError> {
    let mut value = serde_json::to_value(value).map_err(|e| error(e.to_string()))?;
    value.sort_all_objects();
    let mut text = serde_json::to_string_pretty(&value).map_err(|e| error(e.to_string()))?;
    text.push('\n');
    Ok(text)
}

/// Exact original UTF-8 suite identity; this reference alone grants no admission.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SuiteReference {
    /// Admitted suite format.
    pub version: String,
    /// Exactly sha256-json-bytes/1.
    pub digest_profile: String,
    /// SHA-256 of every original suite byte.
    pub digest: String,
}
impl SuiteReference {
    /// Reference the immutable admitted source.
    pub fn of(suite: &AdmittedSuite) -> Self {
        Self {
            version: suite.suite().provenance.suite_version.to_string(),
            digest_profile: "sha256-json-bytes/1".into(),
            digest: suite.digest().into(),
        }
    }
}

/// Checked identity relative to an explicitly supplied authored-input root.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub struct SourceIdentity(String);
impl SourceIdentity {
    /// Check without normalizing Unicode, case or path segments.
    pub fn new(value: impl Into<String>) -> Result<Self, AdmissionError> {
        let value = value.into();
        if value
            .split('/')
            .any(|s| s.is_empty() || s == "." || s == "..")
            || value
                .chars()
                .any(|c| c == '\\' || c == ':' || c.is_control())
        {
            return Err(error(
                "authored source identity must be a checked root-relative path",
            ));
        }
        Ok(Self(value))
    }
    /// Original identity spelling.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl<'de> Deserialize<'de> for SourceIdentity {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Self::new(String::deserialize(d)?).map_err(serde::de::Error::custom)
    }
}

/// Where a retained or rejected candidate originated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Origin {
    /// An explicitly supplied authored file.
    Authored,
    /// Model synthesis.
    Generated,
}
/// The declared sources of obligations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Origins {
    /// Synthesis obligations.
    Generated,
    /// Explicit authored inputs.
    Authored,
    /// Both declared inventories.
    GeneratedAndAuthored,
}
impl Origins {
    /// Whether an origin belongs to this selection.
    pub fn includes(self, origin: Origin) -> bool {
        matches!(
            (self, origin),
            (Self::GeneratedAndAuthored, _)
                | (Self::Generated, Origin::Generated)
                | (Self::Authored, Origin::Authored)
        )
    }
}
/// Declared system or component boundary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Scope {
    /// Whole system.
    System,
    /// One model component.
    Component {
        /// A checked component name.
        #[serde(deserialize_with = "deserialize_component")]
        component: ComponentName,
    },
}
impl Scope {
    /// Select one syntactically checked component; builders additionally resolve it in the model.
    pub fn component(name: &str) -> Result<Self, AdmissionError> {
        Ok(Self::Component {
            component: ComponentName::new(name).map_err(|e| error(e.to_string()))?,
        })
    }
}
fn deserialize_component<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> Result<ComponentName, D::Error> {
    ComponentName::new(String::deserialize(d)?).map_err(serde::de::Error::custom)
}
/// Full selection or an exact narrowing of one admitted parent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Filter {
    /// No explicit ID filtering.
    All,
    /// The exact IDs retained from the named parent.
    Explicit {
        /// Sorted distinct retained IDs; empty is meaningful.
        ids: Vec<ScenarioId>,
        /// Exact original parent bytes.
        parent: SuiteReference,
    },
}
/// The boundary against which completeness is assessed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Selection {
    /// Declared system/component boundary.
    pub scope: Scope,
    /// Declared origin inventory.
    pub origins: Origins,
    /// Optional explicit narrowing.
    pub filter: Filter,
}
/// Whether the producer enumerated its declared inventory.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Knowledge {
    /// Every declared obligation/input was enumerated.
    CompleteInventory,
    /// No completeness claim.
    Unknown,
}
/// Why a known candidate is outside the selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutsideReason {
    /// Requires another component.
    OtherComponent,
    /// Origin excluded when the inventory was built.
    OriginSelection,
    /// Removed by an explicit parent filter.
    SelectionFilter,
}
/// A retained candidate outside the selected executable set.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Outside {
    /// Known scenario identity.
    pub scenario: ScenarioId,
    /// Final survivor origin.
    pub origin: Origin,
    /// The selection decision.
    pub reason: OutsideReason,
    /// Sorted dependencies proving component exclusion, empty otherwise.
    pub needs: Vec<EssSemanticRef>,
}
/// What one refusal occurrence omitted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Effect {
    /// One check in a candidate, possibly beside a retained scenario.
    CheckNotEmitted,
    /// A candidate, possibly rejected beside a retained duplicate.
    CandidateNotEmitted,
}
/// Refusal classification fixed before explicit filtering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RefusalScope {
    /// Prevents complete conformance for this selection.
    InScope,
    /// The refused obligation itself demonstrably requires another component.
    OutsideComponent,
    /// Its origin was excluded at fresh construction.
    OutsideOrigin,
}
/// Final surviving candidate ownership.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Retained {
    /// Survivor origin.
    pub origin: Origin,
    /// Accepted source for an authored survivor; null for generated.
    pub source: Option<SourceIdentity>,
}
/// One actual refusal occurrence. Identical records retain multiplicity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Refusal {
    /// Rejected/omitted candidate origin.
    pub origin: Origin,
    /// Known candidate identity, even when a duplicate survives.
    pub scenario: Option<ScenarioId>,
    /// Known generated construct or optional authored subject.
    pub subject: Option<EssSemanticRef>,
    /// Rejected authored source identity.
    pub source: Option<SourceIdentity>,
    /// Existing closed producer code.
    pub code: String,
    /// Original producer cause text.
    pub message: String,
    /// What this occurrence omitted.
    pub effect: Effect,
    /// Final owner, exactly when a selected or outside candidate survives.
    pub retained: Option<Retained>,
    /// Classification fixed before explicit filtering.
    pub scope: RefusalScope,
    /// Proof dependencies only for outside-component classification.
    pub needs: Vec<EssSemanticRef>,
}
/// Final authored candidate disposition after merging.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Disposition {
    /// Owns exactly one surviving authored scenario.
    Accepted,
    /// Compilation or merging rejected this source.
    Refused,
}
/// An original source and its final inventory result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthoredSource {
    /// SHA-256 of original source bytes.
    pub digest: String,
    /// Known original scenario identity.
    pub scenario: Option<ScenarioId>,
    /// Final disposition.
    pub disposition: Disposition,
}
/// Checked inventory lengths, never additional execution outcomes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Counts {
    /// Selected generated candidates.
    pub generated: u64,
    /// Selected authored candidates.
    pub authored: u64,
    /// Known outside survivors.
    pub outside: u64,
    /// Refusal occurrences including repeated equal records.
    pub refused: u64,
}
/// Unadmitted inventory value; only an admitted suite grants execution authority.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Inventory {
    /// Exact declared boundary.
    pub selection: Selection,
    /// Completeness claim independent of execution.
    pub knowledge: Knowledge,
    /// Sorted selected generated IDs.
    pub generated: Vec<ScenarioId>,
    /// Sorted selected authored IDs.
    pub authored: Vec<ScenarioId>,
    /// Sorted known outside candidates.
    pub outside: Vec<Outside>,
    /// Sorted occurrences preserving multiplicity.
    pub refused: Vec<Refusal>,
    /// Every declared input, including rejected files.
    pub authored_sources: BTreeMap<SourceIdentity, AuthoredSource>,
    /// Exact array lengths.
    pub counts: Counts,
}

fn distinct<T: Ord>(values: &[T]) -> bool {
    values.windows(2).all(|w| w[0] < w[1])
}
fn require(condition: bool, detail: &str) -> Result<(), AdmissionError> {
    if condition {
        Ok(())
    } else {
        Err(error(detail))
    }
}
fn refusal_cmp(a: &Refusal, b: &Refusal) -> std::cmp::Ordering {
    a.origin
        .cmp(&b.origin)
        .then_with(|| a.source.cmp(&b.source))
        .then_with(|| a.subject.cmp(&b.subject))
        .then_with(|| a.scenario.cmp(&b.scenario))
        .then_with(|| a.code.cmp(&b.code))
        .then_with(|| format!("{:?}", a.effect).cmp(&format!("{:?}", b.effect)))
        .then_with(|| {
            a.retained
                .as_ref()
                .map(|r| r.origin)
                .cmp(&b.retained.as_ref().map(|r| r.origin))
        })
        .then_with(|| {
            a.retained
                .as_ref()
                .and_then(|r| r.source.as_ref())
                .cmp(&b.retained.as_ref().and_then(|r| r.source.as_ref()))
        })
        .then_with(|| format!("{:?}", a.scope).cmp(&format!("{:?}", b.scope)))
        .then_with(|| a.needs.cmp(&b.needs))
        .then_with(|| a.message.cmp(&b.message))
}

impl Inventory {
    /// Whether this declared inventory is known and has no in-scope omissions.
    pub fn is_complete(&self) -> bool {
        self.knowledge == Knowledge::CompleteInventory
            && !self
                .refused
                .iter()
                .any(|r| r.scope == RefusalScope::InScope)
    }
    pub(crate) fn sort_and_count(&mut self) -> Result<(), AdmissionError> {
        self.generated.sort();
        self.authored.sort();
        self.outside.sort_by(|a, b| a.scenario.cmp(&b.scenario));
        self.refused.sort_by(refusal_cmp);
        self.counts = Counts {
            generated: u64::try_from(self.generated.len()).map_err(|e| error(e.to_string()))?,
            authored: u64::try_from(self.authored.len()).map_err(|e| error(e.to_string()))?,
            outside: u64::try_from(self.outside.len()).map_err(|e| error(e.to_string()))?,
            refused: u64::try_from(self.refused.len()).map_err(|e| error(e.to_string()))?,
        };
        Ok(())
    }
    pub(crate) fn validate(&self, suite: &ConformanceSuite) -> Result<(), AdmissionError> {
        require(
            suite.provenance.suite_version.major() >= 7
                || self.refused.iter().all(|r| r.code != "ESS-SYNTH-015"),
            "accessor observation refusals require suite/7",
        )?;
        let component = match &self.selection.scope {
            Scope::System => None,
            Scope::Component { component } => Some(component.as_str()),
        };
        require(
            component == suite.provenance.component.as_deref(),
            "scope contradicts provenance.component",
        )?;
        require(
            distinct(&self.generated) && distinct(&self.authored),
            "selected origin lists must be sorted and distinct",
        )?;
        let mut survivors: BTreeMap<&ScenarioId, Origin> = BTreeMap::new();
        for (ids, origin) in [
            (&self.generated, Origin::Generated),
            (&self.authored, Origin::Authored),
        ] {
            require(
                ids.is_empty() || self.selection.origins.includes(origin),
                "selected origin is excluded",
            )?;
            for id in ids {
                require(
                    survivors.insert(id, origin).is_none(),
                    "selected origins overlap",
                )?;
            }
        }
        require(
            survivors.keys().copied().eq(suite.scenarios.keys()),
            "selected origin union differs from scenarios",
        )?;
        require(
            self.outside
                .windows(2)
                .all(|w| w[0].scenario < w[1].scenario),
            "outside IDs must be sorted and unique",
        )?;
        for outside in &self.outside {
            require(
                survivors
                    .insert(&outside.scenario, outside.origin)
                    .is_none(),
                "outside and selected IDs overlap",
            )?;
            require(
                distinct(&outside.needs),
                "outside needs must be sorted and distinct",
            )?;
            require(
                (outside.reason == OutsideReason::OtherComponent) != outside.needs.is_empty(),
                "outside component proof must be nonempty, other needs empty",
            )?;
            match outside.reason {
                OutsideReason::OtherComponent => require(
                    component.is_some() && self.selection.origins.includes(outside.origin),
                    "invalid outside-component scope",
                )?,
                OutsideReason::OriginSelection => require(
                    !self.selection.origins.includes(outside.origin),
                    "outside origin is included",
                )?,
                OutsideReason::SelectionFilter => require(
                    matches!(self.selection.filter, Filter::Explicit { .. })
                        && self.selection.origins.includes(outside.origin),
                    "selection-filter entry without explicit filter",
                )?,
            }
        }
        self.validate_sources_and_refusals(component.is_some(), &survivors)?;
        let mut counted = self.clone();
        counted.sort_and_count()?;
        require(
            self.counts == counted.counts,
            "inventory counts differ from array lengths",
        )?;
        if let Filter::Explicit { ids, parent } = &self.selection.filter {
            require(
                distinct(ids) && ids.iter().eq(suite.scenarios.keys()),
                "explicit IDs differ from selected IDs",
            )?;
            require(
                matches!(
                    parent.version.as_str(),
                    COVERAGE_SUITE_FORMAT | "ess-conformance/7" | "ess-conformance/9"
                ) && parent.version == suite.provenance.suite_version.to_string()
                    && parent.digest_profile == "sha256-json-bytes/1"
                    && valid_digest(&parent.digest),
                "invalid parent reference",
            )?;
        }
        Ok(())
    }
    fn validate_sources_and_refusals(
        &self,
        component: bool,
        survivors: &BTreeMap<&ScenarioId, Origin>,
    ) -> Result<(), AdmissionError> {
        let mut owners = BTreeMap::new();
        for (identity, source) in &self.authored_sources {
            require(valid_digest(&source.digest), "invalid source digest")?;
            if source.disposition == Disposition::Accepted {
                let id = source
                    .scenario
                    .as_ref()
                    .ok_or_else(|| error("accepted source lacks scenario"))?;
                require(
                    survivors.get(id) == Some(&Origin::Authored),
                    "accepted source lacks an authored survivor",
                )?;
                require(
                    owners.insert(id, identity).is_none(),
                    "multiple sources own one survivor",
                )?;
            } else {
                require(
                    self.refused
                        .iter()
                        .any(|r| r.source.as_ref() == Some(identity)),
                    "refused source lacks refusal",
                )?;
            }
        }
        require(
            survivors
                .iter()
                .filter(|(_, o)| **o == Origin::Authored)
                .all(|(id, _)| owners.contains_key(id)),
            "authored survivor lacks accepted source",
        )?;
        require(
            self.refused
                .windows(2)
                .all(|w| !refusal_cmp(&w[0], &w[1]).is_gt()),
            "refusals must be sorted with multiplicity",
        )?;
        for refusal in &self.refused {
            self.validate_refusal(refusal, component, survivors, &owners)?;
        }
        Ok(())
    }
    fn validate_refusal(
        &self,
        r: &Refusal,
        component: bool,
        survivors: &BTreeMap<&ScenarioId, Origin>,
        owners: &BTreeMap<&ScenarioId, &SourceIdentity>,
    ) -> Result<(), AdmissionError> {
        require(!r.message.trim().is_empty(), "empty refusal cause text")?;
        match r.origin {
            Origin::Generated => {
                require(
                    r.subject.is_some() && r.source.is_none(),
                    "generated refusal requires subject and null source",
                )?;
                let expected = (1..=15)
                    .find(|n| r.code == format!("ESS-SYNTH-{n:03}"))
                    .ok_or_else(|| error("unknown generated refusal code"))?;
                let effect = if matches!(expected, 5 | 11 | 12 | 14) {
                    Effect::CheckNotEmitted
                } else {
                    Effect::CandidateNotEmitted
                };
                require(
                    r.effect == effect,
                    "generated refusal effect contradicts code",
                )?;
            }
            Origin::Authored => {
                require(
                    (1..=35).any(|n| r.code == format!("ESS-AUTHOR-{n:03}"))
                        && r.effect == Effect::CandidateNotEmitted,
                    "unknown authored code/effect",
                )?;
                let source = r
                    .source
                    .as_ref()
                    .and_then(|id| self.authored_sources.get(id))
                    .ok_or_else(|| error("authored refusal lacks source inventory"))?;
                require(
                    source.disposition == Disposition::Refused && source.scenario == r.scenario,
                    "authored refusal contradicts source disposition/identity",
                )?;
            }
        }
        let retained = r.scenario.as_ref().and_then(|id| {
            survivors.get(id).map(|origin| Retained {
                origin: *origin,
                source: owners.get(id).map(|s| (*s).clone()),
            })
        });
        require(
            r.retained == retained,
            "refusal survivor identity is inconsistent",
        )?;
        require(
            distinct(&r.needs),
            "refusal needs must be sorted and distinct",
        )?;
        require(
            (r.scope == RefusalScope::OutsideComponent) != r.needs.is_empty(),
            "refusal component proof must be nonempty, other needs empty",
        )?;
        match r.scope {
            RefusalScope::OutsideOrigin => require(
                !self.selection.origins.includes(r.origin),
                "outside-origin refusal is included",
            )?,
            RefusalScope::OutsideComponent => require(
                component && self.selection.origins.includes(r.origin),
                "outside-component refusal has invalid scope",
            )?,
            RefusalScope::InScope => require(
                self.selection.origins.includes(r.origin),
                "in-scope refusal has excluded origin",
            )?,
        }
        Ok(())
    }
}

/// Check raw integer spellings and every required nullable field before typed decoding.
pub(crate) fn parse_inventory(
    value: &Json,
    suite: &ConformanceSuite,
) -> Result<Inventory, AdmissionError> {
    let fields = value.closed(
        &[
            "selection",
            "knowledge",
            "generated",
            "authored",
            "outside",
            "refused",
            "authored_sources",
            "counts",
        ],
        &[],
    )?;
    validate_selection_json(&fields["selection"])?;
    let counts = fields["counts"].closed(&["generated", "authored", "outside", "refused"], &[])?;
    for value in counts.values() {
        value.unsigned()?;
    }
    for source in fields["authored_sources"].object()?.values() {
        source.closed(&["digest", "scenario", "disposition"], &[])?;
    }
    validate_refusals_json(&fields["refused"])?;
    for outside in fields["outside"].array()? {
        let o = outside.closed(&["scenario", "origin", "reason", "needs"], &[])?;
        for need in o["needs"].array()? {
            crate::admission::semantic_reference(need)?;
        }
    }
    let inventory: Inventory = serde_json::from_str(&value.raw)
        .map_err(|e| value.error("InvalidCoverage", e.to_string()))?;
    inventory.validate(suite)?;
    Ok(inventory)
}
fn validate_selection_json(value: &Json) -> Result<(), AdmissionError> {
    let selection = value.closed(&["scope", "origins", "filter"], &[])?;
    let scope = &selection["scope"];
    let kind = scope
        .object()?
        .get("kind")
        .ok_or_else(|| error("scope lacks kind"))?
        .text()?;
    match kind {
        "system" => {
            scope.closed(&["kind"], &[])?;
        }
        "component" => {
            scope.closed(&["kind", "component"], &[])?;
        }
        _ => return Err(error("unsupported coverage scope")),
    }
    let filter = &selection["filter"];
    let kind = filter
        .object()?
        .get("kind")
        .ok_or_else(|| error("filter lacks kind"))?
        .text()?;
    match kind {
        "all" => {
            filter.closed(&["kind"], &[])?;
        }
        "explicit" => {
            let fields = filter.closed(&["kind", "ids", "parent"], &[])?;
            fields["parent"].closed(&["version", "digest_profile", "digest"], &[])?;
        }
        _ => return Err(error("unsupported coverage filter")),
    }
    Ok(())
}
fn validate_refusals_json(value: &Json) -> Result<(), AdmissionError> {
    for refusal in value.array()? {
        let r = refusal.closed(
            &[
                "origin", "scenario", "subject", "source", "code", "message", "effect", "retained",
                "scope", "needs",
            ],
            &[],
        )?;
        if !r["retained"].null() {
            r["retained"].closed(&["origin", "source"], &[])?;
        }
        if !r["subject"].null() {
            crate::admission::semantic_reference(&r["subject"])?;
        }
        for need in r["needs"].array()? {
            crate::admission::semantic_reference(need)?;
        }
    }
    Ok(())
}
pub(crate) fn validate_summary_json(value: &Json, coverage: bool) -> Result<(), AdmissionError> {
    if coverage {
        let fields = value.closed(&["knowledge", "selection", "counts", "refused"], &[])?;
        validate_selection_json(&fields["selection"])?;
        for value in fields["counts"]
            .closed(&["generated", "authored", "outside", "refused"], &[])?
            .values()
        {
            value.unsigned()?;
        }
        validate_refusals_json(&fields["refused"])?;
    } else {
        value.closed(&["knowledge"], &[])?;
    }
    Ok(())
}

/// Structurally parsed carrier. This value alone is not execution authority.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SuiteInputDocument {
    /// Exactly ess-conformance-input/1.
    pub format: String,
    /// Original selected suite JSON, unchanged.
    pub suite_json: String,
    /// Original parent JSON strings, nearest first.
    pub parent_suites: Vec<String>,
}
impl SuiteInputDocument {
    /// Parse a closed carrier without admitting its inner documents.
    pub fn from_json(original: &str) -> Result<Self, AdmissionError> {
        let value = Json::parse(original, "$input")?;
        let fields = value.closed(&["format", "suite_json", "parent_suites"], &[])?;
        require(
            fields["format"].text()? == SUITE_INPUT_FORMAT,
            "unsupported input format",
        )?;
        Ok(Self {
            format: SUITE_INPUT_FORMAT.into(),
            suite_json: fields["suite_json"].text()?.into(),
            parent_suites: fields["parent_suites"]
                .array()?
                .iter()
                .map(|v| v.text().map(str::to_owned))
                .collect::<Result<_, _>>()?,
        })
    }
    /// Canonical presentation; inner original strings are unchanged.
    pub fn to_canonical_json(&self) -> Result<String, AdmissionError> {
        let text = canonical(self)?;
        Self::from_json(&text)?;
        Ok(text)
    }
}
impl<'de> Deserialize<'de> for SuiteInputDocument {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let raw = Box::<serde_json::value::RawValue>::deserialize(d)?;
        Self::from_json(raw.get()).map_err(serde::de::Error::custom)
    }
}
/// Immutable selected suite and its completely admitted original parent chain.
#[derive(Debug, Clone)]
pub struct AdmittedInput {
    selected: AdmittedSuite,
    parents: Vec<AdmittedSuite>,
}
impl AdmittedInput {
    /// Admit a closed original-byte input carrier and every ancestor before execution.
    pub fn from_json(original: &str) -> Result<Self, AdmissionError> {
        let document = SuiteInputDocument::from_json(original)?;
        let selected = AdmittedSuite::parse(&document.suite_json)?;
        require(
            selected.coverage().is_some(),
            "input/1 requires coverage suite/5, suite/7 or suite/9",
        )?;
        let parents = document
            .parent_suites
            .iter()
            .map(|s| AdmittedSuite::parse(s))
            .collect::<Result<Vec<_>, _>>()?;
        let mut seen = BTreeSet::from([selected.digest()]);
        let mut child = &selected;
        for parent in &parents {
            require(
                seen.insert(parent.digest()),
                "duplicate suite in parent chain",
            )?;
            validate_parent(child, parent)?;
            child = parent;
        }
        require(
            matches!(
                child.coverage().map(|c| &c.selection.filter),
                Some(Filter::All)
            ),
            "missing final unfiltered parent",
        )?;
        Ok(Self { selected, parents })
    }
    /// Wrap a directly admitted unfiltered suite/5 without inventing source bytes.
    pub fn from_suite(suite: AdmittedSuite) -> Result<Self, AdmissionError> {
        require(
            suite.coverage().is_some(),
            "input/1 requires coverage suite/5, suite/7 or suite/9",
        )?;
        require(
            matches!(
                suite.coverage().map(|c| &c.selection.filter),
                Some(Filter::All)
            ),
            "direct suite requires no parents",
        )?;
        Ok(Self {
            selected: suite,
            parents: Vec::new(),
        })
    }
    /// Selected immutable execution capability.
    pub fn selected(&self) -> &AdmittedSuite {
        &self.selected
    }
    /// Complete immutable nearest-first parent chain.
    pub fn parents(&self) -> &[AdmittedSuite] {
        &self.parents
    }
    /// Original strings suitable for a new carrier; no bytes are reserialized internally.
    pub fn document(&self) -> SuiteInputDocument {
        SuiteInputDocument {
            format: SUITE_INPUT_FORMAT.into(),
            suite_json: self.selected.original_json().into(),
            parent_suites: self
                .parents
                .iter()
                .map(|s| s.original_json().into())
                .collect(),
        }
    }
    /// Narrow only IDs, preserving the entire admitted inventory and every original parent.
    pub fn select(&self, ids: &[ScenarioId]) -> Result<Self, AdmissionError> {
        let inventory = narrowed_inventory(&self.selected, ids)?;
        let mut suite = self.selected.suite().clone();
        suite
            .scenarios
            .retain(|id, _| ids.binary_search(id).is_ok());
        let text = suite_document(&suite, &inventory)?;
        let mut document = self.document();
        document.parent_suites.insert(0, document.suite_json);
        document.suite_json = text;
        Self::from_json(&document.to_canonical_json()?)
    }
}

#[derive(Serialize)]
struct Document<'a> {
    provenance: &'a crate::SuiteProvenance,
    scenarios: &'a BTreeMap<ScenarioId, crate::ConformanceScenario>,
    coverage: &'a Inventory,
}

pub(crate) fn suite_document(
    suite: &ConformanceSuite,
    coverage: &Inventory,
) -> Result<String, AdmissionError> {
    crate::admission::suite(suite)?;
    canonical(&Document {
        provenance: &suite.provenance,
        scenarios: &suite.scenarios,
        coverage,
    })
}

/// Write a freshly produced coverage suite compactly, retaining its entire inventory.
///
/// Object order matches the existing sorted canonical coverage document. One newline
/// terminates the artifact. Its exact byte digest differs from the pretty artifact;
/// this function does not rewrite an admitted input or its retained parent documents.
pub fn compact_suite_document(
    suite: &ConformanceSuite,
    coverage: &Inventory,
) -> Result<String, AdmissionError> {
    crate::admission::suite(suite)?;
    coverage.validate(suite)?;
    let mut value = serde_json::to_value(Document {
        provenance: &suite.provenance,
        scenarios: &suite.scenarios,
        coverage,
    })
    .map_err(|e| error(e.to_string()))?;
    value.sort_all_objects();
    let mut text = serde_json::to_string(&value).map_err(|e| error(e.to_string()))?;
    text.push('\n');
    Ok(text)
}
fn narrowed_inventory(
    parent: &AdmittedSuite,
    ids: &[ScenarioId],
) -> Result<Inventory, AdmissionError> {
    require(distinct(ids), "explicit IDs must be sorted and distinct")?;
    require(
        ids.iter()
            .all(|id| parent.suite().scenarios.contains_key(id)),
        "explicit ID absent from parent",
    )?;
    let mut inventory = parent
        .coverage()
        .ok_or_else(|| error("legacy parent has no coverage inventory"))?
        .clone();
    inventory.selection.filter = Filter::Explicit {
        ids: ids.to_vec(),
        parent: SuiteReference::of(parent),
    };
    for (selected, origin) in [
        (&mut inventory.generated, Origin::Generated),
        (&mut inventory.authored, Origin::Authored),
    ] {
        for id in selected.iter().filter(|id| ids.binary_search(id).is_err()) {
            inventory.outside.push(Outside {
                scenario: id.clone(),
                origin,
                reason: OutsideReason::SelectionFilter,
                needs: Vec::new(),
            });
        }
        selected.retain(|id| ids.binary_search(id).is_ok());
    }
    inventory.sort_and_count()?;
    Ok(inventory)
}
fn validate_parent(child: &AdmittedSuite, parent: &AdmittedSuite) -> Result<(), AdmissionError> {
    let inventory = child
        .coverage()
        .ok_or_else(|| error("child has no inventory"))?;
    let Filter::Explicit {
        ids,
        parent: reference,
    } = &inventory.selection.filter
    else {
        return Err(error("unused parent after an unfiltered suite"));
    };
    require(
        *reference == SuiteReference::of(parent),
        "parent original-byte reference mismatch",
    )?;
    require(
        child.suite().provenance == parent.suite().provenance,
        "parent/child provenance differs",
    )?;
    require(
        *inventory == narrowed_inventory(parent, ids)?,
        "child rewrote retained parent inventory",
    )?;
    for (id, scenario) in &child.suite().scenarios {
        require(
            parent.suite().scenarios.get(id) == Some(scenario),
            "child changed full scenario semantics or dependencies",
        )?;
    }
    Ok(())
}
