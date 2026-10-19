//! The `value_invariants` view expectation: a value type's invariants observed at a typed position
//! inside a view field, through `List`, `Map`, `Optional` and `Union` wrappers
//! (`docs/design/wrapped-value-invariant-observations.md`).
//!
//! # What it asserts
//!
//! For one nominal type (`value`) at one typed position under one view field (`root`, `position`),
//! every actual occurrence on every returned row satisfies every invariant the type declares, and
//! at least one actual occurrence exists on a row the witness selects. An absent `Optional`, an
//! empty `List` or `Map` and another `Union` variant are legal and are not failures, but none of
//! them is a witness. The declarations the position reaches travel with the expectation, so a
//! runner that holds no IR still checks the whole value: its shape, every declared alphabet and
//! prefix, and the invariants of the selected type.
//!
//! # Wire form
//!
//! ```json
//! {"expect": "value_invariants",
//!  "root": {"name": "tags", "type": "List<demo.Tag>"},
//!  "declarations": {"demo.Tag": {"kind": "newtype", "of": "String",
//!                                "invariants": [{"statement": "value.count >= 1", "predicate": …}]}},
//!  "value": "demo.Tag",
//!  "position": {"kind": "site", "path": [{"op": "element"}]},
//!  "witness": {"kind": "any_observed_occurrence"},
//!  "work_limit": 1048576}
//! ```
//!
//! The scenario carrying it is `<value>/invariant-wrapped/at/<view>/<root>/<digest>`, where
//! `digest` is the lowercase SHA-256 of the position's compact JSON ([`position_digest`]).
//!
//! # What this build executes
//!
//! The native runner executes a `site` position under either witness form with a projected
//! identity selector, and counts the logical work schedule of the design (shape, observation, facts,
//! predicate preflight) against `work_limit`. A `recursive` position and an `identity_query`
//! selector are admitted as wire forms and reported `unsupported`, naming the expectation, as the
//! generated Go and TypeScript runners report every `value_invariants` expectation.
//!
//! A suite carrying one is ordinary suite/[`ORDINARY`] or coverage suite/[`COVERAGE`]; a `/51`
//! reader refuses it by version, and every reader refuses the expectation under a lower major.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use ess_domain::name::QualifiedName;
use ess_domain::types::{Presence, Primitive, TypeRef};
use ess_primitives::facts::{FactPath, FactSource, FactStore, FactValue, Scales};
use ess_primitives::node::Node;
use ess_primitives::predicate::{Operand, Predicate, TextOperand, Truth};
use sha2::{Digest, Sha256};

use crate::admission::AdmissionError;
use crate::scenario::{DeclaredTypeRef, EntityRef};
use crate::{ConformanceSuite, ScenarioId, ScenarioStep, ScenarioValue, ViewExpectation};

/// The ordinary suite major that carries `value_invariants`.
pub const ORDINARY: u32 = 52;

/// Its coverage counterpart.
pub const COVERAGE: u32 = 53;

/// The suite majors this module introduces.
pub const ADMITTED: [u32; 2] = [ORDINARY, COVERAGE];

/// The wire tag.
pub const TAG: &str = "value_invariants";

/// The work budget a producer writes unless it chooses another.
pub const DEFAULT_WORK_LIMIT: u64 = 1_048_576;

/// The largest work budget: the largest integer every JSON reader holds exactly.
pub const MAX_WORK_LIMIT: u64 = 9_007_199_254_740_991;

/// The most declarations one expectation carries.
pub const MAX_DECLARATIONS: usize = 4096;

/// The most typed edges one root's expansion holds.
pub const MAX_EDGES: usize = 16_384;

/// The most canonical compact bytes one expectation takes.
pub const MAX_BYTES: usize = 1_048_576;

/// What a refusal under a lower major says.
pub const REQUIRES: &str = "value_invariants expectations require suite/52 or /53";

/// The closed members of the expectation, in order.
pub const MEMBERS: [&str; 7] = [
    "expect",
    "root",
    "declarations",
    "value",
    "position",
    "witness",
    "work_limit",
];

// ---- the closed DTO ----------------------------------------------------------------------------

/// The expectation's members after `expect`, in their wire order.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ValueInvariants {
    /// The view field the position is under.
    pub root: ObservedField,
    /// Every declaration the root and the selector reach, by fully qualified name.
    pub declarations: BTreeMap<String, Declaration>,
    /// The nominal type whose invariants are checked.
    pub value: DeclaredTypeRef,
    /// Where under the root the type is entered.
    pub position: InvariantPosition,
    /// Which rows may supply the required occurrence.
    pub witness: Witness,
    /// The execution budget of one snapshot, in logical work units.
    pub work_limit: u64,
}

/// A field: its semantic name, its canonical type and, for an `Optional`, its presence policy.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObservedField {
    /// The semantic field name.
    pub name: String,
    /// The canonical `TypeRef` string.
    #[serde(rename = "type")]
    pub type_ref: String,
    /// How an absent `Optional` value travels; absent admits both spellings.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub presence: Option<Presence>,
}

/// One declaration of the closure.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Declaration {
    /// A newtype: its representation, invariants and text constraints.
    Newtype {
        /// The representation, a canonical `TypeRef` string.
        of: String,
        /// Its invariants over `value`, in declaration order.
        invariants: Vec<DeclaredInvariant>,
        /// The characters every value is drawn from.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        alphabet: Option<String>,
        /// The text every value starts with.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        prefix: Option<String>,
    },
    /// A struct: its fields in declaration order and its invariants over them.
    Struct {
        /// Its fields.
        fields: Vec<ObservedField>,
        /// Its invariants.
        invariants: Vec<DeclaredInvariant>,
    },
    /// An enum: its semantic labels in declaration order.
    Enum {
        /// The labels.
        variants: Vec<String>,
    },
    /// A union: its tag field and each variant's payload type.
    Union {
        /// The field carrying the label.
        tag: String,
        /// Each label's payload, a canonical `TypeRef` string.
        variants: BTreeMap<String, String>,
    },
}

/// One invariant: the statement as written, and its compiled predicate.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeclaredInvariant {
    /// The statement, for a diagnostic; not identity.
    pub statement: String,
    /// The predicate, in its canonical serialization.
    pub predicate: Predicate,
}

/// Where the type is entered: an ordinary site, or a site reached across a recursive back-edge.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum InvariantPosition {
    /// The typed path from the root that ends on entry to the type.
    Site {
        /// The path.
        path: Vec<Segment>,
    },
    /// A site that must be reached on one value lineage after crossing a back-edge.
    Recursive {
        /// The back-edge.
        edge: Vec<Segment>,
        /// The site.
        site: Vec<Segment>,
    },
}

/// One typed schema edge.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum Segment {
    /// A struct member.
    Field {
        /// Its name.
        name: String,
    },
    /// An `Optional`'s present value.
    Present,
    /// Each element of a `List`.
    Element,
    /// Each value of a `Map`.
    MapValue,
    /// A union's payload under one label.
    Variant {
        /// The label.
        label: String,
    },
    /// A newtype's representation.
    Unwrap,
}

