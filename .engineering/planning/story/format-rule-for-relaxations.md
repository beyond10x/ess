---
format: aep.planning-md/3
id: story:format-rule-for-relaxations
kind: story
status: draft
title: A relaxation says which format admits it
relations:
- serves: vision:O2
- decomposes: epic:language-consistency-after-adoption
revision: 1
---
## Outcome

SV states when a rule that ESS relaxes needs a format version, so an older build does not refuse a newer document with a misleading diagnostic (review S12).

## Acceptance

- SV:17-19 states the rule for relaxations, citing #204 (gated) and #217/#227/#230 (not gated)
- an older build reading a document that needs a relaxation names the release that admits it

## Origin

Evidence: `docs/design/review-external-requests-2026-09.md` (format-rule-for-relaxations). Drafted, not scheduled; a fit review precedes dispatch.
