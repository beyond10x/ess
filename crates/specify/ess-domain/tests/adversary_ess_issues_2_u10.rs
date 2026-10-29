//! Adversary pass for the refusal of a `when:` branch implied by an earlier input-guarded refusal
//! (beyond10x/ess#702).
//!
//! The risk is a false refusal: a branch some request reaches, refused as unreachable. Each case
//! below builds a branch that is reachable, by the precedence order the interpreter reads, and
//! requires validation to admit it; the implied counterpart beside it shows the check is live in
//! the same model, so a silent pass is not a dead check.

const NEEDLE: &str = "is unreachable: every input satisfying";

fn validate(body: &str) -> String {
    let raw = ess_domain::spec::RawSpecFile::parse(body)
        .unwrap_or_else(|error| panic!("{error}\n{body}"));
    ess_domain::Specification::assemble([(ess_domain::system::Source::new("route.yaml"), raw)])
        .err()
        .map(|errors| errors.to_string())
        .unwrap_or_default()
}

/// `demo.route.Dispatch` in source format `format`, with `branches` before its accepting default.
fn model(format: &str, branches: &str) -> String {
    format!(
        "format: {format}
system: demo
version: v1
domain: demo.route
types:
  - name: demo.route.Route
    kind: enum
    variants: [X, Y]
  - name: demo.route.Kind
    kind: enum
    variants: [A, B, C]
  - name: demo.route.Target
    kind: struct
    fields:
      - {{name: kind, type: demo.route.Kind}}
      - {{name: urgent, type: Boolean}}
entities:
  - name: demo.route.Request
    identity: {{name: request_id, type: Uuid}}
    fields:
      - {{name: route, type: demo.route.Route}}
    lifecycle:
      initial: Open
      states: [Open]
      terminal: [Open]
errors:
  - name: demo.route.Closed
  - name: demo.route.Blocked
  - name: demo.route.Refused
commands:
  - name: demo.route.Dispatch
    input:
      - {{name: route, type: demo.route.Route}}
      - {{name: kind, type: demo.route.Kind}}
      - {{name: flag_a, type: Boolean}}
      - {{name: flag_b, type: Boolean}}
      - {{name: flag_c, type: Boolean}}
      - {{name: target, type: demo.route.Target}}
      - {{name: maybe, type: Optional<demo.route.Kind>}}
    outcomes:
{branches}      - name: dispatched
        creates: demo.route.Request
        instance: request_id
        sets:
          route: input.route
        emits: [demo.route.Dispatched]
        payload:
          demo.route.Dispatched:
            request_id: {{generated: true}}
            route: input.route
events:
  - name: demo.route.Dispatched
    fields:
      - {{name: request_id, type: Uuid}}
      - {{name: route, type: demo.route.Route}}
"
    )
}

fn refusal(name: &str, guard: &str, error: &str) -> String {
    format!("      - name: {name}\n        when: {guard}\n        error: demo.route.{error}\n")
}

fn accepting(name: &str, guard: &str) -> String {
    format!(
        "      - name: {name}
        when: {guard}
        creates: demo.route.Request
        instance: request_id
        sets:
          route: input.route
        emits: [demo.route.Dispatched]
        payload:
          demo.route.Dispatched:
            request_id: {{generated: true}}
            route: input.route
"
    )
}

/// The later branch's #702 refusals in `errors`: how many name it.
fn unreachable(errors: &str, later: &str) -> usize {
    errors.matches(&format!("`{later}` {NEEDLE}")).count()
}

