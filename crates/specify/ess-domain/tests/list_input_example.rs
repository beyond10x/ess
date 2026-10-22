//! `example:` on a command input of `List` type (ess/31, an adopter report, 2026-10-22).
//!
//! From `ess/31` an input whose type is a `List<…>`, through any newtype or `Optional`, may carry
//! an example: a list each of whose elements is a value of the element type, held as a
//! precondition's literal is. Below `ess/31` it keeps the refusal it always had, its hint naming
//! `ess/31`.

use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::error::{ValidationCode, ValidationErrors};
use ess_primitives::node::Node;

const MODEL: &str = include_str!("fixtures/list-input-example.yaml");
const AT: &str = "command.demo.server.Handshake.input";

fn assemble(text: &str) -> Result<Specification, ValidationErrors> {
    let raw = RawSpecFile::parse(text)
        .unwrap_or_else(|error| panic!("the document parses: {error}\n{text}"));
    Specification::assemble([(Source::new("spec.yaml"), raw)])
}

fn admitted(text: &str) -> Specification {
    assemble(text).unwrap_or_else(|errors| panic!("must be admitted: {errors}\n{text}"))
}

fn refused(text: &str) -> ValidationErrors {
    match assemble(text) {
        Ok(_) => panic!("must refuse:\n{text}"),
        Err(errors) => errors,
    }
}

fn with(model: &str, from: &str, to: &str) -> String {
    assert!(model.contains(from), "the fixture carries {from:?}");
    model.replacen(from, to, 1)
}

/// The model with `example: <value>` added to the input `field`, which carries none.
fn example(field: &str, value: &str) -> String {
    let prefix = format!("      - {{name: {field}, type: ");
    let line = MODEL
        .lines()
        .find(|line| line.starts_with(&prefix))
        .unwrap_or_else(|| panic!("the fixture declares the input {field}"));
    let written = line.replace("\"}", &format!("\", example: {value}}}"));
    with(MODEL, line, &written)
}

fn assert_refused(errors: &ValidationErrors, code: ValidationCode, field: &str, says: &str) {
    let site = format!("{AT}.{field}.example");
    assert!(
        errors.as_slice().iter().any(|error| error.code == code
            && error.location.contains(&site)
            && error.message.contains(says)),
        "expected {code:?} at …{site} saying {says:?}, got:\n{errors}"
    );
}

#[test]
fn a_list_example_is_admitted_at_ess_31_and_round_trips() {
    let spec = admitted(MODEL);
    let command = &spec.commands()[&"demo.server.Handshake".parse().unwrap()];
    assert_eq!(
        command.examples["supported_connection_types"],
        Node::Seq(vec![Node::Text("websocket".to_owned())])
    );
    let written = ess_yaml::to_string(command).unwrap();
    let reread: ess_domain::command::RawCommandSpec = ess_yaml::from_str(&written).unwrap();
    let reread = ess_domain::command::CommandSpec::try_from(reread).unwrap();
    assert_eq!(&reread, command, "the list example round-trips");
    for (field, value) in [
        ("tags", "[a, b]"),
        ("tags", "[]"),
        ("nodes", "[{label: first, weight: 2}]"),
        ("hints", "[x]"),
    ] {
        admitted(&example(field, value));
    }
}

#[test]
fn a_list_example_element_that_is_not_a_value_of_the_element_type_is_a_type_mismatch() {
    let errors = refused(&with(
        MODEL,
        "example: [websocket]",
        "example: [carrier-pigeon]",
    ));
    assert_refused(
        &errors,
        ValidationCode::TypeMismatch,
        "supported_connection_types",
        "element 0",
    );
    // An element that breaks its newtype's invariant conflicts, as a scalar example does.
    assert_refused(
        &refused(&example("tags", "[a, \"\"]")),
        ValidationCode::ConflictingDeclaration,
        "tags",
        "element 1",
    );
    // A struct element missing a required member is held as a precondition's struct literal is.
    assert_refused(
        &refused(&example("nodes", "[{label: first}]")),
        ValidationCode::MissingDeclaration,
        "nodes",
        "element 0: `demo.server.Node` requires the field `weight`",
    );
    for (field, value, says) in [
        ("nodes", "x", "is not a list"),
        ("tags", "[[a]]", "element 0"),
    ] {
        assert_refused(
            &refused(&example(field, value)),
            ValidationCode::TypeMismatch,
            field,
            says,
        );
    }
}

#[test]
fn a_null_or_map_example_stays_refused_at_ess_31() {
    assert_refused(
        &refused(&example("tags", "null")),
        ValidationCode::TypeMismatch,
        "tags",
        "cannot be an example",
    );
    assert_refused(
        &refused(&example("codes", "{a: b}")),
        ValidationCode::TypeMismatch,
        "codes",
        "is not a scalar",
    );
}

#[test]
fn a_list_example_below_ess_31_keeps_its_refusal_and_names_ess_31() {
    let errors = refused(&with(MODEL, "format: ess/31", "format: ess/30"));
    assert_refused(
        &errors,
        ValidationCode::TypeMismatch,
        "supported_connection_types",
        "`List<demo.server.ConnectionType>` is not a scalar",
    );
    let error = errors
        .as_slice()
        .iter()
        .find(|error| {
            error
                .location
                .contains("supported_connection_types.example")
        })
        .expect("the refusal");
    assert!(
        error
            .hint
            .as_deref()
            .is_some_and(|hint| hint.contains("ess/31")),
        "{error:?}"
    );
}
