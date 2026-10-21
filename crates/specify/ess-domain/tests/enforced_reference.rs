//! A stored reference whose row cannot be missing needs no `exists: false` branch (`ess/29`,
//! beyond10x/ess#509, `docs/design/generic-language-features.md` § G2, "Enforced references"):
//! the integrity is derived from the commands — every write of the field checks the row, and no
//! branch deletes a row of the referenced entity — and a branch defeating it is named.

const BOARD: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/enforced-reference.yaml");

fn spec(body: &str) -> Result<ess_domain::Specification, String> {
    let raw = ess_domain::spec::RawSpecFile::parse(body).map_err(|e| e.to_string())?;
    ess_domain::Specification::assemble([(ess_domain::system::Source::new("board.yaml"), raw)])
        .map_err(|e| e.to_string())
}

fn refused(body: &str) -> String {
    match spec(body) {
        Ok(_) => panic!("the document validates:\n{body}"),
        Err(errors) => errors,
    }
}

/// `BOARD` with `from` replaced by `to`, after checking `from` is there.
fn with(from: &str, to: &str) -> String {
    assert!(BOARD.contains(from), "the fixture holds `{from}`");
    BOARD.replacen(from, to, 1)
}

/// `BOARD` at another source format.
fn at(format: &str) -> String {
    with("format: ess/29", &format!("format: {format}"))
}

const COMMANDS: &str = "commands:\n";

/// `BOARD` with one more command declared first, and its actor permission.
fn with_command(name: &str, command: &str) -> String {
    let permitted = with(
        "may: [integrity.board.AddNode,",
        &format!("may: [{name}, integrity.board.AddNode,"),
    );
    permitted.replacen(COMMANDS, &format!("{COMMANDS}{command}"), 1)
}

const MISSING: &str = "no branch answers when the row does not exist";

#[test]
fn a_reference_every_write_checks_and_nothing_deletes_needs_no_missing_branch() {
    spec(BOARD).unwrap_or_else(|errors| panic!("{errors}"));
}

#[test]
fn the_missing_branch_may_still_be_declared() {
    let body = with(
        "      - name: node-sealed\n",
        "      - name: no-node\n        when_related: {via: node_id, exists: false}\n        \
         error: integrity.board.NodeNotFound\n      - name: node-sealed\n",
    );
    spec(&body).unwrap_or_else(|errors| panic!("{errors}"));
}

#[test]
fn below_ess_29_the_branch_is_still_required_naming_ess_29() {
    let errors = refused(&at("ess/28"));
    assert!(
        errors.contains("non_exhaustive_branches") && errors.contains(MISSING),
        "{errors}"
    );
    assert!(
        errors.contains("integrity.board.Rename") && errors.contains("format: ess/29"),
        "the hint names ess/29: {errors}"
    );
}

#[test]
fn a_command_deleting_the_referenced_row_keeps_the_branch_required_naming_it() {
    let errors = refused(&with_command(
        "integrity.board.RemoveNode",
        "  - name: integrity.board.RemoveNode\n    input:\n      - {name: node_id, type: \
         integrity.board.NodeId}\n    outcomes:\n      - name: removed\n        deletes: \
         integrity.board.Node\n        instance: node_id\n        emits: \
         [integrity.board.NodeWasSealed]\n        payload: {integrity.board.NodeWasSealed: \
         {node_id: input.node_id}}\n",
    ));
    assert!(
        errors.contains("non_exhaustive_branches") && errors.contains(MISSING),
        "{errors}"
    );
    assert!(
        errors.contains("`integrity.board.RemoveNode/removed` deletes `integrity.board.Node` rows"),
        "the refusal names the deleting branch: {errors}"
    );
}

#[test]
fn an_affects_entry_deleting_the_referenced_rows_keeps_the_branch_required_naming_it() {
    let errors = refused(&with(
        "        sets: {name: input.name}\n",
        "        sets: {name: input.name}\n        affects:\n          - \
         entity: integrity.board.Node\n            where: label == input.name\n            \
         deletes: integrity.board.Node\n",
    ));
    assert!(
        errors.contains("`integrity.board.Rename/renamed` deletes `integrity.board.Node` rows"),
        "the refusal names the deleting branch: {errors}"
    );
}

#[test]
fn a_write_that_does_not_check_the_row_keeps_the_branch_required_naming_it() {
    let errors = refused(&with(
        "      - name: no-node\n        when_related: {via: input.node_id, exists: false}\n        \
         error: integrity.board.NodeNotFound\n",
        "",
    ));
    assert!(
        errors.contains("non_exhaustive_branches") && errors.contains(MISSING),
        "{errors}"
    );
    assert!(
        errors.contains(
            "`integrity.board.AddPort/added` writes `node_id` without first refusing an identity \
             no `integrity.board.Node` row carries"
        ),
        "the refusal names the unchecked write: {errors}"
    );
}

#[test]
fn an_update_writing_the_reference_unchecked_is_named() {
    // `Rename` gains a `node_id` input and stores it, refusing no missing node first.
    const RENAME_INPUT: &str = "      - {name: name, type: String}\n    outcomes:\n      - \
                                name: node-sealed";
    let body = with(
        "        sets: {name: input.name}\n",
        "        sets: {name: input.name, node_id: input.node_id}\n",
    );
    assert!(body.contains(RENAME_INPUT));
    let errors = refused(&body.replacen(
        RENAME_INPUT,
        "      - {name: name, type: String}\n      - {name: node_id, type: \
         integrity.board.NodeId}\n    outcomes:\n      - name: node-sealed",
        1,
    ));
    assert!(
        errors.contains("`integrity.board.Rename/renamed` writes `node_id`"),
        "{errors}"
    );
}

#[test]
fn a_creation_leaving_the_reference_unset_is_named() {
    let errors = refused(&with(
        "        sets: {node_id: input.node_id, name: input.name}\n",
        "        sets: {name: input.name}\n",
    ));
    assert!(
        errors.contains(
            "`integrity.board.AddPort/added` creates a `integrity.board.Port` row without \
             setting `node_id`"
        ),
        "{errors}"
    );
}
