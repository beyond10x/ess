---
format: aep.planning-md/3
id: story:a-create-or-update-writer-is-no-proof-a-stored-reference-is-present
kind: story
status: active
title: A create-or-update writer is no proof that a stored reference is present
summary: The ess/30 enforced-reference proof and synthesis count an unknown_instance creator as refusing a missing related row first, although under the one rule it stores one; found by the 0.70.0 unit I adversary
tags:
- defect
- ess-0.71.0
- severity-high
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-domain-language
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: cited
  path: crates/specify/ess-domain/src/command/reference_integrity.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/related_guard/stored.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/adversary_i_create_or_update.rs
- confidence: inferred
  path: docs/design/generic-language-features.md
revision: 10
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:33:41Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:33:41Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"review_outcome":1}}}
---
## Problem

The `ess/30` proof that a stored reference names a present row
(`crates/specify/ess-domain/src/command/reference_integrity.rs:171-196`, `checked`) counts a
creating command as "refusing an identity no row carries before storing it" when the command has a
`when_related: {via: input.<field>, exists: false}` branch. A create-or-update command with an
`unknown_instance:` branch (`bound`) answers before that refusal under the one precedence rule and
stores the identity anyway. So from `ess/30` (`ess/31` where the proof drops the requirement) a
later command reading the stored reference may leave out its `exists: false` branch although the row
can be missing. Synthesis (`crates/verify/ess-conformance/src/synthesize/related_guard/stored.rs`,
`checks`) calls a declared `exists: false` branch unreachable.

The 0.70.0 unit I adversary found this (`review-result:unit-i-pass-1`); it was already in the
domain code before that wave. Re-checked on 0.92.0.

Reproduction: `crates/verify/ess-conformance/tests/fixtures/create-or-update-beside-related-refusal.yaml`
(`Bind` with `already-bound`, `no-agent`, `rebound`, `bound: unknown_instance: true, creates:`).

1. `ess verify conform synthesize` on the fixture writes `Bind/outcome/bound` as one `Bind` step
   with `agent_id: "agent_id"`, which no `AddAgent` created, and `ess verify conform run --target
   interpreted --report-format 2` passes it: `bound` stores an agent no row carries.
2. The fixture plus

   ```yaml
   - name: demo.desk.Ping
     input:
       - {name: seat_id, type: demo.desk.SeatId}
     outcomes:
       - name: agent-gone
         when_related: {via: agent_id, exists: false}
         error: demo.desk.NoAgent
       - name: pinged
         preserves: demo.desk.Binding
         instance: seat_id
   ```

   is refused for `Ping/agent-gone` with `ESS-SYNTH-003`: "unreachable: every act writing
   `agent_id` (`demo.desk.Bind/rebound`, `demo.desk.Bind/bound`) refuses an identity no row carries
   before storing it".
3. The fixture plus a `Ping` whose branches are `labelled` (`when_related: {via: agent_id,
   predicate: label == "on"}`) and `pinged`, with no `exists: false` branch, validates at `ess/31`.
   At `ess/29` it is refused as `non_exhaustive_branches`.

## Acceptance

- `a_later_command_reading_the_stored_agent` in
  `crates/verify/ess-conformance/tests/adversary_i_create_or_update.rs` asserts
  `every_branch_passes("later stored read", &text, "demo.desk.Ping", &["agent-gone", "pinged"])`
  (today it asserts the refusal) and passes: both branches are synthesized and pass the interpreter.
- `a_stored_agent_written_by_the_creator_is_not_proven_present` asserts `validates(&text).is_err()`
  at `ess/31` and at `ess/29` (today it asserts `is_ok()` at `ess/31`), and the `ess/31` refusal is
  `non_exhaustive_branches` with a hint naming `demo.desk.Bind/bound` as the writer that stores the
  reference unchecked.
- A model whose writers all refuse first (`crates/verify/ess-conformance/tests/fixtures/enforced-reference.yaml`)
  keeps its IR bytes and suite bytes.

## Scope

- cited: crates/specify/ess-domain/src/command/reference_integrity.rs (`checked` at 171-196 treats an input as checked if any branch refuses it with `exists: false`, 186-188; called at 135 and 149; the hint comes out of `unchecked` at 161 and the `(Err, true)` arm of `lift` at 58-63, so no new message code)
- cited: crates/verify/ess-conformance/src/synthesize/related_guard/stored.rs (`checks` at 81-91, used by `unreachable` at 141, the ESS-SYNTH-003 text at 152, and by `relink` at 257)
- cited: crates/verify/ess-conformance/tests/adversary_i_create_or_update.rs (`a_later_command_reading_the_stored_agent` at 322 and `a_stored_agent_written_by_the_creator_is_not_proven_present` at 357 wait for the flip)
- inferred: docs/design/generic-language-features.md (§ G2 "Enforced references", 314-317, needs to say the refusal answers before the writing branch)

Read only: `crates/specify/ess-domain/src/command/precedence.rs` (`Composition::of` 397, `place` 471, `order` 557, `Phase::position` 131, all public); `crates/verify/ess-conformance/src/synthesize/precedence.rs:32` (`answers_before`, `pub(crate)`).

confidence: high — the story names both defect sites and the test file, and all three are where it says (story-scoper, 2026-11-10)

Corrections to the earlier pass: removed `crates/specify/ess-domain/src/command/precedence.rs` (only read); added `docs/design/generic-language-features.md` (inferred).

Shared surfaces this story moves (epic § Shared surfaces, not scope entries): `CHANGELOG.md`; possibly an `ess/37` `stricter:` row of `FORMAT_HISTORY`; the `enforced-reference.yaml` rows of `selection-precedence-table.tsv` (505-509) and of `synthesis-suite-bytes-table.tsv` stay unchanged.