/// Which rows may supply the required occurrence.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
// Held inside the boxed expectation, once per step: the size of the selector costs nothing.
#[allow(clippy::large_enum_variant)]
pub enum Witness {
    /// Any returned row.
    AnyObservedOccurrence,
    /// A row the selector names.
    ArrangedRow {
        /// The selector.
        selector: Selector,
    },
}

/// A source-proved row selector.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Selector {
    /// The row whose projected entity identity is the arranged instance.
    ProjectedIdentity {
        /// The view's source entity.
        entity: EntityRef,
        /// Its identity field, projected unchanged.
        identity: ObservedField,
        /// The arranged instance.
        expected: ScenarioValue,
    },
    /// Every row of a query whose filter equates the identity to a parameter.
    IdentityQuery {
        /// The view's source entity.
        entity: EntityRef,
        /// Its identity field.
        identity: ObservedField,
        /// The parameter the filter equates it to.
        parameter: ObservedField,
        /// The view's full source filter.
        filter: Predicate,
        /// The parameter's value.
        expected: ScenarioValue,
    },
}

impl ValueInvariants {
    /// Every declared invariant predicate the expectation carries, for the feature visitors.
    pub fn predicates(&self) -> impl Iterator<Item = &Predicate> {
        self.declarations
            .values()
            .flat_map(|declaration| match declaration {
                Declaration::Newtype { invariants, .. }
                | Declaration::Struct { invariants, .. } => invariants.as_slice(),
                Declaration::Enum { .. } | Declaration::Union { .. } => &[],
            })
            .map(|invariant| &invariant.predicate)
    }
}

/// The lowercase SHA-256 of the position's canonical compact JSON, with no newline.
pub fn position_digest(position: &InvariantPosition) -> String {
    let compact = serde_json::to_string(position).expect("a position serializes");
    Sha256::digest(compact.as_bytes())
        .iter()
        .fold(String::with_capacity(64), |mut text, byte| {
            write!(text, "{byte:02x}").expect("writing to a String");
            text
        })
}

/// The scenario id an expectation at `view` is filed under.
pub fn scenario_id(view: &crate::scenario::ViewRef, observed: &ValueInvariants) -> ScenarioId {
    ScenarioId::ValueInvariantWrapped {
        value: observed.value.clone(),
        at: view.clone(),
        root: observed.root.name.clone(),
        digest: position_digest(&observed.position),
    }
}

// ---- format authority --------------------------------------------------------------------------

fn observations(
    suite: &ConformanceSuite,
) -> impl Iterator<Item = (&ScenarioId, usize, &ScenarioStep)> {
    suite.scenarios.iter().flat_map(|(id, scenario)| {
        scenario
            .steps
            .iter()
            .enumerate()
            .map(move |(index, step)| (id, index, step))
    })
}

/// The view and observation a step carries, when it carries one.
pub(crate) fn observed_by(
    step: &ScenarioStep,
) -> Option<(&crate::scenario::ViewRef, &ValueInvariants)> {
    match step {
        ScenarioStep::ExpectView {
            view,
            expectation: ViewExpectation::ValueInvariants(observed),
        }
        | ScenarioStep::EventuallyView {
            view,
            expectation: ViewExpectation::ValueInvariants(observed),
            ..
        } => Some((view, observed)),
        _ => None,
    }
}

/// Whether a suite carries a `value_invariants` expectation or a wrapped scenario id.
pub fn used_by(suite: &ConformanceSuite) -> bool {
    suite
        .scenarios
        .keys()
        .any(|id| matches!(id, ScenarioId::ValueInvariantWrapped { .. }))
        || observations(suite).any(|(_, _, step)| observed_by(step).is_some())
}

/// The ordinary major a fresh suite needs at least: [`ORDINARY`] when [`used_by`].
pub fn ordinary_floor(suite: &ConformanceSuite) -> Option<u32> {
    used_by(suite).then_some(ORDINARY)
}

/// The coverage major a fresh coverage suite needs at least: [`COVERAGE`] when [`used_by`].
pub fn coverage_floor(suite: &ConformanceSuite) -> Option<u32> {
    used_by(suite).then_some(COVERAGE)
}

/// Refuse a suite pinned below `/52` that carries the vocabulary, an expectation whose contract
/// does not admit, and an expectation filed under an id that does not name it.
pub(crate) fn admit(suite: &ConformanceSuite) -> Result<(), AdmissionError> {
    if used_by(suite) && suite.provenance.suite_version.major() < ORDINARY {
        return Err(AdmissionError::new(
            "UnsupportedVocabulary",
            "$suite",
            REQUIRES,
        ));
    }
    for (id, scenario) in &suite.scenarios {
        let mut named = false;
        for (index, step) in scenario.steps.iter().enumerate() {
            let Some((view, observed)) = observed_by(step) else {
                continue;
            };
            let at = format!("$suite.scenarios.{id}.steps.{index}.expectation");
            Contract::admit(observed)
                .map_err(|reason| AdmissionError::new("InvalidValueInvariants", &at, reason))?;
            if *id != scenario_id(view, observed) {
                return Err(AdmissionError::new(
                    "InvalidValueInvariants",
                    at,
                    format!(
                        "a value_invariants expectation is filed under `{}`, not `{id}`",
                        scenario_id(view, observed)
                    ),
                ));
            }
            named = true;
        }
        if matches!(id, ScenarioId::ValueInvariantWrapped { .. }) && !named {
            return Err(AdmissionError::new(
                "InvalidValueInvariants",
                format!("$suite.scenarios.{id}"),
                "a wrapped value-invariant scenario carries the value_invariants expectation it \
                 is named for",
            ));
        }
    }
    Ok(())
}

/// Admit one raw expectation of a suite document of `major`, before it is decoded.
pub(crate) fn admit_json(
    value: &crate::count_json::Json,
    major: u32,
) -> Result<(), AdmissionError> {
    if major < ORDINARY {
        return Err(value.error("UnsupportedVocabulary", REQUIRES));
    }
    let members = value.closed(&MEMBERS, &[])?;
    members["work_limit"].unsigned()?;
    for declaration in members["declarations"].object()?.values() {
        if let Some(invariants) = declaration.object()?.get("invariants") {
            for invariant in invariants.array()? {
                let predicate = &invariant
                    .object()?
                    .get("predicate")
                    .ok_or_else(|| invariant.error("MissingField", "predicate"))?;
                predicate.payload()?;
                crate::quoted_predicate_format::admit_predicate(&predicate.raw, major)?;
                crate::text_match_format::admit_predicate(&predicate.raw, major)?;
                crate::expression_format::admit_predicate(&predicate.raw, major)?;
            }
        }
    }
    let decoded: ViewExpectation = serde_json::from_str(&value.raw)
        .map_err(|error| value.error("InvalidValueInvariants", error.to_string()))?;
    let ViewExpectation::ValueInvariants(observed) = decoded else {
        return Err(value.error(
            "InvalidValueInvariants",
            "not a value_invariants expectation",
        ));
    };
    if value.raw.len() > MAX_BYTES {
        return Err(value.error("ValueInvariantsResource", "the expectation exceeds 1 MiB"));
    }
    Contract::admit(&observed).map_err(|reason| value.error("InvalidValueInvariants", reason))?;
    Ok(())
}