/// Below `ess/29` an accepting branch may be declared above an input-guarded refusal; the plan
/// still answers the refusal first (beyond10x/ess#178), so the accepting branch it implies is
/// unreachable whatever the declaration order, and one it does not imply stays valid.
#[test]
fn adversary_u10_declaration_order_does_not_decide_an_accepting_branch() {
    let closed = refusal("closed", "route == X", "Closed");
    let implied = accepting("expedited", "{all: [route == X, flag_b == true]}");
    let errors = validate(&model("ess/23", &format!("{implied}{closed}")));
    assert!(
        errors.contains(
            "`expedited` is unreachable: every input satisfying `(route == X and flag_b == true)` \
             is answered first by `closed` (route == X), the input-guarded refusal taken first"
        ),
        "{errors}"
    );
    let reachable = accepting("expedited", "{all: [route == Y, flag_b == true]}");
    let errors = validate(&model("ess/23", &format!("{reachable}{closed}")));
    assert_eq!(unreachable(&errors, "expedited"), 0, "{errors}");
}

/// Connectives and membership: the prover's implication is exact over a closed domain. Each later
/// guard in `reachable` admits an input the earlier refusal does not claim; each in `implied` does
/// not.
#[test]
fn adversary_u10_any_not_and_membership_are_decided_exactly() {
    let earlier = refusal("held", "{any: [kind == A, flag_a == true]}", "Closed");
    let reachable = [
        "{all: [kind == B, flag_a == false]}",
        "{not: kind == B}",
        "{kind: {any_of: [B, C]}}",
        "{not: {all: [kind == A, flag_a == true]}}",
        "{any: [kind == A, flag_b == true]}",
        "kind != A",
        "flag_a == false",
    ];
    for guard in reachable {
        for later in [
            refusal("later", guard, "Blocked"),
            accepting("later", guard),
        ] {
            let errors = validate(&model("ess/32", &format!("{earlier}{later}")));
            assert_eq!(unreachable(&errors, "later"), 0, "{guard}\n{errors}");
        }
    }
    let implied = [
        "{all: [{kind: {any_of: [A]}}, flag_c == true]}",
        "{all: [{not: kind != A}, flag_b == false]}",
        "{any: [{all: [kind == A, flag_b == true]}, {all: [flag_a == true, kind == C]}]}",
        "{all: [{not: flag_a == false}, kind != B]}",
    ];
    for guard in implied {
        let later = refusal("later", guard, "Blocked");
        let errors = validate(&model("ess/32", &format!("{earlier}{later}")));
        assert_eq!(unreachable(&errors, "later"), 1, "{guard}\n{errors}");
    }
}

/// A guard over a struct member of the input: implication reads the member's closed domain.
#[test]
fn adversary_u10_struct_members_are_decided_by_member() {
    let earlier = refusal("held", "target.kind == A", "Closed");
    let errors = validate(&model(
        "ess/32",
        &format!(
            "{earlier}{}",
            refusal(
                "later",
                "{all: [target.kind == B, target.urgent == true]}",
                "Blocked"
            )
        ),
    ));
    assert_eq!(unreachable(&errors, "later"), 0, "{errors}");
    let errors = validate(&model(
        "ess/32",
        &format!(
            "{earlier}{}",
            refusal(
                "later",
                "{all: [target.kind == A, target.urgent == true]}",
                "Blocked"
            )
        ),
    ));
    assert_eq!(unreachable(&errors, "later"), 1, "{errors}");
}

/// An Optional input is never decided by `analyze`: a later branch that reaches the absent input
/// stays valid.
#[test]
fn adversary_u10_an_optional_input_is_not_refused() {
    for (earlier, later) in [
        ("defined(maybe)", "{not: defined(maybe)}"),
        (
            "defined(maybe)",
            "{all: [{not: defined(maybe)}, flag_a == true]}",
        ),
    ] {
        let branches = format!(
            "{}{}",
            refusal("held", earlier, "Closed"),
            refusal("later", later, "Blocked")
        );
        let errors = validate(&model("ess/32", &branches));
        assert_eq!(unreachable(&errors, "later"), 0, "{later}\n{errors}");
    }
}

