---
format: aep.planning-md/3
id: story:a-create-or-update-command-refuses-a-missing-related-row-before-it-creates
kind: story
status: draft
title: A create-or-update command refuses a missing related row before it creates
summary: 'From ess/27 the creating unknown_instance branch answers before every exists: false, so a new key is stored under a reference no row carries; no single-command form refuses it on 0.94.0 (adopter report)'
tags:
- backlog
- external-request
- from-adopter-report
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-domain-language
revision: 1
---
## Need

A create-or-update command (`unknown_instance: true` on the creating branch, `ess/16`) that stores
a reference to another entity cannot refuse an identity no row of that entity carries when the
addressed key is new. From `ess/27` the one precedence rule answers the addressed record's
existence (step 2, the creating `unknown_instance:`) before every `exists: false` (step 4), so for
a new key the creation is selected and the row is stored with a reference no row carries. For a key
a row already carries, the `exists: false` refusal answers.

Reported by an adopter (2026-11-11): five commands of one specification have this shape, and a
target built to the rule answers a new key under a missing parent with the creation.

Model (`format: ess/37`, validates on 0.94.0), entities `demo.t.Parent` and `demo.t.Child`
(identity `key`, field `parent_id`):

```yaml
- name: demo.t.Record
  input:
    - {name: key, type: demo.t.Key}
    - {name: parent_id, type: demo.t.ParentId}
    - {name: note, type: String}
  outcomes:
    - name: no-parent
      when_related: {via: input.parent_id, exists: false}
      error: demo.t.ParentNotFound
    - name: recorded
      unknown_instance: true
      creates: demo.t.Child
      instance: key
      sets: {parent_id: input.parent_id, note: input.note}
    - name: refreshed
      updates: demo.t.Child
      instance: key
      sets: {note: input.note}
```

Measured on the 0.94.0 release binary:

- `ess verify conform synthesize`: `demo.t.Record/outcome/recorded` sends a new key and a parent
  identity no row carries and expects `recorded`; `demo.t.Record/outcome/no-parent` first records a
  child under a missing parent, then sends that key with a missing parent and expects `no-parent`.
- `ess verify conform run --target interpreted --report-format 2`: 4 scenarios, 4 passed.

This is the rule as written (`website/docs/reference/predicates.md`, "From `ess/27` one precedence
rule"; `docs/design/generic-language-features.md`, the step table, and its sentence that a
create-or-update creator "answers before it and stores the identity, so it is no proof"). The need
is a form, not a change of the rule for existing models.

Forms tried in one command, each refused by ESS-COMMAND-004 on 0.94.0:

| form | refusal |
|---|---|
| the creation without `unknown_instance:` beside the updating branch | outcomes `recorded`, `refreshed` are all unconditional |
| the creation with `when_related: {via: input.parent_id, exists: true}` | the branch taken while the row exists is the default beside `exists: false` |
| `unknown_instance:` and `when_related:` on the creation | a branch has one selection authority |

What works today: two commands. A create-only command (the `exists: false` refusal, then
`creates:` without `unknown_instance:`) refuses a new key under a missing parent; the refresh is a
second command. An adopter whose wire has one route for both cannot use it.

## Acceptance

1. A create-or-update command can declare that a named `exists: false` refusal over a row its input
   names answers before the creating `unknown_instance:` branch, in a form the fit review chooses,
   under a new format version; models of earlier versions keep today's order.
2. For that form synthesis sends a new key with a missing related row and expects the refusal, and
   a new key with a present related row and expects the creation; the interpreter and the generated
   Rust and Go targets answer the same.
3. The integrity derivation of `ess/30` (a creation that stores a reference refused first) counts
   the new form as proof.
4. The precedence plan, `website/docs/reference/predicates.md` and
   `docs/design/selection-plan.md` state where the form places the refusal.

## Fit review

Not done (`.agents/skills/assessing-external-requests/SKILL.md`). Questions it must answer: whether
the need is the general one "a creation refuses a reference no row carries" (then the form may
belong on the relation or the creation, not on the precedence of one branch); what a second,
unrelated adopter's example looks like; the cost of a format version; and "change nothing" with the
two-command split documented as the answer.

## Evidence

Model, suite and the three refused variants: session scratch `ess/adopter7/` (not in the
repository).
