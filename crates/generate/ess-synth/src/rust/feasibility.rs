//! Admission of the existing allocation, before any source renderer runs.
//!
//! Names are checked in their actual scopes. This module never repairs names, changes the neutral
//! plan, renders source to inspect it, or runs a compiler. The admitted layout is the renderer's
//! layout; final collision checks therefore include its existing fallback and suffix decisions.

use std::collections::{BTreeMap, BTreeSet};

use ess_compiler::ir::{
    EssIr, ResolvedBinding, ResolvedBody, ResolvedCommand, ResolvedField, ResolvedMappingValue,
    ResolvedTypeRef,
};
use ess_domain::name::QualifiedName;
use ess_domain::types::Primitive;

use super::{event_variants, items, layout::Layout, name, Emit};
use crate::plan::{conversion_source, mechanical_conversion};
use crate::{
    CapabilityKind, SynthesisPlan, Target, TargetFailure, TargetFailureCause,
    TargetFailureCode as Code,
};

#[derive(Default)]
struct Inventory {
    symbols: BTreeMap<(String, String), Vec<(String, String)>>,
    paths: BTreeMap<String, Vec<String>>,
    helpers: BTreeMap<(String, String), BTreeSet<String>>,
    causes: Vec<TargetFailureCause>,
}

impl Inventory {
    fn cause(&mut self, code: Code, sources: Vec<String>, detail: String) {
        self.causes
            .push(TargetFailureCause::new(code, sources, detail));
    }

    fn symbol(&mut self, scope: &str, token: &str, source: &str, role: &str) {
        if !name::valid_ident(token) {
            self.cause(
                Code::InvalidIdentifier,
                vec![source.to_owned()],
                format!("{role} allocates invalid Rust identifier `{token}` in `{scope}`"),
            );
        }
        let ident = token.strip_prefix("r#").unwrap_or(token);
        self.symbols
            .entry((scope.to_owned(), ident.to_owned()))
            .or_default()
            .push((source.to_owned(), role.to_owned()));
    }

    fn path(&mut self, path: String, source: &str) {
        self.paths.entry(path).or_default().push(source.to_owned());
    }

    fn module(&mut self, scope: &str, token: &str, path: String, source: &str) {
        self.symbol(scope, token, source, "module");
        if token.starts_with("r#") {
            self.cause(
                Code::PathCollision,
                vec![source.to_owned()],
                format!(
                    "module `{token}` resolves `{}` but the current allocation emits `{path}`",
                    token.trim_start_matches("r#")
                ),
            );
        }
        self.path(path, source);
    }

    fn helper(&mut self, scope: &str, ident: &str, source: &str) {
        self.helpers
            .entry((scope.to_owned(), ident.to_owned()))
            .or_default()
            .insert(source.to_owned());
    }

    fn reference(&mut self, scope: &str, reference: &ResolvedTypeRef, source: &str) {
        match reference {
            ResolvedTypeRef::Primitive { name } => match name {
                Primitive::Binary64 => unreachable!("Binary64 is refused before target rendering"),
                Primitive::String => self.helper(scope, "String", source),
                Primitive::Bytes => self.helper(scope, "Vec", source),
                Primitive::Boolean
                | Primitive::Integer
                | Primitive::Decimal
                | Primitive::Timestamp
                | Primitive::Duration
                | Primitive::Uuid => {}
            },
            ResolvedTypeRef::Declared { .. } => {}
            ResolvedTypeRef::Optional { of } => {
                self.helper(scope, "Option", source);
                self.reference(scope, of, source);
            }
            ResolvedTypeRef::List { of } => {
                self.helper(scope, "Vec", source);
                self.reference(scope, of, source);
            }
            ResolvedTypeRef::Map { key, value } => {
                self.helper(scope, "std", source);
                self.reference(scope, &ResolvedTypeRef::Primitive { name: *key }, source);
                self.reference(scope, value, source);
            }
        }
    }

