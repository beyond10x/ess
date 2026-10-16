---
format: aep.planning-md/3
id: story:synthesis-arranges-state-through-the-command-that-reaches-it
kind: story
status: implemented
title: Synthesis arranges a stored field or related state through the command that reaches it
tags:
- adopter-report
refs:
- provider: github
  reference: beyond10x/ess#553
relations:
- serves: vision:O2
- informed_by: epic:generic-row-queries
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-10-16T07:01:19Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-10-16T07:01:20Z", actor: "human:timo", revision: 3}
- {from: "active", to: "implemented", at: "2026-10-16T12:45:26Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

Synthesis arranges a guarded stored value or a related row's state through whichever declared
command reaches it: an `updates:` or `sets:` of a later command, an `affects:` entry of another
command, or a move of the related row, for the subject and for owned rows alike. A branch that
needs such state gets a scenario instead of `ESS-SYNTH-003`/`ESS-SYNTH-001`.

Fits none of the four generic language features: it is synthesis coverage of constructs the
language already has. Related: `epic:generic-row-queries` (row-set guards whose rows need a move).

## Fit review

Findings covered: https://github.com/beyond10x/ess/issues/553 (body and comment of 2026-10-13) and
the synthesis half of the 2026-10-13 comment on https://github.com/beyond10x/ess/issues/473, both
from a downstream incident-ledger specification (ess 0.59.0, `ess/24`).

1. **Need.** A record is locked once a pointer field is set; every command that changes it or its
   owned rows refuses while the pointer is set. The model validates, and synthesis refuses every lock
   branch: "no candidate of the 5 tried satisfies `ledger.Incident` stored approved_by selecting this
   branch, over the rows 5 bounded arrangements left" (5 `when_subject: {predicate:
   defined(approved_by)}` guards, `crates/verify/ess-conformance/src/synthesize/subject_fact.rs:7197`),
   the same for 2 `when_related … predicate: defined(approved_by)` guards, a related row that needs
   a move (Pending to Approved), and a stored `via:` whose writer is not the creating act
   (`synthesize/related_guard/stored.rs:167`). A row-set guard over related rows in a moved state
   refuses `ESS-SYNTH-001` (https://github.com/beyond10x/ess/issues/473 comment). `--synthesis-seed`
   does not reach owned rows. Requester's ask (theirs): arrange through the `updates:` that writes
   the field and the moves that reach the state; fallback, seeds on owned rows.
2. **Class.** Gap in synthesis coverage (no new authored surface).
3. **Already expressible?** The model is; its scenarios are not.
4. **Fit.** Arrangement already searches every creating command
   (`story:arrangement-searches-every-creating-command`) and threads distinctions through reach
   state (`story:arrangement-threads-distinction-through-reach-state`); this extends the search to
   non-creating writers and to related rows' moves, one search for subject and owned rows. The
   pointer in the reporter's current model is written by an `affects:` entry of another command,
   which waits on https://github.com/beyond10x/ess/issues/547 (`story:set-effect-filters-read-the-identity`,
   held by another branch); the arrangement must read `affects:` writers when that lands. Seeds on
   owned rows are the fallback, not the design.
5. **Second adopter.** An order locked once invoiced (an `invoice_id` set by the invoicing command)
   refusing line changes.
6. **Cost.** Synthesis only; suites of affected models gain scenarios; no format change.
7. **Alternatives.** (a) Change nothing: every lock branch stays unwitnessed. (b) Seeds on owned rows
   only: the author restates what the commands already say. (c) Search the declared writers (chosen),
   seeds as fallback.

## Decisions

Accept, redesigned: arrangement through declared writers; seeds on owned rows only if the search
cannot reach a state.

## Acceptance

- `stored_pointer_set_by_a_later_update_is_arranged`: a `when_subject: defined(field)` refusal whose
  field only a later `updates:` writes gets a passing interpreted scenario.
- `related_state_reached_by_a_move_is_arranged`: a `when_related` predicate over a state reached by a
  move gets a scenario, for an owned related row.
- `row_set_over_moved_rows_is_arranged`: the https://github.com/beyond10x/ess/issues/473 comment's
  row-set guard gets a scenario.
- `field_written_by_another_commands_affects_is_arranged`: after the identity-filter story lands.