// ---- the admitted contract ---------------------------------------------------------------------

/// One declaration with its types parsed.
#[derive(Debug, Clone)]
enum Body<'a> {
    Newtype {
        of: TypeRef,
        invariants: &'a [DeclaredInvariant],
        alphabet: Option<BTreeSet<char>>,
        prefix: Option<&'a str>,
    },
    Struct {
        fields: Vec<(&'a str, TypeRef, Option<Presence>)>,
        invariants: &'a [DeclaredInvariant],
    },
    Enum {
        variants: BTreeSet<&'a str>,
    },
    Union {
        tag: &'a str,
        variants: BTreeMap<&'a str, TypeRef>,
    },
}

/// An expectation whose closed structure, closure, position and budget admitted.
#[derive(Debug, Clone)]
pub(crate) struct Contract<'a> {
    observed: &'a ValueInvariants,
    root: TypeRef,
    types: BTreeMap<QualifiedName, Body<'a>>,
    value: QualifiedName,
    site: Option<Vec<Segment>>,
}

fn parse_type(raw: &str) -> Result<TypeRef, String> {
    let parsed = TypeRef::parse(raw).map_err(|error| format!("`{raw}`: {error}"))?;
    if parsed.to_string() != raw {
        return Err(format!(
            "`{raw}` is not the canonical spelling `{parsed}` of its type"
        ));
    }
    Ok(parsed)
}

fn field_name(name: &str) -> Result<(), String> {
    match FactPath::new(name) {
        Ok(path) if path.segments().len() == 1 => Ok(()),
        _ => Err(format!("`{name}` is not a field name")),
    }
}

fn presence_admitted(field: &ObservedField, type_ref: &TypeRef) -> Result<(), String> {
    if field.presence.is_some() && !matches!(type_ref, TypeRef::Optional(_)) {
        return Err(format!(
            "`{}` declares a presence policy and is not Optional",
            field.name
        ));
    }
    Ok(())
}

impl<'a> Contract<'a> {
    /// Check everything the expectation claims that can be checked without the IR.
    #[allow(clippy::too_many_lines)]
    pub(crate) fn admit(observed: &'a ValueInvariants) -> Result<Self, String> {
        if observed.work_limit == 0 || observed.work_limit > MAX_WORK_LIMIT {
            return Err(format!(
                "work_limit is a positive integer of at most {MAX_WORK_LIMIT}"
            ));
        }
        if observed.declarations.len() > MAX_DECLARATIONS {
            return Err(format!("more than {MAX_DECLARATIONS} declarations"));
        }
        field_name(&observed.root.name)?;
        let root = parse_type(&observed.root.type_ref)?;
        presence_admitted(&observed.root, &root)?;
        let mut types = BTreeMap::new();
        for (name, declaration) in &observed.declarations {
            let qualified = QualifiedName::new(name)
                .map_err(|error| format!("`{name}` is not a declaration name: {error}"))?;
            let body = match declaration {
                Declaration::Newtype {
                    of,
                    invariants,
                    alphabet,
                    prefix,
                } => Body::Newtype {
                    of: parse_type(of)?,
                    invariants,
                    alphabet: alphabet.as_ref().map(|text| text.chars().collect()),
                    prefix: prefix.as_deref(),
                },
                Declaration::Struct { fields, invariants } => {
                    let mut parsed = Vec::new();
                    let mut seen = BTreeSet::new();
                    for field in fields {
                        field_name(&field.name)?;
                        if !seen.insert(field.name.as_str()) {
                            return Err(format!("`{name}` declares `{}` twice", field.name));
                        }
                        let type_ref = parse_type(&field.type_ref)?;
                        presence_admitted(field, &type_ref)?;
                        parsed.push((field.name.as_str(), type_ref, field.presence));
                    }
                    Body::Struct {
                        fields: parsed,
                        invariants,
                    }
                }
                Declaration::Enum { variants } => {
                    let labels: BTreeSet<&str> = variants.iter().map(String::as_str).collect();
                    if labels.len() != variants.len() || labels.is_empty() {
                        return Err(format!("`{name}` declares no distinct labels"));
                    }
                    Body::Enum { variants: labels }
                }
                Declaration::Union { tag, variants } => {
                    field_name(tag)?;
                    if variants.is_empty() {
                        return Err(format!("`{name}` declares no variants"));
                    }
                    Body::Union {
                        tag,
                        variants: variants
                            .iter()
                            .map(|(label, type_ref)| Ok((label.as_str(), parse_type(type_ref)?)))
                            .collect::<Result<_, String>>()?,
                    }
                }
            };
            types.insert(qualified, body);
        }
        let value = observed.value.name().clone();
        let mut contract = Self {
            observed,
            root,
            types,
            value,
            site: None,
        };
        contract.closure()?;
        // The expectation asserts the invariants themselves, so it asks for them after binding
        // them, not for `is_constrained()`: an alphabet alone gives it nothing to assert.
        let asserted = match contract.types.get(&contract.value) {
            Some(Body::Newtype { invariants, .. } | Body::Struct { invariants, .. }) => {
                invariants.len()
            }
            _ => 0,
        };
        if asserted == 0 {
            return Err(format!(
                "`{}` is no declared newtype or struct with invariants",
                contract.value
            ));
        }
        for (name, body) in &contract.types {
            if let Body::Newtype { invariants, .. } | Body::Struct { invariants, .. } = body {
                for invariant in *invariants {
                    Self::rooted(name, body, &invariant.predicate)?;
                }
            }
        }
        match &observed.position {
            InvariantPosition::Site { path } => {
                contract.walk(path, true)?;
                contract.site = Some(path.clone());
            }
            InvariantPosition::Recursive { edge, site } => {
                contract.walk(edge, false)?;
                contract.walk(site, true)?;
            }
        }
        match &observed.witness {
            Witness::AnyObservedOccurrence => {}
            Witness::ArrangedRow {
                selector: Selector::ProjectedIdentity { identity, .. },
            } => {
                field_name(&identity.name)?;
                parse_type(&identity.type_ref)?;
            }
            Witness::ArrangedRow {
                selector:
                    Selector::IdentityQuery {
                        identity,
                        parameter,
                        ..
                    },
            } => {
                for field in [identity, parameter] {
                    field_name(&field.name)?;
                    parse_type(&field.type_ref)?;
                }
            }
        }
        Ok(contract)
    }

