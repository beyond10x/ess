---
format: aep.planning-md/3
id: story:synthesis-reads-selection-plan
kind: story
status: implemented
title: Synthesis asks the selection plan which branches answer before a witness
relations:
- decomposes: epic:one-selection-plan
- serves: vision:O2
- depends_on: story:selection-plan-design-and-type
scope:
- confidence: inferred
  path: crates/verify/ess-conformance/src/authored.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/precedence.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/related_guard.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/related_guard/stored.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/row_set.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/external_beside_held_guard.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/fixtures/external-beside-held-guard-base.tsv
revision: 10
transitions:
- {from: "draft", to: "proposed", at: "2026-10-07T23:43:18Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"review_outcome":3}}}
- {from: "proposed", to: "active", at: "2026-10-07T23:43:19Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"review_outcome":3}}}
- {from: "active", to: "implemented", at: "2026-10-08T06:14:18Z", actor: "human:timo", revision: 10, decided_on: {"recorded":{"test_result":2,"review_outcome":10}}}
---
# Story: Synthesis asks the selection plan which branches answer before a witness

## Why

`epic:one-selection-plan`. To write a branch's witness, synthesis has to refute every branch that
would answer first. It re-derives that set in several places: `sibling_refusals` and the #464 claim
search (`crates/verify/ess-conformance/src/synthesize.rs`), the stored-row search with
`Order::FirstDeclared` (`synthesize/subject_fact.rs`), and the related and row-set searches. Commit
`bafb78c` (#464) is the cost: witnesses were chosen to avoid where the interpreter and the design
disagreed.

## What it delivers

One query on `story:selection-plan-design-and-type`'s plan, "the branches that answer before this
one", used by every witness search in place of its own.

## Acceptance

- Unit 1 (before any synthesis code changes) adds a full-bytes pin: the suite and refusal bytes of
  every model the 193-row walker reads, claimable scenarios kept, written at the story's base. Every
  later unit leaves that table identical. `repository_model_suites_change_only_where_claimed` stays
  green too, but it strips claimable scenarios and is not the check.
- `sibling_refusals`, `earlier_accepting_branches`, `later_refusals`, the #464 claim search
  (`held_state_claims`, `held_state_refuted`), `subject_fact::selects`' `FirstDeclared` pick,
  `related_guard::selects` and `row_set::input_for`/`consistent` each call the one plan query; none
  matches on condition kinds to decide order.
- A test using `story:selection-plan-design-and-type`'s phase-order override, with the held-state
  and accepting/external phases exchanged, shows a witness change with it.

## Scope

Derived 2026-10-07 by `aep:story-scoper` on `985f58cc3a`. Every line is **cited** (read from the
story or the tree) or **inferred** (a reading that could be wrong). Paths are under
`crates/verify/ess-conformance/` unless shown in full.

- **Base:** the story starts from a `main` containing PR #487, which edits `tests/external_beside_held_guard.rs` (the epic gates every wave on it) — cited
- **Primary surface:** `src/synthesize.rs` and `src/synthesize/{subject_fact,related_guard,row_set}.rs` — cited
- **Code that derives "which branches answer before this one", with call counts** — cited (`git grep`, each site read):
  - `sibling_refusals` `synthesize.rs:6593`: 15 calls (9 in `synthesize.rs`: 3905, 6232, 6444, 6539, 6874, 6890, 13212, 13342, 13508; `subject_fact.rs:2328,5473,6921`; `related_guard.rs:322`; `existence.rs:627`; `authored.rs:1404`)
  - `earlier_accepting` / `earlier_accepting_branches` `synthesize.rs:6712-6736`: 13 calls
  - `later_refusals` `synthesize.rs:6613`: 3 calls (6337, 6383, 13345)
  - the #464 claim search (`bafb78c`): `held_state_claims` `:3917`, `held_state_refuted` `:3952`, inside `unclaimed_external` `:3702` and `driven_unclaimed` `:5528`; `reaches_external` `:3891`, `reach_external_beside` `:6218` (3 calls)
  - input refusal first, then first declared: `selected_in_state` `synthesize.rs:6065-6074`, `admits_plain` `:6851-6898`
  - `subject_fact.rs`: `Order` `:1810`, `selects` `:1828` (8 calls); `FirstDeclared` pick `:1887`; refusal-first retain `:1877-1882`; `leaves_external` `:2314` (4 calls)
  - `related_guard.rs`: `selects` `:770` (8 calls, one in `related_guard/stored.rs:401`), precedence at `:818-845`; `orders_present_related_refusal` `:853` (3 calls); `leaves_external` `:308`
  - `row_set.rs`: `input_for` refutes by declaration position `:321-338`; `consistent` derives `answers_first` `:681-702`
  - outside the module: `authored.rs:1390-1404` (`not_taken`) rebuilds `earlier_accepting` inline
- **Also likely:** new `src/synthesize/precedence.rs` holding the one query, a new byte-pin test and fixture; `src/authored.rs`, `src/synthesize/related_guard/stored.rs` — inferred
- **Acceptance gap** (cited): `repository_model_suites_change_only_where_claimed` (`tests/external_beside_held_guard.rs:1081`) removes every claimable scenario before the digest (`:958-966`, `:1132-1136`); only 2 of the 193 pinned models have claimable scenarios (`fixtures/external-beside-held-guard-base.tsv:102,141`). The story's named pin does not cover what it changes; a full-bytes pin has to come first.
- **Confidence:** high for where the code lands; medium for the split
- **Would collide with:** any unit on synthesis (`synthesize.rs`, `subject_fact.rs`, `related_guard.rs`, `row_set.rs`); `synthesize.rs` `overlaps` `:13205` and `refusal_pair_overlaps` `:13335` (`story:overlap-witnesses-per-phase`); `interpret/execute.rs:677` holds its own `orders_present_related_refusal` (`story:interpreter-reads-selection-plan`)
- **Safety fact:** every helper takes `(command, outcome)` and returns branches, so rewriting their bodies leaves the 31 call sites in place — inferred. Bytes stay the same only if the plan's phase order matches today's derivation for every repository command, which is `story:selection-plan-design-and-type`'s table — inferred

### Split (inferred)

| unit | lands on | after |
|---|---|---|
| U1 pin | new test + fixture: full suite and refusal bytes of every model the 193-row walker reads, claimable scenarios kept, pinned at the story base | — |
| U2 query | `synthesize/precedence.rs` (new); `synthesize.rs` 3702-3990, 5528-5545, 6032-6084, 6218-6240, 6565-6736, 6851-6898; `authored.rs:1359-1404`; phase-exchange test | U1 |
| U3 | `subject_fact.rs` `Order`/`selects`/`leaves_external` | U2 |
| U4 | `related_guard.rs` + `related_guard/stored.rs` | U2 |
| U5 | `row_set.rs` `input_for`/`consistent` | U2 |

U3, U4 and U5 touch different files and can run in parallel, each gated by U1's table.

### Design decisions for the implementor (inferred)

1. Keep the helpers' names and signatures; rewrite their bodies as filters over the plan's "before" query. Membership comes from the plan's phase, not from `is_input_guarded_refusal` (32 uses, many deciding a branch's family rather than order).
2. `Order::Unique` is not a precedence rule (it prefers an order-independent witness); keep it. Only the `FirstDeclared` pick and the refusal-first retain ask the plan.
3. Swap in a test plan through a scoped thread-local, as `with_dropped_write` does (`synthesize.rs:9382-9400`); keep the phase-exchange test in the crate.
4. Keep refusal texts byte-for-byte (`named_guards` `synthesize.rs:3973`, `external_claimed` `subject_fact.rs:2245`, `external_unwitnessed` `related_guard.rs:268`, `stored_external_refused` `related_guard.rs:213`).
5. Keep the `orders_present_related_refusal` signature so U4 does not touch `synthesize.rs`.

**Could not establish:** whether the plan reproduces today's order (the plan does not exist); the
full pin's run time; whether `authored.rs` `not_taken` belongs here; whether rewriting the #464
search as a cross-phase rule changes which witness is kept first.
