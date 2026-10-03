//! Actual stored related rows decide commands; missing observations never become guessed facts.
use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_conformance::{interpret::Interpreted, report::Status, target::*, AdmittedSuite, Runner};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::{
    consistency::QueryConsistency, ids::CorrelationId, node::Node, time::Timestamp,
};
use std::collections::BTreeMap;

const SIGN_IN: &str = include_str!("fixtures/related-guard-sign-in.yaml");
const RELEASE: &str = include_str!("fixtures/related-guard-release.yaml");
fn model(source: &str) -> EssIr {
    let spec = Specification::assemble([(
        Source::new("related.yaml"),
        RawSpecFile::parse(source).unwrap(),
    )])
    .unwrap_or_else(|error| panic!("{error}\n{source}"));
    compile(&spec, &SourceMap::new()).unwrap()
}
fn context() -> ScenarioContext {
    ScenarioContext::new(
        "demo.signin/authored/related".parse().unwrap(),
        CorrelationId::new("related").unwrap(),
    )
}
fn target(source: &str) -> Interpreted {
    let target = Interpreted::for_model(model(source));
    target.begin_scenario(&context()).unwrap();
    target
}
fn fields(values: &[(&str, &str)]) -> BTreeMap<String, Node> {
    values
        .iter()
        .map(|(key, value)| ((*key).into(), Node::Text((*value).into())))
        .collect()
}
fn invoke(
    target: &Interpreted,
    command: &str,
    input: BTreeMap<String, Node>,
) -> Result<SemanticCommandResult, TargetError> {
    target.execute_command(SemanticCommandRequest {
        command: command.parse().unwrap(),
        actor: None,
        caller: None,
        input,
        correlation: context().correlation,
    })
}
fn configure(target: &Interpreted, client: &str) -> String {
    invoke(
        target,
        "demo.signin.ConfigureTenant",
        fields(&[("redirect_client", client)]),
    )
    .unwrap()
    .direct_events[0]
        .payload["tenant"]
        .as_text()
        .unwrap()
        .to_owned()
}
fn signin(
    target: &Interpreted,
    tenant: &str,
    client: &str,
) -> Result<SemanticCommandResult, TargetError> {
    invoke(
        target,
        "demo.signin.InitiateSignIn",
        fields(&[("tenant", tenant), ("client", client)]),
    )
}
fn outcome(answer: SemanticCommandResult, expected: &str) {
    assert_eq!(answer.outcome.unwrap().outcome.as_str(), expected);
    if expected != "initiated" && expected != "published" {
        assert!(answer.error.is_some());
        assert!(answer.direct_events.is_empty());
    }
}
fn rows(target: &Interpreted, view: &str) -> Vec<ViewRow> {
    target
        .query_view(SemanticViewRequest {
            view: view.parse().unwrap(),
            params: BTreeMap::new(),
            consistency: QueryConsistency::Current,
            correlation: context().correlation,
            deadline: Deadline::at(Timestamp::from_epoch_millis(0)),
        })
        .unwrap()
        .rows
}

#[test]
fn actual_signin_synthesis_executes_all_four_scenarios() {
    let ir = model(SIGN_IN);
    let suite = ess_conformance::synthesize(&ir).suite;
    assert_eq!(suite.scenarios.len(), 4);
    let admitted = AdmittedSuite::from_suite(&suite).unwrap();
    let report = Runner::for_suite(&suite)
        .run_admitted(&admitted, &Interpreted::for_model(ir))
        .into_report();
    assert_eq!(report.scenarios.len(), 4);
    for case in report.scenarios {
        assert_eq!(case.status, Status::Passed, "{case:#?}");
    }
}

#[test]
fn addressed_related_row_wins_over_decoys_and_refusals_preserve_state() {
    let target = target(SIGN_IN);
    let first = configure(&target, "console");
    let second = configure(&target, "other");
    let before = rows(&target, "demo.signin.Configurations");
    outcome(
        signin(&target, &second, "console").unwrap(),
        "no-redirect-entry",
    );
    outcome(
        signin(&target, &first, "other").unwrap(),
        "no-redirect-entry",
    );
    outcome(
        signin(&target, "00000000-0000-4000-8000-999999999999", "console").unwrap(),
        "no-configuration",
    );
    assert!(rows(&target, "demo.signin.SignIns").is_empty());
    assert_eq!(rows(&target, "demo.signin.Configurations"), before);
    outcome(signin(&target, &second, "other").unwrap(), "initiated");
    outcome(signin(&target, &first, "console").unwrap(), "initiated");
    assert_eq!(rows(&target, "demo.signin.SignIns").len(), 2);
}

