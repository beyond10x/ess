//! A `when:` branch whose input guard implies the guard of an input-guarded refusal the precedence
//! order answers before it is refused as unreachable (beyond10x/ess#702).
//!
//! Every input the later guard admits is answered first by the earlier refusal, so no request
//! reaches the later branch. Synthesis found this by searching candidates and named it
//! ESS-SYNTH-003; validation now decides it with the finite prover, in the same wording. Two equal
//! guards count where the prover declines them too (beyond10x/ess#816). Where the prover declines
//! one of two guards that differ, validation stays silent and synthesis keeps the finding. A later
//! branch that some input reaches, because the earlier guard is narrower, stays valid.

const NEEDLE: &str = "is unreachable: every input satisfying";

fn spec(body: &str) -> Result<ess_domain::Specification, String> {
    let raw = ess_domain::spec::RawSpecFile::parse(body).map_err(|e| e.to_string())?;
    ess_domain::Specification::assemble([(ess_domain::system::Source::new("route.yaml"), raw)])
        .map_err(|e| e.to_string())
}

/// `Dispatch` with `branches` before its accepting default.
fn model(branches: &str) -> String {
    format!(
        "format: ess/32
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
commands:
  - name: demo.route.Dispatch
    input:
      - {{name: route, type: demo.route.Route}}
      - {{name: kind, type: demo.route.Kind}}
      - {{name: flag_a, type: Boolean}}
      - {{name: flag_b, type: Boolean}}
      - {{name: flag_c, type: Boolean}}
      - {{name: flag_d, type: Boolean}}
      - {{name: count, type: Integer}}
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

const CLOSED: &str = "      - name: closed
        when: route == X
        error: demo.route.Closed
";

const BLOCKED: &str = "      - name: blocked
        when:
          all:
            - route == X
            - flag_a == false
            - flag_b == true
        error: demo.route.Blocked
";

const EXPEDITED: &str = "      - name: expedited
        when:
          all:
            - route == X
            - flag_a == false
            - flag_b == true
        creates: demo.route.Request
        instance: request_id
        sets:
          route: input.route
        emits: [demo.route.Dispatched]
        payload:
          demo.route.Dispatched:
            request_id: {generated: true}
            route: input.route
";

/// The specification's refusals where it is refused; empty where it validates.
fn refusals(branches: &str) -> String {
    spec(&model(branches)).err().unwrap_or_default()
}

#[test]
fn a_branch_implied_by_an_earlier_input_refusal_is_unreachable() {
    let errors = refusals(&format!("{CLOSED}{BLOCKED}"));
    assert!(
        errors.contains(
            "[unreachable_branch] command.demo.route.Dispatch.outcomes.blocked: `blocked` is \
             unreachable: every input satisfying `(route == X and flag_a == false and flag_b == \
             true)` is answered first by `closed` (route == X), an input-guarded refusal declared \
             before it (hint: declare `blocked` before `closed` to answer those inputs with it, or \
             drop it)"
        ),
        "{errors}"
    );
    assert_eq!(errors.matches(NEEDLE).count(), 1, "{errors}");
}

#[test]
fn an_accepting_branch_implied_by_an_earlier_refusal_is_unreachable() {
    let errors = refusals(&format!("{CLOSED}{EXPEDITED}"));
    assert!(
        errors.contains(
            "[unreachable_branch] command.demo.route.Dispatch.outcomes.expedited: `expedited` is \
             unreachable: every input satisfying `(route == X and flag_a == false and flag_b == \
             true)` is answered first by `closed` (route == X), the input-guarded refusal taken \
             first (hint: drop `expedited`, or narrow `closed` so some input `expedited` admits \
             falls outside it)"
        ),
        "{errors}"
    );
    assert_eq!(errors.matches(NEEDLE).count(), 1, "{errors}");
}

#[test]
fn an_equal_guard_declared_later_is_unreachable() {
    let again = "      - name: closed-again
        when: route == X
        error: demo.route.Blocked
";
    let errors = refusals(&format!("{CLOSED}{again}"));
    assert!(
        errors.contains(
            "[unreachable_branch] command.demo.route.Dispatch.outcomes.closed-again: \
             `closed-again` is unreachable: every input satisfying `route == X` is answered first \
             by `closed` (route == X), an input-guarded refusal declared before it (hint: drop \
             `closed-again`, or narrow `closed` so some input `closed-again` admits falls outside \
             it)"
        ),
        "{errors}"
    );
    assert_eq!(errors.matches(NEEDLE).count(), 1, "{errors}");
}

#[test]
fn a_broader_later_branch_stays_valid() {
    let accepting = "      - name: expedited
        when: route == X
        creates: demo.route.Request
        instance: request_id
        sets:
          route: input.route
        emits: [demo.route.Dispatched]
        payload:
          demo.route.Dispatched:
            request_id: {generated: true}
            route: input.route
";
    for branches in [
        format!("{BLOCKED}{CLOSED}"),
        format!("{BLOCKED}{accepting}"),
        CLOSED.to_owned(),
    ] {
        let errors = refusals(&branches);
        assert!(errors.is_empty(), "{errors}\n{branches}");
    }
}

#[test]
fn an_undecidable_implication_is_not_refused() {
    let busy = "      - name: busy
        when: count > 5
        error: demo.route.Closed
";
    let swamped = "      - name: swamped
        when:
          all:
            - count > 7
            - route == X
        error: demo.route.Blocked
";
    let branches = format!("{busy}{swamped}");
    let errors = refusals(&branches);
    assert!(errors.is_empty(), "{errors}\n{branches}");
}

/// Two refusals with one guard the prover declines (an `Integer` comparison): the second never
/// answers, because the first answers every input that satisfies it (beyond10x/ess#816).
#[test]
fn an_equal_guard_the_prover_declines_is_unreachable() {
    let busy = "      - name: busy
        when: count > 5
        error: demo.route.Closed
";
    let again = "      - name: busy-again
        when: count > 5
        error: demo.route.Blocked
";
    let errors = refusals(&format!("{busy}{again}"));
    assert!(
        errors.contains(
            "[unreachable_branch] command.demo.route.Dispatch.outcomes.busy-again: `busy-again` \
             is unreachable: every input satisfying `count > 5` is answered first by `busy` \
             (count > 5), an input-guarded refusal declared before it (hint: drop `busy-again`, \
             or narrow `busy` so some input `busy-again` admits falls outside it)"
        ),
        "{errors}"
    );
    assert_eq!(errors.matches(NEEDLE).count(), 1, "{errors}");
}

/// Beside an equal guard the prover declines, what stays valid: other literals, a first refusal
/// narrowed by a conjunct the second lacks, and one guard on two commands.
#[test]
fn guards_the_prover_declines_that_differ_stay_valid() {
    let busy = "      - name: busy
        when: count > 5
        error: demo.route.Closed
";
    let other = "      - name: swamped
        when: count > 7
        error: demo.route.Blocked
";
    let narrowed = "      - name: busy-x
        when:
          all:
            - count > 5
            - route == X
        error: demo.route.Closed
";
    let broad = "      - name: busy-any
        when: count > 5
        error: demo.route.Blocked
";
    for branches in [format!("{busy}{other}"), format!("{narrowed}{broad}")] {
        let errors = refusals(&branches);
        assert!(errors.is_empty(), "{errors}\n{branches}");
    }
    let one = model(busy);
    let start = one
        .find("  - name: demo.route.Dispatch")
        .expect("the command");
    let end = one.find("\nevents:\n").expect("the events") + 1;
    let second = one[start..end].replace(
        "name: demo.route.Dispatch\n",
        "name: demo.route.Redispatch\n",
    );
    let two = format!("{}{second}{}", &one[..end], &one[end..]);
    let errors = spec(&two).err().unwrap_or_default();
    assert!(errors.is_empty(), "{errors}\n{two}");
}

#[test]
fn a_later_branch_pinning_a_superset_is_unreachable() {
    let held = "      - name: held
        when:
          all:
            - kind == A
            - flag_a == false
            - flag_b == true
        error: demo.route.Closed
";
    let parked = "      - name: parked
        when:
          all:
            - kind == A
            - flag_a == false
            - flag_b == true
            - flag_c == false
        error: demo.route.Blocked
";
    let errors = refusals(&format!("{held}{parked}"));
    assert!(
        errors.contains(
            "[unreachable_branch] command.demo.route.Dispatch.outcomes.parked: `parked` is \
             unreachable: every input satisfying `(kind == A and flag_a == false and flag_b == \
             true and flag_c == false)` is answered first by `held` (kind == A and flag_a == \
             false and flag_b == true), an input-guarded refusal declared before it (hint: \
             declare `parked` before `held` to answer those inputs with it, or drop it)"
        ),
        "{errors}"
    );
    assert_eq!(errors.matches(NEEDLE).count(), 1, "{errors}");
}
