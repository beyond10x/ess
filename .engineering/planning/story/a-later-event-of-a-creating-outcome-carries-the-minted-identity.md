---
format: aep.planning-md/3
id: story:a-later-event-of-a-creating-outcome-carries-the-minted-identity
kind: story
status: active
title: A later event of a creating outcome carries the minted identity
summary: A creating outcome that mints its identity and emits two events has no payload source giving the second event the new row's id; two generated fields are two mints on 0.94.0 (adopter report)
tags:
- backlog
- external-request
- from-adopter-report
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-domain-language
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-12T21:27:30Z", actor: "human:timo", revision: 3, executor: "agent:b10x", correlation: "operator-2026-11-12-accept-16"}
- {from: "proposed", to: "active", at: "2026-11-12T21:27:32Z", actor: "human:timo", revision: 4, executor: "agent:b10x", correlation: "operator-2026-11-12-accept-16"}
---
## Need

A creating outcome that mints its identity (`creates:` with `instance: <field>` and no input of
that name) and emits two events cannot say that the second event carries the new row's identity.
Both payload fields are `{generated: true}`, each a mint of its own; the synthesized suite ties the
created row to the first listed event's field and checks the second event's field for its shape
only. An implementation that publishes two different identities passes.

Reported by an adopter on 0.90.0 and 0.93.0 (2026-11-12); reproduced on the 0.94.0 release binary.
The adopter's workaround is one event and a read of the row; a caller-supplied `input.<field>`
gives equal values but moves the choice of the identity to the caller.

Model (`format: ess/37`), entity `demo.t.Note` with identity `id`, events `demo.t.Opened` and
`demo.t.Announced`, each with a field `id`:

```yaml
- name: demo.t.Open
  input:
    - {name: text, type: String}
  outcomes:
    - name: opened
      creates: demo.t.Note
      instance: id
      sets: {text: input.text}
      emits: [demo.t.Opened, demo.t.Announced]
      payload:
        demo.t.Opened: {id: {generated: true}}
        demo.t.Announced: {id: <source>}
```

`ess verify conform synthesize` per `<source>`:

| `<source>` | result |
|---|---|
| `{generated: true}` | 1 scenario, 0 refusals; `expect_event` for `demo.t.Announced` checks only the shape of `id`; `expect_view` reads the row's `id` as the value observed in `demo.t.Opened.id` |
| `{payload: demo.t.Opened.id}` | refused, `type_mismatch` (ESS-COMMAND-002): "`{payload: <Event>.<field>}` stores a value this outcome publishes, and is a `sets:` source only" |
| `{subject: id}` | refused, `conflicting_declaration` (ESS-COMMAND-004): "a `creates:` outcome has no row before it" |
| `demo.t.Opened.id` | refused, `type_mismatch`: read as a literal |

`docs/design/minted-value-sources.md` (`ess/27`) gave `sets:` the source `{payload: <Event>.<field>}`
so that a stored value is the published one. It has no counterpart between two payload fields of
one outcome, and its "Not done" list does not name one.

## Acceptance

1. A payload field of a creating outcome can name the created instance's identity, or a field of
   another event the same outcome emits, in a form the fit review chooses, under a new format
   version.
2. Synthesis expects the two event fields equal (the second read as the value observed in the
   first), and the row's identity equal to both; a target that publishes two different values
   fails the scenario.
3. The interpreter and the generated Rust and Go targets publish one value in both fields.
4. A source that names an event the outcome does not emit, a field of another type, or itself is
   refused by name.
5. `docs/design/minted-value-sources.md` and the reference page for payload sources state the form.

## Fit review

Not done (`.agents/skills/assessing-external-requests/SKILL.md`). Questions it must answer:
whether the general need is "one mint feeds several places of an outcome" (then `{payload: …}` may
be admitted in payloads, with an order rule and a cycle refusal) or "the created identity is
nameable" (a source for the identity `instance:` mints, usable in payloads and in a `response:`);
how it relates to beyond10x/ess#498 (a stored generated value returned to the caller); the second,
unrelated adopter's example; the cost of a format version; and "change nothing" with the one-event
workaround documented.

## Evidence

Model, script and output: session scratch `ess/adopter8/` (`run.sh`, `run.txt`; not in the
repository).

## Decisions

**Accepted by the operator (2026-11-12),** with fifteen other drafts from one list of 25 that
he was shown with a recommendation and an outside review per draft. He accepted the drafts both
recommended. The two transitions carry the correlation `operator-2026-11-12-accept-16` and the executor
`agent:b10x`.

The fit review of `.agents/skills/assessing-external-requests/SKILL.md` is not written yet. It
is owed before this story is dispatched and says which form of acceptance applies (as proposed
or redesigned).
