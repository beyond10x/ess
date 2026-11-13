---
format: aep.planning-md/3
id: story:a-predicate-may-add-fields-and-take-a-disjunction
kind: story
status: active
title: An invariant or guard compares arithmetic over fields, and an unquoted or in a string predicate is refused
summary: '`passed + failed == total` is refused, and an unquoted `n < 0 or n > 100` is read as a text literal (workbench report items 17, 18)'
tags:
- backlog
- external-request
- from-consumer-report
relations:
- serves: vision:O2
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-11-12T21:48:11Z", actor: "human:timo", revision: 4, executor: "agent:b10x", correlation: "operator-2026-11-12-round-2"}
- {from: "proposed", to: "active", at: "2026-11-12T21:48:12Z", actor: "human:timo", revision: 5, executor: "agent:b10x", correlation: "operator-2026-11-12-round-2"}
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

## Decisions

**Accepted under the operator's procedure for external-request drafts (2026-11-12).** He set
it that day: the session that maintains the repository recommends per draft, an independent
adversarial review reads each draft, a draft both read as accept is accepted on his behalf, and
a draft where the two differ goes to him. This draft was number 35 of a second list of 13
(numbers 26 to 38); both read accept. The two transitions carry the correlation
`operator-2026-11-12-round-2` and the executor `agent:b10x`.

The fit review of `.agents/skills/assessing-external-requests/SKILL.md` is not written yet. It
is owed before this story is dispatched and says which form of acceptance applies (as proposed
or redesigned).

Scope of the acceptance, narrowed from the request:

- In: acceptance point 1, an invariant or guard that compares an arithmetic expression over
  fields (`passed + failed == total`).
- In: of acceptance point 3, the refusal of an unquoted string predicate that contains ` or `,
  such as `n < 0 or n > 100`, which is still read as a text literal. Quoted operands are refused
  since `story:refuse-misparsed-predicate-disjunctions`. The refusal names `any:`/`all:`.
- Out: acceptance point 2. `when_subject: {predicate: {all: [...]}}` validates on 0.93.0 (shown in
  the body of `story:a-caller-attribute-is-compared-with-each-element-of-a-stored-list`), so
  `when_subject` already takes `any:` and `all:` in that form.
- Out: parsing ` or ` / ` and ` inside a string predicate as a disjunction or conjunction.

The title and the summary are corrected to this scope.
