---
format: aep.planning-md/3
id: story:every-witness-reads-back-its-outcome-effect
kind: story
status: draft
title: Every witness reads the addressed row back and requires exactly its outcome's effect, a compensating delete included
tags:
- adopter-report
- defect
refs:
- provider: github
  reference: beyond10x/ess#581
- provider: github
  reference: beyond10x/ess#585
- provider: github
  reference: beyond10x/ess#587
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 1
---
## Outcome

Every scenario that drives a command against an existing row reads that row back afterwards and
requires exactly the effect of the outcome the step selected: unchanged in every field for any
refusal (input-guarded, held-state, related-row and `external:`, `before_row: true` included), the
declared move or writes for a success or a compensating refusal, and absence for an outcome that
deletes the row. The rule is one obligation, applied the same way to the primary witness and to
every extra witness synthesis adds for a guard's complement. A compensating refusal may delete its
addressed row, read back as absent, as an accepting `deletes:` is.

Findings covered: https://github.com/beyond10x/ess/issues/581,
https://github.com/beyond10x/ess/issues/587 and https://github.com/beyond10x/ess/issues/585
(one downstream audit of an authorization-code redemption on ess 0.60.0, `ess/25`).

## Fit review

1. **Need.** Three findings of one report are one missing rule: *what a witness asserts about the
   addressed row after the step*.
   - #581: an extra witness for the complement of a compound (`all:`) refusal guard selects a
     `deletes:` success and then expects the deleted row in an unfiltered `read_your_writes` view
     (steps 15-19 of the issue's suite); the closed https://github.com/beyond10x/ess/issues/342
     fixed the same contradiction for the closed and invariant-after scenarios, not for extra
     witnesses. The primary witness of the same scenario asserts the row
     absent. With a single-leaf guard the extra witness is not written and the suite passes.
   - #587: an `external:` refusal's scenario checks the outcome, the error and no event, and never
     reads the row back, so a target that deletes or rewrites the row on the forced branch passes.
     A held-state refusal of the same command gets `snapshot_complete_subject` …
     `expect_complete_subject_unchanged`.
   - #585: a single-use row is consumed before work that can still fail, and the failure answers an
     error with the row already gone. `compensates: true` admits only `moves:` or `updates:`
     (`crates/specify/ess-domain/src/command.rs:340-350`, `docs/design/refusal-with-effect.md:81-86`),
     so the answer cannot be declared; the only idiom is an invented `Spent` state the
     implementation does not store.
   Minimal reproduction to write first (brand-free): domain `demo.tickets`, entity `Ticket {id,
   secret}` with one state, `Issue` creating it, `Redeem {id, proof}` with (a) a refusal
   `when_subject: {predicate: {all: [secret != "", secret != input.proof]}}`, (b) an `external:`
   refusal `store-unavailable` with `before_row: true`, (c) an `external:` refusal
   `consumed-on-failure` with `compensates: true` and `deletes:`, (d) a `deletes:` success and
   (e) `unknown_instance:`; one unfiltered immediate view over `Ticket`.
   Requester's proposals (theirs): #581 "assert the effects of the selected outcome"; #587
   "snapshot the row before forcing the branch and require it unchanged"; #585 "`compensates: true`
   admits `deletes:` with `instance:`".
2. **Class.** #581 is a defect: the suite contradicts the model it was synthesized from, and the
   interpreted target fails it on a specification that validates with 0 refusals. #587 is a defect
   against "a refusal changes nothing" (`website/docs/reference/spec-versions.md`, `ess/22` row; the
   guide's "a refused record is asserted unchanged"). #585 is a gap in a sibling: the design kept
   `deletes:` out with no stated domain reason (`refusal-with-effect.md:81-86`, "Not done").
3. **Already expressible?** #581 and #587: no; they are about generated suites. #585: only through a
   lifecycle state the implementation does not store, which makes an adapter report a deleted row as
   a state.
4. **Fit.** The complete-subject observation already exists and is minted per outcome as
   `complete_refusal` (`crates/specify/ess-compiler/src/resolve.rs:5415-5433`): for a named
   wrong-state refusal from `ess/7` and a `when_subject` refusal from `ess/23`. External,
   input-guarded and related-row refusals are the siblings it skips. Extending the same flag, rather
   than adding a second observer, keeps one obligation that synthesis discharges through
   `preserve_refused_subject` (`crates/verify/ess-conformance/src/synthesize/subject_fact.rs`) and
   that `ess verify diff` already classifies. For the extra-witness defect the fix is in the
   witness writer: an extra witness's expectations are derived from the outcome it selects, never
   copied from the arranged row. For #585 the design's own read-back rule already covers absence
   (`refusal-with-effect.md:113-134`, step 5 reads an immediate view); `deletes:` reuses the
   accepting `deletes:` spelling, so no new key. Targets: validation, interpreter (apply the delete
   then answer the error, as `take` already applies a subject), synthesis, Rust/Go/TypeScript
   runners (existing steps only: `snapshot_complete_subject`, `expect_complete_subject_unchanged`,
   `expect_subject_absent`), Entity Runtime (keeps `CompensatingRefusalUnsupported`,
   `website/docs/reference/entity-runtime-lowering.md:88`), `ess verify diff`
   (`outcome-compensates-changed` and `outcome-subject-changed` unchanged), docs.
5. **Second adopter.** A one-time download link deleted before the file is streamed, where a storage
   failure after the delete answers an error; a payment intent consumed before an upstream capture
   that can time out; any service whose refusal path runs after a partial write, where a half-done
   change is most likely during an outage (#587's point).
6. **Cost.** Suite bytes change for every document with an external, input-guarded or related-row
   refusal on an existing row, so the complete observation for those siblings is gated at the next
   source format, as `ess/7` and `ess/23` gated theirs; below it suites keep their bytes. The #581
   correction is a defect fix at every format (the suite it replaces is wrong). `compensates:` with
   `deletes:` is admitted from the same format and refused below it naming it. No new step kind, no
   suite format, no new diagnostic code.
7. **Alternatives.** (a) Change nothing: the three gaps stay, and adopters keep known-failure
   declarations (#581's workaround). (b) Three local fixes, one per issue: three code paths deciding
   the same question. (c) One read-back obligation per selected outcome, through the existing
   `complete_refusal` flag and the existing absence steps (chosen).

## Decisions

Accept, redesigned into one obligation. Every witness requires the addressed row's state after the
step to be exactly what its selected outcome declares: unchanged for a refusal, changed as declared
for a success or a compensating refusal, absent for a delete. `complete_refusal` is minted for
every named refusal that addresses an existing row from the next source format. A compensating
refusal admits `deletes:` with `instance:` on the addressed row from that format; `creates:`,
`affects:` and `emits:` beside it stay refused. #581 is fixed at every format. The proposals of
#581 and #587 are taken; #585's is taken with the format gate.

## Acceptance

Each is a named conformance scenario, written failing first.

- `extra_witness_of_a_deleting_outcome_expects_the_row_absent`: the reproduction's `Redeem`
  `deletes` scenario, with the compound refusal guard, has no step expecting the redeemed row
  present after the delete in any witness, and passes against `--target interpreted`; a scripted
  target that keeps the row fails it (#581).
- `external_refusal_requires_the_row_unchanged`: the `store-unavailable` scenario snapshots the
  complete row before forcing the branch and requires it unchanged after; a target that deletes or
  rewrites the row on that branch fails it (#587).
- `input_and_related_refusals_require_the_row_unchanged`: the same observation on an input-guarded
  and a `when_related:` refusal of a command addressing an existing row.
- `compensating_refusal_deletes_its_row`: `consumed-on-failure` validates at the new format; its
  scenario forces the branch, requires the error and no event, then requires the row absent from an
  immediate view; a target that keeps the row fails, and one that deletes it on the
  `store-unavailable` path fails the previous scenario (#585).
- `compensating_delete_refused_below_the_format`: the same model one format lower is refused
  `refusal_mutated_state`, naming the format.
- `older_formats_keep_their_suite_bytes`: every repository model below the new format synthesizes
  byte-identical suites, except the scenarios the #581 correction changes, each listed in the
  CHANGELOG.

## Scope (inferred)

`crates/specify/ess-compiler/src/resolve.rs` (`complete_refusal`), `crates/specify/ess-domain/src/command.rs`
(`validate_compensation`), `crates/verify/ess-conformance/src/synthesize.rs` and
`synthesize/subject_fact.rs` (witness expectations, `preserve_refused_subject`),
`crates/verify/ess-conformance/src/interpret/execute.rs`, `docs/design/refusal-with-effect.md`,
`website/docs/reference/spec-versions.md`, the command guide.
