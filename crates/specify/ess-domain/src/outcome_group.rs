//! Outcome groups: one outcome declared once for many commands (beyond10x/ess#105).
//!
//! `docs/design/outcome-groups.md` is the binding design.

use std::collections::{BTreeMap, BTreeSet};

use ess_primitives::error::{ParseError, ValidationCode, ValidationError, ValidationErrors};

use crate::command::{OutcomeName, PayloadDeclaration, PayloadTable, RawOutcome};
use crate::name::QualifiedName;
use crate::refs::Refs;
use crate::spec::RawSpecFile;
use crate::system::{FormatVersion, Source};

/// An outcome group's identifier.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
#[serde(transparent)]
pub struct OutcomeGroupName(String);

impl OutcomeGroupName {
    /// What an outcome group name looks like.
    pub const PATTERN: &'static str = "^[a-z][a-z0-9]*(-[a-z0-9]+)*$";

    /// Parses one.
    pub fn new(value: impl AsRef<str>) -> Result<Self, ParseError> {
        let value = value.as_ref();
        let valid = !value.is_empty()
            && value.starts_with(|c: char| c.is_ascii_lowercase())
            && !value.ends_with('-')
            && !value.contains("--")
            && value
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
        if !valid {
            return Err(ParseError::identifier(
                "outcome group name",
                value,
                "an outcome group name is lower-case words joined by single hyphens, such as \
                 `remote-backed`"
                    .to_owned(),
            ));
        }
        Ok(Self(value.to_owned()))
    }

    /// The name as text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> serde::Deserialize<'de> for OutcomeGroupName {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(deserializer)?;
        Self::new(raw).map_err(serde::de::Error::custom)
    }
}

impl std::fmt::Display for OutcomeGroupName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl schemars::JsonSchema for OutcomeGroupName {
    fn schema_name() -> String {
        "OutcomeGroupName".to_owned()
    }

    fn json_schema(_: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        let mut schema = schemars::schema::SchemaObject {
            instance_type: Some(schemars::schema::InstanceType::String.into()),
            ..Default::default()
        };
        schema.string().pattern = Some(Self::PATTERN.to_owned());
        schema.metadata().description =
            Some("An outcome group's identifier, such as `remote-backed`.".to_owned());
        schema.into()
    }
}

/// One outcome group as written: which commands, and the outcomes each of them gains.
#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RawOutcomeGroup {
    /// Its identity, unique across the specification.
    pub name: OutcomeGroupName,
    /// Explicit membership, by fully qualified command name.
    #[serde(default)]
    pub commands: Option<BTreeSet<QualifiedName>>,
    /// Every command this actor `may:` invoke.
    #[serde(default)]
    pub actor: Option<QualifiedName>,
    /// Every command declared in a file whose `domain:` is this name.
    #[serde(default)]
    pub domain: Option<QualifiedName>,
    /// Commands the `actor:` or `domain:` selector would include and the group skips.
    #[serde(default)]
    pub except: BTreeSet<QualifiedName>,
    /// The outcomes each member gains, after its own.
    #[serde(default)]
    pub outcomes: Vec<RawGroupOutcome>,
}

/// One outcome of a group: always an external refusal.
#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RawGroupOutcome {
    /// What the outcome is called in every member.
    pub name: OutcomeName,
    /// What outside the input decides this branch.
    pub external: String,
    /// The error it reports.
    pub error: QualifiedName,
    /// One line for generated documentation.
    #[serde(default)]
    pub summary: Option<String>,
    /// The records outside this model that explain it.
    #[serde(default)]
    pub refs: Refs,
}

impl RawGroupOutcome {
    /// The outcome a member gains: exactly what the author would have written by hand.
    fn written(&self) -> RawOutcome {
        RawOutcome {
            name: self.name.clone(),
            when: None,
            when_subject: None,
            when_subject_state: None,
            when_state_changes: None,
            external: Some(self.external.clone()),
            wrong_state: false,
            refuses: None,
            creates: None,
            moves: None,
            updates: None,
            preserves: None,
            replays: None,
            instance: None,
            emits: Vec::new(),
            payload: PayloadDeclaration::default(),
            sets: PayloadTable::default(),
            error: Some(self.error.clone()),
            summary: self.summary.clone(),
            refs: self.refs.clone(),
        }
    }
}

