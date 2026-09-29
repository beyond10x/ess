//! A relation carried by the entity's own identity field (beyond10x/ess#230).
//!
//! An identity keyed by `user_id` names the user with the same `user_id` and outlives it:
//! `{name: user, kind: references, target: demo.provisioning.User, cardinality: one, via: user_id}`.
//! The model validates, the relation is carried by the identity in every place that asks which
//! field carries what, and a `references` is never taken for an ownership: synthesis does not
//! arrange a user row before an identity row, because the referenced row may be absent.

use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::scenario::{ScenarioStep, ScenarioValue};
use ess_conformance::{ConformanceSuite, ScenarioId};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_gen::artifact::run;
use ess_gen::docs::Docs;
use ess_gen::schema::JsonSchema;
use serde_json::{json, Value};

/// The model, one document per domain, as the issue's two namespaces require.
const MODEL: [(&str, &str); 3] = [
    (
        "system.yaml",
        include_str!("fixtures/relation-via-identity/system.yaml"),
    ),
    (
        "provisioning.yaml",
        include_str!("fixtures/relation-via-identity/provisioning.yaml"),
    ),
    (
        "binding.yaml",
        include_str!("fixtures/relation-via-identity/binding.yaml"),
    ),
];
const BOUND: &str = "demo.binding.BindIdentity/outcome/bound";
const REBOUND: &str = "demo.binding.RebindIdentity/outcome/rebound";

fn ir(model: [(&str, &str); 3]) -> EssIr {
    let parsed = model.map(|(label, text)| {
        let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{label}: {error}"));
        (Source::new(label), raw)
    });
    let spec = Specification::assemble(parsed)
        .unwrap_or_else(|errors| panic!("the model validates:\n{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn suite(ir: &EssIr) -> ConformanceSuite {
    let result = ess_conformance::synthesize::synthesize(ir);
    assert!(result.refusals.is_empty(), "{:#?}", result.refusals);
    result.suite
}

/// The commands a scenario runs, in order, as plain names.
fn commands(suite: &ConformanceSuite, id: &str) -> Vec<String> {
    suite.scenarios[&ScenarioId::parse(id).unwrap()]
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand { command, .. } => Some(command.to_string()),
            _ => None,
        })
        .collect()
}

fn identity_handle(ir: &EssIr) -> &ess_compiler::ir::EntityHandle {
    ir.projections()
        .into_keys()
        .find(|handle| handle.name().to_string() == "demo.binding.Identity")
        .expect("the identity entity has a view")
}

#[test]
fn the_issue_model_validates_and_the_identity_carries_the_relation() {
    let ir = ir(MODEL);
    let carried = ir.relations_carried_by(identity_handle(&ir).name());
    let [(field, relation)] = carried.iter().collect::<Vec<_>>()[..] else {
        panic!("one carried relation, not {carried:?}");
    };
    assert_eq!(*field, "user_id", "the identity is the carrying field");
    assert_eq!(relation.relation.name, "user");
    assert!(
        ir.owner_of(identity_handle(&ir)).is_none(),
        "a `references` carried by the identity is not an ownership"
    );
}

#[test]
fn synthesis_does_not_arrange_the_referenced_row_as_an_owner() {
    // `references` means the user may be absent: an identity row is created on its own, and the
    // scenario that binds one runs exactly that command.
    let suite = suite(&ir(MODEL));
    assert_eq!(
        commands(&suite, BOUND),
        vec!["demo.binding.BindIdentity".to_owned()],
        "binding an identity arranges no user"
    );
    // An update of the identity arranges the identity it acts on, and still no user.
    let rebound = commands(&suite, REBOUND);
    assert_eq!(
        rebound,
        vec![
            "demo.binding.BindIdentity".to_owned(),
            "demo.binding.RebindIdentity".to_owned()
        ],
        "arranging an identity arranges no user"
    );
    // The update is told apart from the arrangement: the subject it writes is not the one the
    // arranged row already holds, and the row it names is the one arranged.
    let inputs = inputs(&suite, REBOUND);
    let [bind, rebind] = &inputs[..] else {
        panic!("two commands, not {inputs:?}");
    };
    assert_ne!(
        bind.get("subject"),
        rebind.get("subject"),
        "an equal subject would not show the update wrote it"
    );
    assert!(
        matches!(rebind.get("user_id"), Some(ScenarioValue::Instance { .. })),
        "the update names the arranged identity: {rebind:?}"
    );
}

