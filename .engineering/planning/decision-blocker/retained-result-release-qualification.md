---
format: aep.planning-md/1
id: decision-blocker:retained-result-release-qualification
kind: decision-blocker
status: cleared
title: Qualify the local gate for the retained-result release
relations:
- blocks: story:retained-command-result-replay
withholds: test_result
revision: 4
---
## Decision boundary

AGENTS.md requires the local task check and exact-tag qualification. Existing
CI and release workflows explicitly use SKIP_CONSUMER_CHECKS=true. The previous
local-release-gate-profile resolution authorizes that profile for 0.28.0 only;
it does not authorize 0.29.0.

## Concrete candidate

The independently reviewed source and its preserved regression cases are in
https://github.com/beyond10x/ess/pull/59. The published delivery tree is identical
to the local qualification candidate. Original development refs and a verified
complete Git bundle preserve the intermediate history; the governed journal is
unchanged by delivery consolidation. Security publication passed under the
unchanged active policy.

Local qualification candidate: b69e6de3213d87579ef6c8946bf76f83b38a3c35.

The workspace/tooling run reports 3312 passed, 0 failed and 14 ignored.
Formatting, strict Clippy, rustdoc, examples, projections and support checks passed before the consumer refusal.

- task fuzz-check: exit 0.
- task release-check: exit 0.
- task action-check: exit 0.
- task site-build: exit 0.

## Remaining refusal

extraction/classification refused: unclassified concrete consumer entry ess_cli::bin(ess)::enum::SuiteTarget/variant/Typescript; finite review required; accounting diagnostics: {"SchemaDocumentMetadata":0,"details":"unknown claimed model wire:RawSpecFile#/definitions/NamedType/oneOf/2/properties/variants/items/type","format":"ess-consumer-accounting/1","no_cells_admitted":true,"status":"PROVISIONAL_ACCOUNTING_REFUSAL"}

Fresh extraction: 213 unclassified concrete entries, including inherited and introduced entries. No accounting cells or completed qualification receipt were admitted.

The documentation fixture that still called ess/7 unreleased was corrected,
and its module and the subsequent full workspace run pass. The original failure
and the refused default gate remain in the retained qualification evidence.
No consumer classification, pinned accounting baseline or assertion was relaxed.

## Operator decision requested

The operator answered the bounded 0.29.0 qualification question on 2026-09-22:

> what exactly is there to be approved? if we should merge you mean, yes

The question explicitly offered the existing CI/release profile for this release,
with the local consumer-accounting refusal retained, or completing that separate
accounting work first. This answer authorizes merging PR 59 and using the existing
SKIP_CONSUMER_CHECKS=true profile for local exact-tag qualification of 0.29.0.

Required CI must remain green. The exact merged commit must pass that profile and
site-lab before tagging, and the published release and assets must be verified.
This approval does not classify the 213 entries, admit any accounting cell, change
the default gate, or complete consumer-accounting-baseline-never-extended.
The default refusal and its original evidence remain retained. This decision
does not establish EKR durability, runtime conformance or phase completion.