/// What the raw files declare, read before any of them is converted.
struct Declared {
    /// Every command name, whichever file declares it.
    commands: BTreeSet<QualifiedName>,
    /// Each file's `domain:` and the commands that file declares.
    by_domain: BTreeMap<QualifiedName, BTreeSet<QualifiedName>>,
    /// Every error name.
    errors: BTreeSet<QualifiedName>,
    /// Each actor's `may:`, from its first declaration in source order: the one assembly keeps.
    actors: BTreeMap<QualifiedName, BTreeSet<QualifiedName>>,
    /// The header's format, when exactly one file carries `system:`.
    format: Option<FormatVersion>,
}

impl Declared {
    fn read(files: &[(Source, RawSpecFile)]) -> Self {
        let mut declared = Self {
            commands: BTreeSet::new(),
            by_domain: BTreeMap::new(),
            errors: BTreeSet::new(),
            actors: BTreeMap::new(),
            format: None,
        };
        let mut headers = 0;
        for (_, file) in files {
            if file.system.is_some() {
                headers += 1;
                declared.format = Some(file.format.unwrap_or(FormatVersion::V1));
            }
            let names = file.commands.iter().map(|command| command.name.clone());
            declared.commands.extend(names.clone());
            if let Some(domain) = &file.domain {
                declared
                    .by_domain
                    .entry(domain.clone())
                    .or_default()
                    .extend(names);
            }
            declared
                .errors
                .extend(file.errors.iter().map(|error| error.name.clone()));
            for actor in &file.actors {
                declared
                    .actors
                    .entry(actor.name.clone())
                    .or_insert_with(|| actor.may.clone());
            }
        }
        // No header, or two: the merge refuses the specification, and there is no format to gate on.
        if headers != 1 {
            declared.format = None;
        }
        declared
    }
}

/// Which way a group chose its members, for the wording of a refusal.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Form {
    List,
    Selector,
}

/// A group that raised no refusal of its own, and the commands it expands into.
struct Admitted {
    group: RawOutcomeGroup,
    form: Form,
    members: BTreeSet<QualifiedName>,
}

/// Expands every file's outcome groups into their member commands, before anything is converted.
///
/// Each member gains the outcomes of every group selecting it, after its own, in ascending group
/// name order, so that the result is the document an author copying the outcomes by hand would have
/// written. A group with a refusal of its own expands into nothing; a collision with a command's
/// own outcome, or with another group's, withholds that group from that command only.
pub fn expand(files: &mut [(Source, RawSpecFile)], errors: &mut ValidationErrors) {
    let declared = Declared::read(files);
    let mut written: Vec<(Source, RawOutcomeGroup)> = Vec::new();
    for (source, file) in files.iter_mut() {
        for group in std::mem::take(&mut file.outcome_groups) {
            written.push((source.clone(), group));
        }
    }

    let mut seen: BTreeSet<OutcomeGroupName> = BTreeSet::new();
    let mut admitted: BTreeMap<OutcomeGroupName, Admitted> = BTreeMap::new();
    for (source, group) in written {
        let first = seen.insert(group.name.clone());
        if !first {
            errors.push(
                ValidationError::new(
                    ValidationCode::DuplicateDeclaration,
                    format!("outcome_groups.{}", group.name),
                    format!(
                        "`{}` is declared more than once; {source} declares it again",
                        group.name
                    ),
                )
                .with_hint("a name identifies one thing; two declarations cannot both be it"),
            );
        }
        if let Some(format) = declared.format {
            if format.major() < FormatVersion::V12.major() {
                errors.push(ValidationError::new(
                    ValidationCode::UnsupportedFormatVersion,
                    format!("outcome_groups.{}", group.name),
                    "outcome groups require specification format ess/12",
                ));
            }
        }
        let before = errors.len();
        let membership = check(&group, &declared, errors);
        if let Some((form, members)) = membership {
            if first && errors.len() == before {
                admitted.insert(
                    group.name.clone(),
                    Admitted {
                        group,
                        form,
                        members,
                    },
                );
            }
        }
    }

    let gains = collisions(files, &admitted, errors);
    for (_, file) in files.iter_mut() {
        for command in &mut file.commands {
            let Some(groups) = gains.get(&command.name) else {
                continue;
            };
            for name in groups {
                command.outcomes.extend(
                    admitted[name]
                        .group
                        .outcomes
                        .iter()
                        .map(RawGroupOutcome::written),
                );
            }
        }
    }
}

