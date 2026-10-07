---
format: aep.planning-md/3
id: story:generated-behaviour-reads-selection-plan
kind: story
status: active
title: The Rust and Go emitters order generated selection by the selection plan
relations:
- decomposes: epic:one-selection-plan
- serves: vision:O2
- depends_on: story:selection-plan-design-and-type
scope:
- confidence: cited
  path: crates/generate/ess-synth/src/determined.rs
- confidence: cited
  path: crates/generate/ess-synth/src/go/behaviour.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/plan.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/behaviour.rs
- confidence: inferred
  path: crates/generate/ess-synth/tests
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-10-07T13:18:13Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-10-07T13:18:14Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"review_outcome":1}}}
---
# Story: The Rust and Go emitters order generated selection by the selection plan

## Why

`epic:one-selection-plan`. `body` in `crates/generate/ess-synth/src/rust/behaviour.rs` and its Go
twin emit the precedence order as a fixed sequence of loops (input refusals, related rows, the
addressed row, subject-guarded branches, accepting branches, default), and `determined.rs` restates
which branches come first (`orders_present_related_refusal`).

## What it delivers

Both emitters take the order of the `if` blocks they write from
`story:selection-plan-design-and-type`'s plan.

## Acceptance

- Every byte under `generated/` is unchanged (`feasibility.rs`
  `complete_committed_valid_artifact_maps_are_unchanged_and_compile`,
  `adversary_related_guard_behaviour_pass1.rs` `adv319_every_committed_artifact_of_the_examples_is_unchanged`).
- The slow per-model byte probes (`task test-slow-probes`: `adversary_e_u2_bytes`,
  `adversary_e_u6_bytes`) give the same Rust and Go digests on the base and on the unit, except the
  epic's named exception (a held-state branch declared after an accepting branch the finite prover
  shows disjoint); each is listed in the PR. `generated/` alone exercises none of the rewired phases.
- A test using `story:selection-plan-design-and-type`'s seam, with the held-state and
  accepting/external phases exchanged, shows the emitted order of two guards change with it, for
  Rust and for Go.
- `orders_present_related_refusal` and `is_present_related_refusal` are gone from
  `crates/generate/ess-synth/src/determined.rs` (`git grep` finds neither in `ess-synth`);
  `subject_guarded` and `collision_answer` stay as admission checks.

## Scope

Derived 2026-10-07 by `aep:story-scoper` on `985f58cc3a`. Every line is **cited** (read from the
story or the tree) or **inferred** (a reading that could be wrong).

- **Primary surface:** `crates/generate/ess-synth`: the Rust and Go behaviour emitters and `determined.rs` — cited (story, Why)
- **Files:** `crates/generate/ess-synth/src/rust/behaviour.rs:1180-1389` (`Writer::body`), `:1527` (`precheck`), `:1657` (`subject_guard`) — cited
- **Files:** `crates/generate/ess-synth/src/go/behaviour.rs:1801-2015` (`Writer::body`), `:2018` (`is_subject_guard`), `:2339` (`precheck`) — cited
- **Files:** `crates/generate/ess-synth/src/determined.rs:165` (`collision_answer`), `:623` (`subject_guarded`), `:1509` (`is_present_related_refusal`), `:1524` (`orders_present_related_refusal`) — cited
- **Order as written today (Rust `body`):** clock `:1183` → input-named related read with `existing_instance` `:1199` → input-guarded refusals `:1211` → `existing_instance` when not input-related `:1226` → stored reference (addressed row, `precheck(false)`, related read) `:1231`, else `precheck(true)` if `orders` `:1254` → present-related refusals `:1257` → collision answers `:1274` → addressed row and subject-guarded loop `:1281-1306` → accepting / related-holds / external / external-when in declaration order `:1311-1364` → default or `undeclared` `:1365` — cited
- **Go `body` is the same sequence** (`:1803`, `:1833`, `:1845`, `:1860`, `:1865`, `:1894`, `:1911`, `:1939`, `:1990`) **with no collision-answer loop**; `determined::collision_answer` is called only at `rust/behaviour.rs:1277` and `determined.rs:201` — cited
- **Second ordering loop:** `precheck` (Rust `:1531`, Go `:2347`) walks accepting branches, and related-holds branches when `with_related`, in declaration order — cited
- **Other targets:** `src/web/` and `src/clap/` read no `ResolvedCondition` — cited; web goes through `rust::feasibility::checked(.., Target::Web)` (`web/mod.rs:306`) and so inherits the Rust body — inferred. `crates/generate/ess-gen` restates precedence only in prose (`docs.rs:1814`, `openapi.rs:897`) — cited, out of scope
- **Also likely:** `crates/generate/ess-synth/src/plan.rs:712` (`related_precedence`) restates `orders_present_related_refusal` (`:724`) as prose in `PLAN.md`/`plan.json` — cited that it restates; inferred that the story covers it
- **Also likely:** a new test in `crates/generate/ess-synth/tests/` for the exchanged-phases acceptance, Rust and Go — inferred
- **Byte pins:** `ess-synth/tests/feasibility.rs:1131` `complete_committed_valid_artifact_maps_are_unchanged_and_compile` (Rust billing, gatepass, Web billing); `ess-synth/tests/adversary_related_guard_behaviour_pass1.rs:1413` `adv319_every_committed_artifact_of_the_examples_is_unchanged` (Rust, Go, Web; the only Go pin); `Taskfile.yml:102-106` `test-slow-probes` runs `adversary_e_u2_bytes` and `adversary_e_u6_bytes` (`--ignored`), which hash generated Rust and Go for every repository model — cited
- **Not a pin:** `cargo xtask generate --check` (`Taskfile.yml:268`) skips `go`, `rust` and `web` (`crates/edge/ess-xtask/src/main.rs:41` `PROJECTION_EXCLUSIONS`) — cited
- **Safety fact:** `examples/billing` and `examples/gatepass` use no `when_related`, `when_subject*`, `when_state_changes`, `existing_instance` or `rows:`, so `generated/` does not exercise the phases rewired here; byte identity has to come from the E-U2/E-U6 probes, base against unit — inferred
- **Confidence:** high — every ordering loop above was read
- **Would collide with:** any unit touching `ess-synth`'s `rust/behaviour.rs`, `go/behaviour.rs`, `determined.rs` or `plan.rs`, and the plan type's module — inferred
- **Design decisions for the implementor:** — inferred unless marked
  1. Pass the plan into the writers rather than building it in `body`: Rust `Writer` is declared at `:1094` and built at `:91` and `:226`; Go `Writer::new` at `:1733`, `:204`, `:251`. That seam lets the exchanged-phases test inject a plan.
  2. `precheck` reads the same phases as `body`, or the two disagree on which branch is selected.
  3. Go and a collision phase: emit it or refuse it by name, never skip it.
  4. `subject_guarded` (`determined.rs:45`, `rekey_composition:146`) and `collision_answer` (`:201`) are admission checks too; keep those uses and remove only the ordering copy.
  5. Emitted comments ("an accepting branch, in declaration order.", "before every accepting branch.") are pinned bytes; key each to its phase verbatim (cited `rust :1319`, `:1267`).
  6. Names taken: `ess_domain::selection::SelectionPlan`, `ess_compiler::ir::ResolvedSelectionPlan` (`ir.rs:2392`, imported by `ess-synth/src/selection.rs:2`), `ess_synth::plan::SynthesisPlan` (cited).

**Could not establish:** whether Go ever generates a re-key command (no collision loop in Go
`body`, no re-key refusal in `go/refusal.rs`); whether the web build compiles the Rust `body`;
whether the story covers the `plan.rs:724` prose copy.
