---
format: aep.planning-md/3
id: story:a-tied-input-also-read-by-a-row-set-guard-keeps-its-tie
kind: story
status: archived
title: A tied input that a row-set guard also reads keeps its tie in the caller swap
summary: When a when_related tied input is also written into a field a row-set guard reads, the caller-swap half redraws it whole and the same-version scenario sends a fresh version without a note.
tags:
- ess-0.70.0
revision: 4
transitions:
- {from: "draft", to: "archived", at: "2026-10-31T23:45:02Z", actor: "human:timo", revision: 4}
---
## Finding

Adversary pass on G-U25 (d077d7640, merged ac08ece30) before 0.69.0, on integrate/ess-batch-0690 f1197bcd4. Pre-existing: the case fails at ac08ece30^1 too. Recorded for 0.70.0 under approval-record:ess-0690-cut-after-gu19.

- `selected` in `crates/verify/ess-conformance/src/synthesize/caller.rs`: a `RetractVersion` row-set guard reads `Review.version`, which `StartReview` writes from its tied `input.version`. In the swapped half `StartReview` sends `version` 524545 while the stored version is 1.
- G-U25 says a tied input is no longer drawn afresh; that does not hold for this model.

## Acceptance

- The swapped half keeps the tie, or the swap is left out with a note naming the row-set guard.
- Red test: `a_tied_input_written_where_a_row_set_reads_keeps_the_tie_or_is_noted` (ess-conformance, from the G-U25 adversary pass).

## Triage (D338)

Closed at the 2026-10-31 findings triage (orchestrator D338, operator approval relayed by host-monitor D53) as a duplicate of story:a-caller-swap-redraw-keeps-unread-members-and-related-ties (implemented; fix 137e2567e changed this code path, and the CHANGELOG 0.69.0 text covers the tie and the note). Inferred from the code; reopen if the red case still fails.
