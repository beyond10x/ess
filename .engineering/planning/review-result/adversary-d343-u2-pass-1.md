---
format: aep.planning-md/3
id: review-result:adversary-d343-u2-pass-1
kind: review-result
status: active
title: 'D343 U2 adversary pass 1: input type rules lower typed or are refused'
relations:
- reviews: story:entity-runtime-lowering-enforces-input-type-invariants
revision: 1
---
unit: story:entity-runtime-lowering-enforces-input-type-invariants, tree ess-d343-u2 at 453cdd71a (plus my untracked test file)
verdict: NEEDS-CHANGE
cases: executed 181→188, red 2
origin: introduced 3 / pre-existing 0 / undecided 1
wrote-outside-worktree: 6 paths (part 6)
needs-coordinator: none

**1. `git --no-pager diff --stat`**: empty. `git status --short` shows one untracked file, `?? crates/generate/ess-entity-runtime/tests/adversary_d343_u2_pass1.rs`, a test file. Nothing committed.

**2. Cases** in `crates/generate/ess-entity-runtime/tests/adversary_d343_u2_pass1.rs` (for the update cases a refusal naming the input also counts as a pass):
- `adv_d343_u2_a_timestamp_ordering_invariant_of_an_update_input_admits_a_valid_instant`: **red** (first draft did not compile on a fixture error; not counted). `CancelInvoice` with `{"since":"2026-01-01T00:00:00Z",…}` takes `PreconditionUnobservable { rule: "nominal_billing_invoice_CancelInvoice_input_since_0000", message: "value >= \"2000-01-01T00:00:00Z\"" }`, expected `cancelled`.
- `adv_d343_u2_a_text_ordering_invariant_of_an_update_input_admits_a_valid_value`: **red**. `CancelInvoice` with `{"code":"q",…}` takes `PreconditionUnobservable { rule: "nominal_billing_invoice_CancelInvoice_input_code_0000", message: "value >= \"m\"" }`, expected `cancelled`.
- Green: a struct input with a newtype member and an `Optional` newtype member; an `Optional<List<Tag>>` input with `prefix:`; three rules across a two-layer newtype chain; one type used as the unstored input of a creation and of an update (only `CreateInvoice.input.tag` refused).
- `adv_d343_u2_a_creation_input_stored_on_one_of_two_accepting_branches_is_refused_by_name`: green, aimed at the mutant `all`→`any` in `stored_on_every_accepting_branch`; not run against the mutant. First ran inside the suite rather than alone (procedure breach); run alone afterwards, passed.

**3. Suite run**: `cargo test -p ess-entity-runtime --locked --no-fail-fast` → 186 passed, 2 failed (the two red cases), exit 101. "Before" is 188 minus the 7 new cases; the suite was not run before writing them.

**4. Findings**
- **F1, in scope, NEEDS-CHANGE, introduced** (`src/lib.rs:2221`): `input_type_rules` → `lower_nominal_invariants` → `lower_predicate` lowers a type rule without type information. An ordering on a `Timestamp` or a text becomes a plain `compare` over two strings, which entity-core 0.28.0 answers `Unknown` when no scale is declared (`runtime.rs:2355-2359`), so every request, valid or not, fails `PreconditionUnobservable`. Before the unit the rule was dropped (update accepted everything); now it refuses everything. The reference page row claims such a rule lowers to `preconditions`. No model pinned in the TSV has such a rule. Fix: lower with typing as guards do (Timestamp → `before`/`after`, text ordering refused by name), or refuse as `InputInvariantUnsupported`.
- **F2, in scope, note, introduced** (`src/subset.rs:121`): the refused row says "on an input no field of that type stores" and the code description "writes the input into no field of that type", but the code refuses unless every accepting branch stores the input, so a partly stored input is refused while the page says otherwise.
- **F3, out of scope, note, introduced** (`src/lib.rs:2193` doc): entity-core answers a refusing branch (step 6) before preconditions (step 7), so an input ESS rejects at admission can get a domain error instead (PayInvoice `rejected` for a negative Money). Nothing is accepted; the error identity differs from ESS.
- **F4, out of scope, undecided** (`src/lib.rs:1499`): the same untyped ordering lowering applies to stored-field type rules (entity invariants). Inferred from code, not run.

**5. Could not break**: several members of a struct input; an absent `Optional` input or member; a list `ForAll` over the empty list; each rule of a newtype chain refusing on its own; unique rule names; the creation refusal naming only its own input when the type is shared with an update; identity-from-input and `sets:`-stored skips. The unit's tests assert literal outcomes.

**6. Paths written outside the worktree**: the wave scratch directory `u2/adv-scratch/` and in it `run1.log`, `run2.log`, `run3.log`, `suite.log`, `suite2.log`.

```findings
[
  {"file": "crates/generate/ess-entity-runtime/src/lib.rs", "line": 2221, "category": "acceptance", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "F1 in: an update input type's Timestamp or text ordering invariant lowers untyped to a string compare entity-core answers Unknown, so every request, valid or not, fails PreconditionUnobservable (red: adv_d343_u2_a_timestamp_ordering_*, adv_d343_u2_a_text_ordering_*)"},
  {"file": "crates/generate/ess-entity-runtime/src/subset.rs", "line": 121, "category": "contract-drift", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "F2 in: the refused row and code description say 'no field of that type stores' while the code refuses unless every accepting branch stores the input"},
  {"file": "crates/generate/ess-entity-runtime/src/lib.rs", "line": 2193, "category": "judgement", "severity": "note", "verdict": "INFEASIBLE", "origin": "introduced", "message": "F3 out: a refusing branch answers before preconditions, so an input ESS rejects at admission can get a domain error instead of an input refusal; nothing is accepted"},
  {"file": "crates/generate/ess-entity-runtime/src/lib.rs", "line": 1499, "category": "judgement", "severity": "note", "verdict": "INFEASIBLE", "origin": "undecided", "message": "F4 out: the same untyped ordering lowering applies to stored-field type rules as entity invariants; inferred from code, not run"}
]
```
