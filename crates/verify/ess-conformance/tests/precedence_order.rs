//! The one precedence order (`docs/design/cross-record-and-stored-field-guards.md`, "The
//! precedence order"), as the model interpreter answers it on a command guarded by a related row:
//! a missing related row is answered by its `exists: false` branch first, then an input-guarded
//! refusal, the first declared whose guard holds.
//!
//! The model is `related-guard-sign-in.yaml` with two input refusals added after
//! `no-configuration`: `blank-client: client == ""` and `blank-again: client == ""`, which
//! overlap it exactly.

use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::interpret::execute::{execute, Externals, Store, Undetermined};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::node::Node;

const SIGN_IN: &str = include_str!("fixtures/related-guard-sign-in.yaml");

fn edit(text: &str, from: &str, to: &str) -> String {
    let out = text.replace(from, to);
    assert_ne!(out, text, "`{from}` is in the fixture");
    out
}

fn model() -> EssIr {
    let text = edit(
        SIGN_IN,
        "  - {name: demo.signin.NoRedirectEntry, summary: The configuration does not register the client., fields: []}\n",
        "  - {name: demo.signin.NoRedirectEntry, summary: The configuration does not register the client., fields: []}\n  - {name: demo.signin.BlankClient, summary: The client is blank., fields: []}\n  - {name: demo.signin.StillBlank, summary: The client is still blank., fields: []}\n",
    );
    let text = edit(
        &text,
        "      - name: initiated\n        creates: demo.signin.SignIn\n",
        "      - name: blank-client\n        when: client == \"\"\n        error: demo.signin.BlankClient\n      - name: blank-again\n        when: client == \"\"\n        error: demo.signin.StillBlank\n      - name: initiated\n        creates: demo.signin.SignIn\n",
    );
    let raw = RawSpecFile::parse(&text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("sign-in.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn run(
    ir: &EssIr,
    store: &Store,
    command: &str,
    input: &[(&str, &str)],
) -> Result<Vec<ess_conformance::interpret::execute::Step>, Undetermined> {
    let input = input
        .iter()
        .map(|(name, value)| ((*name).to_owned(), Node::Text((*value).to_owned())))
        .collect::<BTreeMap<_, _>>();
    execute(
        ir,
        store,
        &command.parse().unwrap(),
        &input,
        &Externals::Withheld,
    )
}

fn taken(steps: &[ess_conformance::interpret::execute::Step]) -> Vec<String> {
    steps
        .iter()
        .map(|step| {
            step.outcome
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_default()
        })
        .collect()
}

/// A store holding one configuration, and its tenant.
fn configured(ir: &EssIr) -> (Store, String) {
    let steps = run(
        ir,
        &Store::default(),
        "demo.signin.ConfigureTenant",
        &[("redirect_client", "client-registered")],
    )
    .unwrap_or_else(|why| panic!("{why}"));
    assert_eq!(steps.len(), 1);
    let next = steps[0].next.clone();
    let tenant = next
        .text_instances()
        .find(|(entity, _, _)| entity.to_string() == "demo.signin.Configuration")
        .map(|(_, identity, _)| identity.to_owned())
        .expect("a configuration is stored");
    (next, tenant)
}

#[test]
fn a_missing_related_row_is_answered_by_exists_false_before_an_input_refusal() {
    let ir = model();
    let (store, tenant) = configured(&ir);
    let missing = "00000000-0000-4000-8000-000000000077";
    assert_ne!(tenant, missing);
    let steps = run(
        &ir,
        &store,
        "demo.signin.InitiateSignIn",
        &[("tenant", missing), ("client", "")],
    )
    .unwrap_or_else(|why| panic!("{why}"));
    assert_eq!(
        taken(&steps),
        ["demo.signin.InitiateSignIn/no-configuration"]
    );
    assert!(steps[0].error.is_some() && steps[0].events.is_empty());
}

#[test]
fn a_stored_related_row_leaves_the_input_refusal_to_answer_first_declared() {
    let ir = model();
    let (store, tenant) = configured(&ir);
    let steps = run(
        &ir,
        &store,
        "demo.signin.InitiateSignIn",
        &[("tenant", &tenant), ("client", "")],
    )
    .unwrap_or_else(|why| panic!("{why}"));
    assert_eq!(taken(&steps), ["demo.signin.InitiateSignIn/blank-client"]);
}

/// The `exists: false` witness is sent with an input the input refusals claim, so a target
/// answering `blank-client` before reading the related row fails `no-configuration`.
#[test]
fn the_missing_row_witness_carries_an_input_a_refusal_claims() {
    let result = ess_conformance::synthesize::synthesize(&model());
    let scenario = result
        .suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == "demo.signin.InitiateSignIn/outcome/no-configuration")
        .map(|(_, scenario)| scenario)
        .expect("no-configuration has a scenario");
    let blank = scenario.steps.iter().any(|step| match step {
        ess_conformance::ScenarioStep::ExecuteCommand { command, input, .. } => {
            command.to_string() == "demo.signin.InitiateSignIn"
                && matches!(input.get("client"), Some(ess_conformance::ScenarioValue::Literal { value })
                    if *value == Node::Text(String::new()))
        }
        _ => false,
    });
    assert!(blank, "{:#?}", scenario.steps);
}
