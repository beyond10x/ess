---
format: aep.planning-md/3
id: approval-record:zero-issues-archive-unreproduced-drafts
kind: approval-record
status: draft
title: Unreproduced and duplicate defect drafts archived with no operator present
tags:
- non-interactive
relations:
- decides: story:a-checked-that-reads-no-path-draws-alike
- decides: story:an-action-declaring-several-effects-runs-them-all-or-is-refused
- decides: story:an-authored-event-claim-names-a-struct-leaf
- decides: story:a-row-set-suite-fails-a-target-refusing-less
- decides: story:a-transition-taken-by-an-affects-move-has-a-cause
- decides: story:binary64-reachability-follows-references
- decides: task:consumer-accounting-wire-behavior-period-parity
- decides: story:refusal-witness-sends-echoed-optional-input
- decides: story:a-create-only-the-system-issues-gets-a-synthesized-scenario
revision: 1
---
## Stop

Archiving defect drafts that no longer reproduce on 0.92.0, and one duplicate, under
`epic:zero-open-issues-2026-11`. A move the store allows (`draft -> archived`), held only for an
operator's confirmation.

## Reason: non-interactive

The run is non-interactive: a Stop hook holds the session until the goal is met, and no operator turn
exists. Operator goal, verbatim (2026-11-09): "zero remaining issues on github, all bugs fixed from
planning store". Narrowed (2026-11-09): "zero remaining issues on github, all bugs fixed from planning
store for repo ess".

## Decided in the operator's absence

Each draft below was checked against main at 4ab2ea1449 and the 0.92.0 release binary by one
verification agent; the evidence (command output, test name, commit or CHANGELOG line) is appended
to each draft's body under `## Verification on 0.92.0 (2026-11-09)` before the move.

- already fixed: `story:a-checked-that-reads-no-path-draws-alike` (0.89.0, ec2cb77cad),
  `story:an-authored-event-claim-names-a-struct-leaf` (0.79.0, 8a4a93fb7d),
  `story:refusal-witness-sends-echoed-optional-input` (0.62.0, CHANGELOG.md:1029-1032);
- not a defect: `story:an-action-declaring-several-effects-runs-them-all-or-is-refused`,
  `story:a-row-set-suite-fails-a-target-refusing-less`,
  `story:a-transition-taken-by-an-affects-move-has-a-cause`,
  `story:binary64-reachability-follows-references`;
- duplicate: `story:a-create-only-the-system-issues-gets-a-synthesized-scenario`, the same gap as
  `story:a-system-issued-creating-command-is-witnessed`, which keeps the reproduction and stays in the
  plan;
- `task:consumer-accounting-wire-behavior-period-parity` (active) was fixed in 0.35.0 (9ea3363387,
  test `the_period_schema_admits_exactly_what_the_parser_reads_across_every_range_split`); it moves
  to `implemented` if the store admits that move on the recorded evidence, else to `archived`.

## What the operator would have been asked

"These drafts do not reproduce, or duplicate another draft; archive them with their evidence?"
