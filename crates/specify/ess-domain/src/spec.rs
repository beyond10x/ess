//! The whole specification, assembled from however many files it was written in.
//!
//! [`SystemSpec`] and [`DomainSpec`] say what *belongs* where, by name. This is where the things
//! themselves live, and where a reference stops being a name and starts being checked.
//!
//! ```text
//! system.yaml         format, system name, version
//! domains/invoice.yaml  types, entities, commands, events, views, errors, actors
//! domains/email.yaml    …
//!         │
//!         ▼  assemble
//! Specification        every member, indexed by identity
//!         │
//!         ▼  validate
//! every reference resolves, or a list of the ones that do not
//! ```
//!
//! # Why assembly and validation are separate passes
//!
//! A domain file may reference something a later file declares. Checking as each file is read would
//! make validity depend on the order the files happened to be listed in — so everything is absorbed
//! first, and only then checked. The cost is that a broken reference is reported against the
//! specification rather than against a line, which is what the compiler's source spans will fix in
//! wave 2.

use std::collections::{BTreeMap, BTreeSet};

use ess_primitives::error::{ValidationCode, ValidationError, ValidationErrors};

use crate::actor::ActorSpec;
use crate::command::{CommandSpec, ErrorSpec, EventSpec};
use crate::domain::{DomainSpec, MemberKind};
use crate::entity::{EntityCatalogue, EntitySpec};
use crate::name::{Naming, QualifiedName, Version};
use crate::system::{Claim, FormatVersion, Source, SpecPart, SystemSpec};
use crate::types::NamedType;
use crate::view::ViewSpec;

/// One source file, as parsed.
///
/// A file may carry the system header, one domain's contents, or both — which is what lets a small
/// specification be one file and a large one be a directory (design §24).
#[derive(Debug, Clone, Default, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RawSpecFile {
    /// The specification language, on the file that carries the header.
    #[serde(default)]
    pub format: Option<FormatVersion>,
    /// The system's namespace.
    #[serde(default)]
    pub system: Option<QualifiedName>,
    /// The system's version.
    #[serde(default)]
    pub version: Option<Version>,
    /// What the system is, in one paragraph.
    #[serde(default)]
    pub summary: Option<String>,
    /// The domains this system has.
    ///
    /// A roster, not a comment: when the header lists any, the list and the domains the sources
    /// contribute have to agree in both directions. Writing it is optional; writing it wrongly is
    /// not, because a header that says `billing.invoce` while a file declares `billing.invoice` is
    /// a typo that would otherwise change nothing.
    #[serde(default)]
    pub domains: Vec<QualifiedName>,
    /// The ambient command invocations every scenario runs inside, on the file that carries the
    /// header (ess/15, `docs/design/outcome-shapes.md`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub preconditions: Vec<crate::system::Precondition>,
    /// The domain this file contributes to, when it contributes to one.
    #[serde(default)]
    pub domain: Option<QualifiedName>,
    /// What that domain is called on the wire and shown as.
    ///
    /// On the file carrying `domain:`, not on the one carrying `system:` — a domain's wire name
    /// belongs to the domain, and putting it in the header would make the header the place every
    /// context has to be edited.
    #[serde(default)]
    pub naming: Naming,
    /// Types declared here.
    #[serde(default)]
    pub types: Vec<crate::types::RawNamedType>,
    /// Conversions this specification permits, each with the reason it is permitted.
    #[serde(default)]
    pub conversions: Vec<crate::types::Conversion>,
    /// Entities declared here.
    #[serde(default)]
    pub entities: Vec<crate::entity::RawEntitySpec>,
    /// Commands declared here.
    #[serde(default)]
    pub commands: Vec<crate::command::RawCommandSpec>,
    /// Events declared here.
    #[serde(default)]
    pub events: Vec<crate::command::RawEventSpec>,
    /// Errors declared here.
    #[serde(default)]
    pub errors: Vec<crate::command::RawErrorSpec>,
    /// Views declared here.
    #[serde(default)]
    pub views: Vec<crate::view::RawViewSpec>,
    /// Actors declared here.
    #[serde(default)]
    pub actors: Vec<crate::actor::RawActorSpec>,
    /// Components declared here.
    ///
    /// Not under a `domain:` — a component owns domains, so it sits above them, and a component
    /// file is one that names no domain of its own.
    #[serde(default)]
    pub components: Vec<crate::component::RawComponentSpec>,
    /// Bindings declared here.
    #[serde(default)]
    pub bindings: Vec<crate::binding::RawBindingSpec>,
    /// The system's runtime shape.
    #[serde(default)]
    pub topology: Option<crate::topology::RawTopology>,
    /// Outcome groups declared here: one outcome written once for many commands (ess/12).
    ///
    /// Above the domains, like `bindings:`, so any file may carry them. Expanded into their member
    /// commands before anything is converted, and never seen by the rest of assembly.
    #[serde(default)]
    pub outcome_groups: Vec<crate::outcome_group::RawOutcomeGroup>,
    /// How this file wrote its command guards and entity invariants, read beside the typed fields
    /// by [`Self::parse`] (`docs/design/expression-family-source22.md`, A1). Never part of the
    /// document, its schema or its bytes.
    #[serde(skip)]
    #[schemars(skip)]
    pub(crate) spelled: crate::expression::lexical::Written,
}

/// Everything a specification declares, indexed by identity.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Specification {
    /// The system and its domains.
    system: SystemSpec,
    /// Every entity.
    entities: BTreeMap<QualifiedName, EntitySpec>,
    /// Every command.
    commands: BTreeMap<QualifiedName, CommandSpec>,
    /// Every event.
    events: BTreeMap<QualifiedName, EventSpec>,
    /// Every error a command may name.
    errors: BTreeMap<QualifiedName, ErrorSpec>,
    /// Every view.
    views: BTreeMap<QualifiedName, ViewSpec>,
    /// Every actor.
    actors: BTreeMap<QualifiedName, ActorSpec>,
    /// Every component.
    components: BTreeMap<crate::component::ComponentName, crate::component::ComponentSpec>,
    /// Every binding.
    bindings: BTreeMap<crate::binding::BindingName, crate::binding::BindingSpec>,
    /// The system's runtime shape, empty when nothing is stated.
    topology: crate::topology::Topology,
    /// Which type crossings are permitted, and why.
    conversions: crate::types::ConversionRegistry,
}

impl RawSpecFile {
    /// Reads one file's text.
    ///
    /// Two stages, and the first one is the point: deserialising straight into this type lets
    /// `serde_yaml` **silently keep the last of two identical mapping keys**, so a document
    /// declaring `a-service` twice loses the first entry and nothing says so. Going through
    /// [`serde_yaml::Value`] first refuses it, with the key and the line.
    ///
    /// That applies to every mapping in the format, not just one section — which is why the check
    /// lives here rather than in each module that happens to hold a map.
    ///
    /// A newtype this file declares may key a map it writes (beyond10x/ess#143), together with any
    /// the caller already put in view — which is how [`Self::parse_all`] makes a sibling file's
    /// declarations count.
    ///
    /// It also keeps how each command guard and entity invariant was written, which a source from
    /// `ess/22` reads its bare words by (`docs/design/expression-family-source22.md`, A1). A file
    /// deserialised directly rather than through here keeps every bare word the text it is
    /// spelled as, which is what every earlier format means by it.
    pub fn parse(text: &str) -> Result<Self, serde_yaml::Error> {
        let document: serde_yaml::Value = serde_yaml::from_str(text)?;
        let spelled = crate::expression::lexical::Written::from_document(&document);
        // A file that names its format reads the operand grammar of that format; one that does
        // not reads what its caller set, `ess/22`'s when nobody did (see `parse_all`).
        let format = written_format(&document);
        let read = move || {
            crate::types::MapKeyNewtypes::current()
                .with_value(&document)
                .scope(|| serde_yaml::from_value(document))
        };
        let mut file: Self = match format {
            Some(format) => reading_format(format, read),
            None => read(),
        }?;
        file.spelled = spelled;
        Ok(file)
    }

    /// Reads every file of one specification, each result in the order the texts were given.
    ///
    /// The same as [`Self::parse`] on each, except that a newtype declared in any of the files may
    /// key a map in any other: validity must not depend on which file declares what. A file that
    /// names no format reads the operand grammar of the one its system header names.
    pub fn parse_all(texts: &[&str]) -> Vec<Result<Self, serde_yaml::Error>> {
        let headers: Vec<Option<FormatVersion>> = texts
            .iter()
            .filter_map(|text| serde_yaml::from_str::<serde_yaml::Value>(text).ok())
            .filter(|document| document.get("system").is_some())
            .map(|document| written_format(&document))
            .collect();
        let read = || {
            crate::types::MapKeyNewtypes::from_documents(texts.iter().copied())
                .scope(|| texts.iter().map(|text| Self::parse(text)).collect())
        };
        match headers.as_slice() {
            [header] => reading_format(header.unwrap_or(FormatVersion::V1), read),
            _ => read(),
        }
    }
}

/// Runs `read` with the grammar `format` admits where the reader cannot otherwise know it: the
/// `ess/22` operands, and filtered reads (`ess/22`, beyond10x/ess#299).
fn reading_format<T>(format: FormatVersion, read: impl FnOnce() -> T) -> T {
    let source22 = format.major() >= FormatVersion::V22.major();
    ess_primitives::predicate::reading_source22_operands(source22, || {
        crate::command::row_set::reading_filtered_reads(source22, read)
    })
}

/// The format a document names in its `format:` key, when it names one this build can read.
fn written_format(document: &serde_yaml::Value) -> Option<FormatVersion> {
    document
        .get("format")
        .and_then(serde_yaml::Value::as_str)
        .and_then(|format| FormatVersion::parse(format).ok())
}

impl Specification {
    /// The system and its domains.
    pub fn system(&self) -> &SystemSpec {
        &self.system
    }

    /// Every entity, indexed by identity.
    pub fn entities(&self) -> &BTreeMap<QualifiedName, EntitySpec> {
        &self.entities
    }

    /// Every command, indexed by identity.
    pub fn commands(&self) -> &BTreeMap<QualifiedName, CommandSpec> {
        &self.commands
    }

    /// Every event, indexed by identity.
    pub fn events(&self) -> &BTreeMap<QualifiedName, EventSpec> {
        &self.events
    }

    /// Every declared error, indexed by identity.
    pub fn errors(&self) -> &BTreeMap<QualifiedName, ErrorSpec> {
        &self.errors
    }

    /// Every view, indexed by identity.
    pub fn views(&self) -> &BTreeMap<QualifiedName, ViewSpec> {
        &self.views
    }

    /// Every actor, indexed by identity.
    pub fn actors(&self) -> &BTreeMap<QualifiedName, ActorSpec> {
        &self.actors
    }

