---
format: aep.planning-md/3
id: review-result:adversary-c-relguard-pass-2
kind: review-result
status: active
title: Adversary pass 2, 0.41 unit related-guard
relations:
- reviews: story:a-guard-reads-another-entity-by-identity
revision: 1
---
unit: related-guard (beyond10x/ess#211), correction 1, working tree ~/.local/state/worktree/trees/b10x/ess/ess-c-relguard on b7ecd8d30e (uncommitted)
verdict: NEEDS-CHANGE
cases: executed 2579→2593, red 6
origin: introduced 4 / pre-existing 0 / undecided 0
wrote-outside-worktree: 11 paths, all under ~/.cache/ess-wave-n2/relguard/adv2/
needs-coordinator: none

## 1. Diff stat

`git --no-pager diff --stat`: `27 files changed, 725 insertions(+), 99 deletions(-)`. That is the implementor's tracked diff, unchanged. I added two untracked test files and nothing else:

- crates/verify/ess-conformance/tests/adversary_related_guard_pass2.rs
- crates/specify/ess-domain/tests/adversary_related_guard_pass2.rs

No implementation file was touched.

## 2. Cases added (run alone before the suite)

| # | case (conformance unless noted) | asserts | now |
|---|---|---|---|
| A1 | `..._orders_arranged_beside_a_ranked_read_carry_distinct_supplied_ids` | with a ranked order view, every scenario placing several orders gives each its own supplied `order_id` | red |
| A2 | `..._a_correct_target_passes_scenarios_with_ranked_companions` | a correct target passes every scenario of that model | red |
| A3 | `..._control_without_the_related_guard_orders_carry_distinct_supplied_ids` | the same model without `when_related` places distinct ids | green (control) |
| B1 | `..._a_correct_target_passes_every_move_scenario_on_a_related_guarded_row` | `close: from [Open, Held]` is also witnessed from `Held` (#111) on an order created through a related-guarded create | red |
| B2 | `..._two_orders_in_one_scenario_are_placed_under_distinct_supplied_ids` | same model: some scenario places two orders, each under its own id | red (no scenario does; same cause as B1) |
| C1 | `..._a_create_guarded_on_its_own_owner_is_witnessed` | `PostEntry` (creates an `Entry` owned by `Account`) guarded by `when_related {via: input.account_id, exists: false}` is synthesized, `AmendEntry` still is, and a correct target passes | red |
| D1 | `..._a_target_reading_one_conjunct_of_the_related_predicate_fails_a_scenario` | a target reading `plan == Basic` alone for `all: [plan == Basic, region == North]` fails some scenario | red |
| D2 | `..._a_predicate_over_two_related_fields_is_witnessed_and_passes_a_correct_target` | all three branches run and pass | green |
| D3 | `..._a_target_reading_the_first_row_for_a_two_field_predicate_fails` | a first-row mutant fails both predicate branches | green |
| E1 | `..._a_cycle_two_entities_long_stops_and_is_witnessed_through_the_seed` | Alpha needs Beta, Beta needs Alpha, and Alpha has an unguarded seed: all four branches are synthesized and pass, and synthesis is deterministic | green |
| F1 | `..._existing_instance_beside_exists_false_and_a_predicate_answers_first` | `existing_instance` with both shapes: all four branches pass, and a related-first mutant fails `duplicate` | green |
| G1 | `..._synthesis_of_related_guards_is_deterministic` | four models synthesized twice give equal suites and refusals | green |
| H1 (domain) | `..._two_related_predicate_refusals_true_on_one_row_are_refused` | overlapping enum predicates are refused with `conflicting_declaration` | green |
| H2 (domain) | `..._two_disjoint_related_predicate_refusals_are_admitted` | the disjoint version is admitted | green |

The first runs had three typos of mine, none of them findings:
- `&&` in a compact predicate: refused at parse, so I respelled it `all:`.
- the in-memory service panicked on an illegal move instead of answering `undeclared`.
- B1 counted `ESS-SYNTH-012` in its refusal list. That code is about my model lacking `wrong_state:`.

I fixed all three and reran. The lines below are from those reruns (`red-conformance-2.log`, `red-conformance-ranked.log`, `red-conformance-companions.log`).

```
A1: assertion `left == right` failed: demo.orders.CloseOrder/outcome/closed places two orders under one supplied id: [
    "Literal { value: Text(\"order_id\") }",
    "Literal { value: Text(\"order_id\") }",
]
A2: a correct target fails: { "demo.orders.CloseOrder/outcome/closed": "Failed: ...", "demo.orders.HoldOrder/outcome/held": ..., "demo.orders.Order/transition/close/by/demo.orders.CloseOrder/closed": ..., "demo.orders.Order/transition/hold/by/demo.orders.HoldOrder/held": ..., "demo.orders.PlaceOrder/outcome/placed": ... }
    each: about: "outcome demo.orders.PlaceOrder/placed", status: Failed ... expected: ["outcome = placed"], observed: ["outcome = duplicate"]
B1: assertion `left == right` failed: the move from `Held` is witnessed (#111)
  left: ["ESS-SYNTH-004: it needs an instance of `demo.orders.Order` resting in `Held`, and no declared move reaches it from `Open`"]
C1: ledger.book.PostEntry/outcome/posted is synthesized; refusals: [
    "ESS-SYNTH-004: it needs an instance of `ledger.book.Entry` to change without moving, and the route runs through `ledger.book.PostEntry/posted`, which no input reaches",
    "ESS-SYNTH-008: its command's strategy is `arrange_related_row`: a row of another entity the input names selects its branch, and this scenario family arranges none",
D1: a target reading only [("plan", "Basic")] passes every scenario: {
    "demo.signin.InitiateSignIn/outcome/initiated": "passed",
    "demo.signin.InitiateSignIn/outcome/no-configuration": "passed",
    "demo.signin.InitiateSignIn/outcome/no-redirect-entry": "passed",
```

## 3. Suite runs (after the cases existed)

- `cargo xtask generate --check`: `projections are up to date`, EXIT=0. The committed generated suites did not change.
- `cargo test -p ess-conformance --no-fail-fast`: 1556 passed, 6 failed, EXIT=101. The six failures are A1, A2, B1, B2, C1 and D1. The c1 gate reported 1550, so every one of those 1550 existing cases still passes. The broad lanes (synthesis, arrangement, owners, aggregates, set_effects, mutation) are all part of this package run.
- `cargo test -p ess-domain --no-fail-fast`: 1031 passed, 0 failed, EXIT=0 (1029 before).

## 4. Findings

1. **blocker, NEEDS-CHANGE, introduced**: `crates/verify/ess-conformance/src/synthesize/related_guard.rs:487`.
   - `drive` takes no distinction. `with_row` chooses the driven creator's input at `Distinction::PLAIN`, whatever distinction `created`/`invoke` was asked for.
   - So every further row made through a related-guarded create reuses the same caller-supplied identity. With a ranked view, the companions are placed as `order_id = "order_id"` twice.
   - A correct target answers `duplicate`, and five scenarios fail on it, `PlaceOrder/outcome/placed` among them.
   - What was measured: A1 and A2 red, A3 green as the control.
   - What reaches it: any `creates:` with a supplied identity (ess/16) plus `when_related`, in any family that arranges more than one row: ranked companions, the #111 other sources, `related` decoys.
   - Fix to name: pass the caller's `distinction` into `drive`/`with_row` for the input search.
2. **warning, NEEDS-CHANGE, introduced**: `crates/verify/ess-conformance/src/synthesize/related_guard.rs:264`.
   - The driven related row sits at a fixed `BLOCK * DRIVEN`, so its instance name is the same on every drive.
   - `arrange_unbound` never finds names disjoint from the scenario's, and gives up with `NoPath`. The #111 witness from a second source state is refused (`ESS-SYNTH-004 ... no declared move reaches it from Open`).
   - What was measured: B1 and B2.
   - What reaches it: a move with several `from` states on an entity created through a related-guarded create.
3. **blocker, NEEDS-CHANGE, introduced**: `crates/verify/ess-conformance/src/synthesize/related_guard.rs:330` (and `:242` in `drive`).
   - When the related row is the created row's owner (`owns ... via: account_id`, and the guard reads `input.account_id`), both `prepare` and `drive` refuse because `bound` already holds `account_id`.
   - `PostEntry/outcome/posted` is not synthesized. Every scenario needing an `Entry` (`AmendEntry`) is refused ESS-SYNTH-004. So adding the guard removes coverage of the owned entity's later commands.
   - What was measured: C1.
   - What reaches it: the natural "post into an account that exists" shape. The docs say the guard composes "with any subject a branch names, a `creates:` included".
   - Fix to name: when `via` is the owner field, point the input at the arranged owner instead of refusing.
4. **note, CONFIRMED, introduced**: `crates/verify/ess-conformance/src/synthesize/related_guard.rs:472`.
   - For `all: [plan == Basic, region == North]`, a target that drops `region == North` passes every scenario.
   - The search witnesses each predicate true once and false once, not each conjunct.
   - What was measured: D1.
   - What reaches it: any conjunctive related predicate. The same one-true/one-false contract as `when_subject`, which I did not run against the base.

## 5. Attacked, not broken

- Regressions from threading the arranging chain: `generate --check` is clean, and all 1550 prior conformance cases and 1029 prior domain cases pass.
- Cycle two entities long (A needs B needs A): terminates, witnessed through the seed, and passes (E1).
- Several events of one type: the folder shape passes pass-1 F1. `CaptureInstance` reads `last_command.direct_events` (`runner.rs:1798`), so earlier `FolderCreated` events from the parent and decoys do not leak in.
- A created row with no event: not constructible. The resolver publishes a `creates:` identity from an event only (`resolve.rs` `InstanceSurface::EmittedEvent`, `synthesize.rs` `created` `unreachable!`).
- Predicate over two related fields: witnessed, and the first-row mutant is killed (D2, D3).
- Overlapping predicate branches: refused over finite fields (H1), and disjoint ones admitted (H2). Over open domains a default is required, the same as `when_subject` (`subject_fact.rs` partition).
- `existing_instance` with both shapes: answers first, and the related-first mutant is killed (F1).
- Determinism (G1).
- The pass-1 domain assertion edits (`adv1-domain-assertions.diff`) match decision #211 (2026-09-29) item 2: D1 now expects `conflicting_declaration` naming `when_related`, and D2 expects admission because `exists: false` answers first.
- The `identifying` change (an Observed subject with a bound instance now uses the instance): every non-related caller either passes `None` or has a `Supplied` subject, and `generate --check` confirms no committed suite moved.

## 6. Paths written outside the worktree

- ~/.cache/ess-wave-n2/relguard/adv2/review.md
- ~/.cache/ess-wave-n2/relguard/adv2/red-domain.log
- ~/.cache/ess-wave-n2/relguard/adv2/red-conformance.log
- ~/.cache/ess-wave-n2/relguard/adv2/red-conformance-2.log
- ~/.cache/ess-wave-n2/relguard/adv2/red-conformance-ranked.log
- ~/.cache/ess-wave-n2/relguard/adv2/red-conformance-companions.log
- ~/.cache/ess-wave-n2/relguard/adv2/control.log
- ~/.cache/ess-wave-n2/relguard/adv2/mutant-first-row.log
- ~/.cache/ess-wave-n2/relguard/adv2/generate-check.log
- ~/.cache/ess-wave-n2/relguard/adv2/suite-ess-conformance.log
- ~/.cache/ess-wave-n2/relguard/adv2/suite-ess-domain.log
- Build output went into the assigned ~/.cache/b10x-target/ess-c-relguard.

## 7. Findings block

```findings
[
  {"file": "crates/verify/ess-conformance/src/synthesize/related_guard.rs", "line": 487, "category": "boundary", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "drive ignores the requested distinction and searches the creator's input at Distinction::PLAIN, so every further row made through a related-guarded create reuses one supplied identity and a correct target answers duplicate"},
  {"file": "crates/verify/ess-conformance/src/synthesize/related_guard.rs", "line": 264, "category": "boundary", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "the driven related row is always arranged at BLOCK*DRIVEN, so arrange_unbound never finds disjoint instance names and the #111 other-source witness is refused"},
  {"file": "crates/verify/ess-conformance/src/synthesize/related_guard.rs", "line": 330, "category": "acceptance", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "a create guarded on its own owner's row is refused because the owner arrangement already binds the via field, which also makes every later command on the owned entity unwitnessable"},
  {"file": "crates/verify/ess-conformance/src/synthesize/related_guard.rs", "line": 472, "category": "mutant", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "a target dropping one conjunct of a conjunctive related predicate passes every scenario"}
]
```
