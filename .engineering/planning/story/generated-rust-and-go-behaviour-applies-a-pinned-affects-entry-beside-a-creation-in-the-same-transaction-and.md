---
format: aep.planning-md/3
id: story:generated-rust-and-go-behaviour-applies-a-pinned-affects-entry-beside-a-creation-in-the-same-transaction-and
kind: story
status: draft
title: 'Generated Rust and Go behaviour applies a pinned affects: entry beside a creation in the same transaction, and Entity Runtime keeps refusing set effects by name'
tags:
- gap-p2
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- derived_from: story:create-beside-a-move-on-the-same-key
- depends_on: story:a-branch-that-creates-its-subject-declares-affects-over-rows-that-existed-before-the-outcome-validated-and
- depends_on: story:synthesis-witnesses-a-creating-branch-s-affects-entries-with-no-selected-row-with-selected-rows-and-decoys-and
- depends_on: story:generated-rust-and-go-behaviour-read-a-related-row-and-the-addressed-instance-through-a-member-of-a-struct
revision: 1
---
## Need

Generated Rust and Go behaviour applies `affects:` entries pinned by identity beside a moving or
updating subject (beyond10x/ess#547; `crates/generate/ess-synth/src/rust/behaviour.rs`
`affected_reads` 2368; `crates/generate/ess-synth/src/go/behaviour.rs` `affected_reads` 3198,
`affected_changes` 3285) and owes every other `affects:` by name
(`crates/generate/ess-synth/src/determined.rs`, `affects`, from 1896). Beside a creation it owes
the whole branch: `if subject.effect == ResolvedEffect::Creates { return Err(OWED) }`
(`determined.rs:1919-1921`). After `g12-affects-beside-create` the reproduction's first shape (an
inspection that holds its shipment, pinned by `shipment_id == input.shipment_id`) is valid and the
generated servers cannot answer it.

## Outcome

Generated Rust and Go behaviour applies a pinned `affects:` entry beside a creation in the same
storage transaction as the creation: the pinned row is read before the outcome, a missing pinned
row selects nothing (zero rows), and the created row is never the pinned one. Entries that are not
pinned, and `each:` entries, stay owed by name, as beside a moving subject. Entity Runtime keeps
refusing set effects by name (`SET_EFFECT`, `crates/generate/ess-entity-runtime/src/subset.rs:98`).

## Acceptance

- Red first: `crates/generate/ess-synth/tests/set_effect_identity_pin_behaviour.rs` (existing binary) gains
  the first reproduction: the plan admits the command (no obligation), and the generated Rust and
  Go servers pass the suite `g12-create-affects-synthesis` writes for it. On the base (after
  `g12-create-affects-synthesis`) the plan owes the command.
- The supersede reproduction (an entry over the created row's own entity, selected by a filter that
  is not an identity pin) stays owed, and the obligation text names `affects:` beside a creation;
  one case in `crates/generate/ess-synth/tests/set_effects.rs` (existing binary).
- `crates/generate/ess-entity-runtime`: a case in `tests/set_effects.rs` (existing binary, beside
  `affects_is_refused_by_name`, `subset.rs:610`) asserts the reproduction is refused naming the set
  effect.
- `ess-synth` byte pins (`go`, `served_codec_names`) and the committed example trees are unchanged.
- The Rust build and the Go module test each run in one test under 60 s, timed in the report; a test
  over 5 s is named with its time, and marked for the `slow` lane
  (`crates/edge/ess-xtask/tests/slow_lane.rs` `MARKED`) if the harness marks its siblings so.

## Fit review

Part of the feature of `g12-affects-beside-create` (report item 12); its fit review applies. This
unit is the generated-target half.

## Decisions

- **How much is implemented?** (a) The pinned entry beside a creation, other forms owed by name, as
  beside a moving subject today. (b) Owe every `affects:` beside a creation. **Recommended: (a)**:
  the pinned form is the reported shape and the emitters already read and write a pinned row in the
  same transaction for a moving subject.

## Scope

- cited: `crates/generate/ess-synth/src/determined.rs` (`affects` from 1896, the `Creates` arm at
  1919)
- cited: `crates/generate/ess-synth/src/rust/behaviour.rs` (`affected_reads` 2368, 2406-2414)
- cited: `crates/generate/ess-synth/src/go/behaviour.rs` (`affected_reads` 3198,
  `affected_changes` 3285)
- cited: `crates/generate/ess-synth/tests/set_effect_identity_pin_behaviour.rs` (existing binary)
- cited: `crates/generate/ess-entity-runtime/tests/set_effects.rs` (existing binary)
- Size: M (about 400 changed lines).
- Overlap: s5 of the wave of 2026-11-24 changes both behaviour emitters; this unit lands after s5,
  and after `g11-subject-read-targets` if that one is in flight (same files, different functions).

## Depends on

- `g12-affects-beside-create`, `g12-create-affects-synthesis`.
- `story:generated-rust-and-go-behaviour-read-a-related-row-and-the-addressed-instance-through-a-member-of-a-struct`
  (s5, same emitters).

## Source

A downstream application, 2026-11-24, report item 12 (its G2).
