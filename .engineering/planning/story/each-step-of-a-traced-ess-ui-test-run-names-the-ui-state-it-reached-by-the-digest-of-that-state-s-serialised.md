---
format: aep.planning-md/3
id: story:each-step-of-a-traced-ess-ui-test-run-names-the-ui-state-it-reached-by-the-digest-of-that-state-s-serialised
kind: story
status: draft
title: Each step of a traced `ess ui test` run names the UI state it reached, by the digest of that state's serialised form
tags:
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:a-traced-ess-ui-test-run-reports-for-every-step-the-page-shown-the-nodes-drawn-and-the-cells-each-one-occupies
- depends_on: story:a-ui-state-prints-as-ess-ui-state-1-with-a-digest-and-two-routes-to-one-state-print-the-same-bytes-and-the
revision: 1
---
## Need

The per-step trace of `g04-step-trace` names the page, the nodes drawn and their cells. It does not
name the UI state the step reached, so a tool cannot tell that two steps, or two tests, reached the
same state, and cannot resume from a recorded state. Report item 4 asks for "the state reached".

ESS has no serialised UI state today. The terminal holds it as private fields of `App`: the page and
its params, the focus (`Focus`, `crates/ui/ess-ui-tui/src/app.rs:311-314`), per-view interaction
state (`Ui`, `:318-336`), the read cache, the channels' states, the open overlay. Report items 1
and 2 (P0 group) design that state's serialised form and its digest. This story uses that form and
designs no second one.

## Outcome

With `--trace`, each step record of `ess-ui-test-report/1` carries `state`: the digest of the UI
state reached after the step, in the form and digest items 1 and 2 define. With `--trace full`, it
carries the serialised state itself beside the digest. Two steps that reach the same state carry
the same digest, in one run and across runs.

## Acceptance

- First: a test opens a page, selects a row, selects another row and selects the first again, under
  `--trace`. Records 2 and 4 carry the same `state` and record 3 another. It fails before this story:
  the records have no `state`.
- Two runs of the same test file give the same `state` in every record (no clock time, no
  temporary path, no map order in the digest).
- `--trace full` carries the state in the serialised form of items 1 and 2, and its digest equals
  `state` under the digest rule items 1 and 2 publish; a test recomputes it.
- Without `--trace`, the report bytes are unchanged; with `--trace` and without `full`, a record
  differs from `g04-step-trace`'s only by the `state` key.
- The reference (`ess-ui-test.md` § The report) and `formats.md:221` name the key and point at the
  state form's reference.

## Fit review

Tree `bcfc51555`, read-only.

1. **Need.** Telling two reached states apart, or equal, from a test run. Report item 4, part
   "the state reached".
2. **Class.** *Gap*: no state form exists to name (above).
3. **Already expressible?** No.
4. **Fit.** One state form for ESS: the one items 1 and 2 define. A trace that minted its own
   state encoding would be a second spelling of one concept, the first red flag of the
   assessment skill.
5. **Second adopter.** A regression tool keeps the digests of a released version's journeys and
   reports the first step whose state differs after a change.
6. **Cost.** One optional key, written only with `--trace`; no version (same reasoning as
   `g04-step-trace`, P1-D1).
7. **Considered.** Full state in every record (P1-D2 B: large reports); nothing (P1-D2 C: the
   application replays every journey to compare). Chosen: the digest, the full state on request.

## Decisions

- P1-D2, recorded in `g04-step-trace`. Recommended: (A), the digest, the full state with
  `--trace full`.

## Scope

- cited: `crates/ui/ess-ui-test/src/lib.rs`, `crates/ui/ess-ui-test/src/runner.rs`
- inferred: the state API of items 1 and 2 (its crate is P0's decision)
- cited: `website/docs/reference/ess-ui-test.md`, `website/docs/reference/formats.md`
- Size: S (about 150 changed lines).

## Depends on

- `g04-step-trace`.
- The UI state form and digest of report items 1 and 2 (P0 group's stories).
