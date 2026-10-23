//! Adversarial pass 1 on "a literal-set field steers the creator of a subject row".
//!
//! Every probe starts from `fixtures/arrangement-literal-set-subject.yaml` (the unit's own
//! fixture) and changes one thing about the literal, the guard or the creator, then asks two
//! questions: is every `Dispatch` branch the model makes reachable arranged rather than refused,
//! and does every scenario the synthesis writes pass on the interpreted target.
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::{
    interpret::Interpreted, report::Status, synthesize::Synthesis, AdmittedSuite, Runner,
};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};

const CONN: &str = include_str!("fixtures/arrangement-literal-set-subject.yaml");

const KEY_TO_D: &str = "all: [mode == Key, route == D]";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("{errors:?}"))
}

/// Every refusal naming a `Dispatch` scenario, as `<code> <scenario>: <cause>`.
fn dispatch_refusals(result: &Synthesis) -> Vec<String> {
    result
        .refusals
        .iter()
        .map(|refusal| {
            format!(
                "{} {}: {}",
                refusal.cause.code(),
                refusal
                    .scenario
                    .as_ref()
                    .map_or_else(String::new, ToString::to_string),
                refusal.cause
            )
        })
        .filter(|line| line.contains("demo.conn.Dispatch/"))
        .collect()
}

/// Every scenario of the synthesized suite that does not pass on the interpreted target.
fn interpreted_failures(ir: EssIr, result: &Synthesis) -> Vec<String> {
    let suite = &result.suite;
    let admitted = AdmittedSuite::from_suite(suite).unwrap();
    let report = Runner::for_suite(suite)
        .run_admitted(&admitted, &Interpreted::for_model(ir))
        .into_report();
    assert_ne!(report.scenarios.len(), 0);
    report
        .scenarios
        .into_iter()
        .filter(|scenario| scenario.status != Status::Passed)
        .map(|scenario| format!("{}: {:?}", scenario.scenario, scenario.checks))
        .collect()
}

/// Both questions at once: no `Dispatch` refusal, and nothing failing on the interpreted target.
fn assert_arranged_and_passing(name: &str, text: &str) {
    let model = ir(text);
    let result = ess_conformance::synthesize::synthesize(&model);
    let refused = dispatch_refusals(&result);
    let failing = interpreted_failures(model, &result);
    assert!(
        refused.is_empty() && failing.is_empty(),
        "{name}: refused {refused:#?}\nfailing on the interpreted target {failing:#?}"
    );
}

/// `key-to-d`'s guard written another way that only a `by-key` row satisfies.
fn with_key_to_d(guard: &str) -> String {
    let text = CONN.replacen(KEY_TO_D, guard, 1);
    assert_ne!(text, CONN);
    text
}

#[test]
fn a_negated_literal_leaf_selecting_only_the_key_creator_is_arranged() {
    assert_arranged_and_passing(
        "not over the literal",
        &with_key_to_d("all: [{not: mode == Token}, {not: mode == Open}, route == D]"),
    );
}

#[test]
fn a_not_equal_literal_leaf_selecting_only_the_key_creator_is_arranged() {
    assert_arranged_and_passing(
        "!= over the literal",
        &with_key_to_d("all: [mode != Token, mode != Open, route == D]"),
    );
}

#[test]
fn an_optional_field_set_to_a_literal_is_arranged() {
    let text = CONN.replacen(
        "{name: mode, type: demo.conn.Mode}",
        "{name: mode, type: Optional<demo.conn.Mode>}",
        2,
    );
    assert_ne!(text, CONN);
    assert_arranged_and_passing("Optional mode", &text);
}

#[test]
fn an_integer_literal_under_an_ordering_is_arranged() {
    let text = CONN
        .replacen(
            "      - {name: mode, type: demo.conn.Mode}\n    lifecycle:",
            "      - {name: mode, type: demo.conn.Mode}\n      - {name: level, type: Integer}\n    lifecycle:",
            1,
        )
        .replacen(
            "      - {name: mode, type: demo.conn.Mode}\n      - {name: state",
            "      - {name: mode, type: demo.conn.Mode}\n      - {name: level, type: Integer}\n      - {name: state",
            1,
        )
        .replacen("mode: Key}", "mode: Key, level: \"3\"}", 1)
        .replacen("mode: Token}", "mode: Token, level: \"1\"}", 1)
        .replacen("mode: Open}", "mode: Open, level: \"0\"}", 1)
        .replacen(KEY_TO_D, "all: [level >= 3, route == D]", 1);
    assert!(text.contains("level >= 3") && text.contains("level: \"0\""));
    assert_arranged_and_passing("Integer level ordering", &text);
}

/// An earlier refusal over the field the hint steers and a field the steered base already holds:
/// `route: D` with `cred: Header` is refused before `by-key`, while `route: D` with `cred: Sub` is
/// created by it. The row the branch needs exists, through a different `cred` than a base input.
#[test]
fn an_earlier_refusal_over_the_steered_field_still_finds_the_key_creator() {
    let text = CONN.replacen(
        "      - name: by-key\n",
        "      - name: d-by-header\n        when: {route: D, cred: Header}\n        error: demo.conn.Refused\n      - name: by-key\n",
        1,
    );
    assert_ne!(text, CONN);
    assert_arranged_and_passing("earlier refusal over route", &text);
}