/// An external refusal answered before the row (`before_row: true`) is taken only where its
/// provider takes it: a later `when:` over the same input is reachable and stays valid.
#[test]
fn adversary_u10_an_external_before_row_refusal_does_not_shadow() {
    let external = "      - name: denied
        external: the upstream service declines the route
        when: route == X
        before_row: true
        error: demo.route.Refused
";
    for later in [
        refusal("later", "route == X", "Blocked"),
        accepting("later", "route == X"),
    ] {
        let errors = validate(&model("ess/32", &format!("{external}{later}")));
        assert_eq!(unreachable(&errors, "later"), 0, "{errors}");
    }
}

/// Only the earlier refusal's guard answers first, not a later one's: two refusals whose guards
/// each imply the other's are refused once, at the second declared.
#[test]
fn adversary_u10_mutually_equal_refusals_refuse_only_the_second() {
    let branches = format!(
        "{}{}",
        refusal("first", "{kind: {any_of: [A, B]}}", "Closed"),
        refusal("second", "kind != C", "Blocked")
    );
    let errors = validate(&model("ess/32", &branches));
    assert_eq!(unreachable(&errors, "first"), 0, "{errors}");
    assert_eq!(unreachable(&errors, "second"), 1, "{errors}");
    assert!(
        errors.contains("(hint: drop `second`, or narrow `first`"),
        "{errors}"
    );
}

/// A command reading a row set reads its input refusals at step 2 as every other command does: a
/// later accepting `when:` the refusal does not imply stays valid, one it implies is refused, and a
/// row-set branch whose input guard the refusal implies is not a `when:` branch and is not read.
#[test]
fn adversary_u10_a_row_set_command_keeps_the_plan() {
    let text = |branches: &str| {
        format!(
            "format: ess/32
system: demo
version: v1
domain: demo.shelf
entities:
  - name: demo.shelf.Library
    identity: {{name: library, type: String}}
    fields:
      - {{name: curator, type: String}}
    lifecycle: {{initial: Open, states: [Open], terminal: [Open]}}
  - name: demo.shelf.Book
    identity: {{name: book, type: String}}
    fields: []
    lifecycle: {{initial: Shelved, states: [Shelved], terminal: [Shelved]}}
errors:
  - {{name: demo.shelf.Refused, summary: A library of the curator is open., fields: []}}
  - {{name: demo.shelf.Rushed, summary: A rush is not taken., fields: []}}
  - {{name: demo.shelf.Idle, summary: Nothing was asked., fields: []}}
events:
  - name: demo.shelf.BookPaused
    fields: [{{name: book, type: String}}]
commands:
  - name: demo.shelf.Act
    input:
      - {{name: book, type: String}}
      - {{name: curator, type: String}}
      - {{name: pause, type: Boolean}}
      - {{name: rush, type: Boolean}}
    outcomes:
      - {{name: rushed, when: rush == true, error: demo.shelf.Rushed}}
{branches}      - {{name: idle, error: demo.shelf.Idle}}
"
        )
    };
    let paused = |guard: &str| {
        format!(
            "      - name: paused
        when: {guard}
        updates: demo.shelf.Book
        instance: book
        emits: [demo.shelf.BookPaused]
        payload:
          demo.shelf.BookPaused: {{book: input.book}}
"
        )
    };
    let refused = "      - name: refused
        when_related:
          entity: demo.shelf.Library
          where: curator == input.curator
          exists: true
        error: demo.shelf.Refused
";
    let errors = validate(&text(&format!(
        "{refused}{}",
        paused("{all: [rush == false, pause == true]}")
    )));
    assert_eq!(unreachable(&errors, "paused"), 0, "{errors}");
    let errors = validate(&text(&format!(
        "{refused}{}",
        paused("{all: [rush == true, pause == true]}")
    )));
    assert_eq!(unreachable(&errors, "paused"), 1, "{errors}");
    assert_eq!(errors.matches(NEEDLE).count(), 1, "{errors}");
}
