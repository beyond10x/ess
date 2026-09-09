use crate::wire::{ArgumentSource, Callable, Command, Plan, Shape, Target, ValueContract};
use crate::{Binding, CompiledBinding, Error};
use ess_compiler::ir::{ResolvedBody, ResolvedField, ResolvedTypeRef};
use ess_compiler::EssIr;
use ess_domain::name::QualifiedName;
use ess_domain::types::{Primitive, TypeRef};
use std::collections::{BTreeMap, BTreeSet};

fn refuse(message: impl Into<String>) -> Error {
    Error(message.into())
}

fn token(value: &str) -> bool {
    value.split('-').all(|word| {
        !word.is_empty()
            && word
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
    }) && value.as_bytes().first().is_some_and(u8::is_ascii_lowercase)
}

fn identifier(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}

fn field_name(field: &ResolvedField) -> String {
    field
        .naming
        .wire
        .clone()
        .unwrap_or_else(|| field.name.clone())
}

fn primitive(primitive: Primitive) -> Result<Shape, Error> {
    match primitive {
        Primitive::String => Ok(Shape::String),
        Primitive::Boolean => Ok(Shape::Boolean),
        Primitive::Integer => Ok(Shape::Integer),
        other => Err(refuse(format!("unsupported CLI primitive `{other}`"))),
    }
}

fn fields_shape(
    model: &EssIr,
    fields: &[ResolvedField],
    stack: &mut BTreeSet<String>,
) -> Result<Shape, Error> {
    let mut resolved = BTreeMap::new();
    for field in fields {
        let shape = resolved_ref(model, &field.type_ref, stack)?;
        if resolved.insert(field_name(field), shape).is_some() {
            return Err(refuse("duplicate wire field identity"));
        }
    }
    Ok(Shape::Struct { fields: resolved })
}

fn named(
    model: &EssIr,
    name: &QualifiedName,
    stack: &mut BTreeSet<String>,
) -> Result<Shape, Error> {
    if stack.len() >= 32 || !stack.insert(name.to_string()) {
        return Err(refuse(format!(
            "recursive or overly deep CLI type `{name}` is unsupported"
        )));
    }
    let declared = model
        .types()
        .get(name)
        .ok_or_else(|| refuse(format!("unresolved CLI type `{name}`")))?;
    let shape = match &declared.body {
        ResolvedBody::Newtype { of, invariants } if invariants.is_empty() => {
            resolved_ref(model, of, stack)
        }
        ResolvedBody::Struct { fields, invariants } if invariants.is_empty() => {
            fields_shape(model, fields, stack)
        }
        ResolvedBody::Enum { variants } => Ok(Shape::Enum {
            variants: variants.clone(),
        }),
        _ => Err(refuse(format!(
            "CLI type `{name}` has unsupported invariants or union semantics"
        ))),
    };
    stack.remove(&name.to_string());
    shape
}

fn resolved_ref(
    model: &EssIr,
    reference: &ResolvedTypeRef,
    stack: &mut BTreeSet<String>,
) -> Result<Shape, Error> {
    match reference {
        ResolvedTypeRef::Primitive { name } => primitive(*name),
        ResolvedTypeRef::Declared { name } => named(model, name.name(), stack),
        ResolvedTypeRef::Optional { of } => Ok(Shape::Optional {
            of: Box::new(resolved_ref(model, of, stack)?),
        }),
        ResolvedTypeRef::List { of } => Ok(Shape::List {
            of: Box::new(resolved_ref(model, of, stack)?),
        }),
        ResolvedTypeRef::Map {
            key: Primitive::String,
            value,
        } => Ok(Shape::Map {
            value: Box::new(resolved_ref(model, value, stack)?),
        }),
        ResolvedTypeRef::Map { .. } => Err(refuse("CLI maps require String keys")),
    }
}

fn type_shape(model: &EssIr, reference: &TypeRef) -> Result<Shape, Error> {
    match reference {
        TypeRef::Primitive(name) => primitive(*name),
        TypeRef::Named(name) => named(model, name, &mut BTreeSet::new()),
        TypeRef::Optional(of) => Ok(Shape::Optional {
            of: Box::new(type_shape(model, of)?),
        }),
        TypeRef::List(of) => Ok(Shape::List {
            of: Box::new(type_shape(model, of)?),
        }),
        TypeRef::Map(Primitive::String, value) => Ok(Shape::Map {
            value: Box::new(type_shape(model, value)?),
        }),
        TypeRef::Map(..) => Err(refuse("CLI maps require String keys")),
    }
}

fn contract(model: &EssIr, text: &str) -> Result<ValueContract, Error> {
    let reference = TypeRef::parse(text).map_err(|e| refuse(e.to_string()))?;
    Ok(ValueContract {
        type_ref: reference.to_string(),
        shape: type_shape(model, &reference)?,
    })
}