    /// The declarations are exactly what the root and the selector reach.
    fn closure(&self) -> Result<(), String> {
        let mut reached = BTreeSet::new();
        let mut pending: Vec<TypeRef> = vec![self.root.clone()];
        if let Witness::ArrangedRow { selector } = &self.observed.witness {
            let fields: Vec<&ObservedField> = match selector {
                Selector::ProjectedIdentity { identity, .. } => vec![identity],
                Selector::IdentityQuery {
                    identity,
                    parameter,
                    ..
                } => vec![identity, parameter],
            };
            for field in fields {
                pending.push(parse_type(&field.type_ref)?);
            }
        }
        let mut edges = 0_usize;
        while let Some(type_ref) = pending.pop() {
            edges += 1;
            if edges > MAX_EDGES {
                return Err(format!(
                    "the declarations reach more than {MAX_EDGES} types"
                ));
            }
            match type_ref {
                TypeRef::Primitive(primitive) => {
                    if primitive == Primitive::Binary64 {
                        return Err("a Binary64 value is outside this observer's profile".into());
                    }
                }
                TypeRef::Optional(of) | TypeRef::List(of) | TypeRef::Map(_, of) => {
                    pending.push(*of);
                }
                TypeRef::Named(name) => {
                    if !reached.insert(name.clone()) {
                        continue;
                    }
                    let Some(body) = self.types.get(&name) else {
                        return Err(format!("`{name}` is reached and not declared"));
                    };
                    match body {
                        Body::Newtype { of, .. } => pending.push(of.clone()),
                        Body::Struct { fields, .. } => {
                            pending.extend(fields.iter().map(|(_, type_ref, _)| type_ref.clone()));
                        }
                        Body::Enum { .. } => {}
                        Body::Union { variants, .. } => pending.extend(variants.values().cloned()),
                    }
                }
            }
        }
        if let Some(unreached) = self.types.keys().find(|name| !reached.contains(*name)) {
            return Err(format!("`{unreached}` is declared and not reached"));
        }
        Ok(())
    }

    /// A declared invariant reads only its own type's roots: `value` for a newtype, its fields for
    /// a struct.
    fn rooted(name: &QualifiedName, body: &Body<'_>, predicate: &Predicate) -> Result<(), String> {
        let roots: BTreeSet<&str> = match body {
            Body::Newtype { .. } => BTreeSet::from(["value"]),
            Body::Struct { fields, .. } => fields.iter().map(|(field, _, _)| *field).collect(),
            Body::Enum { .. } | Body::Union { .. } => BTreeSet::new(),
        };
        for path in predicate.fact_paths() {
            let first = path.namespace();
            if !roots.contains(first) {
                return Err(format!(
                    "an invariant of `{name}` reads `{path}`, which is none of its own paths"
                ));
            }
        }
        Ok(())
    }

    /// Walks a typed path from the root. A site path ends on entry to `value`; an edge path ends on
    /// entry to a named type already being expanded.
    fn walk(&self, path: &[Segment], site: bool) -> Result<(), String> {
        let mut at = self.root.clone();
        let mut active: Vec<QualifiedName> = Vec::new();
        let mut index = 0;
        loop {
            // Entering a named type is the event a path ends on; it is not a segment.
            if let TypeRef::Named(name) = &at {
                if index == path.len() {
                    return if (site && *name == self.value) || (!site && active.contains(name)) {
                        Ok(())
                    } else {
                        Err(format!(
                            "the position ends on `{name}`, not on {}",
                            if site {
                                format!("`{}`", self.value)
                            } else {
                                "a back-edge".to_owned()
                            }
                        ))
                    };
                }
                if active.contains(name) {
                    return Err(format!("the position crosses the back-edge into `{name}`"));
                }
                active.push(name.clone());
            } else if index == path.len() {
                return Err(format!("the position ends on `{at}`, not on a named type"));
            }
            let segment = &path[index];
            index += 1;
            at = match (&at, segment) {
                (TypeRef::Optional(of), Segment::Present)
                | (TypeRef::List(of), Segment::Element)
                | (TypeRef::Map(_, of), Segment::MapValue) => (**of).clone(),
                (TypeRef::Named(name), segment) => match (&self.types[name], segment) {
                    (Body::Newtype { of, .. }, Segment::Unwrap) => of.clone(),
                    (Body::Struct { fields, .. }, Segment::Field { name: wanted }) => fields
                        .iter()
                        .find(|(field, _, _)| field == wanted)
                        .map(|(_, type_ref, _)| type_ref.clone())
                        .ok_or_else(|| format!("`{name}` declares no field `{wanted}`"))?,
                    (Body::Union { variants, .. }, Segment::Variant { label }) => variants
                        .get(label.as_str())
                        .cloned()
                        .ok_or_else(|| format!("`{name}` declares no variant `{label}`"))?,
                    _ => return Err(format!("`{segment:?}` does not follow `{name}`")),
                },
                _ => return Err(format!("`{segment:?}` does not follow `{at}`")),
            };
        }
    }

    /// The reason a runner of this build reports the expectation `unsupported`, if it does.
    pub(crate) fn unsupported(&self) -> Option<&'static str> {
        if self.site.is_none() {
            return Some("a recursive value_invariants position");
        }
        if matches!(
            self.observed.witness,
            Witness::ArrangedRow {
                selector: Selector::IdentityQuery { .. }
            }
        ) {
            return Some("a value_invariants identity_query witness");
        }
        None
    }
}

// ---- the observation ---------------------------------------------------------------------------

/// What one snapshot came to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Observed {
    /// Every occurrence holds and a witness row holds one.
    Held,
    /// An occurrence fails, the representation is malformed, or no witness row holds one.
    Unsatisfied(String),
    /// An invariant could not be decided, or the work budget ran out: not retried.
    Broken(String),
}

/// The logical work counter of one snapshot.
struct Work {
    limit: u64,
    spent: u64,
}

/// The budget ran out.
struct Exhausted;

impl Work {
    fn debit(&mut self, units: u64) -> Result<(), Exhausted> {
        let spent = self.spent.checked_add(units).ok_or(Exhausted)?;
        if spent > self.limit {
            return Err(Exhausted);
        }
        self.spent = spent;
        Ok(())
    }
}

fn len(text: &str) -> u64 {
    u64::try_from(text.len()).unwrap_or(u64::MAX)
}

fn count(size: usize) -> u64 {
    u64::try_from(size).unwrap_or(u64::MAX)
}

/// The UTF-8 length of a number's normalized exact decimal spelling.
fn numeric_size(number: ess_primitives::facts::Number) -> u64 {
    let text = number.exact_text();
    if text
        .trim_start_matches('-')
        .trim_matches(['0', '.'])
        .is_empty()
    {
        return 1;
    }
    len(&text)
}

/// A scalar's semantic size: numeric size, UTF-8 length, 4 or 5 for a boolean, 4 for `null`.
fn node_size(node: &Node) -> u64 {
    match node {
        Node::Null | Node::Bool(true) => 4,
        Node::Bool(false) => 5,
        Node::Number(number) => numeric_size(*number),
        Node::Text(text) => len(text),
        Node::Seq(_) | Node::Map(_) => 0,
    }
}

fn fact_size(value: &FactValue) -> u64 {
    match value {
        FactValue::Bool(true) => 4,
        FactValue::Bool(false) => 5,
        FactValue::Number(number) => numeric_size(*number),
        FactValue::Text(text) => len(text),
    }
}

/// Why a row does not carry its declared representation.
enum Fault {
    Malformed(String),
    Exhausted,
}

impl From<Exhausted> for Fault {
    fn from(_: Exhausted) -> Self {
        Self::Exhausted
    }
}

/// The typed facts one terminal binds, with the metadata the comparisons read.
#[derive(Default)]
struct Typed {
    facts: FactStore,
    instants: BTreeSet<FactPath>,
    durations: BTreeSet<FactPath>,
    records: BTreeSet<(Vec<String>, u8)>,
}

