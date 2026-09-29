//! Adversary pass 2 for beyond10x/ess#230: an identity-carried relation beside a field-carried one
//! to the same entity (only-the-first / equal-witness classes), and a creating branch guarded on
//! the referenced row while the update reads through the identity.
#![allow(
    clippy::too_many_lines,
    clippy::missing_panics_doc,
    clippy::let_and_return
)]

use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::scenario::{ScenarioStep, ScenarioValue};
use ess_conformance::ScenarioId;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const SYSTEM: &str = include_str!("fixtures/relation-via-identity/system.yaml");
const PROVISIONING: &str = include_str!("fixtures/relation-via-identity/provisioning.yaml");

const BINDING_TWO_CARRIERS: &str = "\
domain: demo.binding
summary: An identity keyed by a user's id, naming a second user as its backup.
types:
  - {name: demo.binding.Subject, kind: newtype, of: String}
entities:
  - name: demo.binding.Identity
    identity: {name: user_id, type: demo.provisioning.UserId}
    fields:
      - {name: subject, type: demo.binding.Subject}
      - {name: backup, type: demo.provisioning.UserId}
    relations:
      - {name: user, kind: references, target: demo.provisioning.User, cardinality: one, via: user_id}
      - {name: backup_user, kind: references, target: demo.provisioning.User, cardinality: one, via: backup}
    lifecycle: {initial: Bound, states: [Bound], terminal: [Bound]}
events:
  - name: demo.binding.IdentityBound
    fields:
      - {name: user_id, type: demo.provisioning.UserId}
      - {name: subject, type: demo.binding.Subject}
  - name: demo.binding.IdentityRebound
    fields:
      - {name: user_id, type: demo.provisioning.UserId}
      - {name: primary_email, type: demo.provisioning.Email}
      - {name: backup_email, type: demo.provisioning.Email}
commands:
  - name: demo.binding.BindIdentity
    input:
      - {name: user_id, type: demo.provisioning.UserId}
      - {name: subject, type: demo.binding.Subject}
      - {name: backup, type: demo.provisioning.UserId}
    outcomes:
      - name: bound
        creates: demo.binding.Identity
        instance: user_id
        emits: [demo.binding.IdentityBound]
        payload:
          demo.binding.IdentityBound: {user_id: input.user_id, subject: input.subject}
        sets:
          subject: input.subject
          backup: input.backup
  - name: demo.binding.RebindIdentity
    input:
      - {name: user_id, type: demo.provisioning.UserId}
    outcomes:
      - name: rebound
        updates: demo.binding.Identity
        instance: user_id
        emits: [demo.binding.IdentityRebound]
        payload:
          demo.binding.IdentityRebound: {user_id: input.user_id, primary_email: {related: {via: user_id, field: email}}, backup_email: {related: {via: backup, field: email}}}
views:
  - name: demo.binding.Identities
    source: demo.binding.Identity
    consistency: read_your_writes
    fields:
      - {name: user_id, type: demo.provisioning.UserId}
      - {name: subject, type: demo.binding.Subject}
      - {name: state, type: demo.binding.Identity.State}
";

const BINDING_GUARDED_CREATE: &str = "\
domain: demo.binding
summary: An identity bound only to an existing user; a rebind reads that user's email.
types:
  - {name: demo.binding.Subject, kind: newtype, of: String}
entities:
  - name: demo.binding.Identity
    identity: {name: user_id, type: demo.provisioning.UserId}
    fields:
      - {name: subject, type: demo.binding.Subject}
    relations:
      - {name: user, kind: references, target: demo.provisioning.User, cardinality: one, via: user_id}
    lifecycle: {initial: Bound, states: [Bound], terminal: [Bound]}
errors:
  - {name: demo.binding.NoUser, summary: No user has this id., fields: []}
events:
  - name: demo.binding.IdentityBound
    fields:
      - {name: user_id, type: demo.provisioning.UserId}
      - {name: subject, type: demo.binding.Subject}
  - name: demo.binding.IdentityRebound
    fields:
      - {name: user_id, type: demo.provisioning.UserId}
      - {name: email, type: demo.provisioning.Email}
commands:
  - name: demo.binding.BindIdentity
    input:
      - {name: user_id, type: demo.provisioning.UserId}
      - {name: subject, type: demo.binding.Subject}
    outcomes:
      - name: no-user
        when_related: {via: input.user_id, exists: false}
        error: demo.binding.NoUser
      - name: bound
        creates: demo.binding.Identity
        instance: user_id
        emits: [demo.binding.IdentityBound]
        payload:
          demo.binding.IdentityBound: {user_id: input.user_id, subject: input.subject}
        sets:
          subject: input.subject
  - name: demo.binding.RebindIdentity
    input:
      - {name: user_id, type: demo.provisioning.UserId}
    outcomes:
      - name: rebound
        updates: demo.binding.Identity
        instance: user_id
        emits: [demo.binding.IdentityRebound]
        payload:
          demo.binding.IdentityRebound: {user_id: input.user_id, email: {related: {via: user_id, field: email}}}
