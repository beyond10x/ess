//! A failure policy selected per refusal of the invoked command (ess/22, beyond10x/ess#269).
//!
//! `docs/design/conditional-binding-failure-policies.md`, "Refusal-selected failure policy (#269)",
//! is the binding design. Policy-keyed, never keyed by condition kinds:
//!
//! ```yaml
//! on_failure:
//!   drop: [wrong-state]
//!   retry: {outcomes: [demo.ledger.Unavailable], attempts: 3}
//!   escalate:
//!     emits: demo.ledger.RecordEscalated
//!     except: [wrong-state, demo.ledger.Unavailable]
//! ```
//!
//! Each of `drop`, `retry` and `escalate` appears at most once, with exactly one selector:
//! `outcomes: [names]` or `except: [names]`. `drop` and an unbounded `retry` may write the
//! `outcomes` list alone. `escalate` always writes a block with `emits:`; `retry` keeps `attempts:`
//! and `final:`, and `final` needs a bound. Exactly one policy has `except:`: the explicit fallback
//! for every failure of an invoked command port that carries no declared outcome, and for every
//! declared refusal no other policy selects. An empty `except` is legal.
//!
//! Names resolve as `retry.final` does: an outcome that carries an `error:`, by its name, or the
//! error itself, standing for every outcome of the command that reports it. A condition word such
//! as `wrong_state` means nothing here. Aliases resolve first; the resolved sets are then disjoint
//! and exhaustive, the complement included.
//!
//! A map is in selected mode when any of its policies writes a selector — a list, or a block with
//! `outcomes:` or `except:`. Anything else is read by the universal reader, whose spellings,
//! meanings and diagnostics are unchanged; a selected shape below ess/22 is captured and refused by
//! format, never reinterpreted as a universal policy.
//!
//! | rule | code |
//! |---|---|
//! | a selected policy below ess/22 | [`UnsupportedFormatVersion`](ValidationCode::UnsupportedFormatVersion) |
//! | a policy with both selectors | [`ConflictingDeclaration`](ValidationCode::ConflictingDeclaration) |
//! | a policy with no selector, or an empty `outcomes` | [`MissingDeclaration`](ValidationCode::MissingDeclaration) |
//! | no `except:` / more than one | [`MissingDeclaration`](ValidationCode::MissingDeclaration) / [`ConflictingDeclaration`](ValidationCode::ConflictingDeclaration) |
//! | `escalate` without `emits` | [`MissingDeclaration`](ValidationCode::MissingDeclaration) |
//! | `final` without `attempts`, or fewer than two attempts | [`ConflictingDeclaration`](ValidationCode::ConflictingDeclaration) |
//! | a name that is no refusal of the invoked command | [`UndeclaredReference`](ValidationCode::UndeclaredReference) |
//! | a name that is an accepting outcome | [`ConflictingDeclaration`](ValidationCode::ConflictingDeclaration) |
//! | two names of one list selecting one refusal | [`DuplicateDeclaration`](ValidationCode::DuplicateDeclaration) |
//! | one refusal selected by two policies, or by one and the fallback | [`ConflictingDeclaration`](ValidationCode::ConflictingDeclaration) |
//! | a refusal excepted from the fallback that no policy selects | [`MissingDeclaration`](ValidationCode::MissingDeclaration) |
//! | a `final` refusal the retry does not select | [`ConflictingDeclaration`](ValidationCode::ConflictingDeclaration) |
//! | a selected policy on a periodic cause | [`UnsupportedConstruct`](ValidationCode::UnsupportedConstruct) |

use std::collections::{BTreeMap, BTreeSet};

use ess_primitives::error::{ValidationCode, ValidationError, ValidationErrors};

use crate::binding::retry::RetryBound;
use crate::binding::{BindingSpec, Failure};
use crate::command::{CommandSpec, OutcomeName};
use crate::name::QualifiedName;
use crate::system::FormatVersion;

/// The first source format that admits a refusal-selected `on_failure:`.
pub const FORMAT: FormatVersion = FormatVersion::V22;