/// G2–G13: a group's own refusals. Returns the membership when exactly one form was written.
fn check(
    group: &RawOutcomeGroup,
    declared: &Declared,
    errors: &mut ValidationErrors,
) -> Option<(Form, BTreeSet<QualifiedName>)> {
    let name = &group.name;
    let written: Vec<&str> = [
        ("commands", group.commands.is_some()),
        ("actor", group.actor.is_some()),
        ("domain", group.domain.is_some()),
    ]
    .into_iter()
    .filter_map(|(key, present)| present.then_some(key))
    .collect();

    check_outcomes(group, declared, errors);

    match written.len() {
        0 => {
            errors.push(
                ValidationError::new(
                    ValidationCode::MissingDeclaration,
                    format!("outcome_groups.{name}"),
                    format!("outcome group `{name}` says which commands it applies to nowhere"),
                )
                .with_hint(
                    "write one of `commands: [...]`, `actor: <actor>` or `domain: <domain>`",
                ),
            );
            return None;
        }
        1 => {}
        _ => {
            errors.push(
                ValidationError::new(
                    ValidationCode::ConflictingDeclaration,
                    format!("outcome_groups.{name}"),
                    format!(
                        "outcome group `{name}` writes {}; a group has exactly one membership form",
                        written
                            .iter()
                            .map(|key| format!("`{key}`"))
                            .collect::<Vec<_>>()
                            .join(" and ")
                    ),
                )
                .with_hint("two groups with the same outcomes express a union"),
            );
            return None;
        }
    }

    if let Some(listed) = &group.commands {
        return Some((Form::List, check_list(group, listed, declared, errors)));
    }
    check_selector(group, declared, errors).map(|members| (Form::Selector, members))
}

/// G4, G7 and G9 for an explicit `commands:` list.
fn check_list(
    group: &RawOutcomeGroup,
    listed: &BTreeSet<QualifiedName>,
    declared: &Declared,
    errors: &mut ValidationErrors,
) -> BTreeSet<QualifiedName> {
    let name = &group.name;
    if !group.except.is_empty() {
        errors.push(
            ValidationError::new(
                ValidationCode::ConflictingDeclaration,
                format!("outcome_groups.{name}.except"),
                format!(
                    "outcome group `{name}` lists its commands and also excepts some; a command \
                     both listed and excepted says two opposite things"
                ),
            )
            .with_hint("leave the command out of `commands:`"),
        );
    }
    for command in listed {
        if !declared.commands.contains(command) {
            errors.push(
                ValidationError::new(
                    ValidationCode::UndeclaredReference,
                    format!("outcome_groups.{name}.commands"),
                    format!(
                        "outcome group `{name}` lists `{command}`, which no domain declares as \
                         a command"
                    ),
                )
                .with_hint(format!(
                    "declared commands: {}",
                    listing(&declared.commands)
                )),
            );
        }
    }
    if listed.is_empty() {
        empty(name, errors);
    }
    listed.clone()
}

