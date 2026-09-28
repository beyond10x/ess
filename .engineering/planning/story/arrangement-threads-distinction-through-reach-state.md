---
format: aep.planning-md/3
id: story:arrangement-threads-distinction-through-reach-state
kind: story
status: active
title: 'ESS-SYNTH-004: a state reached only through a when_subject-selected branch cannot be arranged for another transition, though synthesis arranges it for that branch''s own scenarios'
refs:
- provider: github
  reference: beyond10x/ess#199
relations:
- serves: vision:O2
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-09-28T17:59:47Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-09-28T17:59:47Z", actor: "human:timo", revision: 3}
---
# Story: ESS-SYNTH-004: a state reached only through a when_subject-selected branch cannot be arranged for another transition, though synthesis arranges it for that branch's own scenarios

## Why

beyond10x/ess#199. Coordinator decision: `.engineering/waves/ess-0.41-decisions.md` row #199 (summarised in the wave page).

## Scope (story-scoper, 2026-09-28, on e9327819658583ae45953acafa3e41d3bbd5b7f4)

- **Reproduces on this tree (cited).** `from_source` (`crates/verify/ess-conformance/src/synthesize.rs:7122`) arranges each further source through `arrange_unbound` (`synthesize.rs:7051`). That function calls `arrange` with `Distinction::further(nth)` and never with `PLAIN` (`synthesize.rs:7059`).
- **The failing step (cited).** For `Shipped`, `advance` (`synthesize.rs:2838`) walks the route `PlaceUnpaidOrder`, then `ShipOrder/shipped`. `ShipOrder` reads a stored field, so it calls `subject_fact::step` (`synthesize/subject_fact.rs:1408`). That returns `None`, because the plain row has `paid == false` and selects `not-paid`.
- **Why the search never runs (cited).** The fallback search `subject_fact::reach_state` is only taken when `distinction == Distinction::PLAIN && arranging.is_empty()` (`synthesize.rs:2856`). Every other case returns `Unreachable::Unwitnessable` (`synthesize.rs:2865-2872`), which is exactly the reported "the route runs through `…`, which no input reaches" (`synthesize.rs:1286-1291`).
- **Why the branch's own scenarios pass (inferred).** `ShipOrder/outcome/shipped` and the `Shipped/refuses/ShipOrder` scenario arrange at top level with `PLAIN`, so they hit the fallback that inserts `PayOrder`.
- **Not already fixed (cited).** HEAD is `e932781965 chore: release 0.40.0`, and the guard at `synthesize.rs:2856` is unchanged.
- **Fix site 1 (cited).** `subject_fact::reach_state` (`synthesize/subject_fact.rs:1426`) hard-codes `Distinction::PLAIN` into `search` (`subject_fact.rs:1107`, called at line 1447). It needs a `distinction` parameter. `search` and `creations` (`subject_fact.rs:951`) already take one.
- **Fix site 2 (inferred).** `advance`'s fallback at `synthesize.rs:2856` has to accept a non-plain distinction and pass it through. `arrange_unbound` depends on non-plain distinctions to keep instance names disjoint from the ones already bound (`synthesize.rs:7058-7065`).
- **Open question on the `arranging` nest (inferred).** The `arranging.is_empty()` half of the guard (owner-arrangement recursion, `arrange_owner` `synthesize.rs:3005`) may need the same relaxation. It is out of scope unless a test shows it.
- **Test home (inferred).** `crates/verify/ess-conformance/tests/when_subject_witness.rs` already covers SYNTH-004 and when_subject. The issue's shape gives the new case: `close` from `[Placed, Shipped]` must yield the `Shipped` source steps `PlaceUnpaidOrder → PayOrder → ShipOrder → CloseOrder`.

### Collisions

| issue | overlap | basis |
|---|---|---|
| #198 | High. Same module and search: the creator choice in `subject_fact::search`/`creations`, and `arrange` "ties to the first creator declared" (`synthesize.rs:2796-2797`). The #199 fix changes `reach_state`/`search` signatures. Sequence #198 and #199 together or in one unit. | cited for the location, inferred for the overlap |
| #209 | Medium. Same file `synthesize.rs` (`refusal_arrangement` `:6962`, `arrange`), different function. Merge conflict risk only. | inferred |
| #201 | Low to medium. `state_refusals` / wrong_state in `synthesize.rs` (`:2248`) plus command validation in ess-domain. | inferred |
| #196 | Low. Map witnesses live in the `candidates`/`witness.rs` path. | inferred |
| #210 | None. `src/mutate.rs`. | inferred |
| #195 | None in conformance arrangement. It is binding validation (ess-domain/compiler) plus `bindings()` `synthesize.rs:8396`. | inferred |

### Design decisions for the implementor

1. How to name the searched instance under a non-plain distinction. Either thread `Distinction` through `reach_state`/`search`, or keep `PLAIN` and rename the result. The first matches `arrange_unbound`'s disjointness loop. (inferred)
2. Whether the fallback also applies inside owner arrangement (`arranging` non-empty). The recursion guard exists to stop two entities that own each other from arranging forever (`synthesize.rs:2781-2783`). (inferred)
3. Whether `other_sources` should skip `subject_fact::routes` commands (`synthesize.rs:7103`) the way it does today. It does not touch the #199 case, where `CloseOrder` has no subject guard. It matters if the fix is generalised. (inferred)

**Confidence:** high on the mechanism. It was traced by reading the code; nothing was built or run.

**Paths**
- crates/verify/ess-conformance/src/synthesize.rs
- crates/verify/ess-conformance/src/synthesize/subject_fact.rs
- crates/verify/ess-conformance/src/witness.rs (inferred: only if `Distinction` needs a helper)
- crates/verify/ess-conformance/tests/when_subject_witness.rs (inferred: test home)

**Verdict:** open