    /// Every component, indexed by identity.
    pub fn components(
        &self,
    ) -> &BTreeMap<crate::component::ComponentName, crate::component::ComponentSpec> {
        &self.components
    }

    /// Every binding, indexed by identity.
    pub fn bindings(&self) -> &BTreeMap<crate::binding::BindingName, crate::binding::BindingSpec> {
        &self.bindings
    }

    /// The system's runtime shape.
    pub fn topology(&self) -> &crate::topology::Topology {
        &self.topology
    }

    /// The permitted type crossings.
    pub fn conversions(&self) -> &crate::types::ConversionRegistry {
        &self.conversions
    }

    /// Assembles a specification from parsed files, then checks every reference in it.
    ///
    /// Both passes accumulate: a specification with a missing type, an unknown event and a view over
    /// an entity nobody declared reports all three.
    pub fn assemble(
        files: impl IntoIterator<Item = (Source, RawSpecFile)>,
    ) -> Result<Self, ValidationErrors> {
        let mut errors = ValidationErrors::new();
        let mut parts: Vec<SpecPart> = Vec::new();
        let mut collected = Collected::default();

        // Before any command is converted, so an expanded outcome meets every check a hand-written
        // copy would (docs/design/outcome-groups.md, "Where before validation is in this tree").
        let mut files: Vec<(Source, RawSpecFile)> = files.into_iter().collect();
        crate::outcome_group::expand(&mut files, &mut errors);
        crate::command::set_effects::refuse_below_ess_16(
            &mut files,
            &mut errors,
            &mut collected.refused.moves,
        );
        crate::command::set_effects::refuse_affect_moves(
            &mut files,
            &mut errors,
            &mut collected.refused.moves,
        );
        crate::command::set_effects::refuse_deletions(&mut files, &mut errors);

        // A guard reads `input.<field>` only in an `ess/22` source, named by its one header.
        let headers: Vec<Option<FormatVersion>> = files
            .iter()
            .filter(|(_, file)| file.system.is_some())
            .map(|(_, file)| file.format)
            .collect();
        let reads_input_namespace = matches!(
            headers.as_slice(),
            [Some(format)] if format.major() >= FormatVersion::V22.major()
        );
        crate::command::converting_input_namespace(reads_input_namespace, || {
            for (source, file) in files {
                parts.push(collected.absorb(&source, file, &mut errors));
            }
        });

        let lifecycle_types: Vec<NamedType> = collected
            .entities
            .values()
            .map(EntitySpec::state_type)
            .collect();
        let (system, merge_errors) =
            SystemSpec::merge_reporting_with_reference_types(parts, &lifecycle_types);
        errors.extend(merge_errors);
        // A merge that failed still hands back the graph, so an unsupported format no longer hides
        // every broken reference behind it. The one thing that cannot be worked around is a missing
        // system header: without a namespace, a domain list and a type registry there is nothing for
        // the member passes to resolve against, and each of them would report its own cascade.
        let Some(system) = system else {
            return Err(errors);
        };

        // A filtered read is an `ess/22` source; below it the same mapping is the nested one it was.
        crate::command::row_set::read_below_ess_22(
            system.format,
            &mut collected.commands,
            &mut errors,
        );
        // `{related: …}` is an `ess/16` source; below it the same mapping is the nested one it was.
        crate::command::related_value::read_below_ess_16(system.format, &mut collected.commands);
        crate::command::caller_value::read_below_ess_16(system.format, &mut collected.commands);
        crate::command::set_effects::read_below_ess_16(system.format, &mut collected.commands);
        let mut specification = Self {
            system,
            entities: collected.entities,
            commands: collected.commands,
            events: collected.events,
            errors: collected.errors,
            views: collected.views,
            actors: collected.actors,
            components: collected.components,
            bindings: collected.bindings,
            topology: collected.topology,
            conversions: collected.conversions,
        };

        // From `ess/22` a bare word on the right of a comparison may name a field; which one is
        // decided now that the format and every declaration are known (A1).
        specification.resolve_written(&collected.spelled);

        errors.extend(specification.validate_after(&collected.refused));
        errors.extend(specification.validate_roster(&collected.roster));
        errors.into_result(specification)
    }

    /// Replaces every authored predicate an `ess/22` source wrote with what its bare words name
    /// there (`docs/design/expression-family-source22.md`, A1). Nothing moves below `ess/22`.
    fn resolve_written(&mut self, written: &crate::expression::lexical::Written) {
        let registry = self.types_with_lifecycles(&mut ValidationErrors::new());
        let resolved = crate::expression::lexical::resolutions(self, &registry, written);
        for (site, predicate) in resolved {
            if let crate::expression::lexical::Site::TypeInvariant { name, index } = &site {
                // A declared type is held twice — in the registry and in its domain's list — and
                // both copies say the same thing.
                for domain in &mut self.system.domains {
                    for declared in domain
                        .types
                        .iter_mut()
                        .filter(|declared| &declared.name == name)
                    {
                        if let crate::types::TypeBody::Struct { invariants, .. }
                        | crate::types::TypeBody::Newtype { invariants, .. } = &mut declared.body
                        {
                            if let Some(invariant) = invariants.get_mut(*index) {
                                invariant.predicate = predicate.clone();
                            }
                        }
                    }
                }
            }
            if let Some(slot) = crate::expression::lexical::slot(
                &site,
                &mut self.commands,
                &mut self.entities,
                &mut self.views,
                &mut self.system.types,
            ) {
                *slot = predicate;
            }
        }
    }

    /// Checks every reference in the specification.
    pub fn validate(&self) -> ValidationErrors {
        self.validate_after(&Refused::default())
    }

    /// Checks every reference, treating the names `refused` holds as declared: their own
    /// conversion already said what is wrong with them, and a reference to one is not a second
    /// fault.
    fn validate_after(&self, refused: &Refused) -> ValidationErrors {
        let mut errors = crate::primitive_admission::specification(self);
        errors.extend(crate::wire::validate(self));
        errors.extend(crate::binding::periodic::validate_specification(self));
        errors.extend(crate::binding::context::validate_specification(self));
        errors.extend(crate::binding::retry::validate_specification(self));
        errors.extend(crate::binding::condition::validate_specification(self));
        errors.extend(crate::binding::refusal::validate_specification(self));

        // Entities contribute the enum their lifecycle forms, so a view projecting `state` and a
        // filter comparing it are checked against the same set of names.
        let registry = self.types_with_lifecycles(&mut errors);

        errors.extend(registry.validate_invariants());

        for entity in self.entities.values() {
            errors.extend(entity.validate(&registry));
        }

        let event_names: BTreeSet<QualifiedName> =
            Refused::with(&self.events.keys().cloned().collect(), &refused.events);
        let error_names: BTreeSet<QualifiedName> =
            Refused::with(&self.errors.keys().cloned().collect(), &refused.errors);
        for command in self.commands.values() {
            if let Err(command_errors) = command.validate(&registry, &event_names, &error_names) {
                errors.extend(command_errors);
            }
        }
        errors.extend(crate::command::subject_state::validate(self, &registry));
        errors.extend(crate::command::subject_fact::validate(self, &registry));
        errors.extend(crate::command::related_guard::validate(self, &registry));
        errors.extend(crate::command::row_set::validate(self, &registry));
        errors.extend(crate::expression::lexical::identity_orderings(
            self, &registry,
        ));
        for event in self.events.values() {
            if let Err(event_errors) = event.validate(&registry) {
                errors.extend(event_errors);
            }
        }
        // A declared error carries a payload like anything else, and its field types are references
        // that have to resolve. Nothing else in the pipeline resolves them.
        for declared in self.errors.values() {
            if let Err(error_errors) = declared.validate(&registry) {
                errors.extend(error_errors);
            }
        }

        // The link between a command's outcomes and the lifecycles they drive, checked once and in
        // both directions: neither an entity nor a command can see it alone, because it is a
        // relation between them.
        errors.extend(crate::entity::validate_lifecycle_causes_after(
            &self.entities,
            &self.commands,
            &self.events,
            refused,
        ));

        // The other cross-entity rule, here for the same reason: a relation is declared on one
        // entity, carried by a field that may be on another, and the rule that makes `owns` mean
        // anything — one owner per owned entity — is a statement about all of them at once.
        errors.extend(crate::entity::validate_relations(&self.entities));

        let catalogue = EntityCatalogue::new(self.entities.values());
        for view in self.views.values() {
            if let Err(view_errors) = view.validate(&registry, &catalogue) {
                errors.extend(view_errors);
            }
        }

        let command_names: BTreeSet<QualifiedName> =
            Refused::with(&self.commands.keys().cloned().collect(), &refused.commands);
        // A grant may name a view from ess/22 (beyond10x/ess#286).
        let view_names: BTreeSet<QualifiedName> =
            Refused::with(&self.views.keys().cloned().collect(), &refused.views);
        for actor in self.actors.values() {
            errors.extend(actor.validate_grants(&command_names, &view_names, self.system.format));
        }

        errors.extend(self.validate_components(&command_names, &event_names, refused));

        // The payload construct's cross-declaration half, beside the binding's for the reason the
        // two mirror each other: an outcome fills an event's fields from its command's input, and
        // neither declaration can check the pair alone.
        errors.extend(crate::command::validate_payloads(
            &self.commands,
            &self.events,
            &registry,
            &self.conversions,
        ));

        // The other half of the same construct: what an outcome sets on the entity it acts on.
        errors.extend(crate::command::validate_sets(
            &self.commands,
            &self.entities,
            &registry,
            &self.conversions,
        ));

        // And what a creating branch leaves unset, against the invariants that read it.
        errors.extend(crate::command::validate_created_invariant_fields(
            &self.commands,
            &self.entities,
        ));

        errors.extend(crate::binding::validate_bindings_after(
            &self.bindings,
            &self.events,
            &self.commands,
            &registry,
            &self.conversions,
            refused,
        ));

        let component_names: BTreeSet<crate::component::ComponentName> = Refused::with(
            &self.components.keys().cloned().collect(),
            &refused.components,
        );
        errors.extend(crate::topology::validate_topology(
            &self.topology,
            &component_names,
        ));

        errors.extend(self.validate_ownership());
        errors
    }

