---
format: aep.planning-md/3
id: story:payload-fields-have-one-filling-rule
kind: story
status: draft
title: Event and error payloads follow one filling rule
relations:
- serves: vision:O2
- decomposes: epic:language-consistency-after-adoption
revision: 1
---
## Outcome

A `payload:` entry maps every field of an event or error, `{generated: true}` marks the implementation's own values, and generated code never fills a field by name (review S16/S20, worst five #5).

## Acceptance

- under the next format an error `payload:` that leaves a field unmapped is refused as an event's is; ess/19 documents keep today's meaning
- generated Rust and Go fill only mapped fields; same-name filling is removed from SY:208-213 and the code
- CO:156 and CO:220 agree with SY:223

## Origin

Evidence: `docs/design/review-external-requests-2026-09.md` (payload-fields-have-one-filling-rule). Drafted, not scheduled; a fit review precedes dispatch.
