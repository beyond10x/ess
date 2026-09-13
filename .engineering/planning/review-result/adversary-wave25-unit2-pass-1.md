---
format: aep.planning-md/1
id: review-result:adversary-wave25-unit2-pass-1
kind: review-result
status: active
title: Adversary pass 1 against a masked first declaration
relations:
- reviews: story:a-masked-first-declaration-hides-a-duplicate-name
revision: 1
---
# Adversary pass 1 against `story:a-masked-first-declaration-hides-a-duplicate-name`

Worktree `wt-c2495d2e6f3a`, uncommitted over `wave/ess-wave-25` at `e5a97603`. Verdict
**NEEDS-CHANGE**. Cases 2909 → 2916, 3 red. Seven cases added, all in one untracked test file; the
unit's three modified paths are byte-identical before and after the pass.

A methodological note the pass made and the unit's own run did not: `--no-fail-fast` was required.
The plain command aborts after `ess-domain` when red and reports 1873 executed, which is not
comparable to the unit's 2909.

## Findings

| # | Where | Verdict / origin / severity | Measured | Reaches it |
|---|---|---|---|---|
| F1 | `ess-domain/src/spec.rs:541` | CONFIRMED / **pre-existing** / blocker | `masked_declaration_adversary_pass1.rs:125`, exit 101. Control at `:107` passes, so the refusal exists when both copies convert and vanishes when one breaks | **The class has a twelfth row: one name, two kinds, first copy broken.** `declare`'s key is `(kind, name)`, so it closes only the same-kind half. The cross-kind reporters — `system.rs:989 Assembly::claim` and `domain.rs:243 DomainSpec::validate` — read `DomainMembers`, which is filled only from `Ok(..)` arms: the exact blindness the unit removed from the registries. A command `shop.cart.Thing` with a duplicate input plus an event of the same name: the author is told about the input and nothing about the name |
| F2 | `ess-domain/src/spec.rs:542` | CONFIRMED / **introduced** / blocker | same red case; the named delegate produced nothing | `declare`'s new doc comment says, unqualified, that "one name held by two *different* kinds … `DomainSpec::validate_all` is what reports that one". It does not report it when either copy fails its conversion — which is the only case this unit is about. The comment is what would stop the next reader looking, and it is new in this diff |
| F3 | `ess-domain/src/spec.rs:1237` | CONFIRMED / pre-existing / warning | `masked_declaration_adversary_pass1.rs:180`, exit 101. Control at `:162` passes | The declared-type row says a type "can be" masked and points at `SystemSpec::merge` as "a second reporter … with a contract of its own". That reporter receives only types that **converted**, so for the masked case there is no second reporter at all. The bound is wider than the sentence reads |
| F4 | `ess-domain/src/spec.rs:1099` | CONFIRMED / **introduced** / warning | `masked_declaration_adversary_pass1.rs:283`, exit 101 | The `MASKED` table's `entity` row does not do what its comment claims. The comment at `:1084` reads "The second is sound and takes the same name"; that row's second copy has `lifecycle: {states: [Open], initial: Open}`, which is `dead_end_state`. The row is broken-then-**broken**, and no case in the tree exercises a sound entity taking a broken entity's name. The unit's own case cannot notice: it asserts only `declare`'s refusal, which fires before either copy converts |
| F5 | `ess-cli/src/recovery/process.rs:349` | INFEASIBLE / pre-existing / note | read, plus **240 runs at 6-way concurrency, 0 failures**, and green in all three suite runs | The flake the unit reported is **unattributable by construction**. `Outcome.status: None` is produced by two unrelated events — the deadline branch at `:342` and `ExitStatus::code()` returning `None` for a signal-killed child at `:350` — and only `timed_out` separates them, which the failing assertion at `execution_recovery.rs:2528` does not print. `left: None, right: Some(101)` is compatible with both a 30-second stall and an OOM kill. Not load-sensitivity established: an ambiguity that prevents establishing it |

## Attacked and could not break

- **The `conversion` row.** Built it. `RawSpecFile::conversions` is `Vec<Conversion>` with no
  `try_from` between document and registry, so there is no conversion step to fail. Row holds.
- **The `topology` row.** Built it. A first topology with an invalid workload name still sets
  `topology_source` before converting, and the second file is refused. Row holds.
- **The `actor` row's infallibility claim.** `actor.rs`'s `try_from` is an unconditional
  `Ok(Self{..})` over three parser-settled fields. The row really is a control.
- **The eleven-kind enumeration against `RawSpecFile`'s fields**: `types, conversions, entities,
  commands, events, errors, views, actors, components, bindings, topology` — exactly eleven, no
  twelfth *field*. The gap is not a missing field; it is a missing row **shape** (F1).
- **`record`'s dropped `contains_key` check.** Looked for a `try_from` that renames, which would make
  `record` overwrite silently where `insert` refused. `QualifiedName`, `ComponentName` and
  `BindingName` are exact strings with injective `Display` and no normalisation. No such path.
- **Duplicates across two files, three copies in one file, copies differing only by case.**
  `declared` lives on `Collected` and spans files, `push` does not dedupe, distinct names are
  distinct. All behave.
- **The unit's fixture claims.** The `typed_diagnostics.rs` diff is purely additive — 20 `+`, 0 `-`
  — so the five existing `Cited` entries are byte-identical; `repeated_names.yaml` and
  `design_page_matches_the_fixture.rs` are untouched and all three of its cases passed.
- **The design-page edit.** Both premise strings the tests grep for survive it, and "all three" is
  true of the tree: `typed_diagnostics.rs` pins the `Solo` list by full equality at three entries,
  all `located: None` / `source: <document>`.

## Reported against itself

The pass's first cross-kind fixture used an entity whose lifecycle is a dead end, so its **control**
failed too. That was a fixture typo, not a finding; it switched to command/event and re-ran. Both
runs are in the record rather than only the second.

```findings
- file: crates/specify/ess-domain/src/spec.rs
  line: 541
  category: acceptance
  severity: blocker
  verdict: CONFIRMED
  origin: pre-existing
  message: "keying declare by (kind, name) closes only the same-kind half of the masking class, so one name held by a broken command and a sound event of the same name is still refused by nothing."
- file: crates/specify/ess-domain/src/spec.rs
  line: 542
  category: contract-drift
  severity: blocker
  verdict: CONFIRMED
  origin: introduced
  message: "the new declare doc comment states without qualification that DomainSpec::validate_all reports a cross-kind name clash, and it does not report one whose first copy failed its own conversion."
- file: cratesty/specify/ess-domain/src/spec.rs
  line: 1237
  category: contract-drift
  severity: warning
  verdict: CONFIRMED
  origin: pre-existing
  message: "the declared-type row points at SystemSpec::merge as a second reporter with its own contract, but that reporter sees only types that converted, so the masked type case has no reporter at all."
- file: crates/specify/ess-domain/src/spec.rs
  line: 1099
  category: mutant
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: "the MASKED table entity row second declaration is itself broken, so that row tests broken-then-broken while its comment claims broken-then-sound, and no case anywhere covers a sound entity taking a broken entity's name."
- file: crates/edge/ess-cli/src/recovery/process.rs
  line: 349
  category: judgement
  severity: note
  verdict: INFEASIBLE
  origin: pre-existing
  message: "Outcome.status None conflates a deadline kill with a signal-killed child, so the reported flake cannot be attributed to load or to anything else; 240 runs at 6-way concurrency did not reproduce it."
```
