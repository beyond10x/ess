---
format: aep.planning-md/3
id: story:validation-refuses-an-exists-false-branch-declared-below-a-refusal-it-answers-before
kind: story
status: draft
title: 'Validation refuses an exists: false branch declared below a present-row refusal over another row that it answers before'
relations:
- serves: vision:O2
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: inferred
  path: crates/edge/ess-cli/tests/
- confidence: cited
  path: crates/specify/ess-domain/src/command.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command/precedence.rs
- confidence: cited
  path: crates/specify/ess-domain/tests/held_state_order.rs
- confidence: inferred
  path: website/docs/reference/predicates.md
revision: 7
---
## Outcome

`ess specify validate` refuses a command in this shape: a `when_related:` predicate refusal or
row-set refusal over one row is declared above a `when_related: {via: …, exists: false}` branch
over a different row, and one request can satisfy both. The refusal is `conflicting_declaration`.
Its hint says the `exists: false` branch answers first, and names `existing_instance:` when that
branch's `via` is the identity it creates. A command that validates then gets the same answer
from a single pass in declaration order as from the precedence plan. No authored scenario
expecting the refusal passes authoring and then fails at run time with nothing pointing to the
order.

## Need

Reproduced on the released 0.97.1 binary, and on 0.97.0. Generic model `d2/v-fail-with-default`:

```yaml
- name: shop.order.PlaceOrder
  input: [order_id, customer_id]
  outcomes:
    - name: no-customer
      when_related: {via: input.customer_id, exists: false}
      error: shop.order.Refused
    - name: blocked
      when_related: {via: input.customer_id, predicate: blocked == true}
      error: shop.order.Refused
    - name: placed
      when_related: {via: input.order_id, exists: false}
      creates: shop.order.Order
      instance: order_id
      ...
    - name: duplicate
      error: shop.order.Duplicate
```

The scenario opens customer `c1` with `blocked: true`, then calls `PlaceOrder(o1, c1)` and expects
`blocked`.

```console
$ ess specify validate --path .
shop v1 — 2 file(s), 1 scenario(s), valid
$ ess verify conform author --path . --scenarios scenarios --out suite.json
1 authored scenario(s) from 1 file(s), 0 refusal(s), suite ess-conformance/34, written to suite.json
$ ess verify conform run --path . --suite suite.json --target interpreted --report-format 2 --report-out report.json --format json
exit 1
  failed outcome shop.order.PlaceOrder/blocked | expected outcome = blocked observed outcome = placed | shop.order.PlaceOrder(customer_id = "c1", order_id = "o1")
```

Boundary, each varying one thing (all outputs in `run.txt`):

| variant | result |
|---|---|
| `placed` without `when_related` (control) | `blocked`, passes |
| order `o1` exists before the call | `blocked`, passes: the predicate is evaluated wherever `placed`'s `exists: false` does not hold |
| `placed` declared before `blocked` | same answer, `placed`: the plan, not the declaration order, decides |
| `blocked` as a row-set guard, `where: {all: [customer_id == input.customer_id, blocked == true]}` | same answer, `placed` |
| uniqueness through `duplicate: existing_instance: true` and an unguarded `placed` | `blocked`, passes. This is the spelling that means what the reporter wants. |
| without the `duplicate` default, or without `no-customer` | refused as `non_exhaustive_branches` |

## Cause

Verified by reading the code and by the boundary above.

- The interpreter answers by the plan, and the plan follows its design. Under the one rule
  (`ess/27` and later), `precedence::place` puts every `exists: false` over a row the input names
  at `PresentRelated` with `Rank::Lead`, whether it refuses or creates
  (`crates/specify/ess-domain/src/command/precedence.rs:484-487`). A predicate or row-set refusal
  goes at `PresentRelated` with `Rank::Declared` (`:534-536`). The interpreter answers a missing
  input-named row at `PresentRelated` before `select_phases`
  (`crates/verify/ess-conformance/src/interpret/execute.rs:824-839`, `related_absent` at
  `:1612`). Order `o1` is missing, so `placed` answers before `blocked` is read.
- `docs/design/generic-language-features.md:496`, step 4: "first existence (`exists: false` over a
  row the input or a stored field names, in declaration order), then predicate and row-set
  refusals over present rows". `docs/design/selection-plan.md:45` says the same.
- The same page also states the rule as "Within one step, the first declared whose condition
  holds" (`:488`). It promises that "Validation refuses a declaration order that contradicts the
  step order wherever two branches' guards can both hold … A consumer reading one
  declaration-order pass then gives the rule's answer" (`:516-519`).
- Validation does not keep that promise inside step 4. `validate_step_order`
  (`crates/specify/ess-domain/src/command.rs:3948-4026`, from `ess/29`, `STEP_ORDER_DECLARED`
  `:4777`) compares branches of different phases only (`read.phase.position() >
  place.phase.position()`, `:3991`). It also skips every branch not at `Rank::Declared` (`:3986`),
  on the premise that "a marker below a guarded branch … answers its step's question" (`:3943-3944`).
  That premise holds for an `exists: false` over the same row as the refusal above it, because the
  row is either missing or present. It fails for an `exists: false` over another row: there both
  can hold.