impl FactSource for Typed {
    fn fact(&self, path: &FactPath) -> Option<FactValue> {
        self.facts.fact(path)
    }
    fn present(&self, path: &FactPath) -> bool {
        self.facts.present(path)
    }
    fn scales(&self) -> &Scales {
        self.facts.scales()
    }
    fn orders_as_instant(&self, path: &FactPath) -> bool {
        self.instants.contains(path)
    }
    fn orders_text_by_bytes(&self, path: &FactPath) -> bool {
        !self.durations.contains(path) && !self.instants.contains(path)
    }
}

/// A quantifier's binder over the outer environment, for the preflight.
struct Bound<'a> {
    inner: &'a dyn FactSource,
    bind: &'a str,
    prefix: FactPath,
}

impl Bound<'_> {
    fn rebind(&self, path: &FactPath) -> FactPath {
        if path.namespace() != self.bind {
            return path.clone();
        }
        let mut rebound = self.prefix.clone();
        for segment in &path.segments()[1..] {
            rebound = rebound.child(segment);
        }
        rebound
    }
}

impl FactSource for Bound<'_> {
    fn fact(&self, path: &FactPath) -> Option<FactValue> {
        self.inner.fact(&self.rebind(path))
    }
    fn observe(&self, path: &FactPath) -> Option<FactValue> {
        self.inner.observe(&self.rebind(path))
    }
    fn present(&self, path: &FactPath) -> bool {
        self.inner.present(&self.rebind(path))
    }
    fn cardinality(&self, path: &FactPath) -> Option<usize> {
        self.inner.cardinality(&self.rebind(path))
    }
}

/// One terminal: where it was met, and its value.
struct Terminal<'n> {
    location: String,
    value: &'n Node,
}

/// The JSON kind a primitive is carried as, checked.
fn primitive_admits(primitive: Primitive, node: &Node) -> bool {
    match primitive {
        Primitive::String
        | Primitive::Timestamp
        | Primitive::Duration
        | Primitive::Uuid
        | Primitive::Bytes => matches!(node, Node::Text(_)),
        Primitive::Boolean => matches!(node, Node::Bool(_)),
        Primitive::Integer => matches!(node, Node::Number(number) if number.as_i64().is_some()),
        Primitive::Decimal => match node {
            Node::Number(_) => true,
            Node::Text(text) => text.parse::<f64>().is_ok(),
            _ => false,
        },
        Primitive::Json => true,
        Primitive::Binary64 => false,
    }
}

fn map_key_admits(key: Primitive, text: &str) -> bool {
    match key {
        Primitive::Integer => text.parse::<i64>().is_ok(),
        Primitive::Boolean => matches!(text, "true" | "false"),
        Primitive::Uuid => text.len() == 36,
        _ => true,
    }
}

fn describe(node: &Node) -> String {
    match node {
        Node::Null => "null".to_owned(),
        Node::Bool(flag) => flag.to_string(),
        Node::Number(number) => number.exact_text(),
        Node::Text(text) => format!("{text:?}"),
        Node::Seq(items) => format!("a list of {}", items.len()),
        Node::Map(entries) => format!("an object of {} key(s)", entries.len()),
    }
}