/// One policy of a refusal-selected `on_failure:`, as the document writes it.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct RefusalEntry {
    /// The policy.
    pub policy: Failure,
    /// The refusals it selects, when it selects positively.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outcomes: Option<Vec<String>>,
    /// The refusals the fallback leaves to the other policies, when this is the fallback.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub except: Option<Vec<String>>,
    /// The event an `escalate` publishes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emits: Option<QualifiedName>,
    /// The total invocations a bounded `retry` makes for one occurrence, the first included.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attempts: Option<u32>,
    /// The refusals that end a bounded `retry` at once.
    #[serde(rename = "final", skip_serializing_if = "Option::is_none")]
    pub finals: Option<Vec<String>>,
}

impl RefusalEntry {
    fn empty(policy: Failure) -> Self {
        Self {
            policy,
            outcomes: None,
            except: None,
            emits: None,
            attempts: None,
            finals: None,
        }
    }

    /// The bound a `retry` entry states, as the universal reader would hold it.
    pub fn bound(&self) -> Option<RetryBound> {
        self.attempts.map(|attempts| RetryBound {
            attempts,
            finals: self.finals.clone().unwrap_or_default(),
        })
    }
}

/// A refusal-selected `on_failure:`: its policies in the order the document writes them.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(transparent)]
pub struct RefusalPolicy {
    /// One entry per policy, each policy at most once.
    pub entries: Vec<RefusalEntry>,
}

impl RefusalPolicy {
    /// The explicit fallback: the first policy that writes `except:`.
    pub fn fallback(&self) -> Option<&RefusalEntry> {
        self.entries.iter().find(|entry| entry.except.is_some())
    }

    /// The entry for `policy`, where the document writes one.
    pub fn entry(&self, policy: Failure) -> Option<&RefusalEntry> {
        self.entries.iter().find(|entry| entry.policy == policy)
    }

    /// The legacy universal fields a binding carries beside this table: the fallback's word (or
    /// the first policy's, where no single fallback is written and validation refuses the table),
    /// the event the `escalate` policy publishes, and the bound the `retry` policy states.
    ///
    /// Never a policy for every refusal: [`BindingSpec::refusals`] is the authority, and these are
    /// only what lets a consumer that asks which events a binding may publish keep answering.
    pub fn legacy_view(&self) -> (Failure, Option<QualifiedName>, Option<RetryBound>) {
        let failure = self
            .fallback()
            .or_else(|| self.entries.first())
            .map_or(Failure::Drop, |entry| entry.policy);
        let escalation = self
            .entry(Failure::Escalate)
            .and_then(|entry| entry.emits.clone());
        let retry = self.entry(Failure::Retry).and_then(RefusalEntry::bound);
        (failure, escalation, retry)
    }

    /// Which entry each declared refusal of `command` falls to, in the command's declaration
    /// order, and the index of the fallback entry. `None` unless the table is admitted against
    /// `command`: exactly one fallback, every name resolved.
    pub fn assign<'a>(&self, command: &'a CommandSpec) -> Option<Assignment<'a>> {
        let fallback = self
            .entries
            .iter()
            .position(|entry| entry.except.is_some())?;
        let mut chosen: BTreeMap<&OutcomeName, usize> = BTreeMap::new();
        for (index, entry) in self.entries.iter().enumerate() {
            let Some(names) = &entry.outcomes else {
                continue;
            };
            for name in names {
                let Resolved::Refusals(outcomes) = resolve(command, name) else {
                    return None;
                };
                for outcome in outcomes {
                    chosen.insert(outcome, index);
                }
            }
        }
        let refusals = refusals(command)
            .into_iter()
            .map(|outcome| (outcome, chosen.get(outcome).copied().unwrap_or(fallback)))
            .collect();
        Some(Assignment { refusals, fallback })
    }

    /// The outcomes of `command` the `final:` of entry `index` makes final, in declaration order.
    pub fn finals<'a>(&self, index: usize, command: &'a CommandSpec) -> Vec<&'a OutcomeName> {
        let Some(names) = self
            .entries
            .get(index)
            .and_then(|entry| entry.finals.as_ref())
        else {
            return Vec::new();
        };
        let selected: BTreeSet<&OutcomeName> = names
            .iter()
            .filter_map(|name| match resolve(command, name) {
                Resolved::Refusals(outcomes) => Some(outcomes),
                _ => None,
            })
            .flatten()
            .collect();
        refusals(command)
            .into_iter()
            .filter(|outcome| selected.contains(outcome))
            .collect()
    }
}

