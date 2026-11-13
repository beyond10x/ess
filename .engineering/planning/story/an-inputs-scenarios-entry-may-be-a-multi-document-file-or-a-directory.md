---
format: aep.planning-md/3
id: story:an-inputs-scenarios-entry-may-be-a-multi-document-file-or-a-directory
kind: story
status: archived
title: An inputs `scenarios:` entry may hold several documents or name a directory
summary: ess-inputs/2 scenarios refuses a multi-document file and a directory (workbench report item 35)
tags:
- backlog
- external-request
- from-consumer-report
relations:
- serves: vision:O2
revision: 3
transitions:
- {from: "draft", to: "archived", at: "2026-11-12T21:48:21Z", actor: "human:timo", revision: 3, executor: "agent:b10x", correlation: "operator-2026-11-12-round-2"}
---
## Need

`scenarios:` in `ess-inputs/2` takes only single-document files:

- One file with several documents: `unreadable: deserializing from YAML containing more than one
  document is not supported` (example `35b`).
- A directory: `scenarios entry "scenarios/dir" is not a regular file; list a file rather than a
  directory or special node` (example `35c`; control `35a`).

Not the same as #735. Item 35 of the 2026-11-09 workbench report.

## Acceptance

- A scenarios file may hold several YAML documents, each one scenario.
- A scenarios entry may name a directory, read in a stable order, its files as above.
- The order of scenarios in the suite does not depend on the file system.

## Scope

- inferred: the `ess-inputs` reader (`crates/specify/ess-compiler`), the scenarios loader.

## Fit review

Pending. An external request: it is assessed with
`.agents/skills/assessing-external-requests/SKILL.md` before this story leaves draft.

## Source

`.engineering/repro/workbench-2026-11-09/report.md`, with its examples under `examples/`.

## Decisions

**Declined under the operator's procedure for external-request drafts (2026-11-12).** He set
it that day: the session that maintains the repository recommends per draft, an independent
adversarial review reads each draft, and a draft both read as drop is archived with this note.
This draft was number 38 of a second list of 13 (numbers 26 to 38); both read drop. The
transition to `archived` carries the correlation `operator-2026-11-12-round-2` and the executor
`agent:b10x`.

Reason, directory entry: `docs/design/review-authored-discovery.md` (accepted 2026-09-07,
`story:review-authored-discovery`) states for the inputs manifest: "No glob expansion, directory
entries, includes, environment substitution, remote sources, content-based exclusion, default
filenames or inherited manifests are introduced." A directory entry is against that contract.

Reason, several documents in one file: nothing is blocked. Class (fit question 2): convenience.

Idiom: list each scenario file by name under `scenarios:`. The report's control example does so
(`.engineering/repro/workbench-2026-11-09/examples/35a`: `scenarios: [scenarios/one.yaml]`,
`ess specify validate` exit 0, "1 scenario(s), valid"). Both refused forms are refused by name.

Reopened by: a change of the discovery contract in that design, decided by the operator.

Notice to the requester: the request came with the workbench report of 2026-11-09
(`.engineering/repro/workbench-2026-11-09/report.md`). No session of its author is reachable, so
this section is the notice.
