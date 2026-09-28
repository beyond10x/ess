---
format: aep.planning-md/3
id: story:synthesis-kills-connective-and-source-mutants
kind: story
status: implemented
title: Synthesis kills dropped-source and connective mutants, and witnesses partial views
relations:
- decomposes: epic:retrofit-findings-20260927
- serves: vision:O2
scope:
- confidence: inferred
  path: crates/specify/ess-compiler/src/ir.rs
- confidence: inferred
  path: crates/specify/ess-compiler/src/resolve.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/witness.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/mutation_audit.rs
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-09-27T07:34:30Z", actor: "human:timo", revision: 6, imported: true}
- {from: "proposed", to: "active", at: "2026-09-27T07:35:00Z", actor: "human:timo", revision: 7, imported: true}
- {from: "active", to: "implemented", at: "2026-09-27T14:54:10Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"test_result":1}}, imported: true}
---
## Scope

- #154: a state dropped from an all-states transition gets a `…/state/<s>/refuses/…` scenario.
- #155: `any:` gets one witness per disjunct with the others false; `all:` a boundary witness with
  exactly one conjunct false (MC/DC over the connective).
- #132: wrong-state preservation over the fields the declared views publish, and a `help:` line
  that points at views.

## Acceptance

- A spec mutated by dropping one source state from an all-states `from:` synthesizes a scenario
  the unmutated implementation fails.
- `any` → `all` and `all` → `any` mutants each change the synthesized suite.
- The repro in #132 synthesizes its terminal refusal scenario over an eventual identity/state view.

## Derived scope

Derived 2026-09-27 by `aep:story-scoper`. Every line is **cited** or **inferred**.

- **Primary surface:** `crates/verify/ess-conformance` synthesis (`src/synthesize.rs`, `src/synthesize/subject_fact.rs`) — cited
- **Files (#154):** `crates/verify/ess-conformance/src/synthesize.rs:4606-4660` (wrong-state loop over `ir.wrong_states`, filtered by `has_subject_guards`), `:4698` `refused_here` — cited
- **Files (#155):** `crates/verify/ess-conformance/src/synthesize.rs:5481` `boundary_inputs`, `:5400` `conjuncts` (top-level `all` only) — cited
- **Files (#155, stored-field guards):** `crates/verify/ess-conformance/src/synthesize/subject_fact.rs:1122` `conjunct_goals` (`all` only) — cited
- **Files (#132):** `subject_fact.rs:1294` `preserve_complete_subject`, `:1396`; `synthesize.rs:5112` `complete_wrong_state` — cited
- **Files (#132 help line):** `synthesize.rs:686` `RefusalCause::hint`, `NoWitness` arm — cited
- **Also likely:** `src/witness.rs` (`candidates`) — inferred
- **Also likely:** `tests/mutation_audit.rs` (pinned `from-drop` / `guard-connective` counts at :248-309, :389-445) — inferred
- **Also likely:** `crates/specify/ess-compiler/src/ir.rs:2084` `wrong_states`, `crates/specify/ess-domain/src/entity.rs:287` — inferred, only if #154's cause is the subtraction
- **Also likely:** `crates/specify/ess-compiler/src/resolve.rs:1743` (`complete_refusal`) — inferred, only if #132 is fixed in the compiler
- **Documents:** none; no format change — cited from the plan
- **Confidence:** medium — #155 and #132 sites read; #154's root cause not established
- **Would collide with:** any unit touching `synthesize.rs` or `synthesize/subject_fact.rs`, and any unit changing pinned counts in `tests/mutation_audit.rs`
- **Not established:** why #154 happens (`wrong_states` already subtracts the `from` sets); whether `witness.rs` changes; whether `mutation_audit.rs` fixtures carry `any`/`all`; where #132 is fixed; the #154 repro is not a fixture yet.
