//! An actor's `may:` names the views it may read (beyond10x/ess#286), from source `ess/22`.
//!
//! One grant table: `may:` already grants commands, and a view named there is read-granted to that
//! actor. A view no actor names stays open to every caller, so a document without a view grant
//! keeps its meaning. Below `ess/22` a grant naming a view is refused, naming the format that
//! admits it.
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::error::ValidationCode;

const MODEL: &str = include_str!("../../../../docs/design/view-grants.example.yaml");

fn assemble(text: &str) -> Result<Specification, ess_primitives::error::ValidationErrors> {
    let raw = RawSpecFile::parse(text).expect("the document parses");
    Specification::assemble([(Source::new("view-grants.yaml"), raw)])
}

fn name(value: &str) -> ess_domain::QualifiedName {
    value.parse().expect("a qualified name")
}

#[test]
fn a_view_readable_by_one_of_two_actors_validates() {
    let spec = assemble(MODEL).unwrap_or_else(|errors| panic!("ess/22 admits it: {errors}"));
    let clerk = &spec.actors()[&name("desk.tickets.Clerk")];
    let watcher = &spec.actors()[&name("desk.tickets.Watcher")];
    let board = name("desk.tickets.Board");
    assert!(clerk.may_read(&board), "the Clerk names the Board");
    assert!(!watcher.may_read(&board), "the Watcher does not");
}

#[test]
fn a_view_grant_below_ess_22_is_refused_naming_ess_22() {
    for below in ["ess/21", "ess/20"] {
        refused_below(below);
    }
}

fn refused_below(below: &str) {
    let errors = assemble(&MODEL.replace("format: ess/22", &format!("format: {below}")))
        .expect_err("below ess/22 a grant names commands only");
    let refusal = errors
        .as_slice()
        .iter()
        .find(|error| error.code == ValidationCode::UnsupportedFormatVersion)
        .unwrap_or_else(|| panic!("an unsupported-format refusal: {errors}"));
    assert_eq!(refusal.location, "actor desk.tickets.Clerk.may");
    assert!(
        refusal.message.contains("`desk.tickets.Board`") && refusal.message.contains("ess/22"),
        "{below}: {refusal}"
    );
    assert_eq!(
        errors.len(),
        1,
        "one refusal, not a second one calling the view an undeclared command: {errors}"
    );
}

#[test]
fn a_model_without_a_view_grant_validates_below_ess_22_as_before() {
    let text = MODEL
        .replace("format: ess/22", "format: ess/21")
        .replace("      - desk.tickets.Board\n", "");
    assemble(&text).unwrap_or_else(|errors| panic!("{errors}"));
}

#[test]
fn a_grant_naming_neither_a_command_nor_a_view_is_refused() {
    let errors = assemble(&MODEL.replace(
        "      - desk.tickets.Board\n",
        "      - desk.tickets.Bored\n",
    ))
    .expect_err("a grant that names nothing");
    assert_eq!(errors.len(), 1, "{errors}");
    let error = errors.as_slice().first().expect("one");
    assert_eq!(error.code, ValidationCode::UndeclaredReference);
    assert!(
        error
            .message
            .contains("`desk.tickets.Bored`, which no domain declares as a command or a view"),
        "{error}"
    );
    assert!(
        error
            .hint
            .as_deref()
            .unwrap_or_default()
            .contains("desk.tickets.Board"),
        "the hint lists the views a grant may name: {error}"
    );
}

#[test]
fn an_actor_without_view_grants_serializes_as_before() {
    let text = MODEL.replace("      - desk.tickets.Board\n", "");
    let spec = assemble(&text).unwrap_or_else(|errors| panic!("{errors}"));
    let clerk = serde_json::to_value(&spec.actors()[&name("desk.tickets.Clerk")]).unwrap();
    assert_eq!(
        clerk["may"],
        serde_json::json!(["desk.tickets.OpenTicket"]),
        "{clerk}"
    );
}
