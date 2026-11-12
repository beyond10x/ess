---
format: aep.planning-md/3
id: story:an-arranged-input-refusal-reads-back-where-the-view-cannot-observe
kind: story
status: draft
title: An arranged input refusal reads back where the view cannot observe
summary: The arranged half of an input refusal counts an identity-only view as observing the row, so a target moving the record on refusal passes (turn-3 adversary, pre-existing, backlog)
tags:
- backlog
- defect
- from-finding
- review-finding
- severity-low
- zero-open-issues
relations:
- decomposes: epic:zero-open-issues-2026-11
- serves: vision:O2
revision: 1
---
## Need

Found by the adversary pass on
`story:a-held-state-refusal-reads-back-where-the-view-cannot-observe` (turn 3); pre-existing and
outside that story's acceptance. The arranged half of an input refusal, on a command with no
held-state branches, counts a view that projects the identity but not `state` as observing the
row (`refusals_on_an_arranged_row`, `crates/verify/ess-conformance/src/synthesize/existence.rs:447`).
Nothing compares `state`, so a target that moves the stored record on refusal and publishes nothing
passes:

```text
demo.ticket.Annotate/outcome/blank-note passed a target that moves the stored ticket on refusal and publishes nothing
```

The red case is kept, ignored with this story's id:
`crates/verify/ess-conformance/tests/adversary_held_state_view_cannot_observe_pass1.rs::an_identity_only_view_arranged_half_catches_the_mover_or_is_withdrawn`.

## Acceptance

- Where no view observes the record in its state, the arranged half reads the record back after
  the refusal, as the held-state half does since the story above. A target that moves the record
  on refusal fails. Where no read-back has a valid send, the scenario is withdrawn as
  `ESS-SYNTH-001` naming the branches tried.
- A view that publishes `state` keeps its suite bytes.
- The ignored case runs and passes.