The faulty component is validation. The interpreter is right by the plan, and synthesis agrees:
on the reporter's model of this shape, the synthesized suite witnesses the predicate refusal only
against an existing created row, and passes on the interpreted target (`run.txt`). The generated Rust and Go emitters and the Entity
Runtime lowering read the same plan (`selection-plan.md:3-4, 51-52`; inferred, not run).
`ess verify conform mutate` replays against the interpreter (inferred). Authoring is not wrong
either: `ESS-AUTHOR-041` decides input guards only, and accepts an expectation that a related row
decides (`crates/verify/ess-conformance/tests/authored_outcome_guards.rs:1-11`).

## Fix

- Extend `validate_step_order` in `crates/specify/ess-domain/src/command.rs` with one case. A
  branch placed at `PresentRelated` with `Rank::Lead` that is an `exists: false` over an
  input-named or stored row, declared below a `PresentRelated` branch with `Rank::Declared` (a
  predicate or row-set refusal), is refused when two conditions hold. First, the two branches read
  different rows: a different `via` field, or any row-set selector. Second, their input guards can
  hold together (the existing `together`).
- Message: "`placed` answers at step 4 before `blocked`, declared above it, wherever both hold".
- Hint: "declare `placed` before `blocked`". Where the `via` is the identity the branch creates,
  the hint adds: "for a refusal of an identity that already exists, declare `existing_instance:`
  and drop the `when_related`".
- Same gate as the rule it completes (`STEP_ORDER_DECLARED`, `ess/29`), unless D-2 decides
  otherwise. About 30 to 40 lines, with no new `ValidationCode`, no plan change and no evaluator
  change.
- Add a `CHANGELOG.md` entry, and a sentence in `website/docs/reference/predicates.md` (the
  `when_related` row).

## Acceptance

- Write the failing test first, as a case in `crates/specify/ess-domain/tests/held_state_order.rs`
  (the step-order target): `an_exists_false_over_another_row_declared_below_a_present_row_refusal_is_refused`.
  The model is `d2/v-fail-with-default`, refused as `conflicting_declaration` naming `placed` and
  `blocked`. The same case covers the row-set variant `d2/v-rowset-all`.
- Controls in the same target, which keep validating:
  - `exists: false` over the same row as the predicate refusal, declared in either order (`no-customer` below `blocked`);
  - `placed` declared above `blocked`;
  - `d2/v-existing-instance`;
  - `d2/v-control-no-via`;
  - the same shape at `ess/28`.
- The suite authored from `d2/v-existing-instance` passes on `--target interpreted`, as an
  `ess-cli` test. `ess-cli` is gated too, because `ess-domain` changed.
- Every committed fixture and example still validates, or each one now refused is named and
  re-declared in the same change. Check this through the `ess-domain`, `ess-cli`,
  `ess-conformance` and `ess-synth` fixture tests. Run `ess-conformance` one `--test` at a time,
  per `AGENTS.md`.
- Gate: `task test-xtask` and `cargo test -p ess-compiler --test typed_diagnostics`. The
  repository-wide byte pins listed in `AGENTS.md` do not move.
- No test over 60 s, and the test step stays within 120 s.

## Fit review

1. Need: a command refuses a request over one related row (a blocked customer) and creates a
   record under a uniqueness check spelled as `exists: false` over another row. The refusal is
   declared first, yet the creation answers whenever the new identity is free. Validation and
   authoring say nothing, and the run fails. The reporter expects declaration order, and proposes
   no syntax.
2. Class: defect. Validation does not hold `generic-language-features.md:516-519` within step 4,
   and the page's two statements (`:488`, `:496`) disagree for this shape without a refusal
   reconciling them.
3. Expressible today: yes. `existing_instance:` for the identity the command creates, with an
   unguarded creating branch, answers the refusal first and keeps the uniqueness refusal
   (`d2/v-existing-instance`, passes). The reporter's workaround drops the uniqueness refusal
   instead.
4. G4 precedence, step 4: no construct is added. The refusal applies to every pair inside the step
   where both can hold, like the cross-step rule.
5. A second adopter: a booking command refuses a suspended member (a predicate over the member
   row) and creates a booking under `exists: false` over the booking id. It has the same silent
   reordering.
6. Cost: one refusal, no format, no keyword, no diagnostic code. A model that validates on 0.97.1
   and has this shape is refused from `ess/29` (D-2).
7. Considered: (a) change nothing: rejected, because the declared order stays silently overridden;
   (b) change the plan to read declaration order within step 4, or to move a creating
   `exists: false` to step 5: rejected, because that changes the answer of every consumer, which is
   a meaning change and needs a format; (c) refuse, chosen.

## Decisions

- Accepted as a defect, redesigned, by the planning session on 2026-11-22. The interpreter's answer
  stands. Validation refuses the declaration order that the plan overrides.
- D-2, open, with a default: gate the refusal at `ess/29`, with the step-order rule it completes.
  The alternative is to refuse only from the next format.
- This is a draft. No wave and no release is ordered.

## Out of scope

- Changing `precedence::place` or the order of step 4.
- Teaching authoring to decide expectations that depend on related rows.
- `story:a-create-or-update-command-refuses-a-missing-related-row-before-it-creates` (draft):
  the same step, a different shape (`unknown_instance:` against `exists: false`). Not covered here.

## Scope

- cited: crates/specify/ess-domain/src/command.rs
- cited: crates/specify/ess-domain/src/command/precedence.rs
- cited: crates/specify/ess-domain/tests/held_state_order.rs
- inferred: website/docs/reference/predicates.md
- inferred: CHANGELOG.md
- inferred: crates/edge/ess-cli/tests/