/// Which entry of a [`RefusalPolicy`] answers each declared refusal of one command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Assignment<'a> {
    /// Every declared refusal, in the command's declaration order, with its entry's index.
    pub refusals: Vec<(&'a OutcomeName, usize)>,
    /// The index of the fallback entry, which also answers a failure carrying no declared outcome.
    pub fallback: usize,
}

/// What one name in a selector stands for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Resolved<'a> {
    /// These refusals of the command, in its declaration order.
    Refusals(Vec<&'a OutcomeName>),
    /// An outcome of the command that carries no error: an acceptance, not a refusal.
    Accepting,
    /// Nothing the command declares.
    Unknown,
}

/// The declared refusals of `command`: its outcomes that carry an `error:`, in declaration order.
pub fn refusals(command: &CommandSpec) -> Vec<&OutcomeName> {
    command
        .outcomes
        .iter()
        .filter(|outcome| outcome.error.is_some())
        .map(|outcome| &outcome.name)
        .collect()
}

/// What `name` selects of `command`: an outcome by its name, or an error standing for every
/// outcome reporting it — exactly as `retry.final` reads a name.
pub fn resolve<'a>(command: &'a CommandSpec, name: &str) -> Resolved<'a> {
    if let Some(outcome) = command
        .outcomes
        .iter()
        .find(|outcome| outcome.name.as_str() == name)
    {
        return if outcome.error.is_some() {
            Resolved::Refusals(vec![&outcome.name])
        } else {
            Resolved::Accepting
        };
    }
    let reporting: Vec<&OutcomeName> = command
        .outcomes
        .iter()
        .filter(|outcome| {
            outcome
                .error
                .as_ref()
                .is_some_and(|error| error.to_string() == name)
        })
        .map(|outcome| &outcome.name)
        .collect();
    if reporting.is_empty() {
        Resolved::Unknown
    } else {
        Resolved::Refusals(reporting)
    }
}

// ---- reading -------------------------------------------------------------------------------------

/// Whether an `on_failure:` value is a refusal-selected one: a mapping in which some policy writes
/// a selector. Everything else is the universal reader's.
pub(crate) fn is_selected(value: &serde_yaml::Value) -> bool {
    let Some(mapping) = value.as_mapping() else {
        return false;
    };
    mapping.iter().any(|(key, value)| {
        let Some(policy) = key.as_str().and_then(Failure::parse) else {
            return false;
        };
        match value {
            serde_yaml::Value::Sequence(_) => policy != Failure::Escalate,
            serde_yaml::Value::Mapping(block) => block
                .keys()
                .any(|key| matches!(key.as_str(), Some("outcomes" | "except"))),
            _ => false,
        }
    })
}

/// Reads a refusal-selected `on_failure:`, refusing only what cannot be represented: a key that is
/// no policy, a policy written twice, a key its block does not take, and a value of the wrong
/// shape. Everything else is validation's, so that a selected shape below ess/22 still reaches
/// the format refusal.
pub(crate) fn read(value: &serde_yaml::Value) -> Result<RefusalPolicy, String> {
    let mapping = value
        .as_mapping()
        .ok_or_else(|| "a refusal-selected `on_failure:` is a block of policies".to_owned())?;
    let mut entries: Vec<RefusalEntry> = Vec::new();
    for (key, value) in mapping {
        let written = key
            .as_str()
            .ok_or_else(|| "a policy under `on_failure:` is a word".to_owned())?;
        let policy = Failure::parse(written).ok_or_else(|| {
            format!(
                "unknown variant `{written}`, expected one of {}",
                Failure::WORDS
                    .iter()
                    .map(|word| format!("`{word}`"))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        })?;
        if entries.iter().any(|entry| entry.policy == policy) {
            return Err(format!("`on_failure` says `{written}` twice"));
        }
        let mut entry = RefusalEntry::empty(policy);
        match value {
            serde_yaml::Value::Sequence(_) if policy == Failure::Escalate => {
                return Err(
                    "`escalate:` takes a block naming the event it emits — `escalate: {emits: \
                     <event>, outcomes: [...]}` or with `except: [...]` — and has no list \
                     shorthand"
                        .to_owned(),
                );
            }
            serde_yaml::Value::Sequence(_) => {
                entry.outcomes = Some(names(value, written, "outcomes")?);
            }
            serde_yaml::Value::Mapping(block) => {
                for (key, value) in block {
                    let key = key.as_str().unwrap_or_default();
                    match (key, policy) {
                        ("outcomes", _) => entry.outcomes = Some(names(value, written, key)?),
                        ("except", _) => entry.except = Some(names(value, written, key)?),
                        ("emits", Failure::Escalate) => {
                            let text = value.as_str().ok_or_else(|| {
                                "`escalate:` names the event it emits as a qualified name"
                                    .to_owned()
                            })?;
                            entry.emits =
                                Some(QualifiedName::new(text).map_err(|error| error.to_string())?);
                        }
                        ("attempts", Failure::Retry) => {
                            entry.attempts = Some(
                                value
                                    .as_u64()
                                    .and_then(|attempts| u32::try_from(attempts).ok())
                                    .ok_or_else(|| {
                                        "`retry:` states `attempts:` as a whole number".to_owned()
                                    })?,
                            );
                        }
                        ("final", Failure::Retry) => {
                            entry.finals = Some(names(value, written, key)?);
                        }
                        (other, _) => {
                            return Err(format!(
                                "`{written}:` in a refusal-selected `on_failure:` takes {}, and \
                                 `{other}` is none of them",
                                taken(policy)
                            ));
                        }
                    }
                }
            }
            _ => {
                return Err(format!(
                    "`{written}:` in a refusal-selected `on_failure:` names its refusals: a list, \
                     or a block with `outcomes:` or `except:`"
                ));
            }
        }
        entries.push(entry);
    }
    Ok(RefusalPolicy { entries })
}

/// The keys a policy's block takes, for a refusal that names them.
fn taken(policy: Failure) -> &'static str {
    match policy {
        Failure::Drop => "`outcomes:` or `except:`",
        Failure::Retry => "`outcomes:` or `except:`, `attempts:` and `final:`",
        Failure::Escalate => "`emits:` and `outcomes:` or `except:`",
    }
}