fn input_fields<'a>(model: &'a EssIr, text: &str) -> Result<&'a [ResolvedField], Error> {
    let name = QualifiedName::new(text).map_err(|e| refuse(e.to_string()))?;
    match &model
        .types()
        .get(&name)
        .ok_or_else(|| refuse(format!("unresolved input type `{text}`")))?
        .body
    {
        ResolvedBody::Struct { fields, .. } => Ok(fields),
        _ => Err(refuse("CLI input must reference a declared struct")),
    }
}

fn validate_target(
    model: &EssIr,
    target: &Target,
    input: &[ResolvedField],
    result: &Shape,
) -> Result<(), Error> {
    match target {
        Target::Local { owner, action } => {
            QualifiedName::new(owner).map_err(|e| refuse(e.to_string()))?;
            if !identifier(action) {
                return Err(refuse("invalid local action identifier"));
            }
        }
        Target::ServiceForward { owner, operation } => {
            let component = model
                .components()
                .values()
                .find(|c| c.name.to_string() == *owner)
                .ok_or_else(|| refuse(format!("unresolved service owner `{owner}`")))?;
            let name = QualifiedName::new(operation).map_err(|e| refuse(e.to_string()))?;
            let expected = if let Some(command) = model.commands().get(&name) {
                if !component.accepts.iter().any(|h| h.name() == &name) {
                    return Err(refuse(format!(
                        "service `{owner}` does not accept `{operation}`"
                    )));
                }
                &command.input
            } else if let Some(view) = model.views().get(&name) {
                if !component.owns.contains(&view.domain) {
                    return Err(refuse("service does not own view domain"));
                }
                let shape = fields_shape(model, &view.fields, &mut BTreeSet::new())?;
                if result != &shape
                    && result
                        != &(Shape::List {
                            of: Box::new(shape),
                        })
                {
                    return Err(refuse(
                        "service view result does not match the declared row shape",
                    ));
                }
                &view.params
            } else {
                return Err(refuse(format!(
                    "unresolved service operation `{operation}`"
                )));
            };
            let signature = |fields: &[ResolvedField]| {
                fields
                    .iter()
                    .map(|f| (field_name(f), f.type_ref.clone()))
                    .collect::<BTreeMap<_, _>>()
            };
            if signature(input) != signature(expected) {
                return Err(refuse(
                    "service input differs from operation input/parameters",
                ));
            }
        }
        Target::Dynamic {
            owner,
            operation_field,
            schema_field,
            payload_field,
        } => {
            QualifiedName::new(owner).map_err(|e| refuse(e.to_string()))?;
            let fields = [operation_field, schema_field, payload_field];
            if fields.into_iter().collect::<BTreeSet<_>>().len() != 3 {
                return Err(refuse("dynamic selector/payload fields must be distinct"));
            }
            for name in fields {
                let field = input
                    .iter()
                    .find(|f| f.name == *name)
                    .ok_or_else(|| refuse(format!("missing dynamic field `{name}`")))?;
                if resolved_ref(model, &field.type_ref, &mut BTreeSet::new())? != Shape::String {
                    return Err(refuse(
                        "dynamic selector and payload fields must be required String values",
                    ));
                }
                if field_name(field) != *name {
                    return Err(refuse(
                        "dynamic selector fields cannot rename their wire identity",
                    ));
                }
            }
        }
    }
    Ok(())
}

fn reserve(flags: &mut BTreeSet<String>, flag: &str) -> Result<(), Error> {
    if !token(flag) || !flags.insert(flag.to_owned()) {
        return Err(refuse(format!("invalid or conflicting CLI flag `{flag}`")));
    }
    Ok(())
}