    /// The layers above the domains: components, against the names the domains declare.
    fn validate_components(
        &self,
        command_names: &BTreeSet<QualifiedName>,
        event_names: &BTreeSet<QualifiedName>,
        refused: &Refused,
    ) -> ValidationErrors {
        // The three layers above the domains. Each needs the whole specification, because each is
        // about how the parts fit rather than about any one of them.
        let domain_names: BTreeSet<QualifiedName> = self
            .system
            .domains
            .iter()
            .map(|domain| domain.name.clone())
            .collect();
        // The views each domain projects. This was the yes-or-no `projecting` set until a second
        // question was asked of it: a network surface needs to know *whether* a domain projects a
        // row, and a command tree needs to know *which*, so that a view served with no verb is a
        // refusal rather than something somebody notices later.
        let views: BTreeMap<QualifiedName, BTreeSet<QualifiedName>> = self
            .system
            .domains
            .iter()
            .map(|domain| {
                (
                    domain.name.clone(),
                    domain
                        .views
                        .iter()
                        .filter(|view| self.views.contains_key(*view))
                        .cloned()
                        // A view its own conversion refused is still the one its domain declares:
                        // reading it is not a second fault (beyond10x/ess#448).
                        .chain(
                            refused
                                .view_domains
                                .iter()
                                .filter(|(_, owner)| **owner == domain.name)
                                .map(|(view, _)| view.clone()),
                        )
                        .collect(),
                )
            })
            .collect();
        crate::component::validate_components(
            &self.components,
            &domain_names,
            command_names,
            event_names,
            &views,
        )
    }

    /// Checks that every member belongs to a declared domain, and to the right one.
    fn validate_ownership(&self) -> ValidationErrors {
        let mut errors = ValidationErrors::new();
        let members = self
            .entities
            .keys()
            .map(|name| ("entity", name))
            .chain(self.commands.keys().map(|name| ("command", name)))
            .chain(self.events.keys().map(|name| ("event", name)))
            .chain(self.views.keys().map(|name| ("view", name)))
            .chain(self.errors.keys().map(|name| ("error", name)))
            .chain(self.actors.keys().map(|name| ("actor", name)));

        for (kind, name) in members {
            if self.system.owner_of(name).is_none() {
                errors.push(
                    ValidationError::new(
                        ValidationCode::UndeclaredReference,
                        format!("{kind} {name}"),
                        format!("`{name}` is inside no declared domain"),
                    )
                    .with_hint(if self.system.domains.is_empty() {
                        self.declared_domains()
                    } else {
                        format!("declared domains: {}", self.declared_domains())
                    }),
                );
            }
        }
        errors
    }

    /// Build the registry shared by every member validation, retaining duplicate-type errors.
    pub(crate) fn types_with_lifecycles(
        &self,
        errors: &mut ValidationErrors,
    ) -> crate::types::TypeRegistry {
        let mut registry = self.system.types.clone().with_format(self.system.format);
        for entity in self.entities.values() {
            if let Err(error) = registry.insert(entity.state_type()) {
                errors.push(error);
            }
        }
        registry
    }

    /// Checks the header's domain roster against the domains the sources contribute.
    ///
    /// The roster is **authoritative**, not advisory: the schema and the guide both describe it as
    /// "which domains it has", so it is read as a statement about the system rather than as a
    /// comment on it, and it is checked in both directions — a listed domain nobody declares is a
    /// typo or a file that never got written, and a declared domain the header omits is a file
    /// nobody meant to include.
    ///
    /// An absent roster states nothing and is checked against nothing. A specification is free not
    /// to keep one; it is not free to keep a wrong one.
    fn validate_roster(&self, roster: &[QualifiedName]) -> ValidationErrors {
        let mut errors = ValidationErrors::new();
        if roster.is_empty() {
            return errors;
        }

        let listed: BTreeSet<&QualifiedName> = roster.iter().collect();
        let declared: BTreeSet<&QualifiedName> = self
            .system
            .domains
            .iter()
            .map(|domain| &domain.name)
            .collect();

        for name in listed.difference(&declared) {
            errors.push(
                ValidationError::new(
                    ValidationCode::UndeclaredReference,
                    "system.domains",
                    format!(
                        "`{name}` is listed as a domain of the system, and no source declares it"
                    ),
                )
                .with_hint(if self.system.domains.is_empty() {
                    self.declared_domains()
                } else {
                    format!("declared domains: {}", self.declared_domains())
                }),
            );
        }
        for name in declared.difference(&listed) {
            errors.push(
                ValidationError::new(
                    ValidationCode::ConflictingDeclaration,
                    format!("domain {name}"),
                    format!("`{name}` is declared, and the system header does not list it"),
                )
                .with_hint(
                    "the header's `domains:` says what the system has; add it there, or drop the \
                     source that declares it",
                ),
            );
        }
        errors
    }

