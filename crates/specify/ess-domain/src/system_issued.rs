//! A command the system issues itself (`ess/27`, beyond10x/ess#542,
//! `docs/design/system-issued-commands.md`).
//!
//! An actor may be declared the system itself, `system: true`. A command is **system-issued**
//! exactly when it has at least one grant and every actor that grants it is a system actor; a
//! command granted to a system actor and to any other actor stays an ordinary, driven command, and
//! one granted to no actor is unchanged. So grants stay the one place that says who issues a
//! command, and no second key can disagree with them.
//!
//! A system-issued command may declare `await_within:`, how soon the system takes its step. What
//! is refused here, each at its own site:
//!
//! | rule | code |
//! |---|---|
//! | `system: true` below `ess/27` | `unsupported_format_version` |
//! | `system: true` beside `attributes:` | `conflicting_declaration` |
//! | `await_within:` below `ess/27` | `unsupported_format_version` |
//! | `await_within:` on a command that is not system-issued | `conflicting_declaration`, naming the grants that make it driven |
//! | a binding whose `invoke:` names a system-issued command | `conflicting_declaration` |

use std::collections::BTreeSet;

use ess_primitives::error::{
    ConstructKind, ConstructRef, ValidationCode, ValidationError, ValidationErrors,
};

use crate::actor::ActorSpec;
use crate::name::QualifiedName;
use crate::system::FormatVersion;

/// Whether `command` is system-issued among `actors`: granted by at least one, and by none that
/// is not the system.
pub fn is_system_issued<'a>(
    actors: impl IntoIterator<Item = &'a ActorSpec>,
    command: &QualifiedName,
) -> bool {
    let mut granted = false;
    for actor in actors {
        if actor.may_invoke(command) {
            if !actor.system {
                return false;
            }
            granted = true;
        }
    }
    granted
}

/// Every refusal of the table in the [module documentation](self).
pub(crate) fn validate_specification(spec: &crate::Specification) -> ValidationErrors {
    let format = spec.system().format;
    let mut errors = ValidationErrors::new();
    for actor in spec.actors().values() {
        errors.extend(actor.validate_system(format));
    }
    for command in spec.commands().values() {
        if command.await_within.is_none() {
            continue;
        }
        let site =
            ConstructRef::new(ConstructKind::Command, command.name.to_string()).key("await_within");
        if format.major() < FormatVersion::V27.major() {
            errors.push(
                ValidationError::at(
                    site,
                    ValidationCode::UnsupportedFormatVersion,
                    "`await_within:` on a command requires specification format ess/27",
                )
                .with_hint("write `format: ess/27` on the source that declares the system"),
            );
            continue;
        }
        if spec.is_system_issued(&command.name) {
            continue;
        }
        let driving: Vec<String> = spec
            .actors()
            .values()
            .filter(|actor| actor.may_invoke(&command.name) && !actor.system)
            .map(|actor| format!("`{}`", actor.name))
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        let why = match driving.as_slice() {
            [] => "no actor grants it".to_owned(),
            [one] => format!("it is granted to {one}, which is not the system"),
            many => format!(
                "it is granted to {}, which are not the system",
                many.join(", ")
            ),
        };
        errors.push(
            ValidationError::at(
                site,
                ValidationCode::ConflictingDeclaration,
                format!(
                    "`{}` declares how soon the system issues it, and is not system-issued: {why}",
                    command.name
                ),
            )
            .with_hint(
                "`await_within:` belongs on a command every grant of which is an actor declared \
                 `system: true`",
            ),
        );
    }
    for binding in spec.bindings().values() {
        if !spec.is_system_issued(&binding.command) {
            continue;
        }
        errors.push(
            ValidationError::at(
                ConstructRef::new(ConstructKind::Binding, binding.name.to_string()).key("invoke"),
                ValidationCode::ConflictingDeclaration,
                format!(
                    "`{}` invokes `{}`, a command the system issues itself",
                    binding.name, binding.command
                ),
            )
            .with_hint(
                "a binding's invocation is already the system acting; two causes of one command \
                 are not admitted",
            ),
        );
    }
    errors
}