fn validate_command(
    model: &EssIr,
    command: &mut Command,
    callable: &Callable,
    global_flags: &BTreeSet<String>,
) -> Result<(), Error> {
    let fields = callable
        .input
        .as_ref()
        .map_or(Ok(&[][..]), |input| input_fields(model, &input.type_ref))?;
    let mut mapped = BTreeSet::new();
    let mut flags = global_flags.clone();
    let mut positions = BTreeMap::new();
    let mut stdin_sources = 0;
    for argument in &mut command.arguments {
        let field = fields
            .iter()
            .find(|f| f.name == argument.field)
            .ok_or_else(|| refuse(format!("unresolved argument field `{}`", argument.field)))?;
        if !mapped.insert(field.name.clone()) {
            return Err(refuse("input field is mapped more than once"));
        }
        let shape = resolved_ref(model, &field.type_ref, &mut BTreeSet::new())?;
        match &argument.source {
            ArgumentSource::Option { long } => reserve(&mut flags, long)?,
            ArgumentSource::Positional { index } => {
                if *index == 0 || positions.insert(*index, shape.optional()).is_some() {
                    return Err(refuse("duplicate or zero positional index"));
                }
            }
            ArgumentSource::Protected {
                file,
                stdin,
                hidden_tty,
            } => {
                stdin_sources += 1;
                if shape.required() != &Shape::String {
                    return Err(refuse("protected input must be a String"));
                }
                for flag in [file, stdin, hidden_tty] {
                    reserve(&mut flags, flag)?;
                }
            }
            ArgumentSource::Document {
                inline,
                file,
                stdin,
            } => {
                stdin_sources += 1;
                if shape.required() != &Shape::String {
                    return Err(refuse("document input must be a String"));
                }
                for flag in [inline, file, stdin] {
                    reserve(&mut flags, flag)?;
                }
            }
        }
        argument.field = field_name(field);
    }
    if mapped.len() != fields.len() {
        return Err(refuse(
            "every input field needs exactly one argument source",
        ));
    }
    if stdin_sources > 1 {
        return Err(refuse(
            "only one stdin-consuming field is supported per command",
        ));
    }
    let mut optional_seen = false;
    for (offset, (index, optional)) in positions.iter().enumerate() {
        if *index != offset + 1 || (optional_seen && !optional) {
            return Err(refuse(
                "positionals must be consecutive, with optional values last",
            ));
        }
        optional_seen |= optional;
    }
    Ok(())
}

/// Resolve all used ESS types and owners, then validate the supported CLI projection.
pub fn compile(model: &EssIr, binding: &Binding) -> Result<CompiledBinding, Error> {
    if !token(&binding.binary) {
        return Err(refuse("invalid CLI binary name"));
    }
    if ["build", "deps", "examples", "incremental"].contains(&binding.binary.as_str()) {
        return Err(refuse(
            "CLI binary name conflicts with a Cargo output directory",
        ));
    }
    if binding.commands.is_empty() || binding.callables.is_empty() {
        return Err(refuse("a CLI needs commands and callables"));
    }
    let mut flags = BTreeSet::from(["help".to_owned(), "version".to_owned()]);
    for flag in [
        &binding.globals.config,
        &binding.globals.state,
        &binding.globals.output,
    ] {
        reserve(&mut flags, flag)?;
    }
    let mut callables = BTreeMap::new();
    let mut obligations = Vec::new();
    for (key, declared) in &binding.callables {
        if !identifier(key) {
            return Err(refuse("invalid callable key"));
        }
        let fields = declared
            .input
            .as_ref()
            .map_or(Ok(&[][..]), |input| input_fields(model, input))?;
        let input = declared
            .input
            .as_ref()
            .map(|input| contract(model, input))
            .transpose()?;
        let result = contract(model, &declared.result)?;
        validate_target(model, &declared.target, fields, &result.shape)?;
        let mut errors = BTreeMap::new();
        for (code, reference) in &declared.errors {
            if !identifier(code) || code.starts_with("cli_") {
                return Err(refuse("invalid or reserved error code"));
            }
            errors.insert(code.clone(), contract(model, reference)?);
        }
        obligations.push(format!("handler:{key}: implement the owner-qualified callable and its declared result/error contract"));
        if matches!(&declared.target, Target::Dynamic { .. }) {
            obligations.push(format!("dynamic-validator:{key}: resolve operation/schema identity and validate native input, result and errors; absent validator refuses"));
        }
        callables.insert(
            key.clone(),
            Callable {
                target: declared.target.clone(),
                input,
                result,
                errors,
            },
        );
    }
    let mut commands = binding.commands.clone();
    let mut paths = BTreeSet::new();
    for command in &mut commands {
        for path in std::iter::once(&command.path).chain(&command.aliases) {
            if !(1..=2).contains(&path.len())
                || path
                    .iter()
                    .any(|p| !token(p) || ["help", "version", "completions"].contains(&p.as_str()))
                || !paths.insert(path.clone())
            {
                return Err(refuse(format!(
                    "invalid or conflicting command path `{}`",
                    path.join(" ")
                )));
            }
        }
        let callable = callables
            .get(&command.callable)
            .ok_or_else(|| refuse("command references an undeclared callable"))?;
        validate_command(model, command, callable, &flags)?;
        command.aliases.sort();
    }
    for path in &paths {
        if path.len() == 2 && paths.contains(&vec![path[0].clone()]) {
            return Err(refuse("a root command cannot also be a group"));
        }
    }
    commands.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(CompiledBinding(Plan {
        format: "ess-cli-plan/1".to_owned(),
        binary: binding.binary.clone(),
        about: binding.about.clone(),
        globals: binding.globals.clone(),
        callables,
        commands,
        obligations,
    }))
}