/// A list of refusal names.
fn names(value: &serde_yaml::Value, policy: &str, key: &str) -> Result<Vec<String>, String> {
    let items = value
        .as_sequence()
        .ok_or_else(|| format!("`{policy}:` writes `{key}` as a list of refusal names"))?;
    items
        .iter()
        .map(|item| {
            item.as_str()
                .map(ToOwned::to_owned)
                .ok_or_else(|| format!("`{policy}:` writes each name under `{key}` as text"))
        })
        .collect()
}

// ---- validation ----------------------------------------------------------------------------------

/// Everything a binding can be wrong about in its own refusal-selected policy without the rest of
/// the specification: a periodic cause, and a legacy view assembled in code out of step with the
/// table.
pub(crate) fn check_local(binding: &BindingSpec) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    let Some(policy) = &binding.refusals else {
        return errors;
    };
    let at = format!("binding.{}.on_failure", binding.name);
    if binding.cause.periodic().is_some() {
        errors.push(
            ValidationError::new(
                ValidationCode::UnsupportedConstruct,
                at.clone(),
                "a periodic cause invokes no command a refusal could come from, and its polls \
                 drop every failure",
            )
            .with_hint("write `on_failure: drop` for a periodic cause"),
        );
    }
    let (failure, escalation, retry) = policy.legacy_view();
    if binding.failure != failure || binding.escalation != escalation || binding.retry != retry {
        errors.push(
            ValidationError::new(
                ValidationCode::ConflictingDeclaration,
                at,
                format!(
                    "binding `{}` carries a universal failure policy that disagrees with its \
                     refusal table",
                    binding.name
                ),
            )
            .with_hint(
                "a document cannot write this: the reader derives the universal fields from the \
                 table, so this binding was assembled in code; derive them with \
                 `RefusalPolicy::legacy_view`",
            ),
        );
    }
    errors
}

/// The format gate, each policy's shape, and every name against the invoked command, for every
/// binding of the specification with a refusal-selected policy, including one assembled in code.
pub fn validate_specification(spec: &crate::Specification) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    for binding in spec.bindings().values() {
        let Some(policy) = &binding.refusals else {
            continue;
        };
        let at = format!("binding.{}.on_failure", binding.name);
        if spec.system().format.major() < FORMAT.major() {
            errors.push(
                ValidationError::new(
                    ValidationCode::UnsupportedFormatVersion,
                    at.clone(),
                    "a failure policy selected per refusal requires specification format ess/22",
                )
                .with_hint(
                    "declare `format: ess/22`, or write one universal policy: `retry`, `drop`, \
                     or `escalate:` with `emits:`",
                ),
            );
        }
        errors.extend(check_shape(binding, policy, &at));
        // A command nobody declares is reported once, by the binding's own validation.
        if let Some(command) = spec.commands().get(&binding.command) {
            errors.extend(check_names(binding, policy, command, &at));
        }
    }
    errors
}