views:
  - name: demo.binding.Identities
    source: demo.binding.Identity
    consistency: read_your_writes
    fields:
      - {name: user_id, type: demo.provisioning.UserId}
      - {name: subject, type: demo.binding.Subject}
      - {name: state, type: demo.binding.Identity.State}
";

fn compiled(binding: &str) -> EssIr {
    let parsed = [
        ("system.yaml", SYSTEM),
        ("provisioning.yaml", PROVISIONING),
        ("binding.yaml", binding),
    ]
    .map(|(label, text)| {
        let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{label}: {error}"));
        (Source::new(label), raw)
    });
    let spec = Specification::assemble(parsed)
        .unwrap_or_else(|errors| panic!("the model validates:\n{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

/// Every `RegisterUser`: the captured instance name and the email it was sent.
fn users(steps: &[ScenarioStep]) -> BTreeMap<String, ScenarioValue> {
    let mut out = BTreeMap::new();
    let mut email = None;
    for step in steps {
        match step {
            ScenarioStep::ExecuteCommand { command, input, .. }
                if command.to_string() == "demo.provisioning.RegisterUser" =>
            {
                email = input.get("email").cloned();
            }
            ScenarioStep::CaptureInstance {
                instance, entity, ..
            } if entity.to_string() == "demo.provisioning.User" => {
                out.insert(
                    instance.to_string(),
                    email.take().expect("a capture follows"),
                );
            }
            _ => {}
        }
    }
    out
}

/// The instance `BindIdentity` sends as `input`.
fn bind_input(steps: &[ScenarioStep], input_name: &str) -> String {
    steps
        .iter()
        .find_map(|step| match step {
            ScenarioStep::ExecuteCommand { command, input, .. }
                if command.to_string() == "demo.binding.BindIdentity" =>
            {
                match input.get(input_name) {
                    Some(ScenarioValue::Instance { instance }) => Some(instance.to_string()),
                    other => panic!("`{input_name}` names an arranged user: {other:?}"),
                }
            }
            _ => None,
        })
        .expect("the scenario binds an identity")
}

fn asserted(
    steps: &[ScenarioStep],
    event: &str,
    field: &str,
) -> Option<ess_primitives::node::Node> {
    steps
        .iter()
        .find_map(|step| match step {
            ScenarioStep::ExpectEvent {
                event: seen,
                payload,
                ..
            } if seen.to_string() == event => Some(payload.get(field).cloned()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("`{event}` is expected"))
}

fn literal(value: &ScenarioValue) -> ess_primitives::node::Node {
    match value {
        ScenarioValue::Literal { value } => value.clone(),
        other => panic!("a literal email: {other:?}"),
    }
}

#[test]
fn an_identity_carrier_and_a_field_carrier_to_one_entity_are_each_witnessed_apart() {
    let ir = compiled(BINDING_TWO_CARRIERS);
    let result = ess_conformance::synthesize::synthesize(&ir);
    let id = ScenarioId::parse("demo.binding.RebindIdentity/outcome/rebound").unwrap();
    let scenario = result.suite.scenarios.get(&id).unwrap_or_else(|| {
        panic!(
            "the rebind reading through both carriers is synthesised: {:#?}",
            result.refusals
        )
    });
    let steps = &scenario.steps;
    let users = users(steps);
    let primary = bind_input(steps, "user_id");
    let backup = bind_input(steps, "backup");
    assert_ne!(
        primary, backup,
        "the identity and the backup name different users, or a swapped carrier passes"
    );
    let primary_email = literal(&users[&primary]);
    let backup_email = literal(&users[&backup]);
    assert_ne!(
        primary_email, backup_email,
        "the two referenced users hold different emails"
    );
    assert_eq!(
        asserted(steps, "demo.binding.IdentityRebound", "primary_email"),
        Some(primary_email),
        "the read through the identity asserts the identity's user"
    );
    assert_eq!(
        asserted(steps, "demo.binding.IdentityRebound", "backup_email"),
        Some(backup_email),
        "the read through the field asserts the backup user"
    );
}

/// The relation declared on the other side as well — `User` names the identity keyed by its own
/// generated id — and a user command reading the identity's `subject` through it.
fn provisioning_reading_the_identity() -> String {
    let replace = |text: &str, from: &str, to: &str| {
        let out = text.replace(from, to);
        assert_ne!(out, text, "`{from}` is in the fixture");
        out
    };
    let text = replace(
        PROVISIONING,
        "      - {name: email, type: demo.provisioning.Email}\n    lifecycle:",
        "      - {name: email, type: demo.provisioning.Email}\n    relations:\n      - {name: identity, kind: references, target: demo.binding.Identity, cardinality: one, via: user_id}\n    lifecycle:",
    );
    let text = replace(
        &text,
        "    may: [demo.provisioning.RegisterUser,",
        "    may: [demo.provisioning.RegisterUser, demo.provisioning.TouchUser,",
    );
    let text = replace(
        &text,
        "commands:\n",
        "  - name: demo.provisioning.UserTouched\n    fields:\n      - {name: user_id, type: demo.provisioning.UserId}\n      - {name: subject, type: demo.binding.Subject}\ncommands:\n  - name: demo.provisioning.TouchUser\n    input:\n      - {name: user_id, type: demo.provisioning.UserId}\n    outcomes:\n      - name: touched\n        updates: demo.provisioning.User\n        instance: user_id\n        emits: [demo.provisioning.UserTouched]\n        payload:\n          demo.provisioning.UserTouched: {user_id: input.user_id, subject: {related: {via: user_id, field: subject}}}\n",
    );
    text
}

#[test]
fn a_read_through_a_generated_identity_is_arranged_or_refused_never_dropped() {
    let provisioning = provisioning_reading_the_identity();
    let binding = include_str!("fixtures/relation-via-identity/binding.yaml");
    let parsed = [
        ("system.yaml", SYSTEM),
        ("provisioning.yaml", provisioning.as_str()),
        ("binding.yaml", binding),
    ]
    .map(|(label, text)| {
        let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{label}: {error}"));
        (Source::new(label), raw)
    });
    let spec = Specification::assemble(parsed)
        .unwrap_or_else(|errors| panic!("the other-side read validates:\n{errors}"));
    let ir = compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"));
    let result = ess_conformance::synthesize::synthesize(&ir);
    let id = ScenarioId::parse("demo.provisioning.TouchUser/outcome/touched").unwrap();
    let Some(scenario) = result.suite.scenarios.get(&id) else {
        assert!(
            result
                .refusals
                .iter()
                .any(|refusal| format!("{refusal:?}").contains("TouchUser")),
            "`{id}` is neither synthesised nor refused by name: {:#?}",
            result.refusals
        );
        return;
    };
    let steps = &scenario.steps;
    let bound_subject = steps.iter().find_map(|step| match step {
        ScenarioStep::ExecuteCommand { command, input, .. }
            if command.to_string() == "demo.binding.BindIdentity" =>
        {
            input.get("subject").cloned()
        }
        _ => None,
    });
    let commands: Vec<String> = steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand { command, .. } => Some(command.to_string()),
            _ => None,
        })
        .collect();
    let Some(bound_subject) = bound_subject else {
        panic!(
            "`{id}` reads the identity row keyed by the user and arranges none: commands {commands:?}, \
             asserted subject {:?}, refusals {:#?}",
            asserted(steps, "demo.provisioning.UserTouched", "subject"),
            result.refusals
        );
    };
    assert_eq!(
        asserted(steps, "demo.provisioning.UserTouched", "subject"),
        Some(literal(&bound_subject)),
        "the scenario asserts the subject of the identity keyed by the user"
    );
}

#[test]
fn a_guard_on_the_creating_branch_does_not_stop_the_update_reading_through_the_identity() {
    let ir = compiled(BINDING_GUARDED_CREATE);
    let result = ess_conformance::synthesize::synthesize(&ir);
    let id = ScenarioId::parse("demo.binding.RebindIdentity/outcome/rebound").unwrap();
    let scenario = result.suite.scenarios.get(&id).unwrap_or_else(|| {
        panic!(
            "the unguarded rebind is synthesised although only the bind is guarded: {:#?}",
            result.refusals
        )
    });
    let steps = &scenario.steps;
    let users = users(steps);
    let bound = bind_input(steps, "user_id");
    let email = literal(&users[&bound]);
    let others: Vec<_> = users
        .iter()
        .filter(|(name, _)| **name != bound)
        .map(|(_, value)| literal(value))
        .collect();
    assert!(
        !others.is_empty() && others.iter().all(|other| *other != email),
        "decoy users hold other emails: {users:?}"
    );
    assert_eq!(
        asserted(steps, "demo.binding.IdentityRebound", "email"),
        Some(email),
        "the rebind asserts the email of the user the identity is keyed by"
    );
}
