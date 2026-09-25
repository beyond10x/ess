//! A declaration refused by its own conversion is still *declared*, so a reference to it is not
//! refused a second time as a reference to nothing (beyond10x/ess#79).
//!
//! One bad payload source in `LendCopy` used to produce four refusals: the real one, and three that
//! said the command did not exist — an actor's grant, a component's `accepts`, and the lifecycle
//! transition only `LendCopy` takes. The author wrote one fault and the first refusal is its cause,
//! so it is the only one reported. A reference that names nothing at all is still refused.

use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::error::{ValidationCode, ValidationErrors};

const SYSTEM: &str = include_str!("fixtures/lending/system.yaml");
const COMPONENTS: &str = include_str!("fixtures/lending/components.yaml");
const LENDING: &str = include_str!("fixtures/lending/domains/lending.yaml");

fn assemble(domain: &str, components: &str) -> Result<Specification, ValidationErrors> {
    let file = |source: &str, yaml: &str| {
        (
            Source::new(source),
            RawSpecFile::parse(yaml).expect("the document is well formed YAML"),
        )
    };
    Specification::assemble(vec![
        file("system.yaml", SYSTEM),
        file("components.yaml", components),
        file("domains/lending.yaml", domain),
    ])
}

/// `LENDING` with `from` replaced by `to` once, at or after the declaration named `within`.
fn edit(within: &str, from: &str, to: &str) -> String {
    let start = LENDING
        .find(&format!("name: {within}\n"))
        .unwrap_or_else(|| panic!("the fixture declares `{within}`"));
    let offset = start
        + LENDING[start..]
            .find(from)
            .unwrap_or_else(|| panic!("`{within}` holds `{from}`"));
    format!(
        "{}{to}{}",
        &LENDING[..offset],
        &LENDING[offset + from.len()..]
    )
}

#[test]
fn the_fixture_is_valid_as_written() {
    assemble(LENDING, COMPONENTS).expect("the lending example validates");
}

#[test]
fn one_bad_payload_source_is_one_refusal() {
    let domain = edit(
        "library.lending.LendCopy",
        "copy_id: input.copy_id",
        "copy_id: input.no_such_field",
    );
    let errors = assemble(&domain, COMPONENTS).expect_err("the payload source is refused");
    assert_eq!(errors.len(), 1, "{errors}");
    let only = &errors.as_slice()[0];
    assert_eq!(only.code, ValidationCode::UndeclaredReference, "{errors}");
    assert!(only.message.contains("no_such_field"), "{errors}");
}

#[test]
fn a_refused_event_is_not_reported_again_by_the_command_and_component_that_name_it() {
    let domain = edit(
        "library.lending.CopyLent",
        "      - name: copy_id\n        type: library.lending.CopyId\n",
        "      - name: copy_id\n        type: library.lending.CopyId\n      - name: copy_id\n        type: library.lending.CopyId\n",
    );
    let errors = assemble(&domain, COMPONENTS).expect_err("the duplicated field is refused");
    assert!(
        errors
            .as_slice()
            .iter()
            .all(|error| error.code != ValidationCode::UndeclaredReference),
        "a reference to the refused event was reported as a reference to nothing: {errors}"
    );
}

#[test]
fn a_reference_to_a_name_nobody_declared_is_still_refused() {
    let components = COMPONENTS.replace(
        "        - library.lending.ReturnCopy\n",
        "        - library.lending.ReturnCopy\n        - library.lending.RenewCopy\n",
    );
    let domain = edit(
        "library.lending.LendCopy",
        "copy_id: input.copy_id",
        "copy_id: input.no_such_field",
    );
    let errors = assemble(&domain, &components).expect_err("two faults");
    assert_eq!(errors.len(), 2, "{errors}");
    assert!(
        errors.as_slice().iter().any(|error| error
            .message
            .contains("`library.lending.RenewCopy`, which nothing declares as a command")),
        "{errors}"
    );
}