/// G5, G6, G8 and G9 for an `actor:` or `domain:` selector.
fn check_selector(
    group: &RawOutcomeGroup,
    declared: &Declared,
    errors: &mut ValidationErrors,
) -> Option<BTreeSet<QualifiedName>> {
    let name = &group.name;
    let selected: BTreeSet<QualifiedName> = if let Some(actor) = &group.actor {
        let Some(may) = declared.actors.get(actor) else {
            errors.push(ValidationError::new(
                ValidationCode::UndeclaredReference,
                format!("outcome_groups.{name}"),
                format!(
                    "outcome group `{name}` selects the grants of `{actor}`, which no domain \
                     declares as an actor"
                ),
            ));
            return None;
        };
        // A grant naming no command is the actor's own refusal, reported by the actor.
        may.intersection(&declared.commands).cloned().collect()
    } else {
        let domain = group.domain.as_ref()?;
        let Some(commands) = declared.by_domain.get(domain) else {
            errors.push(ValidationError::new(
                ValidationCode::UndeclaredReference,
                format!("outcome_groups.{name}"),
                format!(
                    "outcome group `{name}` selects the commands of `{domain}`, which no source \
                     contributes as a domain"
                ),
            ));
            return None;
        };
        commands.clone()
    };

    for excepted in &group.except {
        if !declared.commands.contains(excepted) {
            errors.push(
                ValidationError::new(
                    ValidationCode::UndeclaredReference,
                    format!("outcome_groups.{name}.except"),
                    format!(
                        "outcome group `{name}` excepts `{excepted}`, which no domain declares \
                         as a command"
                    ),
                )
                .with_hint(format!(
                    "declared commands: {}",
                    listing(&declared.commands)
                )),
            );
        } else if !selected.contains(excepted) {
            errors.push(
                ValidationError::new(
                    ValidationCode::ConflictingDeclaration,
                    format!("outcome_groups.{name}.except"),
                    format!(
                        "outcome group `{name}` excepts `{excepted}`, which its selector does not \
                         select; the exception excludes nothing now, and would start excluding \
                         silently when the selection grew"
                    ),
                )
                .with_hint(format!("remove `{excepted}` from `except:`")),
            );
        }
    }
    let members: BTreeSet<QualifiedName> = selected.difference(&group.except).cloned().collect();
    if members.is_empty() {
        empty(name, errors);
    }
    Some(members)
}

/// G9: a group that selects no command.
fn empty(name: &OutcomeGroupName, errors: &mut ValidationErrors) {
    errors.push(
        ValidationError::new(
            ValidationCode::EmptyDeclaration,
            format!("outcome_groups.{name}"),
            format!("outcome group `{name}` applies to no command"),
        )
        .with_hint("a group that gives its outcomes to nothing says nothing; remove it"),
    );
}

/// G10–G13: the group's own outcomes.
fn check_outcomes(group: &RawOutcomeGroup, declared: &Declared, errors: &mut ValidationErrors) {
    let name = &group.name;
    if group.outcomes.is_empty() {
        errors.push(
            ValidationError::new(
                ValidationCode::EmptyDeclaration,
                format!("outcome_groups.{name}.outcomes"),
                format!("outcome group `{name}` declares no outcome"),
            )
            .with_hint("a group exists to give its members an outcome; declare one"),
        );
    }
    let mut seen: BTreeSet<&OutcomeName> = BTreeSet::new();
    for outcome in &group.outcomes {
        let at = &outcome.name;
        if !seen.insert(at) {
            errors.push(
                ValidationError::new(
                    ValidationCode::DuplicateDeclaration,
                    format!("outcome_groups.{name}.outcomes.{at}"),
                    format!(
                        "outcome group `{name}` declares outcome `{at}` more than once; every \
                         member would gain two branches with one name"
                    ),
                )
                .with_hint("name the second outcome after what makes it different"),
            );
        }
        if outcome.external.trim().is_empty() {
            errors.push(
                ValidationError::new(
                    ValidationCode::UnexplainedDecision,
                    format!("outcome_groups.{name}.outcomes.{at}.external"),
                    format!(
                        "outcome `{at}` of group `{name}` is decided outside the input but states \
                         no cause; a test runner cannot inject a fault nobody named"
                    ),
                )
                .with_hint("write what fails, as in `the service rejects the session credential`"),
            );
        }
        if !declared.errors.contains(&outcome.error) {
            errors.push(
                ValidationError::new(
                    ValidationCode::UndeclaredReference,
                    format!("outcome_groups.{name}.outcomes.{at}.error"),
                    format!(
                        "outcome `{at}` of group `{name}` reports `{}`, which no domain declares \
                         as an error",
                        outcome.error
                    ),
                )
                .with_hint(format!("declared errors: {}", listing(&declared.errors))),
            );
        }
    }
}

