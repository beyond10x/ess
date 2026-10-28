---
format: aep.planning-md/3
id: approval-record:d132-types-across-specifications-by-vendor
kind: approval-record
status: approved
title: 'Types across specifications: ess specify vendor generates the copy (D132)'
summary: 'Option B for #663; no source-format change'
refs:
- provider: github
  reference: beyond10x/ess#663
relations:
- decides: story:a-specification-imports-another-specifications-types
revision: 2
transitions:
- {from: "draft", to: "approved", at: "2026-10-28T10:43:04Z", actor: "human:timo", revision: 2, executor: "agent:ess-issues-lead", correlation: "ess-issues-1"}
---
## Decision

D132, taken by the orchestrator under the operator's grant on 2026-10-28, in the message that handed
the open-issue batches to the batch lead: "D132 #663: option B (`ess specify vendor` generates the
renamed copy, no format change)."

Option B of the three in `docs/plan/wave-ess-issues-1.md` § Decisions for Timo, #663:
`ess specify vendor --from <owner ir> --type <T> --into <domain>` generates the renamed copy of the
owner's type, its nested types and the `compose conformances:` entry that proves the copy, with the
owner's IR digest in a header. No source-format change; ownership (ESS-DOMAIN-004) is unchanged and
the copy is still owned by the importing domain.

Not chosen: A (a source `imports:` key, a new `ess/N` and IR shape) stays possible later; C (decline
with the idiom) would leave the copies hand-written.

## Effect

`story:a-specification-imports-another-specifications-types` is redesigned around the generated
copy. Its reserved scenario names change when its acceptance is written; it is sized S-M and goes
into a later batch, not batch 1.

Decider: operator