fn check_shape(binding: &BindingSpec, policy: &RefusalPolicy, at: &str) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    for entry in &policy.entries {
        let here = format!("{at}.{}", entry.policy);
        errors.extend(check_selector(entry, &here));
        errors.extend(check_block(binding, entry, &here));
        errors.extend(check_repeats(entry, &here));
    }
    errors.extend(check_fallback_count(binding, policy, at));
    errors
}

/// Exactly one selector, and a positive one that selects something.
fn check_selector(entry: &RefusalEntry, here: &str) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    let word = entry.policy;
    match (&entry.outcomes, &entry.except) {
        (Some(_), Some(_)) => errors.push(
            ValidationError::new(
                ValidationCode::ConflictingDeclaration,
                here,
                format!(
                    "`{word}` names both `outcomes:` and `except:`; a policy selects its \
                     refusals one way"
                ),
            )
            .with_hint(
                "keep `outcomes:` for the refusals it takes, or `except:` if it is the fallback",
            ),
        ),
        (None, None) => errors.push(
            ValidationError::new(
                ValidationCode::MissingDeclaration,
                here,
                format!(
                    "`{word}` names no refusals, beside a policy that does; in a \
                     refusal-selected `on_failure:` every policy selects"
                ),
            )
            .with_hint(
                "write `outcomes: [<refusal>]`, or `except: [<refusal>]` on the one policy that \
                 is the fallback",
            ),
        ),
        (Some(outcomes), None) if outcomes.is_empty() => errors.push(
            ValidationError::new(
                ValidationCode::MissingDeclaration,
                here,
                format!("`{word}` selects no refusal: its `outcomes` list is empty"),
            )
            .with_hint("name the refusals it takes, or remove the policy"),
        ),
        _ => {}
    }
    errors
}

/// What the policy's own block owes: an escalation's event, a retry's bound.
fn check_block(binding: &BindingSpec, entry: &RefusalEntry, here: &str) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    if entry.policy == Failure::Escalate && entry.emits.is_none() {
        errors.push(
            ValidationError::new(
                ValidationCode::MissingDeclaration,
                here,
                format!(
                    "binding `{}` escalates and does not say what that emits, so nothing can be \
                     asked to prove the escalation happened",
                    binding.name
                ),
            )
            .with_hint("write `emits: <event>` in the `escalate:` block"),
        );
    }
    if entry.policy != Failure::Retry {
        return errors;
    }
    if entry.finals.is_some() && entry.attempts.is_none() {
        errors.push(
            ValidationError::new(
                ValidationCode::ConflictingDeclaration,
                format!("{here}.final"),
                format!(
                    "binding `{}` names `final:` refusals for a retry that states no \
                     `attempts:`; a final refusal ends a bounded retry",
                    binding.name
                ),
            )
            .with_hint("state `attempts:`, or remove `final:`"),
        );
    }
    if let Some(attempts) = entry
        .attempts
        .filter(|attempts| *attempts < RetryBound::MIN_ATTEMPTS)
    {
        errors.push(
            ValidationError::new(
                ValidationCode::ConflictingDeclaration,
                format!("{here}.attempts"),
                format!(
                    "binding `{}` retries with `attempts: {attempts}`, which makes no second \
                     attempt",
                    binding.name
                ),
            )
            .with_hint(
                "`attempts` counts invocations including the first; select those refusals under \
                 `drop`, or state at least 2",
            ),
        );
    }
    errors
}

/// A name written twice in one list.
fn check_repeats(entry: &RefusalEntry, here: &str) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    for (key, names) in [
        ("outcomes", &entry.outcomes),
        ("except", &entry.except),
        ("final", &entry.finals),
    ] {
        let mut seen = BTreeSet::new();
        for name in names.iter().flatten() {
            if !seen.insert(name.as_str()) {
                errors.push(
                    ValidationError::new(
                        ValidationCode::DuplicateDeclaration,
                        format!("{here}.{key}"),
                        format!(
                            "`{name}` is written twice under `{}`'s `{key}:`",
                            entry.policy
                        ),
                    )
                    .with_hint("name each refusal once"),
                );
            }
        }
    }
    errors
}