    fn fields(&mut self, scope: &str, owner: &QualifiedName, fields: &[ResolvedField]) {
        let field_scope = format!("fields:{owner}");
        for field in fields {
            let source = format!("{owner}.{}", field.name);
            self.symbol(
                &field_scope,
                &name::value_ident(&field.name),
                &source,
                "field",
            );
            self.reference(scope, &field.type_ref, &source);
        }
    }

    fn finish(mut self) -> Vec<TargetFailureCause> {
        for ((scope, ident), entries) in &self.symbols {
            if entries.len() > 1 {
                let code = if scope == "wire functions" {
                    Code::WireCollision
                } else {
                    Code::SymbolCollision
                };
                self.causes.push(TargetFailureCause::new(
                    code,
                    entries.iter().map(|(source, _)| source.clone()).collect(),
                    format!(
                        "`{ident}` is allocated {} times in `{scope}` ({})",
                        entries.len(),
                        entries
                            .iter()
                            .map(|(_, role)| role.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                ));
            }
            if let Some(users) = self.helpers.get(&(scope.clone(), ident.clone())) {
                self.causes.push(TargetFailureCause::new(
                    Code::SymbolCollision,
                    entries
                        .iter()
                        .map(|(source, _)| source.clone())
                        .chain(users.iter().cloned())
                        .collect(),
                    format!("`{ident}` in `{scope}` shadows a helper referenced by generated code"),
                ));
            }
        }
        for (path, sources) in &self.paths {
            if sources.len() > 1 {
                self.causes.push(TargetFailureCause::new(
                    Code::PathCollision,
                    sources.clone(),
                    format!("multiple generated artifacts allocate `{path}`"),
                ));
            }
        }
        self.causes
    }
}

/// The checked allocation, shared with the renderer of either target.
pub(crate) fn checked(
    ir: &EssIr,
    plan: &SynthesisPlan,
    target: Target,
) -> Result<Layout, TargetFailure> {
    let layout = Layout::of(ir);
    let mut inventory = Inventory::default();
    for declared in ir.types().keys() {
        if !layout.has_owner(declared) {
            inventory.cause(
                Code::MissingTypeOwner,
                vec![declared.to_string()],
                "the Rust workspace has no domain module allocated for this system-level type"
                    .to_owned(),
            );
        }
    }
    // All subsequent naming helpers require an owner. No source-driven owner lookup may run
    // until this prerequisite is established, including on a domainless, unused type.
    if !inventory.causes.is_empty() {
        return Err(TargetFailure::new(ir, target, plan, inventory.finish()));
    }
    paths_and_packages(&mut inventory, ir, plan, &layout);
    declarations(&mut inventory, ir, plan, &layout);
    components_and_system(&mut inventory, ir, plan, &layout);
    conversions(&mut inventory, ir, plan, &layout);
    size_cycles(&mut inventory, ir);
    bindings(&mut inventory, ir, plan);
    if !super::http::served(ir).is_empty() {
        wire(&mut inventory, ir, plan, &layout, |_| true);
    }
    if target == Target::Web {
        web_dependencies(&mut inventory, ir, &layout);
    }
    let causes = inventory.finish();
    if causes.is_empty() {
        Ok(layout)
    } else {
        Err(TargetFailure::new(ir, target, plan, causes))
    }
}

fn web_dependencies(inventory: &mut Inventory, ir: &EssIr, layout: &Layout) {
    let scope = "web root";
    for module in ["catalog", "json", "wire"] {
        inventory.symbol(scope, module, &ir.system().to_string(), "fixed Web module");
    }
    // The bridge constructs every component by its final dependency path. A module in this
    // root captures that path; a same-named value or a helper in another module does not.
    for component in ir.components().keys() {
        inventory.helper(
            scope,
            &Layout::crate_ident(layout.component_package(component)),
            &component.to_string(),
        );
    }
}

fn paths_and_packages(
    inventory: &mut Inventory,
    ir: &EssIr,
    plan: &SynthesisPlan,
    layout: &Layout,
) {
    let system = ir.system().to_string();
    let mut packages = vec![(layout.package(), system.clone())];
    if !ir.components().is_empty() || !ir.bindings().is_empty() {
        packages.push((layout.system_package(), system.clone()));
    }
    let served = super::http::served(ir);
    if !served.is_empty() {
        packages.push((layout.server_package(), system.clone()));
    }
    for component in ir.components().keys() {
        packages.push((layout.component_package(component), component.to_string()));
    }
    for (package, owner) in packages {
        inventory.symbol(
            "workspace crates",
            &Layout::crate_ident(package),
            &owner,
            "crate identifier",
        );
        inventory.path(format!("crates/{package}/Cargo.toml"), &owner);
        inventory.path(format!("crates/{package}/src/lib.rs"), &owner);
    }
    inventory.symbol("types root", "primitives", &system, "fixed module");
    inventory.path(
        format!("crates/{}/src/primitives.rs", layout.package()),
        &system,
    );
    if plan.obligations().next().is_some()
        || !ir.components().is_empty()
        || !ir.bindings().is_empty()
    {
        inventory.symbol("types root", "obligation", &system, "fixed module");
        inventory.path(
            format!("crates/{}/src/obligation.rs", layout.package()),
            &system,
        );
    }
    for (domain, module) in layout.modules() {
        inventory.module(
            "types root",
            module,
            layout.module_path(domain),
            &domain.to_string(),
        );
    }
    if !served.is_empty() {
        for fixed in ["http", "json", "wire"] {
            inventory.symbol("server root", fixed, &system, "fixed module");
            inventory.path(
                format!("crates/{}/src/{fixed}.rs", layout.server_package()),
                &system,
            );
        }
        for component in served {
            let module = name::value_ident(&component.name.to_string());
            inventory.module(
                "server root",
                &module,
                format!("crates/{}/src/{module}.rs", layout.server_package()),
                &component.name.to_string(),
            );
        }
    }
}

fn domain_scope(layout: &Layout, source: &QualifiedName) -> String {
    format!("domain:{}", layout.owner(source))
}

fn declarations(inventory: &mut Inventory, ir: &EssIr, plan: &SynthesisPlan, layout: &Layout) {
    type_declarations(inventory, ir, layout);
    entity_declarations(inventory, ir, layout);
    records_and_commands(inventory, ir, layout);
    domain_obligations(inventory, plan, layout);
}

fn type_declarations(inventory: &mut Inventory, ir: &EssIr, layout: &Layout) {
    for declared in ir.types().values() {
        let source = declared.name.to_string();
        let scope = domain_scope(layout, &declared.name);
        inventory.symbol(&scope, &layout.type_name(&declared.name), &source, "type");
        match &declared.body {
            ResolvedBody::Newtype { of, .. } => inventory.reference(&scope, of, &source),
            ResolvedBody::Struct { fields, .. } => inventory.fields(&scope, &declared.name, fields),
            ResolvedBody::Enum { variants } => {
                for variant in variants {
                    inventory.symbol(
                        &format!("variants:{source}"),
                        &name::pascal(variant),
                        &format!("{source}.{variant}"),
                        "enum variant",
                    );
                }
            }
            ResolvedBody::Union { variants, .. } => {
                for (variant, reference) in variants {
                    inventory.symbol(
                        &format!("variants:{source}"),
                        &name::pascal(variant),
                        &format!("{source}.{variant}"),
                        "union variant",
                    );
                    inventory.reference(&scope, reference, &format!("{source}.{variant}"));
                }
            }
        }
    }
}

fn entity_declarations(inventory: &mut Inventory, ir: &EssIr, layout: &Layout) {
    for entity in ir.entities().values() {
        let source = entity.name.to_string();
        let scope = domain_scope(layout, &entity.name);
        let ty = layout.type_name(&entity.name);
        for (token, role) in [
            (ty.clone(), "entity"),
            (format!("{ty}Data"), "entity data"),
            (
                layout.entity_snapshot(&entity.name).to_owned(),
                "entity snapshot",
            ),
            (format!("Any{ty}"), "entity sum"),
            (format!("{}_state", name::value_ident(&ty)), "state module"),
        ] {
            inventory.symbol(&scope, &token, &source, role);
        }
        inventory.helper(&scope, "core", &source);
        let generic_scope = format!("entity generic:{source}");
        inventory.symbol(&generic_scope, "S", &source, "state type parameter");
        inventory.symbol(&generic_scope, &ty, &source, "entity type reference");
        inventory.fields(&scope, &entity.name, std::slice::from_ref(&entity.identity));
        inventory.fields(&scope, &entity.name, &entity.fields);
        let states = format!("states:{source}");
        for fixed in ["Marker", "sealed"] {
            inventory.symbol(&states, fixed, &source, "state helper");
        }
        for state in &entity.lifecycle.states {
            inventory.symbol(
                &states,
                state.as_str(),
                &format!("{source}.{state}"),
                "state marker",
            );
            let methods = format!("entity:{source}:{state}");
            for fixed in ["state", "data", "into_data"] {
                inventory.symbol(&methods, fixed, &source, "entity method");
            }
            if *state == entity.lifecycle.initial {
                inventory.symbol(&methods, "new", &source, "constructor");
            }
            for transition in entity.lifecycle.outgoing(state) {
                inventory.symbol(
                    &methods,
                    &name::value_ident(&transition.name),
                    &format!("{source}.{}", transition.name),
                    "transition method",
                );
            }
        }
    }
}

fn records_and_commands(inventory: &mut Inventory, ir: &EssIr, layout: &Layout) {
    for (owner, fields) in ir
        .events()
        .values()
        .map(|value| (&value.name, value.fields.as_slice()))
        .chain(
            ir.errors()
                .values()
                .map(|value| (&value.name, value.fields.as_slice())),
        )
        .chain(
            ir.views()
                .values()
                .map(|value| (&value.name, value.fields.as_slice())),
        )
    {
        let scope = domain_scope(layout, owner);
        inventory.symbol(
            &scope,
            &layout.type_name(owner),
            &owner.to_string(),
            "record",
        );
        inventory.fields(&scope, owner, fields);
    }
    for command in ir.commands().values() {
        let source = command.name.to_string();
        let scope = domain_scope(layout, &command.name);
        let ty = layout.type_name(&command.name);
        inventory.symbol(&scope, &ty, &source, "command input");
        inventory.symbol(&scope, &format!("{ty}Outcome"), &source, "command outcome");
        inventory.fields(&scope, &command.name, &command.input);
        if !command.response.is_empty() {
            inventory.symbol(
                &scope,
                &format!("{ty}Response"),
                &source,
                "command response",
            );
            inventory.fields(
                &format!("response:{source}"),
                &command.name,
                &command.response,
            );
        }
        let emit = Emit {
            ir,
            layout,
            domain: layout.owner(&command.name),
        };
        for outcome in &command.outcomes {
            let branch = format!("{source}.{}", outcome.name);
            inventory.symbol(
                &format!("outcomes:{source}"),
                &name::pascal(outcome.name.as_str()),
                &branch,
                "outcome variant",
            );
            for carried in items::outcome_event_fields(&emit, outcome) {
                inventory.symbol(
                    &format!("outcome fields:{branch}"),
                    &carried.field,
                    &carried.event.to_string(),
                    "published event field",
                );
            }
            if let Some(error) = &outcome.error {
                inventory.symbol(
                    &format!("outcome fields:{branch}"),
                    "error",
                    &error.to_string(),
                    "refusal field",
                );
            }
        }
    }
}

fn domain_obligations(inventory: &mut Inventory, plan: &SynthesisPlan, layout: &Layout) {
    for (domain, _) in layout.modules() {
        let owed = plan
            .obligations()
            .filter(|(capability, _)| {
                matches!(
                    capability.kind,
                    CapabilityKind::CommandBehavior | CapabilityKind::ViewQuery
                )
            })
            .filter(|(capability, _)| {
                QualifiedName::new(&capability.source)
                    .is_ok_and(|source| layout.owner(&source) == domain)
            })
            .collect::<Vec<_>>();
        if owed.is_empty() {
            continue;
        }
        inventory.symbol(
            &format!("domain:{domain}"),
            "obligations",
            &domain.to_string(),
            "obligation module",
        );
        let scope = format!("domain obligations:{domain}");
        inventory.symbol(
            &scope,
            "Unimplemented",
            &domain.to_string(),
            "obligation stub",
        );
        for (capability, _) in owed {
            let declared =
                QualifiedName::new(&capability.source).expect("typed declaration source");
            let suffix = if capability.kind == CapabilityKind::CommandBehavior {
                "Behavior"
            } else {
                "Query"
            };
            inventory.symbol(
                &scope,
                &format!("{}{suffix}", layout.type_name(&declared)),
                &capability.source,
                "obligation trait",
            );
        }
    }
}

fn delivery_initializers(
    inventory: &mut Inventory,
    plan: &SynthesisPlan,
    delivered: &[&ResolvedBinding],
) {
    // deliver_fn groups this ordered list by event. Each arm starts with its event pattern,
    // and every delivery (including an obligated transformation) introduces input only after
    // evaluating its initializer. A later bare call can therefore see a prior input local.
    let mut prior_inputs: BTreeMap<&QualifiedName, String> = BTreeMap::new();
    for binding in delivered {
        let source = binding.name.to_string();
        if plan.is_generated(CapabilityKind::BindingTransformation, &source) {
            let scope = format!("delivery initializer:{source}");
            inventory.symbol(
                &scope,
                &name::value_ident(&source),
                &source,
                "transformation function reference",
            );
            inventory.helper(
                &scope,
                "event",
                &binding
                    .cause
                    .event()
                    .expect("generated event capability")
                    .to_string(),
            );
            if let Some(prior) = prior_inputs.get(
                binding
                    .cause
                    .event()
                    .expect("generated event capability")
                    .name(),
            ) {
                inventory.helper(&scope, "input", prior);
            }
        }
        prior_inputs.insert(
            binding
                .cause
                .event()
                .expect("generated event capability")
                .name(),
            source,
        );
    }
}

fn components_and_system(
    inventory: &mut Inventory,
    ir: &EssIr,
    plan: &SynthesisPlan,
    layout: &Layout,
) {
    let owner = ir.system().to_string();
    selection_symbols(inventory, ir, &owner);
    let delivered = ir
        .bindings()
        .values()
        .filter(|binding| {
            plan.is_generated(CapabilityKind::BindingDelivery, &binding.name.to_string())
        })
        .collect::<Vec<_>>();
    delivery_initializers(inventory, plan, &delivered);
    let has_obligations = delivered.iter().any(|binding| {
        !plan.is_generated(
            CapabilityKind::BindingTransformation,
            &binding.name.to_string(),
        ) || binding.escalation.is_some()
    });
    let has_retry = delivered.iter().any(|binding| {
        matches!(
            binding.on_failure(),
            ess_compiler::ir::ResolvedFailure::Retry
        )
    });
    // Each component is a system dependency. The implicit standard prelude is a real generated
    // reference too: a dependency named std replaces it before any explicit path is resolved.
    inventory.helper("system dependencies", "std", &owner);
    if has_retry {
        inventory.helper("system dependencies", "core", &owner);
    }
    for fixed in ["published", "cursor"] {
        inventory.symbol("system fields", fixed, &owner, "system field");
    }
    if !delivered.is_empty() {
        inventory.symbol("system fields", "invocations", &owner, "system field");
    }
    if has_obligations {
        inventory.symbol("system fields", "obligations", &owner, "system field");
        inventory.symbol(
            "system generics",
            "Obligations",
            &owner,
            "obligation parameter",
        );
    }
    if has_retry {
        inventory.symbol("system fields", "retries", &owner, "system field");
    }
    component_scopes(inventory, ir, layout);
    let mut events: BTreeSet<_> = ir
        .components()
        .values()
        .flat_map(|component| component.publishes.iter())
        .collect();
    for binding in &delivered {
        events.insert(binding.cause.event().expect("generated event capability"));
        if let Some(event) = &binding.escalation {
            events.insert(event);
        }
        inventory.symbol(
            "binding invocations",
            &name::pascal(&binding.name.to_string()),
            &binding.name.to_string(),
            "binding variant",
        );
    }
    for (event, variant) in event_variants(ir, layout, &events) {
        inventory.symbol(
            "system events",
            &variant,
            &event.to_string(),
            "event variant",
        );
    }
    for binding in ir.bindings().values() {
        let source = binding.name.to_string();
        if plan.is_generated(CapabilityKind::BindingTransformation, &source) {
            inventory.symbol(
                "system values",
                &name::value_ident(&source),
                &source,
                "binding function",
            );
        }
        for (kind, suffix) in [
            (CapabilityKind::BindingTransformation, "Transformation"),
            (CapabilityKind::BindingEscalation, "Escalation"),
        ] {
            if plan.obligation_of(kind, &source).is_some() {
                inventory.symbol(
                    "system obligations",
                    &format!("{}{suffix}", name::pascal(&source)),
                    &source,
                    "binding obligation",
                );
            }
        }
    }
}

fn component_scopes(inventory: &mut Inventory, ir: &EssIr, layout: &Layout) {
    for component in ir.components().values() {
        let source = component.name.to_string();
        inventory.symbol(
            "system dependencies",
            &Layout::crate_ident(layout.component_package(&component.name)),
            &source,
            "component dependency",
        );
        let scope = format!("port:{source}");
        let port = name::pascal(&source);
        inventory.symbol(&scope, &port, &source, "port type");
        inventory.symbol(&scope, "PublishedEvent", &source, "outbox type");
        for helper in ["Vec", "B"] {
            inventory.helper(&scope, helper, &source);
        }
        if !component.accepts.is_empty()
            || ir
                .views()
                .values()
                .any(|view| component.owns.contains(&view.domain))
        {
            inventory.helper(&scope, "Result", &source);
        }
        let methods = format!("port methods:{source}");
        for fixed in ["new", "drain_outbox"] {
            inventory.symbol(&methods, fixed, &source, "port method");
        }
        for command in &component.accepts {
            inventory.symbol(
                &methods,
                &name::value_ident(&layout.type_name(command.name())),
                &command.to_string(),
                "command method",
            );
        }
        for view in ir
            .views()
            .values()
            .filter(|view| component.owns.contains(&view.domain))
        {
            inventory.symbol(
                &methods,
                &name::value_ident(&layout.type_name(&view.name)),
                &view.name.to_string(),
                "view method",
            );
        }
        let events = component.publishes.iter().collect();
        for (event, variant) in event_variants(ir, layout, &events) {
            inventory.symbol(
                &format!("outbox:{source}"),
                &variant,
                &event.to_string(),
                "published event variant",
            );
        }
        inventory.symbol(
            "system fields",
            &name::value_ident(&source),
            &source,
            "component field",
        );
        inventory.symbol(
            "system generics",
            &format!("{port}Behaviors"),
            &source,
            "component parameter",
        );
    }
}

fn conversions(inventory: &mut Inventory, ir: &EssIr, plan: &SynthesisPlan, layout: &Layout) {
    for conversion in ir.conversions() {
        let source = conversion_source(conversion);
        if let Some((_, to)) = mechanical_conversion(ir, conversion) {
            inventory.helper(&domain_scope(layout, to.name()), "From", &source);
        }
        if mechanical_conversion(ir, conversion).is_none()
            && plan
                .obligation_of(CapabilityKind::Conversion, &source)
                .is_some()
        {
            inventory.symbol(
                "conversion obligations",
                &format!(
                    "{}To{}Conversion",
                    name::type_fragment(&conversion.from.to_string()),
                    name::type_fragment(&conversion.to.to_string())
                ),
                &source,
                "conversion trait",
            );
        }
    }
}

fn bindings(inventory: &mut Inventory, ir: &EssIr, plan: &SynthesisPlan) {
    for binding in ir.bindings().values() {
        if !plan.is_generated(
            CapabilityKind::BindingTransformation,
            &binding.name.to_string(),
        ) {
            continue;
        }
        for mapping in &binding.mapping {
            if let ResolvedMappingValue::EventField { field, type_ref } = &mapping.value {
                if mapping.conversion.is_none() && type_ref != &mapping.target_type {
                    inventory.cause(Code::BindingAssignment, vec![binding.name.to_string(), format!("{}.{}", binding.cause.event().expect("generated event capability"), field), format!("{}.{}", binding.command, mapping.target)], format!("binding `{}` emits a plain clone of `{type_ref}` for `{}` of type `{}`; this assignment needs an explicit target representation", binding.name, mapping.target, mapping.target_type));
                }
            }
        }
    }
}

fn size_reference(reference: &ResolvedTypeRef) -> Option<&QualifiedName> {
    match reference {
        ResolvedTypeRef::Declared { name } => Some(name.name()),
        ResolvedTypeRef::Optional { of } => size_reference(of),
        ResolvedTypeRef::Primitive { .. }
        | ResolvedTypeRef::List { .. }
        | ResolvedTypeRef::Map { .. } => None,
    }
}

fn size_cycles(inventory: &mut Inventory, ir: &EssIr) {
    let mut graph: BTreeMap<&QualifiedName, Vec<(&QualifiedName, String)>> = BTreeMap::new();
    for declared in ir.types().values() {
        let edges = graph.entry(&declared.name).or_default();
        match &declared.body {
            ResolvedBody::Newtype { of, .. } => {
                if let Some(target) = size_reference(of) {
                    edges.push((target, format!("{}.value", declared.name)));
                }
            }
            ResolvedBody::Struct { fields, .. } => {
                for field in fields {
                    if let Some(target) = size_reference(&field.type_ref) {
                        edges.push((target, format!("{}.{}", declared.name, field.name)));
                    }
                }
            }
            ResolvedBody::Union { variants, .. } => {
                for (variant, reference) in variants {
                    if let Some(target) = size_reference(reference) {
                        edges.push((target, format!("{}.{variant}", declared.name)));
                    }
                }
            }
            ResolvedBody::Enum { .. } => {}
        }
    }
    // Iterative depth-first traversal: declaration depth is source data, not a Rust call stack.
    let mut done = BTreeSet::new();
    for start in graph.keys().copied() {
        if done.contains(start) {
            continue;
        }
        let mut active = BTreeMap::new();
        let mut stack = vec![(start, 0_usize, String::new())];
        active.insert(start, 0_usize);
        while let Some((node, next, _)) = stack.last_mut() {
            let edges = &graph[node];
            if *next == edges.len() {
                let (node, _, _) = stack.pop().expect("active frame");
                active.remove(node);
                done.insert(node);
                continue;
            }
            let (target, via) = &edges[*next];
            *next += 1;
            if let Some(position) = active.get(target) {
                let sources = stack[*position..]
                    .iter()
                    .map(|(node, _, _)| node.to_string())
                    .collect();
                let mut path = stack[*position + 1..]
                    .iter()
                    .map(|(_, _, via)| via.as_str())
                    .collect::<Vec<_>>();
                path.push(via);
                inventory.cause(Code::RecursiveLayout, sources, format!("by-value representation cycle {} -> {target}; Optional preserves size, while List/Map break the cycle", path.join(" -> ")));
            } else if !done.contains(target) {
                active.insert(target, stack.len());
                stack.push((target, 0, via.clone()));
            }
        }
    }
}

fn wire_fields(inventory: &mut Inventory, owner: &QualifiedName, fields: &[ResolvedField]) {
    let mut names: BTreeMap<&str, Vec<String>> = BTreeMap::new();
    for field in fields {
        names
            .entry(ess_gen::schema::wire_field_name(field))
            .or_default()
            .push(format!("{owner}.{}", field.name));
    }
    for (key, sources) in names {
        if sources.len() > 1 {
            inventory.cause(
                Code::WireCollision,
                sources,
                format!(
                    "generated codec for `{owner}` maps multiple fields to JSON member `{key}`"
                ),
            );
        }
    }
}

fn wire(
    inventory: &mut Inventory,
    ir: &EssIr,
    plan: &SynthesisPlan,
    layout: &Layout,
    command_present: impl Fn(&QualifiedName) -> bool,
) {
    for declared in ir.types().values() {
        if !plan.is_generated(CapabilityKind::DomainType, &declared.name.to_string()) {
            continue;
        }
        for prefix in ["encode", "decode"] {
            inventory.symbol(
                "wire functions",
                &format!("{prefix}_{}", super::wire::ident(&declared.name)),
                &declared.name.to_string(),
                "type codec",
            );
        }
        if let ResolvedBody::Struct { fields, .. } = &declared.body {
            wire_fields(inventory, &declared.name, fields);
        }
        if let ResolvedBody::Union { tag, .. } = &declared.body {
            debug_assert_ne!(tag, ess_gen::schema::union_content_key(tag));
        }
    }
    for (owner, fields, prefix) in ir
        .events()
        .values()
        .map(|value| (&value.name, value.fields.as_slice(), "encode_event"))
        .chain(
            ir.errors()
                .values()
                .map(|value| (&value.name, value.fields.as_slice(), "encode_error")),
        )
        .chain(
            ir.views()
                .values()
                .map(|value| (&value.name, value.fields.as_slice(), "encode_view")),
        )
    {
        inventory.symbol(
            "wire functions",
            &format!("{prefix}_{}", super::wire::ident(owner)),
            &owner.to_string(),
            "record codec",
        );
        wire_fields(inventory, owner, fields);
    }
    for command in ir
        .commands()
        .values()
        .filter(|command| command_present(&command.name))
    {
        for prefix in ["encode_command", "decode_command", "encode_outcome"] {
            inventory.symbol(
                "wire functions",
                &format!("{prefix}_{}", super::wire::ident(&command.name)),
                &command.name.to_string(),
                "command codec",
            );
        }
        wire_fields(inventory, &command.name, &command.input);
        outcome_codec_locals(inventory, ir, layout, command);
    }
}

fn outcome_codec_locals(
    inventory: &mut Inventory,
    ir: &EssIr,
    layout: &Layout,
    command: &ResolvedCommand,
) {
    let emit = Emit {
        ir,
        layout,
        domain: &ir.domain(&command.domain).name,
    };
    for outcome in &command.outcomes {
        let source = format!("{}.{}", command.name, outcome.name);
        let scope = format!("outcome codec:{source}");
        inventory.helper(&scope, "out", &source);
        for carried in items::outcome_event_fields(&emit, outcome) {
            let event = carried.event.name();
            inventory.symbol(
                &scope,
                &carried.field,
                &event.to_string(),
                "event pattern binding",
            );
            inventory.helper(
                &scope,
                &format!("encode_event_{}", super::wire::ident(event)),
                &event.to_string(),
            );
        }
        if let Some(error) = &outcome.error {
            inventory.symbol(&scope, "error", &error.to_string(), "error pattern binding");
            inventory.helper(
                &scope,
                &format!("encode_error_{}", super::wire::ident(error.name())),
                &error.to_string(),
            );
        }
    }
}

/// Additional codecs of the Web target, restricted to the commands it actually presents.
pub(crate) fn web_codecs(
    ir: &EssIr,
    plan: &SynthesisPlan,
    layout: &Layout,
    command_present: impl Fn(&QualifiedName) -> bool,
) -> Result<(), TargetFailure> {
    let mut inventory = Inventory::default();
    wire(&mut inventory, ir, plan, layout, command_present);
    let causes = inventory.finish();
    if causes.is_empty() {
        Ok(())
    } else {
        Err(TargetFailure::new(ir, Target::Web, plan, causes))
    }
}

fn selection_symbols(inventory: &mut Inventory, ir: &EssIr, owner: &str) {
    if ir
        .bindings()
        .values()
        .any(|binding| binding.selection.is_some())
    {
        for symbol in ["selection_all", "selection_any"] {
            inventory.helper("system values", symbol, owner);
        }
        for binding in ir
            .bindings()
            .values()
            .filter(|binding| crate::selection::prepared_helper(ir, binding))
        {
            inventory.symbol(
                "system values",
                &format!(
                    "{}_from_prepared",
                    name::value_ident(&binding.name.to_string())
                ),
                &binding.name.to_string(),
                "prepared selection helper",
            );
        }
    }
}
