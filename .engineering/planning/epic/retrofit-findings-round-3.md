---
format: aep.planning-md/3
id: epic:retrofit-findings-round-3
kind: epic
status: implemented
title: 'Retrofit findings round 3: seventeen issues filed against 0.36.0'
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-09-28T07:46:00Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-09-28T07:46:00Z", actor: "human:timo", revision: 3}
- {from: "active", to: "implemented", at: "2026-09-28T07:46:01Z", actor: "human:timo", revision: 4}
---
## Outcome

The seventeen findings filed on 2026-09-27 after the 0.36.0 retrofit (beyond10x/ess #162-#176,
#178, #179) are each specified, synthesized and released, or declined with a written reason.
#177 is answered by `unknown_instance:` in 0.37.0 and is not part of this epic.

## Planned order

| wave | stories | why this order |
|---|---|---|
| 3a | literal-fallback-after-else, nested-struct-per-leaf-comparison, defined-over-optional-aggregates, when-subject-witness-and-diagnostics, optional-input-narrowed-after-refusal, input-guard-overlap-precedence | extend designs that already exist (value expressions, presence, subject guards, command typing); no new noun |
| 3b | absent-command-input-outcome, related-record-value-source, caller-value-source-and-guard, current-time-guard-operand | new value sources and guard operands; each needs a source-format bump and witness support |
| 3c | set-effects-over-filtered-instances, upsert-outcome-by-existence | an outcome over more than one instance, or selected by existence; share the multi-row witness |
| 3d | bounded-retry-bindings, view-paging-and-caller-filters, consumer-imports-owner-types | binding, view and composition surfaces; each revisits a recorded design decision |

## Acceptance

Every story below is implemented or declined, and each issue is closed naming the release or the
decline.
