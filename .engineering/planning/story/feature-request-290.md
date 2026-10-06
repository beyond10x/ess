---
format: aep.planning-md/3
id: story:feature-request-290
kind: story
status: implemented
title: 'ess verify diff: no way to fail on a breaking change; a narrowing exits 0'
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#290
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-10-04T11:42:44Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-10-04T11:42:44Z", actor: "human:timo", revision: 4}
- {from: "active", to: "implemented", at: "2026-10-06T09:43:13Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"test_result":1,"review_outcome":1,"verification":1}}}
---
## Outcome

Resolve beyond10x/ess#290: ess verify diff: no way to fail on a breaking change; a narrowing exits 0.

## Origin

beyond10x/ess#290, from a downstream hardening run on ess 0.48.0; reproduced minimally (triage item 1, `~/.cache/ess-gaps/triage-cb/`).

## Fit review

Pending.

## Current source and required design scope (2026-10-02)

The gap remains at source candidate482609: VerifyCommand::Diff in crates/edge/ess-cli/src/main.rs:302 accepts from/to/format only; diff at :2606 prints a valid delta and returns success without classifying compatibility. The consumer request includes direction-aware classification in the document AND an opt-in failure/acknowledgement gate; an exit-code-only narrowing check would not satisfy it.

Current ess-diff/src/delta.rs supports documents/1 through/12. raw.rs:43,57 rejects unknown fields on the delta and change envelopes. Adding persisted classification therefore needs an explicit format decision and legacy-reader tests. The existing semantic dependency graph is owned by ess-compiler (ess-diff/src/graph.rs re-exports it); reuse that graph/resolved types rather than a second unrelated dependency walk. The design record ess-semantic-diff-impact-evolution-design-v0.1.md sections29-31 explicitly distinguishes old-caller/new-command, old-reader/new-event, historical state and projection assumptions, and requires Unknown when the model supplies no answer. A single blanket narrowed=breaking rule contradicts both that record and the issue.

Implementation planning must cover nested type reachability, input-only/output-only/shared uses, variant additions as well as removals, wire changes and structural changes with unknown consumer assumptions, exact change-id acknowledgements bound to the compared revisions, and unchanged default CLI exit behavior. No red execution or implementation is claimed by this source reconciliation, and no narrowing-only partial fix is scheduled.