/// The input each command of a scenario was handed, in order.
fn inputs(suite: &ConformanceSuite, id: &str) -> Vec<BTreeMap<String, ScenarioValue>> {
    suite.scenarios[&ScenarioId::parse(id).unwrap()]
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand { input, .. } => Some(input.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn the_entity_schema_states_the_relation_on_the_identity_property() {
    let ir = ir(MODEL);
    let published = run(&JsonSchema, &ir).expect("no two artifacts claim one path");
    let document: Value = serde_json::from_str(
        &published["schema/entities/demo.binding.Identity.schema.json"].contents,
    )
    .expect("the entity document is JSON");
    assert_eq!(
        document["properties"]["user_id"]["x-ess-relation"],
        json!({
            "name": "user",
            "kind": "references",
            "source": "demo.binding.Identity",
            "target": "demo.provisioning.User",
            "cardinality": "one",
            "via": "user_id",
        }),
        "the identity property carries the whole relation"
    );
    let user: Value = serde_json::from_str(
        &published["schema/entities/demo.provisioning.User.schema.json"].contents,
    )
    .expect("the user document is JSON");
    assert!(
        user["properties"]["user_id"]["x-ess-relation"].is_null(),
        "the user's own identity, of the same name and type, carries nothing: the relation is \
         carried by the source of a `references`"
    );
}

#[test]
fn the_generated_docs_say_the_identity_carries_the_relation() {
    let ir = ir(MODEL);
    let published = run(&Docs, &ir).expect("no two artifacts claim one path");
    let sentence = "as `user`, carried by `Identity.user_id`.";
    assert!(
        published
            .values()
            .any(|artifact| artifact.contents.contains(sentence)),
        "a docs page states the relation and its carrying identity: {:?}",
        published.keys().collect::<Vec<_>>()
    );
}

/// The fixture with the user's email read through the identity, by the update's event through
/// `rebind` and, where `bind` names a `via`, by the creating branch's event too.
///
/// `UserId` is the identity type of both entities, so the type alone names two; the relation the
/// identity carries is what says which. Without it the read is refused as ambiguous.
fn reading_through_identity(
    with_relation: bool,
    bind: Option<&str>,
    rebind: &str,
) -> [(&'static str, String); 3] {
    let replace = |text: &str, from: &str, to: &str| {
        let out = text.replace(from, to);
        assert_ne!(out, text, "`{from}` is in the fixture");
        out
    };
    let mut binding = MODEL[2].1.to_owned();
    for event in ["IdentityBound", "IdentityRebound"] {
        binding = replace(
            &binding,
            &format!(
                "  - name: demo.binding.{event}\n    fields:\n      - {{name: user_id, type: \
                 demo.provisioning.UserId}}\n"
            ),
            &format!(
                "  - name: demo.binding.{event}\n    fields:\n      - {{name: user_id, type: \
                 demo.provisioning.UserId}}\n      - {{name: email, type: \
                 demo.provisioning.Email}}\n"
            ),
        );
    }
    binding = replace(
        &binding,
        "demo.binding.IdentityRebound: {user_id: input.user_id, subject: input.subject}",
        &format!(
            "demo.binding.IdentityRebound: {{user_id: input.user_id, subject: input.subject, \
             email: {{related: {{via: {rebind}, field: email}}}}}}"
        ),
    );
    let bound = match bind {
        Some(via) => format!("{{related: {{via: {via}, field: email}}}}"),
        // The creating branch then publishes an email it takes from nowhere related; an input.
        None => "input.email".to_owned(),
    };
    binding = replace(
        &binding,
        "demo.binding.IdentityBound: {user_id: input.user_id, subject: input.subject}",
        &format!(
            "demo.binding.IdentityBound: {{user_id: input.user_id, subject: input.subject, email: \
             {bound}}}"
        ),
    );
    if bind.is_none() {
        binding = replace(
            &binding,
            "      - {name: subject, type: demo.binding.Subject}\n    outcomes:\n      - name: \
             bound",
            "      - {name: subject, type: demo.binding.Subject}\n      - {name: email, type: \
             demo.provisioning.Email}\n    outcomes:\n      - name: bound",
        );
    }
    if !with_relation {
        binding = replace(
            &binding,
            "    relations:\n      - {name: user, kind: references, target: \
             demo.provisioning.User, cardinality: one, via: user_id}\n",
            "",
        );
    }
    [
        (MODEL[0].0, MODEL[0].1.to_owned()),
        (MODEL[1].0, MODEL[1].1.to_owned()),
        (MODEL[2].0, binding),
    ]
}

fn assembled(model: &[(&'static str, String); 3]) -> Result<Specification, String> {
    let parsed = model.clone().map(|(label, text)| {
        let raw = RawSpecFile::parse(&text).unwrap_or_else(|error| panic!("{label}: {error}"));
        (Source::new(label), raw)
    });
    Specification::assemble(parsed).map_err(|errors| errors.to_string())
}

fn compiled(model: &[(&'static str, String); 3]) -> EssIr {
    let spec = assembled(model)
        .unwrap_or_else(|errors| panic!("the relation the identity carries decides:\n{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

#[test]
fn a_related_read_through_an_existing_subjects_identity_follows_the_relation_it_carries() {
    for rebind in ["user_id", "input.user_id"] {
        let refused = assembled(&reading_through_identity(false, None, rebind))
            .expect_err("without the relation, `UserId` names two entities");
        assert!(
            refused.contains("demo.provisioning.User") && refused.contains("demo.binding.Identity"),
            "the refusal names both candidates: {refused}"
        );
    }
    compiled(&reading_through_identity(true, None, "user_id"));
    compiled(&reading_through_identity(true, None, "input.user_id"));
}

#[test]
fn a_related_read_through_the_identity_a_creating_branch_takes_from_its_input_is_admitted() {
    // On `creates:` there is no row before the branch; the identity holds the input it is filled
    // from, as a field `sets:` fills from its input does, and the relation it carries decides.
    compiled(&reading_through_identity(true, Some("user_id"), "user_id"));
    // The same input read directly: it supplies the identity, so it is carried by the identity.
    compiled(&reading_through_identity(
        true,
        Some("input.user_id"),
        "user_id",
    ));
}

/// Every `RegisterUser` the scenario runs: the instance it captures and the email it sends, in
/// order.
fn users(steps: &[ScenarioStep]) -> Vec<(String, ScenarioValue)> {
    let mut out = Vec::new();
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
                out.push((
                    instance.to_string(),
                    email.take().expect("a capture follows its RegisterUser"),
                ));
            }
            _ => {}
        }
    }
    out
}

/// The user the scenario's `BindIdentity` keys the identity by.
fn bound_to(steps: &[ScenarioStep]) -> String {
    steps
        .iter()
        .find_map(|step| match step {
            ScenarioStep::ExecuteCommand { command, input, .. }
                if command.to_string() == "demo.binding.BindIdentity" =>
            {
                match input.get("user_id") {
                    Some(ScenarioValue::Instance { instance }) => Some(instance.to_string()),
                    other => panic!("the identity is keyed by an arranged user: {other:?}"),
                }
            }
            _ => None,
        })
        .expect("the scenario binds an identity")
}

/// The email the scenario's expectation of `event` asserts.
fn asserted_email(steps: &[ScenarioStep], event: &str) -> Option<ess_primitives::node::Node> {
    steps
        .iter()
        .find_map(|step| match step {
            ScenarioStep::ExpectEvent {
                event: seen,
                payload,
                ..
            } if seen.to_string() == event => Some(payload.get("email").cloned()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("`{event}` is expected"))
}

/// The scenario keys the identity by the middle of three users, each decoy holding another email,
/// and asserts the middle user's email on `event`.
fn assert_read_between_decoys(suite: &ConformanceSuite, id: &str, event: &str) {
    let steps = &suite.scenarios[&ScenarioId::parse(id).unwrap()].steps;
    let users = users(steps);
    assert_eq!(users.len(), 3, "three users arranged for `{id}`: {users:?}");
    assert_eq!(
        users[1].0,
        bound_to(steps),
        "`{id}` keys the identity by the middle user: {users:?}"
    );
    assert_ne!(
        users[0].1, users[1].1,
        "the first decoy holds another email"
    );
    assert_ne!(users[2].1, users[1].1, "the last decoy holds another email");
    let ScenarioValue::Literal { value } = &users[1].1 else {
        panic!("the referenced user's email is a literal: {:?}", users[1].1);
    };
    assert_eq!(
        asserted_email(steps, event).as_ref(),
        Some(value),
        "`{id}` asserts the referenced user's email on `{event}`"
    );
}

#[test]
fn synthesis_arranges_the_row_the_identity_references_for_a_related_read() {
    for (bind, rebind) in [("user_id", "input.user_id"), ("input.user_id", "user_id")] {
        let suite = suite(&compiled(&reading_through_identity(
            true,
            Some(bind),
            rebind,
        )));
        assert_read_between_decoys(&suite, BOUND, "demo.binding.IdentityBound");
        assert_read_between_decoys(&suite, REBOUND, "demo.binding.IdentityRebound");
    }
}

/// The fixture with both commands of the identity guarded on the user row the identity names.
fn guarded() -> [(&'static str, String); 3] {
    let binding = MODEL[2]
        .1
        .replace(
            "commands:\n",
            "errors:\n  - {name: demo.binding.NoUser, summary: No user has this id., fields: []}\ncommands:\n",
        )
        .replace(
            "    outcomes:\n      - name: bound\n",
            "    outcomes:\n      - name: no-user\n        when_related: {via: input.user_id, exists: false}\n        error: demo.binding.NoUser\n      - name: bound\n",
        )
        .replace(
            "    outcomes:\n      - name: rebound\n",
            "    outcomes:\n      - name: no-user\n        when_related: {via: input.user_id, exists: false}\n        error: demo.binding.NoUser\n      - name: rebound\n",
        );
    assert_eq!(binding.matches("when_related").count(), 2, "{binding}");
    [
        (MODEL[0].0, MODEL[0].1.to_owned()),
        (MODEL[1].0, MODEL[1].1.to_owned()),
        (MODEL[2].0, binding),
    ]
}

#[test]
fn a_guard_on_the_row_the_identity_names_is_witnessed_or_refused_by_name() {
    let ir = compiled(&guarded());
    let result = ess_conformance::synthesize::synthesize(&ir);

    // Creating: the input that becomes the identity is pointed at the middle of three users, or,
    // for the refusal, at an id no user carries.
    let steps = |id: &str| &result.suite.scenarios[&ScenarioId::parse(id).unwrap()].steps;
    let arranged = users(steps(BOUND));
    assert_eq!(arranged.len(), 3, "three users: {arranged:?}");
    assert_eq!(bound_to(steps(BOUND)), arranged[1].0, "the middle user");
    let missing = steps("demo.binding.BindIdentity/outcome/no-user")
        .iter()
        .find_map(|step| match step {
            ScenarioStep::ExecuteCommand { command, input, .. }
                if command.to_string() == "demo.binding.BindIdentity" =>
            {
                input.get("user_id").cloned()
            }
            _ => None,
        });
    assert!(
        matches!(missing, Some(ScenarioValue::Literal { .. })),
        "the refused branch is sent an id no arranged user carries: {missing:?}"
    );

    // Updating: the input names the identity and, through it, the user. That arrangement is not
    // built, and each branch is refused naming the strategy rather than dropped.
    for outcome in ["no-user", "rebound"] {
        let id = format!("demo.binding.RebindIdentity/outcome/{outcome}");
        assert!(
            !result
                .suite
                .scenarios
                .keys()
                .any(|key| key.to_string() == id),
            "`{id}` is not synthesised"
        );
        assert!(
            result.refusals.iter().any(|refusal| {
                format!("{:?}", refusal.scenario).contains(&format!("OutcomeName({outcome})"))
                    && format!("{:?}", refusal.scenario).contains("RebindIdentity")
                    && refusal.cause.code().to_string() == "ESS-SYNTH-008"
            }),
            "`{id}` is refused by name: {:#?}",
            result.refusals
        );
    }
}
