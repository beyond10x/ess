---
format: aep.planning-md/3
id: story:a-predicate-may-add-fields-and-take-a-disjunction
kind: story
status: draft
title: A predicate may add fields and take a disjunction
summary: '`passed + failed == total` and `n < 0 or n > 100` are refused; when_subject has no any/all (workbench report items 17, 18)'
tags:
- backlog
- external-request
- from-consumer-report
relations:
- serves: vision:O2
revision: 1
---
## Need

- An invariant cannot relate several fields: `ESS-SPEC-012 cannot parse predicate "passed + failed
  == total": left-hand side must be a fact path` (item 17, example `17`).
- A string predicate has no `or`: `ESS-COMMAND-002 \`n < 0 or n > 100\`: operator \`<\` does not admit
  \`Integer\` (Number) and \`Text literal \`0 or n > 100\`\`` and `ESS-SPEC-012 ... invalid fact path
  identifier "(n"`. `any:` works in `when:` and view filters, but `when_subject` has no `any`/`all` form
  (item 18, examples `18a` to `18e`).

Items 17 and 18 of the 2026-11-09 workbench report.

## Acceptance

- An invariant or guard may compare an arithmetic expression over fields (`passed + failed ==
  total`). Synthesis witnesses both sides.
- `when_subject` takes `any:` and `all:` as `when:` does.
- A string predicate containing ` or ` / ` and ` either parses as a disjunction or conjunction, or
  is refused with a message naming `any:`/`all:`; it is never read as a text literal.

## Scope

- inferred: the predicate parser (`ESS-SPEC-012`), `crates/specify/ess-domain` (guards, invariants),
  synthesis guard witnesses.

## Fit review

Pending. An external request: it is assessed with
`.agents/skills/assessing-external-requests/SKILL.md` before this story leaves draft.

## Source

`.engineering/repro/workbench-2026-11-09/report.md`, with its examples under `examples/`.
