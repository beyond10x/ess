---
format: aep.planning-md/3
id: story:host-context-has-one-shape
kind: story
status: draft
title: Host-bound context has one block and one prefix
relations:
- serves: vision:O2
- decomposes: epic:language-consistency-after-adoption
revision: 1
---
## Outcome

Event and periodic bindings read host-bound context through one `context: {authority, fields}` block and one `context.` prefix (review S6/S7, worst five #4).

## Acceptance

- in the next format, both binding causes accept `context: {authority, fields}` and read `context.<field>`; `host:`/`host_context.` and `context_fields`/`context_authority` remain as aliases
- `context.x` below ess/18 keeps its literal meaning, as `{caller:}` and `response.item` did
- ess-diff classifies the alias rewrite as no change

## Origin

Evidence: `docs/design/review-external-requests-2026-09.md` (host-context-has-one-shape). Drafted, not scheduled; a fit review precedes dispatch.
