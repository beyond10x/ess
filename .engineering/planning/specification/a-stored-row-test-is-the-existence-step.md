---
format: aep.planning-md/3
id: specification:a-stored-row-test-is-the-existence-step
kind: specification
status: draft
title: A stored-row test is the existence step, not a when_subject identity read
summary: Decline of the when_subject identity read (github:beyond10x/ess#643) with the idiom, run on 0.74.0
tags:
- adopter-report
refs:
- provider: github
  reference: beyond10x/ess#643
relations:
- decides: story:when-subject-reads-the-addressed-identity
- informed_by: epic:downstream-reported-gaps
- serves: vision:O2
revision: 1
---
## Statement

A test on whether the addressed row is stored is the existence step, not a `when_subject:` predicate on the identity. A command that must answer "the row is stored" and "the row is not" writes the stored case as its unguarded update, and the unknown case as a creator with `unknown_instance: true`. A `when_subject:` predicate that reads the identity field is refused: on every row `when_subject:` reads, the identity equals `input.<id>` and is defined, so the predicate cannot distinguish anything.

Answers the adopter report of story:when-subject-reads-the-addressed-identity (github:beyond10x/ess#643). Decision: **decline, with this idiom**.

## Sources

- `crates/specify/ess-domain/src/command/subject_fact.rs:455`: the subject environment is "minus the identity".
- `crates/specify/ess-domain/src/entity.rs:985`: the identity is a separate `Field`, not one of `entity.fields`.
- `docs/design/language-grids.md`: grid R has no identity cell for `when_subject:`.

## The idiom

```yaml
format: ess/32
system: repro
version: v1
domain: repro.g
summary: A grant per parent; minting is create-or-update and refuses a missing parent first.
types:
  - {name: repro.g.ParentId, kind: newtype, of: Uuid}
entities:
  - name: repro.g.Parent
    identity: {name: parent_id, type: repro.g.ParentId}
    fields:
      - {name: label, type: String}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
  - name: repro.g.Grant
    identity: {name: parent_id, type: repro.g.ParentId}
    fields:
      - {name: enabled, type: Boolean}
    lifecycle: {initial: Live, states: [Live], terminal: [Live]}
errors:
  - {name: repro.g.ParentNotFound, summary: No parent carries the identity., fields: []}
events:
  - name: repro.g.Minted
    fields: [{name: parent_id, type: repro.g.ParentId}]
  - name: repro.g.ParentAdded
    fields: [{name: parent_id, type: repro.g.ParentId}]
actors:
  - name: repro.g.User
    may: [repro.g.AddParent, repro.g.Mint]
commands:
  - name: repro.g.AddParent
    input:
      - {name: label, type: String}
    outcomes:
      - name: added
        creates: repro.g.Parent
        instance: parent_id
        emits: [repro.g.ParentAdded]
        payload: {repro.g.ParentAdded: {parent_id: {generated: true}}}
        sets: {label: input.label}
  - name: repro.g.Mint
    input: [{name: parent_id, type: repro.g.ParentId}]
    outcomes:
      - name: first
        unknown_instance: true
        creates: repro.g.Grant
        instance: parent_id
        emits: [repro.g.Minted]
        payload: {repro.g.Minted: {parent_id: input.parent_id}}
        sets: {enabled: true}
      - name: already
        updates: repro.g.Grant
        instance: parent_id
        emits: [repro.g.Minted]
        payload: {repro.g.Minted: {parent_id: input.parent_id}}
        sets: {enabled: true}
views:
  - name: repro.g.Grants
    source: repro.g.Grant
    consistency: read_your_writes
    fields:
      - {name: parent_id, type: repro.g.ParentId}
      - {name: enabled, type: Boolean}
```

The `already` branch is the unguarded default: it fires when the row is stored. The `first` branch is the `unknown_instance: true` creator.

## Run

Commands run on `ess` 0.74.0 against the idiom above, kept outside the repository as `idiom.yaml`:

```console
$ ess specify validate --path idiom.yaml
repro v1 — 1 file(s), valid

$ ess conform run --path idiom.yaml --target interpreted --report-format 2
repro v1 against interpreted 0.74.0 — passed
  passed repro.g.AddParent/outcome/added
  passed repro.g.Mint/outcome/already
  passed repro.g.Mint/outcome/first
  3 scenarios: 3 passed, 0 failed, 0 error, 0 unsupported
```

The same model at `format: ess/14` is refused, `unknown_instance:` on a `creates:` branch requiring `ess/16`. At `ess/20` and `ess/24` it validates. The adopter shape, with `when_subject: {predicate: parent_id == input.parent_id}` on the update, is refused on 0.74.0 with ESS-COMMAND-003 whose hint names the stored fields (`unobservable_fact`), which is the refusal this idiom replaces.
