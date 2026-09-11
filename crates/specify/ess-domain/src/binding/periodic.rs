//! The bounded session poll and the host authority supplying its inputs.
use crate::component::ComponentName;
use crate::types::{Field, TypeRegistry};
use ess_primitives::error::{ParseError, ValidationCode, ValidationError, ValidationErrors};
use std::collections::BTreeSet;
use std::num::NonZeroU32;

/// Positive, whole seconds; unlike elapsed observations, a period cannot be zero.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(try_from = "String", into = "String")]
pub struct Period(NonZeroU32);
impl Period {
    /// Parse canonical `PT<seconds>S`, with no alternate spellings.
    pub fn parse(value: &str) -> Result<Self, ParseError> {
        if value
            .strip_prefix("PT")
            .and_then(|rest| rest.strip_suffix('S'))
            .is_some_and(|digits| !digits.bytes().all(|byte| byte.is_ascii_digit()))
        {
            return Err(ParseError::reference(
                "period",
                value,
                "a positive period uses decimal digits only",
            ));
        }
        let seconds = ess_primitives::time::parse_elapsed_seconds(value)?;
        NonZeroU32::new(seconds).map(Self).ok_or_else(|| {
            ParseError::reference(
                "period",
                value,
                "a periodic cause requires a positive period, not PT0S",
            )
        })
    }
    /// The whole-second period.
    pub const fn seconds(self) -> u32 {
        self.0.get()
    }
    /// Exact millisecond representation, without narrowing.
    pub const fn milliseconds(self) -> u64 {
        self.0.get() as u64 * 1000
    }
    /// A nominal deadline relative to a target mark; overflow is a refusal.
    pub fn deadline(self, anchor_ms: u64, ordinal: u64) -> Option<u64> {
        if ordinal == 0 {
            return None;
        }
        self.milliseconds()
            .checked_mul(ordinal)?
            .checked_add(anchor_ms)
    }
}
impl TryFrom<String> for Period {
    type Error = ParseError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::parse(&value)
    }
}
impl From<Period> for String {
    fn from(value: Period) -> Self {
        format!("PT{}S", value.seconds())
    }
}
impl std::fmt::Display for Period {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PT{}S", self.seconds())
    }
}

macro_rules! profile_word {
    ($name:ident, $variant:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(
            Debug,
            Clone,
            Copy,
            PartialEq,
            Eq,
            serde::Serialize,
            serde::Deserialize,
            schemars::JsonSchema,
        )]
        #[serde(rename_all = "snake_case")]
        pub enum $name {
            #[doc = $doc]
            $variant,
        }
    };
}
profile_word!(
    Anchor,
    HostActivation,
    "The actual host poll activation anchors the grid."
);
profile_word!(First, AfterPeriod, "There is no immediate periodic call.");
profile_word!(
    Cadence,
    FixedRate,
    "Completion does not reset the deadline phase."
);
profile_word!(
    Overlap,
    SerialPerInstance,
    "One complete read/apply operation per host lifetime."
);
profile_word!(
    Missed,
    OnePendingDropExcess,
    "Retain at most one pending tick; drop excess deadlines."
);
profile_word!(
    Lifetime,
    HostInstance,
    "The poll belongs to one authenticated host instance."
);
profile_word!(
    Cancellation,
    StopAcknowledged,
    "Only acknowledged quiescence ends the poll lifetime."
);
profile_word!(
    Eligibility,
    HostBoolean,
    "The host samples eligibility at received-tick dispatch."
);

/// A binding-local required interface; a declaration is not proof of authentication.
#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct HostInputContract {
    /// Existing component responsible for the adapter boundary.
    #[serde(deserialize_with = "component_name")]
    pub owner: ComponentName,
    /// Binding-local authority name; no credentials are serialized here.
    pub authority: super::BindingName,
    /// Required host decision before starting a read.
    pub eligibility: Eligibility,
    /// Lifetime-constant fields, supplied by the authenticated session.
    pub context_fields: Vec<Field>,
    /// Fresh required read results for each eligible occurrence.
    pub read_fields: Vec<Field>,
}

fn component_name<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<ComponentName, D::Error> {
    let raw = <String as serde::Deserialize>::deserialize(deserializer)?;
    ComponentName::new(raw).map_err(serde::de::Error::custom)
}

/// Cross-declaration authority and format admission, including in-memory models.
pub fn validate_specification(spec: &crate::Specification) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    for binding in spec.bindings().values() {
        let Some(periodic) = binding.cause.periodic() else {
            continue;
        };
        let at = format!("binding.{}.when.periodic", binding.name);
        if spec.system().format.major() < 3 {
            errors.push(ValidationError::new(
                ValidationCode::UnsupportedFormatVersion,
                &at,
                "periodic binding causes require ess/3",
            ));
        }
        if !spec.components().contains_key(&periodic.host.owner) {
            errors.push(ValidationError::new(
                ValidationCode::UndeclaredReference,
                format!("{at}.host.owner"),
                "periodic host owner must be a declared component",
            ));
        }
        for field in periodic
            .host
            .context_fields
            .iter()
            .chain(&periodic.host.read_fields)
        {
            crate::primitive_admission::reference(
                &field.type_ref,
                Some(spec.system().format),
                &format!("{at}.host.{}", field.name),
                &mut errors,
            );
        }
    }
    errors
}

/// The only initially admitted periodic profile, with every choice explicit.
#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct PeriodicCause {
    /// A strictly positive whole-second interval.
    pub every: Period,
    /// Deadline anchor.
    pub anchor: Anchor,
    /// Initial-fire rule.
    pub first: First,
    /// Cadence rule.
    pub cadence: Cadence,
    /// Concurrency bound.
    pub overlap: Overlap,
    /// Missed-period policy.
    pub missed: Missed,
    /// Lifetime owner.
    pub lifetime: Lifetime,
    /// End-of-lifetime observation.
    pub cancellation: Cancellation,
    /// Required typed host input authority.
    pub host: HostInputContract,
}
impl PeriodicCause {
    /// Re-admit public programmatically constructed contracts.
    pub fn validate(&self, at: &str) -> ValidationErrors {
        let mut errors = ValidationErrors::new();
        for (label, fields) in [
            ("context_fields", &self.host.context_fields),
            ("read_fields", &self.host.read_fields),
        ] {
            let path = format!("{at}.host.{label}");
            if fields.len() > 64 {
                errors.push(ValidationError::new(
                    ValidationCode::UnsupportedConstruct,
                    &path,
                    "periodic host field limit is 64 per table",
                ));
            }
            let mut names = BTreeSet::new();
            for field in fields {
                if crate::types::field_name(&field.name).is_err() || !names.insert(&field.name) {
                    errors.push(ValidationError::new(
                        ValidationCode::DuplicateDeclaration,
                        &path,
                        "periodic host fields require unique valid declared names",
                    ));
                }
            }
        }
        errors
    }
    /// Host fields resolve against exactly the model's declared types.
    pub fn validate_types(&self, types: &TypeRegistry, at: &str) -> ValidationErrors {
        let mut errors = ValidationErrors::new();
        for field in self
            .host
            .context_fields
            .iter()
            .chain(&self.host.read_fields)
        {
            errors.extend(types.resolve(&field.type_ref, &format!("{at}.host.{}", field.name)));
        }
        errors
    }
}