impl Contract<'_> {
    /// Observes one snapshot of rows. `eligible` says which rows may supply the occurrence.
    pub(crate) fn observe(
        &self,
        rows: &[crate::ViewRow],
        eligible: &dyn Fn(&crate::ViewRow) -> bool,
    ) -> Observed {
        let mut work = Work {
            limit: self.observed.work_limit,
            spent: 0,
        };
        let exhausted = |work: &Work| {
            Observed::Broken(format!(
                "the observation of `{}` exhausted its work budget of {} units after {}",
                self.value, work.limit, work.spent
            ))
        };
        let mut failures: Vec<String> = Vec::new();
        let mut occurred = false;
        let root_name = self.observed.root.name.as_str();
        for (index, row) in rows.iter().enumerate() {
            let value = row.get(root_name);
            let location = format!("row {index} `{root_name}`");
            match self.shape(
                &self.root,
                value,
                self.observed.root.presence,
                &location,
                &mut work,
            ) {
                Ok(()) => {}
                Err(Fault::Exhausted) => return exhausted(&work),
                Err(Fault::Malformed(reason)) => {
                    failures.push(reason);
                    continue;
                }
            }
            let mut terminals = Vec::new();
            let mut path = Vec::new();
            if self
                .visit(
                    &self.root,
                    value,
                    &mut path,
                    &location,
                    &mut terminals,
                    &mut work,
                )
                .is_err()
            {
                return exhausted(&work);
            }
            for terminal in terminals {
                let facts = match self.facts(terminal.value, &mut work) {
                    Ok(facts) => facts,
                    Err(Exhausted) => return exhausted(&work),
                };
                for invariant in self.invariants() {
                    if preflight(&invariant.predicate, &facts, &mut work).is_err() {
                        return exhausted(&work);
                    }
                    let outcome = invariant.predicate.outcome(&facts);
                    match outcome.truth {
                        Truth::True => {}
                        Truth::False => failures.push(format!(
                            "{} holds {}, and `{}` does not hold of it ({})",
                            terminal.location,
                            describe(terminal.value),
                            invariant.statement,
                            outcome.expression
                        )),
                        Truth::Unknown => {
                            return Observed::Broken(format!(
                                "`{}` cannot be decided of {} at {}",
                                invariant.statement,
                                describe(terminal.value),
                                terminal.location
                            ))
                        }
                    }
                }
                if eligible(row) {
                    occurred = true;
                }
            }
        }
        if !failures.is_empty() {
            return Observed::Unsatisfied(failures.join("; "));
        }
        if !occurred {
            return Observed::Unsatisfied(format!(
                "no {} row holds an actual occurrence of `{}` at `{}` {}; legal absence and \
                 empty collections witness none",
                if matches!(self.observed.witness, Witness::AnyObservedOccurrence) {
                    "returned"
                } else {
                    "selected"
                },
                self.value,
                root_name,
                serde_json::to_string(&self.observed.position).unwrap_or_default()
            ));
        }
        Observed::Held
    }

    fn invariants(&self) -> &[DeclaredInvariant] {
        match &self.types[&self.value] {
            Body::Newtype { invariants, .. } | Body::Struct { invariants, .. } => invariants,
            Body::Enum { .. } | Body::Union { .. } => &[],
        }
    }

    /// S: the whole value against its declared representation, in preorder.
    #[allow(clippy::too_many_lines)]
    fn shape(
        &self,
        type_ref: &TypeRef,
        value: Option<&Node>,
        presence: Option<Presence>,
        location: &str,
        work: &mut Work,
    ) -> Result<(), Fault> {
        work.debit(1)?;
        let malformed = |reason: String| Err(Fault::Malformed(format!("{location} {reason}")));
        if let TypeRef::Optional(of) = type_ref {
            return match value {
                None if presence == Some(Presence::NullWhenAbsent) => {
                    malformed("is left out, and its presence is null_when_absent".into())
                }
                None => Ok(()),
                Some(Node::Null) if presence == Some(Presence::OmittedWhenAbsent) => {
                    malformed("is null, and its presence is omitted_when_absent".into())
                }
                Some(Node::Null) => {
                    work.debit(4)?;
                    Ok(())
                }
                Some(present) => {
                    work.debit(1)?;
                    self.shape(of, Some(present), None, location, work)
                }
            };
        }
        let Some(value) = value else {
            return if self.represents_absence(type_ref) {
                Ok(())
            } else {
                malformed(format!("is missing, and `{type_ref}` is required"))
            };
        };
        match type_ref {
            TypeRef::Optional(_) => unreachable!("handled above"),
            TypeRef::Primitive(primitive) => {
                if *primitive == Primitive::Json {
                    return Self::json(value, work);
                }
                if !primitive_admits(*primitive, value) {
                    return malformed(format!(
                        "holds {}, which is no `{primitive}`",
                        describe(value)
                    ));
                }
                work.debit(node_size(value))?;
                Ok(())
            }
            TypeRef::List(of) => {
                let Node::Seq(items) = value else {
                    return malformed(format!("holds {}, which is no list", describe(value)));
                };
                for (index, item) in items.iter().enumerate() {
                    work.debit(1)?;
                    self.shape(of, Some(item), None, &format!("{location}[{index}]"), work)?;
                }
                Ok(())
            }
            TypeRef::Map(key, of) => {
                let Node::Map(entries) = value else {
                    return malformed(format!("holds {}, which is no map", describe(value)));
                };
                let keys: u64 = entries.keys().map(|key| len(key)).sum();
                work.debit(keys)?;
                work.debit(sort_keys(entries.len(), keys))?;
                for (name, entry) in entries {
                    if !map_key_admits(*key, name) {
                        return malformed(format!("holds the key {name:?}, which is no `{key}`"));
                    }
                    work.debit(1)?;
                    self.shape(
                        of,
                        Some(entry),
                        None,
                        &format!("{location}[{name:?}]"),
                        work,
                    )?;
                }
                Ok(())
            }
            TypeRef::Named(name) => match &self.types[name] {
                Body::Newtype {
                    of,
                    alphabet,
                    prefix,
                    ..
                } => {
                    work.debit(1)?;
                    self.shape(of, Some(value), None, location, work)?;
                    if let Node::Text(text) = value {
                        if let Some(alphabet) = alphabet {
                            if let Some(outside) = text.chars().find(|c| !alphabet.contains(c)) {
                                return malformed(format!(
                                    "holds {text:?}, whose {outside:?} is outside the alphabet of \
                                     `{name}`"
                                ));
                            }
                        }
                        if let Some(prefix) = prefix {
                            if !text.starts_with(prefix) {
                                return malformed(format!(
                                    "holds {text:?}, which does not start with the prefix {prefix:?} \
                                     of `{name}`"
                                ));
                            }
                        }
                    }
                    Ok(())
                }
                Body::Struct { fields, .. } => {
                    let Node::Map(entries) = value else {
                        return malformed(format!(
                            "holds {}, which is no `{name}`",
                            describe(value)
                        ));
                    };
                    work.debit(entries.keys().map(|key| len(key)).sum())?;
                    if let Some(undeclared) = entries
                        .keys()
                        .find(|key| !fields.iter().any(|(field, _, _)| field == key))
                    {
                        return malformed(format!(
                            "carries `{undeclared}`, which `{name}` does not declare"
                        ));
                    }
                    for (field, type_ref, presence) in fields {
                        let member = entries.get(*field);
                        if member.is_some() {
                            work.debit(1)?;
                        }
                        self.shape(
                            type_ref,
                            member,
                            *presence,
                            &format!("{location}.{field}"),
                            work,
                        )?;
                    }
                    Ok(())
                }
                Body::Enum { variants } => match value {
                    Node::Text(label) if variants.contains(label.as_str()) => {
                        work.debit(len(label))?;
                        Ok(())
                    }
                    other => malformed(format!(
                        "holds {}, which is no label of `{name}`",
                        describe(other)
                    )),
                },
                Body::Union { tag, variants } => {
                    let Node::Map(entries) = value else {
                        return malformed(format!(
                            "holds {}, which is no `{name}`",
                            describe(value)
                        ));
                    };
                    work.debit(entries.keys().map(|key| len(key)).sum())?;
                    let content = ess_gen::schema::union_content_key(tag);
                    let Some(Node::Text(label)) = entries.get(*tag) else {
                        return malformed(format!("carries no `{tag}` label of `{name}`"));
                    };
                    let Some(payload) = variants.get(label.as_str()) else {
                        return malformed(format!(
                            "is labelled {label:?}, which `{name}` does not declare"
                        ));
                    };
                    work.debit(len(label))?;
                    if let Some(extra) = entries.keys().find(|key| *key != tag && *key != content) {
                        return malformed(format!(
                            "carries `{extra}`, which `{name}` does not declare"
                        ));
                    }
                    let carried = entries.get(content);
                    if carried.is_some() {
                        work.debit(1)?;
                    }
                    self.shape(payload, carried, None, &format!("{location}.{label}"), work)
                }
            },
        }
    }

    /// Whether a missing value of this type is its own legal absence: a newtype over `Optional`.
    fn represents_absence(&self, type_ref: &TypeRef) -> bool {
        match type_ref {
            TypeRef::Optional(_) => true,
            TypeRef::Named(name) => match &self.types[name] {
                Body::Newtype { of, .. } => self.represents_absence(of),
                _ => false,
            },
            _ => false,
        }
    }

    /// An opaque `Json` value, walked structurally.
    fn json(value: &Node, work: &mut Work) -> Result<(), Fault> {
        match value {
            Node::Seq(items) => {
                for item in items {
                    work.debit(2)?;
                    Self::json(item, work)?;
                }
            }
            Node::Map(entries) => {
                let keys: u64 = entries.keys().map(|key| len(key)).sum();
                work.debit(keys)?;
                work.debit(sort_keys(entries.len(), keys))?;
                for entry in entries.values() {
                    work.debit(2)?;
                    Self::json(entry, work)?;
                }
            }
            scalar => work.debit(node_size(scalar))?,
        }
        Ok(())
    }

    /// O: the typed graph again, collecting each entry to the selected site.
    #[allow(clippy::too_many_lines)]
    fn visit<'n>(
        &self,
        type_ref: &TypeRef,
        value: Option<&'n Node>,
        path: &mut Vec<Segment>,
        location: &str,
        terminals: &mut Vec<Terminal<'n>>,
        work: &mut Work,
    ) -> Result<(), Exhausted> {
        work.debit(1)?;
        let Some(value) = value else {
            return Ok(());
        };
        // Every entry to the selected site is one terminal, whatever its row.
        if let TypeRef::Named(name) = type_ref {
            if *name == self.value && self.site.as_deref() == Some(path.as_slice()) {
                work.debit(1)?;
                terminals.push(Terminal {
                    location: location.to_owned(),
                    value,
                });
            }
        }
        let mut child = |segment: Segment,
                         type_ref: &TypeRef,
                         value: Option<&'n Node>,
                         location: String,
                         work: &mut Work,
                         terminals: &mut Vec<Terminal<'n>>|
         -> Result<(), Exhausted> {
            if value.is_none() {
                return Ok(());
            }
            work.debit(1)?;
            path.push(segment);
            let result = self.visit(type_ref, value, path, &location, terminals, work);
            path.pop();
            result
        };
        match type_ref {
            TypeRef::Primitive(_) => Ok(()),
            TypeRef::Optional(of) => match value {
                Node::Null => Ok(()),
                present => child(
                    Segment::Present,
                    of,
                    Some(present),
                    location.to_owned(),
                    work,
                    terminals,
                ),
            },
            TypeRef::List(of) => {
                let Node::Seq(items) = value else {
                    return Ok(());
                };
                for (index, item) in items.iter().enumerate() {
                    child(
                        Segment::Element,
                        of,
                        Some(item),
                        format!("{location}[{index}]"),
                        work,
                        terminals,
                    )?;
                }
                Ok(())
            }
            TypeRef::Map(_, of) => {
                let Node::Map(entries) = value else {
                    return Ok(());
                };
                for (name, entry) in entries {
                    child(
                        Segment::MapValue,
                        of,
                        Some(entry),
                        format!("{location}[{name:?}]"),
                        work,
                        terminals,
                    )?;
                }
                Ok(())
            }
            TypeRef::Named(name) => match &self.types[name] {
                Body::Newtype { of, .. } => child(
                    Segment::Unwrap,
                    of,
                    Some(value),
                    location.to_owned(),
                    work,
                    terminals,
                ),
                Body::Struct { fields, .. } => {
                    let Node::Map(entries) = value else {
                        return Ok(());
                    };
                    for (field, type_ref, _) in fields {
                        child(
                            Segment::Field {
                                name: (*field).to_owned(),
                            },
                            type_ref,
                            entries.get(*field),
                            format!("{location}.{field}"),
                            work,
                            terminals,
                        )?;
                    }
                    Ok(())
                }
                Body::Enum { .. } => Ok(()),
                Body::Union { tag, variants } => {
                    let Node::Map(entries) = value else {
                        return Ok(());
                    };
                    let Some(Node::Text(label)) = entries.get(*tag) else {
                        return Ok(());
                    };
                    let Some(payload) = variants.get(label.as_str()) else {
                        return Ok(());
                    };
                    child(
                        Segment::Variant {
                            label: label.clone(),
                        },
                        payload,
                        entries.get(ess_gen::schema::union_content_key(tag)),
                        format!("{location}.{label}"),
                        work,
                        terminals,
                    )
                }
            },
        }
    }

    /// F: one terminal's typed facts. A newtype's predicates read `value`; a struct's, its fields.
    fn facts(&self, value: &Node, work: &mut Work) -> Result<Typed, Exhausted> {
        let mut typed = Typed::default();
        let named = TypeRef::Named(self.value.clone());
        match &self.types[&self.value] {
            Body::Struct { fields, .. } => {
                work.debit(1)?;
                if let Node::Map(entries) = value {
                    for (field, type_ref, _) in fields {
                        if let Some(member) = entries.get(*field) {
                            work.debit(1)?;
                            self.bind(
                                type_ref,
                                member,
                                FactPath::from_segments([*field]),
                                &mut typed,
                                work,
                            )?;
                        }
                    }
                }
            }
            _ => self.bind(
                &named,
                value,
                FactPath::from_segments(["value"]),
                &mut typed,
                work,
            )?,
        }
        // One debit per unique logical record: scalar, presence and type, by path.
        let records = std::mem::take(&mut typed.records);
        for (segments, kind) in &records {
            match kind {
                0 => {
                    let path = FactPath::from_segments(segments.iter().map(String::as_str));
                    let size = typed.facts.fact(&path).map_or(0, |value| fact_size(&value));
                    work.debit(1 + size)?;
                }
                _ => work.debit(1)?,
            }
        }
        typed.facts.order_text_by_bytes();
        Ok(typed)
    }

    #[allow(clippy::too_many_lines)]
    fn bind(
        &self,
        type_ref: &TypeRef,
        value: &Node,
        path: FactPath,
        typed: &mut Typed,
        work: &mut Work,
    ) -> Result<(), Exhausted> {
        work.debit(1)?;
        let key = path.segments().to_vec();
        match type_ref {
            TypeRef::Optional(of) => {
                if !matches!(value, Node::Null) {
                    work.debit(1)?;
                    self.bind(of, value, path, typed, work)?;
                }
            }
            TypeRef::Primitive(primitive) => {
                let scalar = match value {
                    Node::Text(text) => Some(FactValue::text(text.clone())),
                    Node::Number(number) => Some(FactValue::from(*number)),
                    Node::Bool(flag) => Some(FactValue::bool(*flag)),
                    Node::Null | Node::Seq(_) | Node::Map(_) => None,
                };
                match scalar {
                    Some(scalar) if *primitive != Primitive::Json => {
                        typed.facts.set(path.clone(), scalar);
                        typed.records.insert((key.clone(), 0));
                        typed.records.insert((key, 2));
                        if *primitive == Primitive::Timestamp {
                            typed.instants.insert(path);
                        } else if *primitive == Primitive::Duration {
                            typed.durations.insert(path);
                        }
                    }
                    _ => {
                        typed.facts.mark_present(path);
                        typed.records.insert((key.clone(), 1));
                        typed.records.insert((key, 2));
                    }
                }
            }
            TypeRef::List(of) => {
                let Node::Seq(items) = value else {
                    return Ok(());
                };
                typed.facts.mark_present(path.clone());
                typed.records.insert((key.clone(), 1));
                typed.records.insert((key, 2));
                let counted = path.child("count");
                typed
                    .facts
                    .set(counted.clone(), FactValue::count(items.len()));
                typed.records.insert((counted.segments().to_vec(), 0));
                typed.records.insert((counted.segments().to_vec(), 2));
                for (index, item) in items.iter().enumerate() {
                    work.debit(1)?;
                    self.bind(of, item, path.child(&index.to_string()), typed, work)?;
                }
            }
            TypeRef::Map(_, of) => {
                let Node::Map(entries) = value else {
                    return Ok(());
                };
                typed.facts.mark_present(path.clone());
                typed.records.insert((key.clone(), 1));
                typed.records.insert((key, 2));
                let counted = path.child("count");
                typed
                    .facts
                    .set(counted.clone(), FactValue::count(entries.len()));
                typed.records.insert((counted.segments().to_vec(), 0));
                typed.records.insert((counted.segments().to_vec(), 2));
                for (index, entry) in entries.values().enumerate() {
                    work.debit(1)?;
                    self.bind(of, entry, path.child(&index.to_string()), typed, work)?;
                }
            }
            TypeRef::Named(name) => match &self.types[name] {
                Body::Newtype { of, .. } => {
                    work.debit(1)?;
                    self.bind(of, value, path, typed, work)?;
                }
                Body::Enum { .. } => {
                    if let Node::Text(label) = value {
                        typed.facts.set(path, FactValue::text(label.clone()));
                        typed.records.insert((key.clone(), 0));
                        typed.records.insert((key, 2));
                    }
                }
                Body::Struct { fields, .. } => {
                    typed.facts.mark_present(path.clone());
                    typed.records.insert((key.clone(), 1));
                    typed.records.insert((key, 2));
                    if let Node::Map(entries) = value {
                        for (field, type_ref, _) in fields {
                            if let Some(member) = entries.get(*field) {
                                work.debit(1)?;
                                self.bind(type_ref, member, path.child(field), typed, work)?;
                            }
                        }
                    }
                }
                Body::Union { .. } => {
                    typed.facts.mark_present(path);
                    typed.records.insert((key.clone(), 1));
                    typed.records.insert((key, 2));
                }
            },
        }
        Ok(())
    }
}