#[test]
fn related_lifecycle_state_selects_actual_rows_beside_opposite_state_decoys() {
    let target = target(RELEASE);
    let propose = || {
        invoke(&target, "demo.release.ProposeCandidate", BTreeMap::new())
            .unwrap()
            .direct_events[0]
            .payload["candidate_id"]
            .as_text()
            .unwrap()
            .to_owned()
    };
    let first = propose();
    let second = propose();
    invoke(
        &target,
        "demo.release.AcceptCandidate",
        fields(&[("candidate_id", &first)]),
    )
    .unwrap();
    let release = invoke(&target, "demo.release.DraftRelease", BTreeMap::new())
        .unwrap()
        .direct_events[0]
        .payload["release_id"]
        .as_text()
        .unwrap()
        .to_owned();
    let before = rows(&target, "demo.release.Releases");
    outcome(
        invoke(
            &target,
            "demo.release.PublishRelease",
            fields(&[("release_id", &release), ("candidate", &second)]),
        )
        .unwrap(),
        "not-accepted",
    );
    assert_eq!(rows(&target, "demo.release.Releases"), before);
    outcome(
        invoke(
            &target,
            "demo.release.PublishRelease",
            fields(&[("release_id", &release), ("candidate", &first)]),
        )
        .unwrap(),
        "published",
    );
}

#[test]
fn optional_related_presence_is_observed_and_not_guessed() {
    let source = SIGN_IN
        .replace(
            "{name: redirect_client, type: demo.signin.ClientId}",
            "{name: redirect_client, type: 'Optional<demo.signin.ClientId>'}",
        )
        .replace(
            "predicate: redirect_client != input.client",
            "predicate: {redirect_client: {exists: false}}",
        );
    let target = target(&source);
    let present = configure(&target, "console");
    let absent = invoke(&target, "demo.signin.ConfigureTenant", BTreeMap::new())
        .unwrap()
        .direct_events[0]
        .payload["tenant"]
        .as_text()
        .unwrap()
        .to_owned();
    outcome(
        signin(&target, &absent, "console").unwrap(),
        "no-redirect-entry",
    );
    outcome(signin(&target, &present, "console").unwrap(), "initiated");
}

#[test]
fn all_related_predicate_conjuncts_and_input_eligibility_are_required() {
    let source = SIGN_IN
        .replace(
            "predicate: redirect_client != input.client",
            "predicate: {all: [redirect_client != input.client, redirect_client != console]}",
        )
        .replace(
            "      - name: no-redirect-entry\n",
            "      - name: no-redirect-entry\n        when: client != bypass\n",
        );
    let target = target(&source);
    let console = configure(&target, "console");
    let other = configure(&target, "other");
    for (row, client, expected) in [
        (&other, "console", "no-redirect-entry"),
        (&console, "other", "initiated"),
        (&other, "other", "initiated"),
        (&other, "bypass", "initiated"),
    ] {
        outcome(signin(&target, row, client).unwrap(), expected);
    }
}

#[test]
fn missing_required_related_fact_is_unknown_but_false_input_dominates() {
    let source = SIGN_IN
        .replace(
            "        sets:\n          redirect_client: input.redirect_client\n",
            "",
        )
        .replace(
            "      - name: no-redirect-entry\n",
            "      - name: no-redirect-entry\n        when: client != bypass\n",
        );
    let target = target(&source);
    let tenant = configure(&target, "console");
    assert!(matches!(
        signin(&target, &tenant, "console"),
        Err(TargetError::Unsupported { .. })
    ));
    assert!(rows(&target, "demo.signin.SignIns").is_empty());
    outcome(signin(&target, &tenant, "bypass").unwrap(), "initiated");
}

#[test]
fn missing_related_row_precedes_input_refusal_which_precedes_present_predicate() {
    let source = SIGN_IN.replace("      - name: no-redirect-entry\n", "      - name: blocked\n        when: client == blocked\n        error: demo.signin.NoRedirectEntry\n      - name: no-redirect-entry\n");
    let target = target(&source);
    let tenant = configure(&target, "console");
    outcome(
        signin(&target, "00000000-0000-4000-8000-999999999999", "blocked").unwrap(),
        "no-configuration",
    );
    outcome(signin(&target, &tenant, "blocked").unwrap(), "blocked");
    outcome(
        signin(&target, &tenant, "other").unwrap(),
        "no-redirect-entry",
    );
    assert!(rows(&target, "demo.signin.SignIns").is_empty());
}