/// Exactly one explicit fallback.
fn check_fallback_count(
    binding: &BindingSpec,
    policy: &RefusalPolicy,
    at: &str,
) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    let fallbacks: Vec<Failure> = policy
        .entries
        .iter()
        .filter(|entry| entry.except.is_some())
        .map(|entry| entry.policy)
        .collect();
    match fallbacks.as_slice() {
        [_] => {}
        [] => errors.push(
            ValidationError::new(
                ValidationCode::MissingDeclaration,
                at,
                format!(
                    "binding `{}` names no explicit fallback: exactly one policy takes `except:`, \
                     and it answers every failure that carries no declared outcome",
                    binding.name
                ),
            )
            .with_hint(
                "write `except: [...]` on one policy, listing the refusals the others select",
            ),
        ),
        several => errors.push(
            ValidationError::new(
                ValidationCode::ConflictingDeclaration,
                at,
                format!(
                    "{} all take `except:`; exactly one policy is the fallback",
                    several
                        .iter()
                        .map(|word| format!("`{word}`"))
                        .collect::<Vec<_>>()
                        .join(" and ")
                ),
            )
            .with_hint("select the others' refusals with `outcomes:`"),
        ),
    }
    errors
}

/// One binding's names, resolved against the command it invokes.
struct Names<'a> {
    binding: &'a BindingSpec,
    command: &'a CommandSpec,
    at: &'a str,
}

impl<'a> Names<'a> {
    /// The refusals one list selects, refusing a name that is no refusal, an accepting outcome,
    /// and two names of the list that select one refusal.
    fn resolve(
        &self,
        word: Failure,
        key: &str,
        names: &[String],
        errors: &mut ValidationErrors,
    ) -> BTreeSet<&'a OutcomeName> {
        let here = format!("{}.{word}", self.at);
        let mut selected: BTreeMap<&OutcomeName, &str> = BTreeMap::new();
        for name in names {
            match resolve(self.command, name) {
                Resolved::Refusals(outcomes) => {
                    for outcome in outcomes {
                        match selected.get(outcome) {
                            None => {
                                selected.insert(outcome, name.as_str());
                            }
                            // The same name twice is `check_repeats`'s.
                            Some(earlier) if *earlier == name.as_str() => {}
                            Some(earlier) => errors.push(
                                ValidationError::new(
                                    ValidationCode::DuplicateDeclaration,
                                    format!("{here}.{key}"),
                                    format!(
                                        "`{earlier}` and `{name}` both select `{outcome}` under \
                                         `{word}`'s `{key}:`"
                                    ),
                                )
                                .with_hint(
                                    "an error stands for every outcome reporting it; name each \
                                     refusal once, by its outcome or by its error",
                                ),
                            ),
                        }
                    }
                }
                Resolved::Accepting => errors.push(
                    ValidationError::new(
                        ValidationCode::ConflictingDeclaration,
                        here.clone(),
                        format!(
                            "`{name}` under `{word}:` is an accepting outcome of `{}`; only a \
                             refusal — an outcome with `error:` — selects a failure policy",
                            self.command.name
                        ),
                    )
                    .with_hint(available(self.command)),
                ),
                Resolved::Unknown => errors.push(
                    ValidationError::new(
                        ValidationCode::UndeclaredReference,
                        here.clone(),
                        format!(
                            "`{name}` under `{word}:` is no refusal of `{}`, the command `{}` \
                             invokes",
                            self.command.name, self.binding.name
                        ),
                    )
                    .with_hint(available(self.command)),
                ),
            }
        }
        selected.into_keys().collect()
    }
}