/// `n + ceil(log2(max(1, n))) * sum(key lengths)`.
fn sort_keys(n: usize, keys: u64) -> u64 {
    let n = count(n);
    let log = u64::from(64 - n.max(1).saturating_sub(1).leading_zeros());
    n.saturating_add(log.saturating_mul(keys))
}

/// One operand read: `operand_read`, then its value's semantic size, and for a text's derived
/// `.count` the scan of the text it is derived from (`text_count`).
fn read_operand(path: &FactPath, facts: &dyn FactSource, work: &mut Work) -> Result<(), Exhausted> {
    work.debit(1)?;
    if let Some(value) = facts.observe(path) {
        work.debit(fact_size(&value))?;
        if facts.fact(path).is_none() {
            if let Some((_, parent)) = path.segments().split_last() {
                if let Some(FactValue::Text(text)) =
                    facts.fact(&FactPath::from_segments(parent.iter().map(String::as_str)))
                {
                    work.debit(len(&text))?;
                }
            }
        }
    }
    Ok(())
}

/// P: one invariant's logical preflight against its facts.
fn preflight(
    predicate: &Predicate,
    facts: &dyn FactSource,
    work: &mut Work,
) -> Result<(), Exhausted> {
    work.debit(1)?;
    let read = |path: &FactPath, work: &mut Work| read_operand(path, facts, work);
    let literal = |value: &FactValue, work: &mut Work| -> Result<(), Exhausted> {
        work.debit(1)?;
        work.debit(fact_size(value))
    };
    match predicate {
        Predicate::Always | Predicate::Never => Ok(()),
        Predicate::All(children) | Predicate::Any(children) => {
            for child in children {
                preflight(child, facts, work)?;
            }
            Ok(())
        }
        Predicate::Not(inner) | Predicate::Trimmed(inner) => preflight(inner, facts, work),
        Predicate::Forall(quantified) | Predicate::Exists(quantified) => {
            work.debit(1)?;
            // Validation keeps a test over the rows of an entity out of every invariant
            // (beyond10x/ess#551): it reads no fact of this value, so there is nothing to preflight.
            let Some(over) = quantified.over.fact() else {
                return Ok(());
            };
            let members = facts.cardinality(over).unwrap_or(0);
            for index in 0..members {
                work.debit(1)?;
                let bound = Bound {
                    inner: facts,
                    bind: &quantified.bind,
                    prefix: over.child(&index.to_string()),
                };
                preflight(&quantified.body, &bound, work)?;
            }
            Ok(())
        }
        Predicate::Distinct(distinct) => {
            work.debit(1)?;
            let members = facts.cardinality(&distinct.over).unwrap_or(0);
            for index in 0..members {
                work.debit(1)?;
                let bound = Bound {
                    inner: facts,
                    bind: &distinct.bind,
                    prefix: distinct.over.child(&index.to_string()),
                };
                read_operand(&distinct.key_path(), &bound, work)?;
            }
            Ok(())
        }
        Predicate::Compare { left, right, .. } => {
            for operand in [left, right] {
                match operand {
                    Operand::Fact(path) => read(path, work)?,
                    Operand::Literal(value) => literal(value, work)?,
                    Operand::Offset(offset) => read(&offset.base, work)?,
                    Operand::Derived(derived) => read(derived.parent(), work)?,
                }
            }
            Ok(())
        }
        Predicate::Truthy(path) => read(path, work),
        Predicate::Defined(_) => work.debit(1),
        Predicate::AnyOf { path, values }
        | Predicate::NoneOf { path, values }
        | Predicate::FoldMatch { path, values, .. } => {
            read(path, work)?;
            for value in values {
                literal(value, work)?;
            }
            Ok(())
        }
        Predicate::TextMatch { path, value, .. } => {
            read(path, work)?;
            match value {
                TextOperand::Literal(value) => literal(value, work),
                TextOperand::Fact { path, .. } => read(path, work),
            }
        }
        Predicate::Window(window) => match &window.at {
            ess_primitives::window::WindowInstant::Fact(path) => read(path, work),
            ess_primitives::window::WindowInstant::Now => work.debit(1),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_digest_is_the_sha256_of_the_compact_position() {
        let position = InvariantPosition::Site {
            path: vec![Segment::Element],
        };
        assert_eq!(
            serde_json::to_string(&position).unwrap(),
            r#"{"kind":"site","path":[{"op":"element"}]}"#
        );
        assert_eq!(position_digest(&position).len(), 64);
    }

    #[test]
    fn a_wrapped_scenario_id_round_trips_and_refuses_a_malformed_digest() {
        let position = InvariantPosition::Site {
            path: vec![Segment::Element],
        };
        let digest = position_digest(&position);
        let id = ScenarioId::ValueInvariantWrapped {
            value: DeclaredTypeRef::new(QualifiedName::new("demo.tags.Tag").unwrap()),
            at: crate::scenario::ViewRef::new(QualifiedName::new("demo.tags.Boards").unwrap()),
            root: "tags".to_owned(),
            digest: digest.clone(),
        };
        let rendered = id.to_string();
        assert_eq!(
            rendered,
            format!("demo.tags.Tag/invariant-wrapped/at/demo.tags.Boards/tags/{digest}")
        );
        assert_eq!(ScenarioId::parse(&rendered).unwrap(), id);
        for forged in [
            digest.to_uppercase(),
            digest[..63].to_owned(),
            String::new(),
        ] {
            assert!(ScenarioId::parse(&rendered.replace(&digest, &forged)).is_err());
        }
    }

    #[test]
    fn sorting_keys_costs_n_plus_log_n_times_their_bytes() {
        assert_eq!(sort_keys(0, 0), 0);
        assert_eq!(sort_keys(1, 3), 1);
        assert_eq!(sort_keys(2, 4), 2 + 4);
        assert_eq!(sort_keys(5, 10), 5 + 3 * 10);
    }

    #[test]
    fn a_number_costs_its_normalized_decimal_spelling() {
        let number = |text: &str| match serde_json::from_str::<Node>(text).unwrap() {
            Node::Number(number) => number,
            _ => unreachable!(),
        };
        assert_eq!(numeric_size(number("1")), 1);
        assert_eq!(numeric_size(number("1.0")), 1);
        assert_eq!(numeric_size(number("1e0")), 1);
        assert_eq!(numeric_size(number("0.0")), 1);
        assert_eq!(numeric_size(number("-0")), 1);
        assert_eq!(numeric_size(number("0.0100")), 4);
    }
}
