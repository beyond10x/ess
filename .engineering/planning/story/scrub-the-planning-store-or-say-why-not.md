---
format: aep.planning-md/2
id: story:scrub-the-planning-store-or-say-why-not
kind: story
status: draft
title: The reason for leaving the planning store unscanned was false; decide again on the real numbers
relations:
- serves: vision:O2
- informed_by: story:planning-store-carries-workstation-paths
scope:
- confidence: inferred
  path: .engineering/planning
- confidence: cited
  path: crates/edge/ess-xtask/tests/host_paths.rs
revision: 3
---
## Why this exists

`story:planning-store-carries-workstation-paths` offered two branches: scrub the store, or declare
it out of scope and say so in the lane. Wave 24 took the second, on an argument
`aep-drive:adversary` then falsified (`review-result:adversary-wave24-unit2-pass-1`).

The argument was that a widened scan carving out `journal.jsonl` **"would report the store covered
while exempting the file holding the largest single share of the defect."** Measured in the unit
the same sentence counts in:

| file | carrying lines | share |
|---|---|---|
| `.engineering/planning/review-result/authored-discovery-source-pass1.md` | 28,730 | 87.6% |
| every other document | 3,989 | 12.2% |
| `.engineering/planning/journal.jsonl` | **87** | **0.27%** |

A journal carve-out covers **32,719 of 32,806 carrying lines and 59 of the 60 files**. The sentence
is true only under a different, unstated unit — findings per line, where the journal is first with
32,163 against 28,730 — and the two units answer the decision oppositely.

So the reason the scrub branch was rejected does not hold, and the branch is open again.

## What makes it a story of its own rather than a correction

The 59 documents are **planning-store bodies**, and `AGENTS.md:117` says the store is mutated only
through `aep artifact`, never by editing a store file. So the scrub is 59 `aep plan artifact body`
calls — which the wave protocol reserves to the coordinator, not to an implementor, because the
journal is append-only and one file and two writers race on it. It is a different unit of work from
the lane change, done by a different actor, and it should not be smuggled into a correction round.

## Acceptance

One of these, decided on the corrected numbers rather than the false one:

- **Scrub.** The 59 documents are rewritten through `aep plan artifact body`, the scan widens to
  `.engineering/` with `journal.jsonl` carved out, the carve-out is stated in the lane's module doc
  with its true share (0.27% of carrying lines, 87 lines, one file), and `aep plan artifact
  validate` still prints `valid`.
- **Still out of scope, on a true reason.** Whatever that reason is, it is stated with the numbers
  above rather than with the falsified comparison, and the lane's module doc carries it.

Either way the sentence in the lane and the state of the repository agree, and the sentence is one
a reader can check.

## What is already true and need not be re-measured

60 tracked files, 87 journal lines, 699 unread files, 65,838 findings, `examined == 699` with no
file skipped — all confirmed twice, by the story's own grep and by the lane's transcribed detector.
No carrying tree exists outside `.engineering/`.