/// G14 and G15, per member command. Returns, for each command, the groups it gains in name order.
fn collisions(
    files: &[(Source, RawSpecFile)],
    admitted: &BTreeMap<OutcomeGroupName, Admitted>,
    errors: &mut ValidationErrors,
) -> BTreeMap<QualifiedName, Vec<OutcomeGroupName>> {
    let mut own: BTreeMap<&QualifiedName, BTreeSet<&OutcomeName>> = BTreeMap::new();
    for (_, file) in files {
        for command in &file.commands {
            own.entry(&command.name)
                .or_default()
                .extend(command.outcomes.iter().map(|outcome| &outcome.name));
        }
    }
    let members: BTreeSet<&QualifiedName> = admitted
        .values()
        .flat_map(|admitted| admitted.members.iter())
        .collect();

    let mut gains = BTreeMap::new();
    for command in members {
        let declared_here = own.get(command).cloned().unwrap_or_default();
        let mut candidates: Vec<&Admitted> = Vec::new();
        for admitted in admitted.values() {
            if !admitted.members.contains(command) {
                continue;
            }
            let mut collides = false;
            for outcome in &admitted.group.outcomes {
                if !declared_here.contains(&outcome.name) {
                    continue;
                }
                collides = true;
                let group = &admitted.group.name;
                let at = &outcome.name;
                let leave = match admitted.form {
                    Form::List => format!("leave `{command}` out of `commands:`"),
                    Form::Selector => format!("list `{command}` under `except:`"),
                };
                errors.push(
                    ValidationError::new(
                        ValidationCode::DuplicateDeclaration,
                        format!("outcome_groups.{group}.outcomes.{at}"),
                        format!(
                            "outcome group `{group}` gives `{command}` an outcome `{at}`, and \
                             `{command}` already declares one of that name; a group never \
                             overrides a command's own outcome"
                        ),
                    )
                    .with_hint(format!(
                        "rename one of the two, drop the command's own, or {leave}"
                    )),
                );
            }
            if !collides {
                candidates.push(admitted);
            }
        }

        let mut offered: BTreeMap<&OutcomeName, &OutcomeGroupName> = BTreeMap::new();
        let mut withheld: BTreeSet<&OutcomeGroupName> = BTreeSet::new();
        for admitted in &candidates {
            let group = &admitted.group.name;
            for outcome in &admitted.group.outcomes {
                let at = &outcome.name;
                match offered.get(at) {
                    None => {
                        offered.insert(at, group);
                    }
                    Some(earlier) => {
                        withheld.insert(earlier);
                        withheld.insert(group);
                        errors.push(
                            ValidationError::new(
                                ValidationCode::DuplicateDeclaration,
                                format!("outcome_groups.{group}.outcomes.{at}"),
                                format!(
                                    "outcome groups `{earlier}` and `{group}` both give \
                                     `{command}` an outcome `{at}`; neither is expanded into it"
                                ),
                            )
                            .with_hint(format!(
                                "rename one of the two outcomes, or leave `{command}` out of one \
                                 group"
                            )),
                        );
                    }
                }
            }
        }
        let gained: Vec<OutcomeGroupName> = candidates
            .iter()
            .map(|admitted| &admitted.group.name)
            .filter(|group| !withheld.contains(group))
            .cloned()
            .collect();
        if !gained.is_empty() {
            gains.insert(command.clone(), gained);
        }
    }
    gains
}

/// Names for a hint, or `none`.
fn listing(names: &BTreeSet<QualifiedName>) -> String {
    if names.is_empty() {
        return "none".to_owned();
    }
    names
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}
