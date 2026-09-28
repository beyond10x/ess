---
format: aep.planning-md/3
id: story:arrangement-searches-every-creating-command
kind: story
status: active
title: 'ESS-SYNTH-003: a when_subject branch is unwitnessed when only the second of two creating commands produces the row it needs'
refs:
- provider: github
  reference: beyond10x/ess#198
relations:
- serves: vision:O2
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-09-28T17:59:47Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-09-28T17:59:47Z", actor: "human:timo", revision: 3}
---
# Story: ESS-SYNTH-003: a when_subject branch is unwitnessed when only the second of two creating commands produces the row it needs

## Why

beyond10x/ess#198. Coordinator decision: `.engineering/waves/ess-0.41-decisions.md` row #198 (summarised in the wave page).

## Scope (story-scoper, 2026-09-28, on e9327819658583ae45953acafa3e41d3bbd5b7f4)

Verdict: **open.** The defect is still in the code on this tree (`e932781965`, chore: release 0.40.0) and no commit fixes it. I read the code and did not run it.

Paths below are relative to `crates/verify/ess-conformance/`.

**Reproduces (cited):**
- `src/synthesize/subject_fact.rs:1118-1120`: `search()` sets `let creator = drivers.iter().find(|driver| matches!(driver.effect, ResolvedEffect::Creates))`. That takes the first-declared creating driver only.
- `subject_fact.rs:1126`: the breadth-first search starts only from `creations(ir, entity, creator, …)`.
- `subject_fact.rs:1170-1173`: the expansion step drops every other `Creates` driver (`ResolvedEffect::Creates | ResolvedEffect::Preserves` → `continue`). A second creator is never tried at any depth. This matches the issue's report that swapping the two `sets:` literals swaps which branch is refused.
- `subject_fact.rs:1187-1196`: this is where the reported message comes from ("stored {field} selecting this branch, over the rows {} bounded arrangements left").
- `subject_fact.rs:949-980`: `creations()` returns the rows one creator can leave: its plain witness, plus one row per hinted input.
- Already fixed: no. No fix commit for #198 is in the recent history.

**Fix site (cited):**
- `subject_fact.rs` `search()` at 1107: seed `level` from `creations()` over every `Creates` driver, in declaration order.
- The same pattern already exists in `src/synthesize.rs:2794-2815`, the lifecycle route arrangement. It iterates all creators ("Every creating branch is a place to start… ties to the first creator declared"), so it can serve as the model.
- All `search()` callers inherit the fix: `subject_fact.rs:597, 1330, 1362, 1442, 1967, 2123`.
- Regression test (inferred placement): `tests/when_subject_witness.rs`. Use the issue's two-creator Order shape in both declaration orders and assert that both `not-paid` and `shipped` synthesize.

**Design decisions for the implementor:**
- Determinism (cited, rule at 1103-1106 "ties to the earlier… (§37)"): with several creators seeded, the tie-break must stay a function of the model. Needed: declaration order of the creator, then the existing `score` rank and index.
- Budget (cited, `MAX_NODES = 64` at `subject_fact.rs:42`, checked at 1135): extra seed rows fill it sooner. Decide whether to keep 64 as the cap or scale it per creator.
- Errors (inferred): `creations()` fails with `?` (`InstanceRequired`). With many creators, skip a creator that cannot produce a row and refuse only if all fail, keeping the first cause the way `first` does today.
- Scope (inferred): the issue's "Expected" also says "(and update)". Updates are already expanded as successors at 1165-1180, so only creators need changing.
- Unguarded creators only (inferred): `creations()` returns just the plain witness for a creator with subject guards or `InjectFault` (961-966). That stays per creator.

**Collisions:**
- #199 (inferred, high): the same Order/`paid`/`when_subject` shape. Its refusal `Unreachable::Unwitnessable` is raised in `src/synthesize.rs:2858, 2867`, inside the route arrangement next to 2794. It probably also depends on `subject_fact.rs` `search()`. Land #198 first or put both in one unit.
- #209 (inferred, medium): refusal-scenario arrangement of an existing instance. It likely lands in `src/synthesize.rs` around 2790-2870. The issue itself says it is possibly related to #198.
- #201 (inferred, low-medium): `wrong_state` / subject-state guards. The recent commits touching `subject_fact.rs` are wrong-state work (`69884aecfe`, `6355cd7999`); the actual edit sites are probably the validator in `ess-domain` `command.rs`.
- #196 (inferred, low): Map witness values sit in the value and witness generator, not in `search()`.
- #210 (inferred, none): `mutate --collect` scoring.
- #195 (inferred, none): binding and `host_context` in the compiler and domain.

**Confidence:** high that the defect reproduces and where the fix lands (the single-creator `find` is explicit). Medium on the collision guesses, which come from file layout only.

**Paths:**
- `crates/verify/ess-conformance/src/synthesize/subject_fact.rs` (cited, primary)
- `crates/verify/ess-conformance/src/synthesize.rs` (cited, reference pattern at 2794; the #199/#209 overlap is inferred)
- `crates/verify/ess-conformance/tests/when_subject_witness.rs` (inferred, test home)
- `crates/specify/ess-compiler/src/ir.rs:2346` `drivers()` (cited, read-only dependency)

Verdict: open
