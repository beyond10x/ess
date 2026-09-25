---
format: aep.planning-md/2
id: review-result:wave25-unit3-correction-1
kind: review-result
status: active
title: 'Unit 3 correction round 1: the refusal with nothing left to evade'
relations:
- reviews: story:component-declares-its-settings
revision: 1
---
# Unit 3, correction round 1: the refusal with nothing left to evade

Worktree `wt-e0cb562e0319`, uncommitted over `wave/ess-wave-25` at `e5a97603`. Green.

```
cargo test --workspace --exclude ess-xtask --locked --no-fail-fast    exit 0
  2929 passed, 0 failed, 5 ignored, 295 test result: lines
cargo clippy --workspace --all-targets --locked -- -D warnings        exit 0
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps --locked   exit 0
cargo fmt -p ess-domain -p ess-compiler -p ess-deployment -- --check  exit 0
cargo xtask schema --check      exit 0   schemas/generated/ess.schema.json: current
cargo xtask generate --check    exit 0   projections are up to date
```

The executed count fell, 2930 → 2929, and the unit accounted for it rather than leaving it to be
noticed: −2 for the two seam cases now `#[ignore]`d, +1 for a new case. Total cases rose 2933 → 2934,
so no lane stopped selecting.

## F1 — fix 1, derive from the type

`requires_a_value(required, type_is_optional) = required.unwrap_or(!type_is_optional)`, a free
`pub fn` in `ess-domain`.

Why fix 2 — refuse silence — is worse, in the unit's own words: it refuses the story's own example,
which writes no `required:`; it contradicts the shipped and adversary-confirmed rustdoc that a
restatement can only be wrong when it is made; and it is a published-schema contract change, since
`RawComponentSetting.required` is `Option<bool>`, `default: null`, so making the key mandatory
breaks every document already written against the published schema — for a key whose whole purpose
is to be a reading aid.

Fix 1 costs nothing: `validate_settings` already refuses both disagreeing combinations, so for every
**accepted** document `required.unwrap_or(!is_optional)` is identically `!is_optional`. The refusal
now has nothing to evade.

### A second member of the class the adversary did not name

| reader | state |
|---|---|
| `ess_compiler::ir::ResolvedComponentSetting::is_required` (`ir.rs:863`) | the one the adversary named; the one `runtime.rs:871` calls — fixed |
| `ess_domain::component::ComponentSetting::is_required` (`component.rs:528`) | a **second** `unwrap_or(false)` copy, `pub`, not reachable today — fixed |

Both were wrong; only one was reachable. The rule now has a single site, so a third reader cannot
disagree without writing a visible second copy.

### The mechanism was measured, not assumed

The unit reverted `requires_a_value` to `unwrap_or(false)` in the otherwise-finished tree and re-ran.
Red exactly where predicted, including the new grid case:

```
test a_setting_whose_type_admits_no_absence_does_not_derive_an_optional_slot ... FAILED
  left: [("state-root", Optional)]   right: [("state-root", Required)]
test an_accepted_required_restates_the_type_and_never_overrides_it ... FAILED
  left: false   right: true
```

Restored immediately after. So the case is not a tautology and the mechanism is the one.

## F2 — fixed rather than ignored, and the naive fix would have been wrong

`derive_component_settings` now resolves the container-role → components edge across every workload
**first**, then visits each container role once, so the guard reads the runtime document and
structurally cannot read its own output.

The unit reports that the fix named in the finding — capture the hand-authored names once, before
any derivation — would have left a second defect the same assertion catches: a shared role would
have derived both workloads' slots twice and been refused for duplicates instead.

## F3 — held, with both patches verified

`spec.rs` belongs to unit 2, which was live. Two patches in scratch, both `git apply --check` clean:
`coordinator/spec-rs-placement.patch` then `coordinator/unignore-settings-seam-cases.patch`. The two
seam cases carry the review-result id and the patch path in their `#[ignore]` attribute, following
the tree's existing convention, with the adversary's assertions and panic messages unchanged.

## The gate caught three things the unit then fixed

`cargo doc` found `rustdoc::redundant_explicit_links` at `ir.rs:867`, because `requires_a_value` is
now in scope there. `cargo fmt --check` found the widened import. `cargo xtask schema --check` went
stale because `RawComponentSetting.required`'s doc comment reaches the published schema as its
`description`; regenerated, +60 lines, re-checked.

## Declared rather than left to be found

The unit names one thing it would have done with more budget: `validate_setting_types`' rustdoc at
`component.rs:1366-1376` says the seam "is worth closing", which understates F3 — the promise is
already broken. It left the wording because F3 is not its fix, the seam is named accurately there,
the escalation is recorded in three other in-tree places, and changing a doc comment would have cost
a second whole-workspace run of about 25 minutes for no behavioural change.

Its `TMPDIR` reached 4,765 entries and 4.2 GB and was deleted, with the note that fixture directories
are left read-only so removal needs `chmod -R u+rwX` first.
