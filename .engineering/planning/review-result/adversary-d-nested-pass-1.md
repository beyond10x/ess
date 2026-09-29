---
format: aep.planning-md/3
id: review-result:adversary-d-nested-pass-1
kind: review-result
status: active
title: Adversary pass 1, 0.42 unit nested
relations:
- reviews: story:synthesized-inputs-satisfy-invariants-over-nested-members
revision: 1
---
unit: nested (beyond10x/ess#234), tree ess-e-nested, uncommitted diff on 300bfd3f5
verdict: NEEDS-CHANGE (5 blockers, 1 warning; 4 blockers are synthesized scenarios a correct target fails)
cases: executed 1726→1740, red 7
origin: introduced 0 / pre-existing 0 / undecided 6
wrote-outside-worktree: 4 paths (listed in part 6)
needs-coordinator: yes — gap (b) touched-refusal contradicts acceptance line 3; rule on deferral to #239

## 1. diff --stat (tracked)

```
 .../ess-conformance/src/synthesize/subject_fact.rs | 172 +++++++--
 crates/verify/ess-conformance/src/witness.rs       | 394 +++++++++++++++++++--
 website/docs/reference/predicates.md               |  14 +-
 3 files changed, 533 insertions(+), 47 deletions(-)
```
These three are the implementor's. Mine is one untracked test file: `crates/verify/ess-conformance/tests/adversary_nested_pass1.rs`. No non-test path touched.

## 2. Cases (file alone: 7 passed, 7 failed; `cargo test -p ess-conformance --test adversary_nested_pass1`, EXIT=101)

The shared property: no scenario in a synthesized suite fails against the interpreter of its own model. A refusal at synthesis with a named cause also passes the case.

| case | red/green | asserts |
|---|---|---|
| `adv_an_entity_invariant_out_of_bounded_reach_…` :217 | RED | 5-member strict chain `a<b<c<d<e`, which cannot be reached with the ladder values {-1,0,1,2} |
| `adv_an_entity_invariant_over_a_list_members_count_…` :240 | RED | entity invariant `fingerprint.parts.count == 2` |
| `adv_a_guard_moving_a_coupled_member_…` :265 | RED | invariant `origin == route` + guard `origin == "eu"` |
| `adv_a_guard_raising_an_ordered_member_…` :285 | RED | invariant `low < high` + guard `low > 10` |
| `adv_each_branch_meets_its_own_entitys_invariant_…` :411 | RED | two creating branches, entities pin `version` to "v2" / "v1" |
| `adv_a_further_instance_differs_on_a_member_an_invariant_bounds_…` :480 | RED | `start >= 5`: the PLAIN and further(1) witnesses differ at `start` |
| `adv_the_default_branch_beside_a_guard_over_a_stored_optional_struct_…` :597 | RED | `touched` is synthesized (acceptance line 3) |
| text length `digest.count == 64`, disjunction, `starts_with`, optional member, struct bounding two members, gap 4 via newtype, gap 5 `!=` refusal-first via newtype | green | — |

Verbatim red lines:

```
a correct target fails 2 synthesized scenario(s): {
    "demo.req.Submit/outcome/submitted": "... observed: [\"invoking `demo.req.Submit` failed: the model's own outcome leaves `demo.req.Request` `00000000-0000-4000-8000-000000000001` violating its invariant `fingerprint.origin == fingerprint.route`\"] ..."
refusals: []
```
```
... violating its invariant `fingerprint.low < fingerprint.high`\"] ...
refusals: []
```
```
... violating its invariant `fingerprint.a < fingerprint.b`\"] ...
refusals: []
```
```
a correct target fails 4 synthesized scenario(s): {
    "demo.two.Submit/outcome/current": "... violating its invariant `fingerprint.version == \\\"v2\\\"`\"] ...",
    "demo.two.Submit/outcome/old": "... violating its invariant `fingerprint.version == \\\"v1\\\"`\"] ...",
```
```
    "demo.req.Submit/outcome/submitted": "... violating its invariant `fingerprint.parts.count == 2`\"] ..."
```
```
assertion `left != right` failed: two instances carry one start: {"window": Map({"end": Number(... 1 ...), "start": Number(... 5 ...), "zone": Text("window.zone")})} / {"window": Map({"end": Number(... 2 ...), "start": Number(... 5 ...), "zone": Text("window.zone-1")})}
```
```
touched is another outcome of the command; refusals: [
    "demo.meta.Touch/outcome/gold-locked: ESS-SYNTH-003: no candidate of the 1 tried satisfies ``demo.meta.Item` stored meta selecting this branch, over the rows 1 bounded arrangements left`",
    "demo.meta.Touch/outcome/touched: ESS-SYNTH-003: no candidate of the 1 tried satisfies ``demo.meta.Item` stored meta selecting this branch, over the rows 1 bounded arrangements left`",
]
```
In the gap (b) model, `Annotate` writes `meta: {tier: Basic, note: {generated: true}}`, so `tier` is a literal the arrangement determines. `Make` alone leaves `meta` absent.

## 3. Suite (run after the cases existed)

`cargo test -p ess-conformance --no-fail-fast` → the last lines:
```
error: 1 target failed:
    `-p ess-conformance --test adversary_nested_pass1`
EXIT=101
```
Executed 1740 (1733 passed, 7 failed). The only failures are the 7 adversary cases. The before-count of 1726 is taken from the implementor's `after-conformance.log`.
`cargo clippy -p ess-conformance --test adversary_nested_pass1 -- -D warnings` → `Finished`. File formatted with rustfmt.

## 4. Findings

| file:line | severity | verdict | origin | finding | fix |
|---|---|---|---|---|---|
| crates/verify/ess-conformance/src/witness.rs:2356 | blocker | NEEDS-CHANGE | undecided | Gap (a) is the dangerous direction. When repair cannot solve an entity invariant, it silently keeps the base, and synthesis emits outcome and invariant scenarios that a correct target fails (chain, `.count`). Acceptance line 1 says "so its scenarios run". The predicates.md:810 sentence documents this false red as intended. | Carry the unsolved entity constraints out of `repair`. Refuse each outcome that copies the input into that entity with a named cause, e.g. ESS-SYNTH-003 "entity invariant `<stmt>` over input `<path>` is not met by any of the N bounded candidates". Change predicates.md:810 to say so. |
| crates/verify/ess-conformance/src/witness.rs:569 | blocker | NEEDS-CHANGE | undecided | Entity invariants are solved only for the base. The guard ladders then move a coupled leaf (`origin` to "eu", `low` to 11) and leave its partner where it was. `admitted_inputs` filters by type only, so the accepted input creates a row the invariant refuses. | Filter or re-solve every candidate against `entity_constraints`, as `admitted_inputs` does for the type. Or add the partner leaf's equality/order copies to the guard's ladder. |
| crates/verify/ess-conformance/src/witness.rs:2288 | blocker | NEEDS-CHANGE | undecided | Class 3. `entity_constraints` merges every branch's invariants into one conjunction. When two branches' entities conflict (v1 vs v2), the conjunction is unsolvable, and both branches send an invalid input with no refusal. | Solve per outcome. Candidates for an outcome are held to that outcome's written entities only. |
| crates/verify/ess-conformance/src/witness.rs:2417 | blocker | NEEDS-CHANGE | undecided | `solve` keeps only `Choice::Value`, so no candidate can change a list's length. `.count` over a struct list member, one of the shapes the issue names, gives a failing scenario. | Keep the list-length choices in the repair ladder, or refuse as in row 1. |
| crates/verify/ess-conformance/src/witness.rs:2870 | warning | NEEDS-CHANGE | undecided | Class 1. `fixed` pins a leaf to an invariant literal at every distinction. `start >= 5` bounds `start` without pinning it, yet PLAIN and further(1) both send `start=5`. The docs claim that instances differ wherever no invariant pins to one literal. | In `solve`, try the moved base first (the literal plus the distinction's ordinal, or the base shifted into range) before the bare literal. |
| crates/verify/ess-conformance/src/synthesize/subject_fact.rs:2127 | blocker | NEEDS-CHANGE | undecided | Gap (b): acceptance line 3 is "refuses at most its own branch, never the command's other outcomes". `touched` is refused although a row with `meta` absent (from `Make`) and a row with `tier: Basic` are both reachable. The implementor's own test `nested_input_invariants.rs` asserts the refusal. | Treat an undetermined optional-struct member as absent/Unknown for the default branch: reach `touched` on the `Make`-only row. Or get a coordinator ruling that moves line 3 to #239. |

Gap (c), `witness::fields` not repaired: struct invariants there are checked by `validate_typed_value`, which refuses with WitnessGap. That is an honest refusal and not a false red. Not tested further. Gap (d), `related_guard::selects`: not attacked.

## 5. Attacked and not broken

- Disjunctive entity invariant (`any:` of two literals): met.
- `starts_with` entity invariant: met.
- Text length `digest.count == 64`: met.
- Entity invariant over an optional member: met.
- Struct type bounding two members: synthesizable.
- Gap 4 through a String newtype: all four scenarios witnessed and correct.
- Gap 5 `!=` refusal declared first, through a newtype: default branch sends equal values, refusal sends different ones, the ticket's members differ.
- Swapped operands for gaps 4 and 5 cannot be expressed: a bare right-hand side is a literal, and the compiler refuses it.
- A newtype with invariants on a view field is refused ESS-SYNTH-001 ("replay response invariant/reading observer is unsupported"). That is not this unit's defect.
- Not run: byte stability via `cargo xtask generate --check`. I relied on the implementor's `generate.log` ("projections are up to date"). Nor did I test `equality_copies` making an inequality's two sides equal by accident.

## 6. Paths written outside the worktree

- ~/.cache/ess-wave-n2/nested/adv1/review.md
- ~/.cache/ess-wave-n2/nested/adv1/red.log
- ~/.cache/ess-wave-n2/nested/adv1/suite.log
- ~/.cache/ess-wave-n2/nested/adv1/clippy.log
- build dir used, as assigned: ~/.cache/b10x-target/ess-e-nested

## 7. Findings block

```findings
- file: crates/verify/ess-conformance/src/witness.rs
  line: 2356
  category: acceptance
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: undecided
  message: an entity invariant the bounded repair cannot solve leaves the base and emits scenarios a correct target fails instead of refusing with a named cause
- file: crates/verify/ess-conformance/src/witness.rs
  line: 569
  category: property
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: undecided
  message: guard ladders move a leaf an entity invariant couples or orders without moving its partner, and no candidate is filtered by entity invariants
- file: crates/verify/ess-conformance/src/witness.rs
  line: 2288
  category: boundary
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: undecided
  message: invariants of every branch's entity are solved as one conjunction, so branches with conflicting invariants over one input both send an invalid input
- file: crates/verify/ess-conformance/src/witness.rs
  line: 2417
  category: boundary
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: undecided
  message: repair drops list-length choices, so a count invariant over a struct list member yields a scenario a correct target fails
- file: crates/verify/ess-conformance/src/witness.rs
  line: 2870
  category: property
  severity: warning
  verdict: NEEDS-CHANGE
  origin: undecided
  message: a repaired leaf is pinned to the invariant literal at every distinction, so two instances share a value the invariant only bounds
- file: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
  line: 2127
  category: acceptance
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: undecided
  message: the default branch beside a when_subject over a stored optional struct member is refused although reachable rows exist, contradicting acceptance line 3
```