    /// The domains the specification declares, for a diagnostic hint.
    fn declared_domains(&self) -> String {
        // "declared domains: " with nothing after it reads as a truncated message rather than as an
        // answer. Validating a header on its own is the common way to reach this, and there the
        // empty list is the whole explanation.
        if self.system.domains.is_empty() {
            return "no source declares any domain — is this the whole specification, or only its \
                    header?"
                .to_owned();
        }
        self.system
            .domains
            .iter()
            .map(|domain| domain.name.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// How many members the specification declares, of every kind.
    pub fn member_count(&self) -> usize {
        self.entities.len()
            + self.commands.len()
            + self.events.len()
            + self.errors.len()
            + self.views.len()
            + self.actors.len()
    }
}

/// Records that a name is declared here, reporting a second declaration of it.
///
/// `true` for the first declaration of a name and `false` for every later one, so that of two
/// declarations that both convert the caller keeps what the first one meant and drops the second —
/// the rule a registry keyed by name has always had, said once instead of once per kind. It is
/// only that, and [`record`] is where the rest of it lives: a first declaration that fails its own
/// conversion never means anything, and the copy that does is what the name means then.
///
/// **Asked before the declaration is converted, and that is the whole point.** Finding the first
/// copy *in the registry* is the same question only for a copy that got there: a declaration whose
/// own `try_from` failed is never recorded, so the second copy of its name found the registry empty
/// and took the name in silence. The author was told about one copy's error and never about the two
/// declarations, and fixing that error made a refusal appear that was true all along. A name is
/// declared where it is written, whether or not what is written under it converts.
///
/// **The only refusal of a same-kind repeat, so it names both files.** [`Collected::write`] keeps
/// the repeat away from `Assembly::claim`, which used to refuse it a second time and was, for two
/// files, the only sentence that named the first one and the other domain. The sentence here is
/// unchanged; the hint names the file that wrote the name first, as any kind, whenever that is
/// another file, and both domains whenever the two copies are filed under different ones. The
/// first writing is `earlier`, from [`Collected::write`]'s record, because this function's own key
/// carries the kind and would name the first file of *this* kind instead.
///
/// Keyed by kind as well as by name, which is the same-kind half of the class. One name held by two
/// *different* kinds — and one type name written twice, since types do not come through here — is
/// a different fault with a different message, and is [`Collected::write`]'s: it asks the same
/// question across kinds, after conversion, of every writing whether it converted or not, and
/// keeps a second copy of a name away from `Assembly::claim` in `system.rs`, so each is refused
/// once. `Assembly::claim` alone read member lists a failed conversion never reaches, so a
/// command carrying a duplicate input plus an event of the same name used to tell the author about
/// the input field and nothing about the name; `tests/masked_declaration_boundaries.rs` pins that
/// in all four soundness combinations.
///
/// Keying by name alone was the other way to close it, and is worse: it would move every same-kind
/// refusal this function already makes onto a new sentence and location, a diagnostic contract
/// that cases already reported today depend on, to fix cases that are not reported at all.
fn declare(
    declared: &mut BTreeMap<(&'static str, String), Source>,
    kind: &'static str,
    name: &impl std::fmt::Display,
    source: &Source,
    (earlier, owner): (Option<&Claim>, Option<&QualifiedName>),
    errors: &mut ValidationErrors,
) -> bool {
    let first = match declared.entry((kind, name.to_string())) {
        std::collections::btree_map::Entry::Vacant(entry) => {
            entry.insert(source.clone());
            return true;
        }
        std::collections::btree_map::Entry::Occupied(entry) => entry.get().clone(),
    };
    let first = earlier.map_or(&first, Claim::source);
    let mut hint = Vec::new();
    if first != source {
        hint.push(format!("{first} declares it first"));
    }
    if let (Some(there), Some(here)) = (earlier.and_then(Claim::owner), owner) {
        if there != here {
            hint.push(format!("claimed by `{there}` and by `{here}`"));
        }
    }
    hint.push("a name identifies one thing; two declarations cannot both be it".to_owned());
    let hint = hint.join("; ");
    errors.push(
        ValidationError::new(
            ValidationCode::DuplicateDeclaration,
            format!("{kind} {name}"),
            format!("`{name}` is declared more than once; {source} declares it again"),
        )
        .with_hint(hint),
    );
    false
}

/// Records what a name means, for the one declaration that owns it.
///
/// A later copy of a duplicated name is converted like any other — its own errors are the author's
/// to see, and dropping them would trade one silence for another — and then it is dropped, because
/// the first declaration is what the name means. [`declare`] is what decides which copy that is,
/// and `first` is its answer.
///
/// **Except when the first declaration is broken, where the name would otherwise mean nothing.**
/// A declaration that fails its own `try_from` never reaches here, so answering `first` alone drops
/// the second copy into the same hole the first one fell in: the name is in no registry, and every
/// reference to it is then refused as undeclared *in the same run* that reports it declared more
/// than once — two refusals, one of which is false, where the author wrote one fault. The registry
/// [`declare`] replaced never did that, because the question it was really answering was which copy
/// **converted**, and that is the part of it worth keeping. So a name absent from `into` is
/// recorded whichever copy it came from, and `first` decides only between copies that both
/// converted. Both refusals still fire; the cascade behind them does not.
///
/// Measured for all eight registries at once by this file's
/// `every_masked_rows_name_is_recorded_from_the_copy_that_converts`, and where an author meets it —
/// a view on the entity, an actor's grant on the command — in
/// `tests/masked_declaration_reference_cascade.rs`.
fn record<K: Ord, T>(first: bool, into: &mut BTreeMap<K, T>, name: K, value: T) {
    if first || !into.contains_key(&name) {
        into.insert(name, value);
    }
}

/// What a file contributed to one domain.
#[derive(Debug, Default)]
struct DomainMembers {
    entities: Vec<QualifiedName>,
    commands: Vec<QualifiedName>,
    events: Vec<QualifiedName>,
    errors: Vec<QualifiedName>,
    views: Vec<QualifiedName>,
    actors: Vec<QualifiedName>,
}

impl DomainMembers {
    fn is_empty(&self) -> bool {
        self.entities.is_empty()
            && self.commands.is_empty()
            && self.events.is_empty()
            && self.errors.is_empty()
            && self.views.is_empty()
            && self.actors.is_empty()
    }

    fn into_domain(
        self,
        name: QualifiedName,
        mut naming: Naming,
        summary: Option<String>,
    ) -> DomainSpec {
        // `summary:` at the top of a domain file is the domain's summary; `naming.summary` is the
        // same thing said the long way, so the short spelling wins when both appear rather than one
        // silently shadowing the other.
        if naming.summary.is_none() {
            naming.summary = summary;
        }
        DomainSpec {
            name,
            types: Vec::new(),
            entities: self.entities,
            commands: self.commands,
            events: self.events,
            views: self.views,
            errors: self.errors,
            actors: self.actors,
            naming,
        }
    }
}

/// The members gathered from every file so far.
///
/// Members are keyed by qualified name across the whole specification rather than per file: two
/// files both declaring `billing.invoice.CreateInvoice` is a collision whichever files they are, and
/// the error has to name both sources to be actionable.
#[derive(Default)]
struct Collected {
    entities: BTreeMap<QualifiedName, EntitySpec>,
    commands: BTreeMap<QualifiedName, CommandSpec>,
    events: BTreeMap<QualifiedName, EventSpec>,
    errors: BTreeMap<QualifiedName, ErrorSpec>,
    views: BTreeMap<QualifiedName, ViewSpec>,
    actors: BTreeMap<QualifiedName, ActorSpec>,
    components: BTreeMap<crate::component::ComponentName, crate::component::ComponentSpec>,
    conversions: crate::types::ConversionRegistry,
    bindings: BTreeMap<crate::binding::BindingName, crate::binding::BindingSpec>,
    /// The runtime shape, from whichever file carries it.
    topology: crate::topology::Topology,
    /// Which file carried it, so a second one can be named in the refusal.
    topology_source: Option<Source>,
    /// The domains the system header says it has, from whichever file carries the header.
    roster: Vec<QualifiedName>,
    /// Every name declared so far in the kinds that go through [`declare`], whether or not the
    /// declaration converted.
    ///
    /// Eight kinds, not every kind: a declared type, a conversion and the topology are absorbed
    /// elsewhere and are not in here. A type written twice is refused through `written` instead;
    /// a conversion and the topology cannot be masked.
    ///
    /// Held beside the registries rather than read off them, because a registry holds what was
    /// *recorded* and this has to answer what was *declared*. See [`declare`] for the difference
    /// and for what the registries alone could not see.
    declared: BTreeMap<(&'static str, String), Source>,
    /// Every type and domain member name written so far, whether or not its declaration
    /// converted: the cross-kind half of what `declared` is for. See [`Collected::write`].
    written: BTreeMap<QualifiedName, Written>,
    /// The declarations whose own conversion failed, so the reference pass can tell a name that
    /// was declared and refused from a name nobody declared. See [`Refused`].
    refused: Refused,
    /// How each authored predicate was written, until the header's format says what its bare
    /// words name (`docs/design/expression-family-source22.md`, A1).
    spelled: crate::expression::lexical::Written,
}

/// What has been written under one name, converted or not.
#[derive(Debug)]
struct Written {
    /// The first writing of the name, which a later one is refused against.
    first: Claim,
    /// Every kind the name has been written as.
    kinds: BTreeSet<MemberKind>,
    /// Whether a copy of the name has been kept for `Assembly::claim`: converted, and a type or a
    /// member of a domain.
    claimed: bool,
    /// The domain the claimed copy was filed under, which `DomainSpec::validate` checks it against.
    claimed_under: Option<QualifiedName>,
}

/// Declarations refused by their own conversion: declared, and absent from every registry.
///
/// A reference to one of these is not refused again as a reference to nothing. The declaration's
/// own refusal is the cause and is already reported; a second refusal saying the name does not
/// exist is false — the author declared it — and reads as a separate fault. So the reference pass
/// treats these names as declared (beyond10x/ess#79). A reference that names nothing at all is
/// still refused, and once the declaration is fixed every reference to it is checked as usual.
#[derive(Debug, Default)]
pub(crate) struct Refused {
    pub(crate) entities: BTreeSet<QualifiedName>,
    pub(crate) commands: BTreeSet<QualifiedName>,
    pub(crate) events: BTreeSet<QualifiedName>,
    pub(crate) errors: BTreeSet<QualifiedName>,
    /// Views whose conversion failed, so a grant naming one (ess/22, beyond10x/ess#286) is not
    /// reported as naming nothing.
    pub(crate) views: BTreeSet<QualifiedName>,
    /// The domain each refused view was declared in, so a command tree reading one is not
    /// reported as reading a view its domain does not project (beyond10x/ess#448).
    pub(crate) view_domains: BTreeMap<QualifiedName, QualifiedName>,
    pub(crate) components: BTreeSet<crate::component::ComponentName>,
    /// The `moves:` a refused command's outcomes name, so a transition only that command takes is
    /// not reported as one nothing takes.
    pub(crate) moves: BTreeSet<QualifiedName>,
}

impl Refused {
    /// `declared` with the refused names of the same kind added.
    fn with<T: Ord + Clone>(declared: &BTreeSet<T>, refused: &BTreeSet<T>) -> BTreeSet<T> {
        declared.union(refused).cloned().collect()
    }
}

impl Collected {
    /// Reads the system header, if this is the file that carries one.
    ///
    /// A file that sets a system-level key without naming a system is refused rather than having
    /// the key dropped: a typo in `system:` would otherwise cost the header silently, and the
    /// format a document declares is exactly the thing that must not be guessed at. `summary:` is
    /// not one of those keys — in this document shape it is the *domain's* summary whenever the
    /// file carries `domain:`. This is the same rule `RawSystemSpec::split` applies to the other
    /// document shape, which had it while this one did not.
    fn absorb_header(
        &mut self,
        source: &Source,
        file: &RawSpecFile,
        errors: &mut ValidationErrors,
    ) -> Option<crate::system::SpecHeader> {
        let Some(name) = file.system.clone() else {
            if file.format.is_some()
                || file.version.is_some()
                || !file.domains.is_empty()
                || !file.preconditions.is_empty()
            {
                errors.push(
                    ValidationError::new(
                        ValidationCode::MissingDeclaration,
                        format!("{source}.system"),
                        "this source sets system-level fields but does not declare `system:`",
                    )
                    .with_hint(
                        "the source that carries the format, the version and the domain list is \
                         the one that names the system",
                    ),
                );
            }
            return None;
        };

        self.roster.extend(file.domains.iter().cloned());
        Some(crate::system::SpecHeader {
            name,
            version: file.version.unwrap_or(Version::V1),
            format: file.format.unwrap_or(FormatVersion::V1),
            naming: Naming::default(),
            summary: file.summary.clone(),
            preconditions: file.preconditions.clone(),
        })
    }

    /// Records that `name` is written as `claim`'s kind, refusing it when it is written already and
    /// no other reporter will say so, and answers whether a converted copy is kept as a member.
    ///
    /// **Called after the declaration is converted, and asked of every writing, converted or not.**
    /// One name held by two kinds, or one type name written twice, was refused only by
    /// `Assembly::claim` in `system.rs` — but the assembly sees only declarations that converted
    /// and belong to a domain, so a pair with one broken copy, or one copy in a file with no
    /// `domain:`, reached it as one claim and the other copy took the name in silence. This is the
    /// question [`declare`] asks for one kind, asked across kinds.
    ///
    /// Every later writing of a name is refused exactly once:
    ///
    /// * the same member kind again is [`declare`]'s, which has already reported it;
    /// * everything else — another kind, or a type again, since types do not go through
    ///   [`declare`] — is refused here, whether or not either copy converted.
    ///
    /// **`Assembly::claim` is kept out of it by what this answers.** A converted copy is kept as a
    /// member only while no earlier copy of the name has reached the assembly, so the assembly
    /// never receives a name twice from a specification and never reports it a second time; it
    /// still receives the one copy that owns the name, which is the copy it would have kept. A
    /// member of a file with no `domain:` never reaches the assembly, so it is always kept — the
    /// file is refused for having no domain by the members it holds — and never counts as the
    /// copy that does. The refusal here is [`Claim::refuse`]'s sentence, the one the assembly
    /// writes for the same pair, and it is written against the first copy in the order the
    /// assembly claims them, so which copy converted does not change it.
    ///
    /// A copy kept out of its domain is also kept out of `DomainSpec::validate`, which is what
    /// refuses a name its domain cannot hold. So a dropped copy is asked that here, through the same
    /// [`DomainSpec::misplaced`], unless it is filed under the domain the kept copy was — there
    /// `validate` already asks it of the kept one.
    fn write(
        &mut self,
        name: &QualifiedName,
        claim: Claim,
        converted: bool,
        errors: &mut ValidationErrors,
    ) -> bool {
        let kind = claim.kind();
        let reaches = kind == MemberKind::Type || claim.owner().is_some();
        let Some(written) = self.written.get_mut(name) else {
            let claimed_under = claim.owner().filter(|_| converted).cloned();
            self.written.insert(
                name.clone(),
                Written {
                    first: claim,
                    kinds: BTreeSet::from([kind]),
                    claimed: converted && reaches,
                    claimed_under,
                },
            );
            return converted;
        };
        let declared_already = kind != MemberKind::Type && written.kinds.contains(&kind);
        if !declared_already {
            errors.push(written.first.refuse(name, &claim));
        }
        written.kinds.insert(kind);
        let keep = converted && !(reaches && written.claimed);
        written.claimed |= keep && reaches;
        if keep && reaches {
            written.claimed_under = claim.owner().cloned();
        }
        // A copy kept out of its domain is not seen by `DomainSpec::validate`, so it is asked
        // here whether that domain could hold it at all.
        if converted && !keep {
            if let Some(owner) = claim
                .owner()
                .filter(|&owner| written.claimed_under.as_ref() != Some(owner))
            {
                if let Some(misplaced) = DomainSpec::misplaced(owner, kind, name) {
                    errors.push(misplaced);
                }
            }
        }
        keep
    }

    /// [`declare`] for a domain member, told the first writing of its name of any kind.
    ///
    /// `declare` is keyed by kind, so on its own it knows only the first copy of *this* kind; the
    /// file that wrote the name first, and the domain it filed it under, are
    /// [`Collected::write`]'s. `owner` is the domain this file files the member under.
    fn declare_member(
        &mut self,
        kind: &'static str,
        name: &QualifiedName,
        source: &Source,
        owner: Option<&QualifiedName>,
        errors: &mut ValidationErrors,
    ) -> bool {
        let earlier = self.written.get(name).map(|written| &written.first);
        declare(
            &mut self.declared,
            kind,
            name,
            source,
            (earlier, owner),
            errors,
        )
    }

    /// Takes one file's members, and returns what that file contributes to the system.
    ///
    /// A member whose own conversion failed is not recorded. Its name goes into [`Refused`]
    /// instead, so the later reference pass neither resolves it nor reports a reference to it as
    /// a reference to nothing.
    ///
    /// Kinds are taken in the order `Assembly::absorb_domain` claims them — types, entities,
    /// commands, events, errors, views, actors — so that [`Collected::write`] names the same first
    /// copy the assembly would.
    fn absorb(
        &mut self,
        source: &Source,
        file: RawSpecFile,
        errors: &mut ValidationErrors,
    ) -> SpecPart {
        let mut part = SpecPart::new(source.clone());
        part.header = self.absorb_header(source, &file, errors);
        self.spelled.absorb(file.spelled.clone());
        // The owner every member of this file is claimed for, as `Assembly::claim` will claim it.
        let owner = file.domain.clone();
        let owner = owner.as_ref();
        for mut raw in file.types {
            // An invariant that does not parse, or a boolean written as a variant, is refused and
            // withheld, not the type: a refused type would make every field declared with it a
            // second refusal (beyond10x/ess#448, beyond10x/ess#426).
            errors.extend(raw.withhold_unread());
            let name = raw.name.clone();
            let converted = match NamedType::try_from(raw) {
                Ok(declared) => Some(declared),
                Err(type_errors) => {
                    errors.extend(type_errors);
                    None
                }
            };
            let claim = Claim::new(source, None, MemberKind::Type);
            if self.write(&name, claim, converted.is_some(), errors) {
                part.types.extend(converted);
            }
        }

        let mut members = DomainMembers::default();

        for mut raw in file.entities {
            // As for a type: the invariant is withheld, not the entity (beyond10x/ess#448).
            errors.extend(raw.withhold_unparsed_invariants());
            let first = self.declare_member("entity", &raw.name, source, owner, errors);
            let name = raw.name.clone();
            let converted = match EntitySpec::try_from(raw) {
                Ok(entity) => {
                    let converted = entity.name.clone();
                    record(first, &mut self.entities, entity.name.clone(), entity);
                    Some(converted)
                }
                Err(member_errors) => {
                    self.refused.entities.insert(name.clone());
                    errors.extend(member_errors);
                    None
                }
            };
            let claim = Claim::new(source, owner, MemberKind::Entity);
            if self.write(&name, claim, converted.is_some(), errors) {
                members.entities.extend(converted);
            }
        }
        self.absorb_commands(source, owner, file.commands, &mut members, errors);
        for raw in file.events {
            let first = self.declare_member("event", &raw.name, source, owner, errors);
            let name = raw.name.clone();
            let converted = match EventSpec::try_from(raw) {
                Ok(event) => {
                    let converted = event.name.clone();
                    record(first, &mut self.events, event.name.clone(), event);
                    Some(converted)
                }
                Err(member_errors) => {
                    self.refused.events.insert(name.clone());
                    errors.extend(member_errors);
                    None
                }
            };
            let claim = Claim::new(source, owner, MemberKind::Event);
            if self.write(&name, claim, converted.is_some(), errors) {
                members.events.extend(converted);
            }
        }
        self.absorb_errors_views_and_actors(
            source,
            owner,
            (file.errors, file.views, file.actors),
            &mut members,
            errors,
        );

        self.absorb_system_level(
            source,
            file.conversions,
            file.components,
            file.bindings,
            file.topology,
            errors,
        );

        if let Some(name) = file.domain {
            part.domains
                .push(members.into_domain(name, file.naming.clone(), file.summary.clone()));
        } else if !members.is_empty() {
            errors.push(
                // Nothing is referenced: a required declaration is absent, not empty.
                ValidationError::new(
                    ValidationCode::MissingDeclaration,
                    source.to_string(),
                    "declares members but no `domain:` they belong to",
                )
                .with_hint(
                    "a member has to belong to a bounded context; add `domain: billing.invoice`",
                ),
            );
        }

        part
    }

    /// One file's errors, views and actors, in that order — the order `Assembly::absorb_domain`
    /// claims them in — each declared, converted and written like any other member.
    fn absorb_errors_views_and_actors(
        &mut self,
        source: &Source,
        owner: Option<&QualifiedName>,
        (declared_errors, views, actors): (
            Vec<crate::command::RawErrorSpec>,
            Vec<crate::view::RawViewSpec>,
            Vec<crate::actor::RawActorSpec>,
        ),
        members: &mut DomainMembers,
        errors: &mut ValidationErrors,
    ) {
        for raw in declared_errors {
            let first = self.declare_member("error", &raw.name, source, owner, errors);
            let name = raw.name.clone();
            let converted = match ErrorSpec::try_from(raw) {
                Ok(error) => {
                    let converted = error.name.clone();
                    record(first, &mut self.errors, error.name.clone(), error);
                    Some(converted)
                }
                Err(member_errors) => {
                    self.refused.errors.insert(name.clone());
                    errors.extend(member_errors);
                    None
                }
            };
            let claim = Claim::new(source, owner, MemberKind::Error);
            if self.write(&name, claim, converted.is_some(), errors) {
                members.errors.extend(converted);
            }
        }
        for raw in views {
            let first = self.declare_member("view", &raw.name, source, owner, errors);
            let name = raw.name.clone();
            let converted = match ViewSpec::try_from(raw) {
                Ok(view) => {
                    let converted = view.name.clone();
                    record(first, &mut self.views, view.name.clone(), view);
                    Some(converted)
                }
                Err(member_errors) => {
                    self.refused.views.insert(name.clone());
                    if let Some(owner) = owner {
                        self.refused
                            .view_domains
                            .insert(name.clone(), owner.clone());
                    }
                    errors.extend(member_errors);
                    None
                }
            };
            let claim = Claim::new(source, owner, MemberKind::View);
            if self.write(&name, claim, converted.is_some(), errors) {
                members.views.extend(converted);
            }
        }
        for raw in actors {
            let first = self.declare_member("actor", &raw.name, source, owner, errors);
            let name = raw.name.clone();
            let converted = match ActorSpec::try_from(raw) {
                Ok(actor) => {
                    let converted = actor.name.clone();
                    record(first, &mut self.actors, actor.name.clone(), actor);
                    Some(converted)
                }
                Err(member_errors) => {
                    errors.extend(member_errors);
                    None
                }
            };
            let claim = Claim::new(source, owner, MemberKind::Actor);
            if self.write(&name, claim, converted.is_some(), errors) {
                members.actors.extend(converted);
            }
        }
    }

    /// One file's commands. A refused one keeps its name and the transitions its outcomes
    /// `moves:`, so [`Refused`] can tell what it would have referred to.
    fn absorb_commands(
        &mut self,
        source: &Source,
        owner: Option<&QualifiedName>,
        commands: Vec<crate::command::RawCommandSpec>,
        members: &mut DomainMembers,
        errors: &mut ValidationErrors,
    ) {
        for raw in commands {
            let first = self.declare_member("command", &raw.name, source, owner, errors);
            let name = raw.name.clone();
            let moves: Vec<QualifiedName> = raw
                .outcomes
                .iter()
                .filter_map(|outcome| outcome.moves.clone())
                .collect();
            let converted = match CommandSpec::try_from(raw) {
                Ok(command) => {
                    let converted = command.name.clone();
                    record(first, &mut self.commands, command.name.clone(), command);
                    Some(converted)
                }
                Err(member_errors) => {
                    self.refused.commands.insert(name.clone());
                    self.refused.moves.extend(moves);
                    errors.extend(member_errors);
                    None
                }
            };
            let claim = Claim::new(source, owner, MemberKind::Command);
            if self.write(&name, claim, converted.is_some(), errors) {
                members.commands.extend(converted);
            }
        }
    }

    /// Takes the declarations that sit above the domains rather than inside one.
    ///
    /// A component owns domains and a binding joins two of them, so neither can belong to
    /// either — which is why these never reach `DomainMembers` and so never make a file that
    /// holds only components look like a file of orphaned members.
    fn absorb_system_level(
        &mut self,
        source: &Source,
        conversions: Vec<crate::types::Conversion>,
        components: Vec<crate::component::RawComponentSpec>,
        bindings: Vec<crate::binding::RawBindingSpec>,
        topology: Option<crate::topology::RawTopology>,
        errors: &mut ValidationErrors,
    ) {
        for conversion in conversions {
            if let Err(error) = self.conversions.insert(conversion) {
                errors.push(error);
            }
        }

        // Components, bindings and topology sit above the domains rather than inside one: a
        // component owns domains, and a binding joins two of them, so neither can belong to either.
        for raw in components {
            let first = declare(
                &mut self.declared,
                "component",
                &raw.name,
                source,
                (None, None),
                errors,
            );
            let name = raw.name.clone();
            match crate::component::ComponentSpec::try_from(raw) {
                Ok(component) => {
                    record(
                        first,
                        &mut self.components,
                        component.name.clone(),
                        component,
                    );
                }
                Err(member_errors) => {
                    // A name that is not a component name is refused on its own account and
                    // cannot be referred to either, so there is nothing to hold.
                    if let Ok(name) = crate::component::ComponentName::new(name) {
                        self.refused.components.insert(name);
                    }
                    errors.extend(member_errors);
                }
            }
        }
        for raw in bindings {
            let first = declare(
                &mut self.declared,
                "binding",
                &raw.name,
                source,
                (None, None),
                errors,
            );
            match crate::binding::BindingSpec::try_from(raw) {
                Ok(binding) => {
                    record(first, &mut self.bindings, binding.name.clone(), binding);
                }
                Err(member_errors) => errors.extend(member_errors),
            }
        }
        if let Some(raw) = topology {
            // One topology per system. Merging two would mean silently choosing which replica floor
            // wins, and a floor chosen by file order is not a decision anyone made.
            if let Some(first) = &self.topology_source {
                errors.push(
                    ValidationError::new(
                        ValidationCode::DuplicateDeclaration,
                        format!("{source}.topology"),
                        format!("the topology is already declared in {first}"),
                    )
                    .with_hint("a system has one runtime shape; put every workload in one file"),
                );
            } else {
                self.topology_source = Some(source.clone());
                match crate::topology::Topology::try_from(raw) {
                    Ok(topology) => self.topology = topology,
                    Err(topology_errors) => errors.extend(topology_errors),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ITEM_ID: &str = "types:\n  - {name: demo.orders.ItemId, kind: newtype, of: String}\n";
    const CHECKED: &str = "events:\n  - name: demo.orders.Checked\n    fields:\n      - {name: \
                           results, type: \"Map<demo.orders.ItemId, Boolean>\"}\n";

    #[test]
    fn a_file_may_key_a_map_by_a_newtype_it_declares() {
        RawSpecFile::parse(&format!("domain: demo.orders\n{ITEM_ID}{CHECKED}"))
            .expect("beyond10x/ess#143: a newtype of String keys a map");
        RawSpecFile::parse(&format!("domain: demo.orders\n{CHECKED}"))
            .expect_err("read alone, a file cannot see a sibling's newtype");
    }

    #[test]
    fn files_read_together_see_each_others_key_newtypes_in_any_order() {
        let types = format!("domain: demo.orders\n{ITEM_ID}");
        let events = format!("domain: demo.orders\n{CHECKED}");
        for texts in [
            [types.as_str(), events.as_str()],
            [events.as_str(), types.as_str()],
        ] {
            for parsed in RawSpecFile::parse_all(&texts) {
                parsed.expect("the declaring file's position does not matter");
            }
        }
    }

    #[test]
    fn revalidation_rejects_an_expression_mutated_inside_the_sealed_module() {
        let text = "format: ess/1\nsystem: sample\nversion: v1\ndomain: sample.data\ncommands:\n  - name: sample.data.Update\n    input:\n      - {name: amount, type: Decimal}\n    outcomes:\n      - name: changed\n        when: amount > 0\n        emits: [sample.data.Changed]\n      - name: refused\n        error: sample.data.Refused\nevents:\n  - name: sample.data.Changed\nerrors:\n  - name: sample.data.Refused\n";
        let mut spec = Specification::assemble([file("mutation.yaml", text)]).unwrap();
        assert!(spec.validate().is_empty());
        let command = spec.commands.values_mut().next().unwrap();
        command.outcomes[0].condition = crate::command::OutcomeCondition::When(
            ess_primitives::predicate::Predicate::parse_expression("amount.nonexistent > 0")
                .unwrap(),
        );
        let errors = spec.validate();
        assert!(
            errors
                .as_slice()
                .iter()
                .any(|error| error.code == ValidationCode::UnobservableFact
                    && error.location.contains("command.sample.data.Update")
                    && error.message.contains("amount.nonexistent")),
            "{errors}"
        );
    }

    /// Parses one file's worth of YAML.
    fn file(source: &str, yaml: &str) -> (Source, RawSpecFile) {
        (
            Source::new(source),
            serde_yaml::from_str(yaml).expect("well formed"),
        )
    }

    /// The smallest specification that says anything.
    ///
    /// No `domains:` roster: it is a statement the header may make, and the tests that check what
    /// it means make it explicitly rather than having every other test satisfy it in passing.
    fn minimal() -> Vec<(Source, RawSpecFile)> {
        vec![file(
            "system.yaml",
            r"
format: ess/1
system: shop
version: v1
",
        )]
    }

    #[test]
    fn a_specification_may_be_one_file() {
        let mut files = minimal();
        files.push(file(
            "system.yaml",
            r"
domain: shop.cart
entities:
  - name: shop.cart.Cart
    identity:
      name: cart_id
      type: String
    fields:
      - name: total
        type: Decimal
    lifecycle:
      initial: Open
      states: [Open, Closed]
      terminal: [Closed]
      transitions:
        - name: close
          from: [Open]
          to: Closed

commands:
  - name: shop.cart.CloseCart
    input:
      - name: cart_id
        type: String
    outcomes:
      - name: closed
        moves: shop.cart.Cart.close
        instance: cart_id
        emits: [shop.cart.CartClosed]

events:
  - name: shop.cart.CartClosed
    fields: []
",
        ));
        let specification = Specification::assemble(files).expect("valid");
        assert_eq!(specification.entities.len(), 1);
    }

    #[test]
    fn a_reusable_view_shape_may_carry_its_entity_lifecycle_state() {
        let mut files = minimal();
        files.push(file(
            "cart.yaml",
            r"
domain: shop.cart
types:
  - name: shop.cart.CartRow
    kind: struct
    fields:
      - name: cart_id
        type: String
      - name: state
        type: shop.cart.Cart.State
entities:
  - name: shop.cart.Cart
    identity:
      name: cart_id
      type: String
    fields: []
    lifecycle:
      initial: Open
      states: [Open, Closed]
      terminal: [Closed]
      transitions:
        - name: close
          from: [Open]
          to: Closed
commands:
  - name: shop.cart.CloseCart
    input:
      - name: cart_id
        type: String
    outcomes:
      - name: closed
        moves: shop.cart.Cart.close
        instance: cart_id
        emits: [shop.cart.CartClosed]
events:
  - name: shop.cart.CartClosed
    fields: []
views:
  - name: shop.cart.CartById
    source: shop.cart.Cart
    shape: shop.cart.CartRow
    consistency: read_your_writes
",
        ));

        let specification =
            Specification::assemble(files).expect("the derived state type resolves");
        let view = specification
            .views
            .get(&QualifiedName::new("shop.cart.CartById").expect("valid name"))
            .expect("view");
        let fields = view
            .projected_fields(&specification.system.types)
            .expect("the named shape resolves");
        assert_eq!(fields.len(), 2);
        assert_eq!(fields[1].name, "state");
    }

    #[test]
    fn a_member_without_a_domain_is_refused_with_what_to_add() {
        let mut files = minimal();
        files.push(file(
            "orphan.yaml",
            r"
events:
  - name: shop.cart.CartOpened
    fields: []
",
        ));
        let errors = Specification::assemble(files).expect_err("no domain owns it");
        let error = errors
            .as_slice()
            .iter()
            .find(|error| error.location == "orphan.yaml")
            .expect("the file that declares members and no domain");
        assert_eq!(
            error.code,
            ValidationCode::MissingDeclaration,
            "a required key is missing; nothing here is a dangling reference: {error}"
        );
        assert!(error.message.contains("domain:"), "{error}");
    }

    #[test]
    fn the_same_name_declared_in_two_files_names_both_files() {
        let declaration = r"
domain: shop.cart
events:
  - name: shop.cart.CartOpened
    fields: []
";
        let mut files = minimal();
        files.push(file("a.yaml", declaration));
        files.push(file("b.yaml", declaration));

        let errors = Specification::assemble(files).expect_err("declared twice");
        let rendered = errors.to_string();
        assert!(
            errors.contains(ValidationCode::DuplicateDeclaration),
            "{rendered}"
        );
        assert!(
            rendered.contains("a.yaml") && rendered.contains("b.yaml"),
            "both sources have to be named or neither can be fixed: {rendered}"
        );
    }

    /// A sound domain, for the rows whose copies sit above the domains and refer into one.
    ///
    /// A component owns a domain and a binding joins a cause to a command, so neither row can be
    /// written as one file. Nothing here is duplicated and nothing here is wrong.
    const SHOP_CART: &str = r"
domain: shop.cart
events:
  - name: shop.cart.ItemAdded
    fields: []
commands:
  - name: shop.cart.AddItem
    outcomes:
      - name: added
        emits: [shop.cart.ItemAdded]
";

    /// One name, declared twice, with the first copy broken — one document per kind.
    ///
    /// A row is the pieces a document is built from rather than the document itself, so that what
    /// this comment claims of each piece can be asked instead of read: the first copy carries an
    /// error of its own and nothing else — a duplicate field, a duplicate input, a projection of
    /// nothing, a component of nothing, a `when:` that names no cause — and the second is sound and
    /// takes the same name. [`every_masked_row_is_broken_then_sound_as_this_table_claims`] builds
    /// each piece on its own and fails the row that does not do what is written here. This table is
    /// the whole evidence that the class is closed, so a row that says one thing and does another
    /// is the evidence failing, not a comment going stale.
    struct Masked {
        /// The kind whose name is declared twice.
        kind: &'static str,
        /// The file both copies are written in.
        label: &'static str,
        /// Everything the two copies need, and nothing that is wrong on its own.
        preamble: &'static str,
        /// The first copy of the name.
        first: &'static str,
        /// The second copy of the name.
        second: &'static str,
        /// The location the refusal of the second declaration has to carry.
        location: &'static str,
        /// Files the row's copies refer to, which are part of neither copy.
        ///
        /// Empty for a row whose preamble can carry everything it needs. A component owns a domain
        /// and a binding joins two, and neither can be written in the document that declares one,
        /// so those rows name the domain document here instead. Whatever is named has to be sound
        /// on its own, or the row's own assembly is refused for something that is not either copy
        /// — which is how the `component` row came to own a domain nothing declared.
        support: &'static [(&'static str, &'static str)],
        /// Whether `first` is the copy carrying an error of its own.
        ///
        /// `false` for the `actor` control row alone, whose conversion cannot fail.
        first_is_broken: bool,
    }

    impl Masked {
        /// The specification one of this row's documents is read in: the system header, whatever
        /// the copies refer to, and the document itself.
        fn spec(&self, document: &str) -> Vec<(Source, RawSpecFile)> {
            let mut files = minimal();
            files.extend(
                self.support
                    .iter()
                    .map(|(label, support)| file(label, support)),
            );
            files.push(file(self.label, document));
            files
        }

        /// Both copies, in the order they are written.
        fn both(&self) -> String {
            format!("{}{}{}", self.preamble, self.first, self.second)
        }

        /// The first copy alone, to ask whether it is the broken one this table claims.
        fn first_alone(&self) -> String {
            format!("{}{}", self.preamble, self.first)
        }

        /// The second copy alone, to ask whether it is the sound one this table claims.
        fn second_alone(&self) -> String {
            format!("{}{}", self.preamble, self.second)
        }
    }

    const MASKED: &[Masked] = &[
        Masked {
            kind: "entity",
            label: "domains/cart.yaml",
            preamble: r"
domain: shop.cart
entities:
",
            first: r"  - name: shop.cart.Cart
    identity: {name: id, type: Uuid}
    fields:
      - {name: total, type: Decimal}
      - {name: total, type: Decimal}
    lifecycle: {states: [Open], initial: Open, terminal: [Open]}
",
            second: r"  - name: shop.cart.Cart
    identity: {name: id, type: Uuid}
    lifecycle: {states: [Open], initial: Open, terminal: [Open]}
",
            location: "entity shop.cart.Cart",
            support: &[],
            first_is_broken: true,
        },
        Masked {
            kind: "command",
            label: "domains/cart.yaml",
            preamble: r"
domain: shop.cart
events:
  - name: shop.cart.ItemAdded
    fields: []
commands:
",
            first: r"  - name: shop.cart.AddItem
    input:
      - {name: note, type: String}
      - {name: note, type: String}
    outcomes:
      - name: added
        emits: [shop.cart.ItemAdded]
",
            second: r"  - name: shop.cart.AddItem
    outcomes:
      - name: added
        emits: [shop.cart.ItemAdded]
",
            location: "command shop.cart.AddItem",
            support: &[],
            first_is_broken: true,
        },
        Masked {
            kind: "event",
            label: "domains/cart.yaml",
            preamble: r"
domain: shop.cart
events:
",
            first: r"  - name: shop.cart.ItemAdded
    fields:
      - {name: total, type: Decimal}
      - {name: total, type: Decimal}
",
            second: r"  - name: shop.cart.ItemAdded
    fields: []
",
            location: "event shop.cart.ItemAdded",
            support: &[],
            first_is_broken: true,
        },
        Masked {
            kind: "error",
            label: "domains/cart.yaml",
            preamble: r"
domain: shop.cart
errors:
",
            first: r"  - name: shop.cart.CartFull
    fields:
      - {name: total, type: Decimal}
      - {name: total, type: Decimal}
",
            second: r"  - name: shop.cart.CartFull
    fields: []
",
            location: "error shop.cart.CartFull",
            support: &[],
            first_is_broken: true,
        },
        Masked {
            kind: "view",
            label: "domains/cart.yaml",
            preamble: r"
domain: shop.cart
entities:
  - name: shop.cart.Cart
    identity: {name: id, type: Uuid}
    lifecycle: {states: [Open], initial: Open, terminal: [Open]}
views:
",
            first: r"  - name: shop.cart.CartById
    source: shop.cart.Cart
",
            second: r"  - name: shop.cart.CartById
    source: shop.cart.Cart
    fields:
      - {name: id, type: Uuid}
",
            location: "view shop.cart.CartById",
            support: &[],
            first_is_broken: true,
        },
        // Two sound copies, because an actor cannot have the first kind of error: its conversion
        // is documented as one that cannot fail ("nothing here can be wrong on its own",
        // `actor.rs`), so no copy of one can be masked. A control rather than a case, and
        // `first_is_broken: false` is what says so to the check as well as to the reader.
        Masked {
            kind: "actor",
            label: "domains/cart.yaml",
            preamble: r"
domain: shop.cart
actors:
",
            first: r"  - name: shop.cart.Shopper
",
            second: r"  - name: shop.cart.Shopper
",
            location: "actor shop.cart.Shopper",
            support: &[],
            first_is_broken: false,
        },
        Masked {
            kind: "component",
            label: "components.yaml",
            preamble: r"
components:
",
            first: r"  - component: cart-service
",
            second: r"  - component: cart-service
    owns:
      domains: [shop.cart]
",
            location: "component cart-service",
            support: &[("domains/cart.yaml", SHOP_CART)],
            first_is_broken: true,
        },
        Masked {
            kind: "binding",
            label: "bindings.yaml",
            preamble: r"
bindings:
",
            first: r"  - id: tell-the-shopper
    when: {}
    invoke: {command: shop.cart.AddItem}
    delivery: at_least_once
    on_failure: drop
",
            second: r"  - id: tell-the-shopper
    when: {event: shop.cart.ItemAdded}
    invoke: {command: shop.cart.AddItem}
    delivery: at_least_once
    on_failure: drop
",
            location: "binding tell-the-shopper",
            support: &[("domains/cart.yaml", SHOP_CART)],
            first_is_broken: true,
        },
    ];

    /// Each [`MASKED`] row is the broken-then-sound pair its table says it is.
    ///
    /// The table is the evidence that the masking class is closed, and a row proves nothing about
    /// masking unless the second copy is sound: [`declare`] refuses before either copy is
    /// converted, so `a_name_declared_twice_is_refused_even_when_a_copy_is_broken_itself` passes
    /// just as happily on a broken-then-*broken* row, which tests the registry blindness against
    /// nothing at all. One row was exactly that — its second entity's only state was a dead end —
    /// and no case in the tree could have noticed. So the claim is asked of every row here, each
    /// copy assembled on its own, rather than left to the comment.
    #[test]
    fn every_masked_row_is_broken_then_sound_as_this_table_claims() {
        for row in MASKED {
            if let Err(errors) = Specification::assemble(row.spec(&row.second_alone())) {
                panic!(
                    "the {} row's second declaration is not sound on its own, so the row pairs a \
                     broken copy with a broken copy and says nothing about a name being taken in \
                     silence: {errors}",
                    row.kind
                );
            }

            let outcome = Specification::assemble(row.spec(&row.first_alone()));
            if row.first_is_broken {
                assert!(
                    outcome.is_err(),
                    "the {} row's first declaration carries no error of its own, so the row is \
                     two sound copies and the masking it claims to exercise never happens",
                    row.kind
                );
            } else if let Err(errors) = outcome {
                panic!(
                    "the {} row is marked a control because its conversion cannot fail, and one \
                     copy of it was refused: {errors}",
                    row.kind
                );
            }
        }
    }

    /// Every kind that is recorded by conversion, declared twice with one copy broken.
    ///
    /// [`Collected::absorb`] reached a registry only with a declaration whose own `try_from`
    /// succeeded, and a registry reports a second declaration by finding the first one in it. So a
    /// copy with any error of its own was never recorded, and the *other* copy found the registry
    /// empty and took the name in silence: the author was told about one copy's error and never
    /// about the two declarations, and fixing that error made a refusal appear that was true all
    /// along.
    ///
    /// All eight kinds that go through [`declare`] are in [`MASKED`], because all eight are
    /// absorbed by the same shape. The three that are *not* are absorbed by something else: a
    /// conversion is one crossing per system and is refused by
    /// [`crate::types::ConversionRegistry`] without converting anything, and a topology is one per
    /// system and records the source that carried it before converting it, so neither can be
    /// masked this way. A declared *type* can be, and is not a [`MASKED`] row because it is not
    /// refused by [`declare`]: [`Collected::write`] refuses a second type declaration whether or
    /// not either copy converts — the same reporter as the cross-kind half — and carries only the
    /// first converted one to `SystemSpec::merge`. Measured in all four soundness combinations in
    /// `tests/masked_declaration_boundaries.rs`.
    #[test]
    fn a_name_declared_twice_is_refused_even_when_a_copy_is_broken_itself() {
        for row in MASKED {
            let (kind, location) = (row.kind, row.location);
            let errors =
                Specification::assemble(row.spec(&row.both())).expect_err("declared twice");
            assert!(
                errors.as_slice().iter().any(|error| {
                    error.code == ValidationCode::DuplicateDeclaration && error.location == location
                }),
                "one {kind} name, two declarations, and nothing at `{location}` says so — a \
                 name is declared where it is written, whether or not what is written under it \
                 converts: {errors}"
            );
        }
    }

    /// Every [`MASKED`] row's name still means something once both copies have been read.
    ///
    /// [`declare`] answers *was this name written* and [`record`] answers *what does it mean*, and
    /// separating the two made the second question reachable for a copy the first had already
    /// refused. Answering it with `first` alone drops that copy — so when the **first** copy is the
    /// one that failed its own conversion, nothing is recorded at all and the name is in no
    /// registry: every reference to it is then refused as undeclared, in the same run that reports
    /// it declared twice. `insert` did not do that, because the registry it consulted was empty and
    /// the copy that converted went into it. [`record`] keeps the first copy that *converts*, which
    /// is what that registry was answering all along.
    ///
    /// What an author meets is the cascade, and that is measured where they meet it, in
    /// `tests/masked_declaration_reference_cascade.rs`. This asks the rule of all eight registries
    /// at once, which is the only place the eight can be enumerated: the `match` below is
    /// exhaustive by panic, so a ninth kind routed through [`declare`] and [`record`] fails here
    /// rather than being left unmeasured.
    #[test]
    fn every_masked_rows_name_is_recorded_from_the_copy_that_converts() {
        for row in MASKED {
            let mut collected = Collected::default();
            let mut errors = ValidationErrors::new();
            for (source, file) in row.spec(&row.both()) {
                collected.absorb(&source, file, &mut errors);
            }

            let (kind, name) = row
                .location
                .split_once(' ')
                .expect("every row's location is its kind and then its name");
            let recorded: Vec<String> = match kind {
                "entity" => collected.entities.keys().map(ToString::to_string).collect(),
                "command" => collected.commands.keys().map(ToString::to_string).collect(),
                "event" => collected.events.keys().map(ToString::to_string).collect(),
                "error" => collected.errors.keys().map(ToString::to_string).collect(),
                "view" => collected.views.keys().map(ToString::to_string).collect(),
                "actor" => collected.actors.keys().map(ToString::to_string).collect(),
                "component" => collected
                    .components
                    .keys()
                    .map(ToString::to_string)
                    .collect(),
                "binding" => collected.bindings.keys().map(ToString::to_string).collect(),
                other => panic!(
                    "the {other} row goes through `declare` and `record` and this check does not \
                     know which registry holds it, so nothing here measures whether its name \
                     survives a broken first copy"
                ),
            };

            assert!(
                recorded.iter().any(|held| held == name),
                "`{name}` is declared twice as a {kind}, the first copy fails its own conversion \
                 and the second is sound — and no {kind} registry holds the name, so every \
                 reference to it is refused as undeclared in the same run that reports it declared \
                 more than once. The registry holds {recorded:?}; the refusals were: {errors}"
            );
        }
    }

    /// The same name twice, in all four combinations of sound and broken.
    ///
    /// The masking is not a property of *which* copy is broken. A registry is reached only by a
    /// declaration that converted, so a sound first copy and a broken second one is masked in the
    /// same way, and two broken copies report two local errors and nothing at all about the name.
    /// One kind is enough to state that: the four rows differ only in which conversion fails.
    #[test]
    fn a_name_declared_twice_is_refused_in_all_four_soundness_combinations() {
        let sound = r"
  - name: shop.cart.AddItem
    outcomes:
      - name: added
        emits: [shop.cart.ItemAdded]
";
        let broken = r"
  - name: shop.cart.AddItem
    input:
      - {name: note, type: String}
      - {name: note, type: String}
    outcomes:
      - name: added
        emits: [shop.cart.ItemAdded]
";
        for (label, first, second) in [
            ("sound, sound", sound, sound),
            ("broken, sound", broken, sound),
            ("sound, broken", sound, broken),
            ("broken, broken", broken, broken),
        ] {
            let document = format!(
                r"
domain: shop.cart
events:
  - name: shop.cart.ItemAdded
    fields: []
commands:{first}{second}"
            );
            let mut files = minimal();
            files.push(file("domains/cart.yaml", &document));
            let errors = Specification::assemble(files).expect_err("declared twice");
            assert!(
                errors.as_slice().iter().any(|error| {
                    error.code == ValidationCode::DuplicateDeclaration
                        && error.location == "command shop.cart.AddItem"
                }),
                "`shop.cart.AddItem` is declared twice ({label}) and nothing says so: {errors}"
            );
        }
    }

    #[test]
    fn every_problem_is_reported_at_once() {
        // Accumulation is the point: an author fixing one error per run is an author running the
        // tool ten times to learn what a single pass already knew.
        let mut files = minimal();
        files.push(file(
            "domains/cart.yaml",
            r"
domain: shop.cart
views:
  - name: shop.cart.CartById
    source: shop.cart.Nonexistent
    fields:
      - name: total
        type: Decimal
errors:
  - name: shop.cart.CartClosed
    summary: The cart is already closed.
commands:
  - name: shop.cart.AddItem
    input: []
    outcomes:
      - name: added
        emits:
          - shop.cart.ItemAdded
",
        ));
        let errors = Specification::assemble(files).expect_err("two problems");
        assert_eq!(errors.len(), 2, "{errors}");
        assert!(
            errors
                .as_slice()
                .iter()
                .all(|error| error.code == ValidationCode::UndeclaredReference),
            "counting errors is not checking them; two of the same cascade would pass that: \
             {errors}"
        );
        let rendered = errors.to_string();
        assert!(
            rendered.contains("shop.cart.Nonexistent"),
            "the missing entity: {rendered}"
        );
        assert!(
            rendered.contains("shop.cart.ItemAdded"),
            "and the missing event, in the same run: {rendered}"
        );
    }

    #[test]
    fn an_errors_payload_is_resolved_against_the_types_the_system_declares() {
        let mut files = minimal();
        files.push(file(
            "domains/cart.yaml",
            r"
domain: shop.cart
errors:
  - name: shop.cart.CartFull
    summary: The cart holds as much as it can.
    fields:
      - name: limit
        type: shop.cart.Nonexistent
",
        ));

        let errors = Specification::assemble(files).expect_err("the payload names nothing");
        assert_eq!(errors.len(), 1, "{errors}");
        let error = &errors.as_slice()[0];
        assert_eq!(error.code, ValidationCode::UndeclaredReference);
        assert_eq!(error.location, "error.shop.cart.CartFull.fields[0].type");
    }

    #[test]
    fn an_event_that_records_one_name_twice_is_refused() {
        let mut files = minimal();
        files.push(file(
            "domains/cart.yaml",
            r"
domain: shop.cart
events:
  - name: shop.cart.ItemAdded
    fields:
      - name: total
        type: Decimal
      - name: total
        type: Decimal
",
        ));

        let errors = Specification::assemble(files).expect_err("one field, twice");
        assert_eq!(errors.len(), 1, "{errors}");
        assert_eq!(
            errors.as_slice()[0].code,
            ValidationCode::DuplicateDeclaration
        );
    }

    #[test]
    fn a_header_this_build_cannot_read_no_longer_hides_the_references_under_it() {
        let files = vec![
            file(
                "system.yaml",
                r"
format: ess/99
system: shop
version: v1
",
            ),
            file(
                "domains/cart.yaml",
                r"
domain: shop.cart
views:
  - name: shop.cart.CartById
    source: shop.cart.Nonexistent
    fields:
      - name: total
        type: Decimal
commands:
  - name: shop.cart.AddItem
    input: []
    outcomes:
      - name: added
        emits:
          - shop.cart.ItemAdded
",
            ),
        ];

        let errors = Specification::assemble(files).expect_err("three unrelated problems");
        assert_eq!(
            errors.len(),
            3,
            "a merge that failed still hands back a graph, so one run reports all three: {errors}"
        );
        assert!(
            errors.contains(ValidationCode::UnsupportedFormatVersion),
            "{errors}"
        );
        let rendered = errors.to_string();
        assert!(rendered.contains("shop.cart.ItemAdded"), "{rendered}");
        assert!(rendered.contains("shop.cart.Nonexistent"), "{rendered}");
    }

    #[test]
    fn nothing_can_be_checked_against_a_specification_that_names_no_system() {
        let errors = Specification::assemble(vec![file(
            "domains/cart.yaml",
            r"
domain: shop.cart
commands:
  - name: shop.cart.AddItem
    input: []
    outcomes:
      - name: added
        emits:
          - shop.cart.ItemAdded
",
        )])
        .expect_err("nothing says what system this is");

        assert_eq!(
            errors.len(),
            1,
            "with no namespace and no registry every reference below would cascade: {errors}"
        );
        assert_eq!(
            errors.as_slice()[0].code,
            ValidationCode::MissingDeclaration
        );
        assert_eq!(errors.as_slice()[0].location, "system");
    }

    #[test]
    fn a_fragment_that_sets_system_level_fields_without_naming_the_system_is_refused() {
        let mut files = minimal();
        files.push(file(
            "domains/cart.yaml",
            r"
version: v9
format: ess/2
domain: shop.cart
",
        ));

        let errors = Specification::assemble(files).expect_err("a version without a system");
        assert_eq!(errors.len(), 1, "{errors}");
        let error = &errors.as_slice()[0];
        assert_eq!(error.code, ValidationCode::MissingDeclaration);
        assert_eq!(error.location, "domains/cart.yaml.system");
    }

    #[test]
    fn a_domain_summary_is_not_a_system_level_field() {
        let mut files = minimal();
        files.push(file(
            "domains/cart.yaml",
            r"
domain: shop.cart
summary: Everything a customer is about to buy.
",
        ));

        let specification = Specification::assemble(files).expect("valid");
        assert_eq!(
            specification
                .system
                .domain(&QualifiedName::new("shop.cart").expect("a name"))
                .expect("declared")
                .naming
                .summary
                .as_deref(),
            Some("Everything a customer is about to buy.")
        );
    }

    #[test]
    fn a_domain_the_header_lists_and_nobody_declares_is_refused() {
        let files = vec![
            file(
                "system.yaml",
                r"
format: ess/1
system: shop
domains:
  - shop.carts
",
            ),
            file(
                "domains/cart.yaml",
                "domain: shop.cart
",
            ),
        ];

        let errors = Specification::assemble(files).expect_err("a typo in the roster");
        assert_eq!(errors.len(), 2, "the roster disagrees both ways: {errors}");
        let listed = errors
            .as_slice()
            .iter()
            .find(|error| error.location == "system.domains")
            .expect("the listed name nobody declares");
        assert_eq!(listed.code, ValidationCode::UndeclaredReference);
        assert!(listed.message.contains("shop.carts"), "{listed}");
    }

    #[test]
    fn a_domain_the_header_does_not_list_is_refused() {
        let files = vec![
            file(
                "system.yaml",
                r"
format: ess/1
system: shop
domains:
  - shop.cart
",
            ),
            file(
                "domains/cart.yaml",
                "domain: shop.cart
",
            ),
            file(
                "domains/wishlist.yaml",
                "domain: shop.wishlist
",
            ),
        ];

        let errors = Specification::assemble(files).expect_err("a domain nobody listed");
        assert_eq!(errors.len(), 1, "{errors}");
        let error = &errors.as_slice()[0];
        assert_eq!(
            error.code,
            ValidationCode::ConflictingDeclaration,
            "the header says what the system has, so it and the sources have to agree: {error}"
        );
        assert_eq!(error.location, "domain shop.wishlist");
    }

    #[test]
    fn a_header_that_keeps_no_roster_is_not_checked_against_one() {
        let mut files = minimal();
        files.push(file(
            "domains/cart.yaml",
            "domain: shop.cart
",
        ));
        Specification::assemble(files).expect("a roster is optional; a wrong one is not");
    }

    #[test]
    fn an_actor_belongs_to_a_domain_like_every_other_member() {
        let mut files = minimal();
        files.push(file(
            "domains/cart.yaml",
            r"
domain: shop.cart
actors:
  - name: shop.cart.Shopper
    may: []
",
        ));

        let specification = Specification::assemble(files).expect("valid");
        let shopper = QualifiedName::new("shop.cart.Shopper").expect("a name");
        assert_eq!(
            specification
                .system
                .owner_of(&shopper)
                .map(|domain| domain.name.to_string()),
            Some("shop.cart".to_owned()),
            "design \u{a7}22 puts every object under the system's identity, actors included"
        );
    }

    #[test]
    fn an_actor_inside_no_declared_domain_is_refused() {
        let mut files = minimal();
        files.push(file(
            "domains/cart.yaml",
            r"
domain: shop.cart
actors:
  - name: evil.Hacker
    may: []
",
        ));

        let errors = Specification::assemble(files).expect_err("nobody owns it");
        assert_eq!(errors.len(), 1, "{errors}");
        let error = &errors.as_slice()[0];
        assert_eq!(error.code, ValidationCode::ConflictingDeclaration);
        assert_eq!(
            error.location, "domain shop.cart.actors",
            "design \u{a7}22 puts an actor under the system's identity like anything else: {error}"
        );
    }

    #[test]
    fn a_file_of_actors_with_no_domain_leaves_them_owned_by_nobody_and_is_refused() {
        let mut files = minimal();
        files.push(file(
            "actors.yaml",
            r"
actors:
  - name: shop.cart.Shopper
    may: []
",
        ));

        let errors = Specification::assemble(files).expect_err("no domain owns them");
        assert!(
            errors
                .as_slice()
                .iter()
                .any(|error| error.code == ValidationCode::MissingDeclaration
                    && error.location == "actors.yaml"),
            "an actor is a member, so a file of them still needs a `domain:`: {errors}"
        );
        assert!(
            errors
                .as_slice()
                .iter()
                .any(|error| error.code == ValidationCode::UndeclaredReference
                    && error.location == "actor shop.cart.Shopper"),
            "and the ownership pass has to see it: {errors}"
        );
    }

    #[test]
    fn an_actor_may_not_take_a_name_a_command_already_has() {
        let mut files = minimal();
        files.push(file(
            "domains/cart.yaml",
            r"
domain: shop.cart
events:
  - name: shop.cart.ItemAdded
    fields: []
commands:
  - name: shop.cart.AddItem
    input: []
    outcomes:
      - name: added
        emits:
          - shop.cart.ItemAdded
actors:
  - name: shop.cart.AddItem
    may: []
",
        ));

        let errors = Specification::assemble(files).expect_err("one name, two things");
        assert!(
            errors.contains(ValidationCode::DuplicateDeclaration),
            "a grant and a command answering to one name is two answers to `who is this`: {errors}"
        );
    }
}

#[cfg(test)]
mod parse_tests {
    use super::RawSpecFile;

    #[test]
    fn a_key_written_twice_is_refused_with_the_key_and_the_line() {
        // Deserialising straight into `RawSpecFile` keeps the *last* of two identical keys and says
        // nothing, so a workload, a type or a whole system could be silently discarded. This is the
        // one check that catches it for every mapping in the format at once.
        let text = "\
topology:
  workloads:
    a-service:
      stateless: true
    a-service:
      stateless: false
";
        let error = RawSpecFile::parse(text).expect_err("the key is written twice");
        let rendered = error.to_string();
        assert!(rendered.contains("a-service"), "{rendered}");
        assert!(
            rendered.contains("duplicate"),
            "the reader has to be told which fault this is: {rendered}"
        );
    }

    #[test]
    fn a_top_level_key_written_twice_is_refused_too() {
        let error = RawSpecFile::parse("system: billing\nsystem: other\n")
            .expect_err("the key is written twice");
        assert!(error.to_string().contains("system"), "{error}");
    }

    #[test]
    fn the_two_stage_parse_reads_what_the_one_stage_parse_read() {
        // Going through `serde_yaml::Value` must not change what a valid document means. A type
        // with a hand-written `Deserialize` is where that would break, so the fixture uses several.
        let text = "\
format: ess/1
system: billing
version: v3
domains: [billing.invoice]
";
        let staged = RawSpecFile::parse(text).expect("valid");
        let direct: RawSpecFile = serde_yaml::from_str(text).expect("valid");
        assert_eq!(
            staged.system.map(|name| name.to_string()),
            direct.system.map(|name| name.to_string())
        );
        assert_eq!(staged.version, direct.version);
        assert_eq!(
            staged.format.map(|f| f.to_string()),
            direct.format.map(|f| f.to_string())
        );
        assert_eq!(staged.domains.len(), direct.domains.len());
    }
}
