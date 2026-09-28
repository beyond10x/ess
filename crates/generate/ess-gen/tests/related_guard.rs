//! `when_related:` (ess/18, beyond10x/ess#211) in the HTTP projection: a refusal decided by the
//! related row is a conflict with that row's state, as a refusal decided by a stored row is, and
//! beside `existing_instance:` (which answers first) both refusals are `409` — the published
//! statuses name no precedence a client could read the wrong way round.
use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const SIGN_IN: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/related-guard-sign-in.yaml");

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("{errors:?}"))
}

fn statuses(ir: &EssIr, command: &str) -> Vec<(String, &'static str)> {
    let command = &ir.commands()[&command.parse().unwrap()];
    command
        .outcomes
        .iter()
        .map(|outcome| (outcome.name.to_string(), ess_gen::http::status(outcome)))
        .collect()
}

#[test]
fn issue_211_related_row_refusals_are_conflicts() {
    assert_eq!(
        statuses(&ir(SIGN_IN), "demo.signin.InitiateSignIn"),
        vec![
            ("no-configuration".to_owned(), ess_gen::http::CONFLICT),
            ("no-redirect-entry".to_owned(), ess_gen::http::CONFLICT),
            ("initiated".to_owned(), ess_gen::http::TAKEN),
        ]
    );
}

#[test]
fn issue_211_beside_existing_instance_both_refusals_are_conflicts() {
    let text = SIGN_IN
        .replace(
            "  - {name: demo.signin.SignInId, kind: newtype, of: Uuid}\n",
            "  - {name: demo.signin.SignInId, kind: newtype, of: String}\n",
        )
        .replace(
            "      - {name: client, type: demo.signin.ClientId}\n    outcomes:\n",
            "      - {name: client, type: demo.signin.ClientId}\n      - {name: sign_in_id, type: demo.signin.SignInId}\n    outcomes:\n      - {name: taken, existing_instance: true, error: demo.signin.NoConfiguration}\n",
        )
        .replace(
            "          demo.signin.SignInInitiated: {sign_in_id: {generated: true}}\n",
            "          demo.signin.SignInInitiated: {sign_in_id: input.sign_in_id}\n",
        );
    let statuses = statuses(&ir(&text), "demo.signin.InitiateSignIn");
    assert_eq!(statuses[0], ("taken".to_owned(), ess_gen::http::CONFLICT));
    assert_eq!(
        statuses[1],
        ("no-configuration".to_owned(), ess_gen::http::CONFLICT)
    );
}