/// Each name resolved against the invoked command, then the resolved sets disjoint and exhaustive.
fn check_names(
    binding: &BindingSpec,
    policy: &RefusalPolicy,
    command: &CommandSpec,
    at: &str,
) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    let names = Names {
        binding,
        command,
        at,
    };
    let mut positive: Vec<(Failure, BTreeSet<&OutcomeName>)> = Vec::new();
    let mut excepted: Option<(Failure, BTreeSet<&OutcomeName>)> = None;
    for entry in &policy.entries {
        if let Some(list) = &entry.outcomes {
            let set = names.resolve(entry.policy, "outcomes", list, &mut errors);
            positive.push((entry.policy, set));
        }
        if let Some(list) = &entry.except {
            let set = names.resolve(entry.policy, "except", list, &mut errors);
            excepted.get_or_insert((entry.policy, set));
        }
    }
    let finals = policy
        .entry(Failure::Retry)
        .and_then(|entry| entry.finals.as_ref())
        .map(|list| names.resolve(Failure::Retry, "final", list, &mut errors));
    let owner = check_disjoint(&positive, at, &mut errors);
    if let Some((fallback, except)) = &excepted {
        check_complement(&owner, *fallback, except, at, &mut errors);
        if let Some(finals) = &finals {
            // `final` resolves wholly inside the retry's own selection.
            let selection: BTreeSet<&OutcomeName> = if *fallback == Failure::Retry {
                refusals(command)
                    .into_iter()
                    .filter(|outcome| !except.contains(outcome))
                    .collect()
            } else {
                owner
                    .iter()
                    .filter(|(_, word)| **word == Failure::Retry)
                    .map(|(outcome, _)| *outcome)
                    .collect()
            };
            for outcome in finals.difference(&selection) {
                errors.push(
                    ValidationError::new(
                        ValidationCode::ConflictingDeclaration,
                        format!("{at}.retry.final"),
                        format!(
                            "`{outcome}` under `final:` is a refusal `retry` does not select; a \
                             final refusal is one the retry itself answers"
                        ),
                    )
                    .with_hint(format!(
                        "select `{outcome}` under `retry`, or remove it from `final:`"
                    )),
                );
            }
        }
    }
    errors
}

/// No refusal in two positive policies; the policy each positively selected refusal falls to.
fn check_disjoint<'a>(
    positive: &[(Failure, BTreeSet<&'a OutcomeName>)],
    at: &str,
    errors: &mut ValidationErrors,
) -> BTreeMap<&'a OutcomeName, Failure> {
    let mut owner: BTreeMap<&OutcomeName, Failure> = BTreeMap::new();
    for (word, set) in positive {
        for outcome in set {
            if let Some(other) = owner.get(outcome) {
                errors.push(
                    ValidationError::new(
                        ValidationCode::ConflictingDeclaration,
                        at,
                        format!(
                            "`{outcome}` is selected by both `{other}` and `{word}`; each refusal \
                             has exactly one policy"
                        ),
                    )
                    .with_hint("select it under one policy only"),
                );
            } else {
                owner.insert(outcome, *word);
            }
        }
    }
    owner
}

/// Exhaustive and disjoint with the complement: the fallback's `except` is exactly what the
/// positive policies select.
fn check_complement(
    owner: &BTreeMap<&OutcomeName, Failure>,
    fallback: Failure,
    except: &BTreeSet<&OutcomeName>,
    at: &str,
    errors: &mut ValidationErrors,
) {
    for (outcome, word) in owner {
        if !except.contains(outcome) {
            errors.push(
                ValidationError::new(
                    ValidationCode::ConflictingDeclaration,
                    format!("{at}.{fallback}.except"),
                    format!(
                        "`{outcome}` is selected by `{word}` and also falls to `{fallback}`, whose \
                         `except:` does not leave it out; each refusal has exactly one policy"
                    ),
                )
                .with_hint(format!("add `{outcome}` to `{fallback}`'s `except:`")),
            );
        }
    }
    for outcome in except {
        if !owner.contains_key(outcome) {
            errors.push(
                ValidationError::new(
                    ValidationCode::MissingDeclaration,
                    format!("{at}.{fallback}.except"),
                    format!(
                        "`{outcome}` is excepted from `{fallback}` and no other policy selects \
                         it; every refusal has exactly one policy"
                    ),
                )
                .with_hint(format!(
                    "select `{outcome}` under another policy, or remove it from `except:`"
                )),
            );
        }
    }
}

/// The refusals a name may select, for a hint.
fn available(command: &CommandSpec) -> String {
    let refusals: Vec<String> = command
        .outcomes
        .iter()
        .filter_map(|outcome| {
            outcome
                .error
                .as_ref()
                .map(|error| format!("`{}` ({error})", outcome.name))
        })
        .collect();
    if refusals.is_empty() {
        format!(
            "`{}` declares no outcome with `error:`; write one universal policy instead",
            command.name
        )
    } else {
        format!(
            "name an outcome of `{}` that carries `error:`, or its error: {}",
            command.name,
            refusals.join(", ")
        )
    }
}