/// A row no creator writes `mode: Key` on, which a later move promotes to `Key`: the creators'
/// literals contradict the leaf, which must stay as written so the move's row is still reached.
#[test]
fn a_literal_a_later_move_writes_is_arranged_through_the_move() {
    let text = CONN
        .replacen(
            "      states: [Admitted]\n      terminal: [Admitted]\n      transitions: []",
            "      states: [Admitted, Promoted]\n      terminal: [Promoted]\n      transitions:\n        - {name: promote, from: [Admitted], to: Promoted}",
            1,
        )
        .replacen("mode: Key}", "mode: Token}", 1)
        .replacen(
            "may: [demo.conn.Connect, demo.conn.Dispatch]",
            "may: [demo.conn.Connect, demo.conn.Dispatch, demo.conn.Promote]",
            1,
        )
        .replacen(
            "events:\n",
            "  - name: demo.conn.Promote\n    input:\n      - {name: conn_id, type: demo.conn.ConnId}\n    outcomes:\n      - name: promoted\n        moves: demo.conn.Conn.promote\n        instance: conn_id\n        sets: {mode: Key}\n        emits: [demo.conn.Admitted]\n        payload:\n          demo.conn.Admitted: {conn_id: input.conn_id}\n      - name: wrong-state\n        wrong_state: true\n        error: demo.conn.Refused\nevents:\n",
            1,
        );
    assert!(text.contains("demo.conn.Conn.promote") && !text.contains("mode: Key}\n        emits: [demo.conn.Admitted]\n        payload:\n          demo.conn.Admitted:\n            conn_id: {generated: true}"));
    assert_arranged_and_passing("literal written by a later move", &text);
}

/// The `by-key` row demoted by a later move to `mode: Open`: a scenario selecting `key-to-d`
/// must not arrange the row through the demotion.
#[test]
fn a_literal_a_later_move_overwrites_passes_on_the_interpreted_target() {
    let text = CONN
        .replacen(
            "      states: [Admitted]\n      terminal: [Admitted]\n      transitions: []",
            "      states: [Admitted, Demoted]\n      terminal: [Demoted]\n      transitions:\n        - {name: demote, from: [Admitted], to: Demoted}",
            1,
        )
        .replacen(
            "may: [demo.conn.Connect, demo.conn.Dispatch]",
            "may: [demo.conn.Connect, demo.conn.Dispatch, demo.conn.Demote]",
            1,
        )
        .replacen(
            "events:\n",
            "  - name: demo.conn.Demote\n    input:\n      - {name: conn_id, type: demo.conn.ConnId}\n    outcomes:\n      - name: demoted\n        moves: demo.conn.Conn.demote\n        instance: conn_id\n        sets: {mode: Open}\n        emits: [demo.conn.Admitted]\n        payload:\n          demo.conn.Admitted: {conn_id: input.conn_id}\n      - name: wrong-state\n        wrong_state: true\n        error: demo.conn.Refused\nevents:\n",
            1,
        );
    assert!(text.contains("demo.conn.Conn.demote"));
    assert_arranged_and_passing("literal overwritten by a later move", &text);
}

/// The fixture with `route` and `cred` sent inside one struct input `req`, `route` stored from
/// `input.req.route`.
fn nested() -> String {
    let text = CONN
        .replacen(
            "      - {name: route, type: demo.conn.Route}\n      - {name: cred, type: demo.conn.Cred}\n",
            "      - {name: req, type: demo.conn.Req}\n",
            1,
        )
        .replacen(
            "  - {name: demo.conn.Mode, kind: enum, variants: [Key, Token, Open]}\n",
            "  - {name: demo.conn.Mode, kind: enum, variants: [Key, Token, Open]}\n  - name: demo.conn.Req\n    kind: struct\n    fields:\n      - {name: route, type: demo.conn.Route}\n      - {name: cred, type: demo.conn.Cred}\n",
            1,
        )
        .replacen(
            "when: {cred: None, token_on: true}",
            "when: {all: [req.cred == None, token_on == true]}",
            1,
        )
        .replace("route: input.route", "route: input.req.route");
    assert!(text.contains("input.req.route") && !text.contains("{cred: None"));
    text
}

#[test]
fn a_route_mapped_from_a_struct_input_is_arranged() {
    assert_arranged_and_passing("route inside a struct input", &nested());
}

/// [`nested`] with `req.route == D` refused before `by-key` when `req.cred` is `Header`: a
/// creator input holding `D` and `Sub` in `req` still creates the row `key-to-d` needs.
#[test]
fn a_struct_input_the_steer_copies_whole_still_finds_the_key_creator() {
    let text = nested().replacen(
        "      - name: by-key\n",
        "      - name: d-by-header\n        when: {all: [req.route == D, req.cred == Header]}\n        error: demo.conn.Refused\n      - name: by-key\n",
        1,
    );
    assert!(text.contains("d-by-header"));
    assert_arranged_and_passing("struct input with an earlier refusal", &text);
}

#[test]
fn the_same_model_synthesizes_byte_identical_suites() {
    for text in [
        CONN.to_owned(),
        with_key_to_d("all: [mode != Token, mode != Open, route == D]"),
    ] {
        let one = ess_conformance::synthesize::synthesize(&ir(&text))
            .suite
            .to_canonical_json()
            .unwrap();
        let two = ess_conformance::synthesize::synthesize(&ir(&text))
            .suite
            .to_canonical_json()
            .unwrap();
        assert_eq!(one, two);
    }
}
