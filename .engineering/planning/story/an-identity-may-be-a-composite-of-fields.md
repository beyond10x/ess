---
format: aep.planning-md/3
id: story:an-identity-may-be-a-composite-of-fields
kind: story
status: draft
title: 'An entity may declare a unique tuple of fields beside its identity (ADR option C), answered by existing_instance:'
summary: 'A composite identity is refused both as instance: map and as a published struct (a2a wave 1, M1).'
tags:
- adopter-report
- ess-0.71.0
- feature-request
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#638
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-domain-language
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:ess-37-and-conformance-64-65-are-open-for-the-epics-constructs
scope:
- confidence: inferred
  path: crates/specify/ess-compiler/src/ir.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command.rs
- confidence: cited
  path: crates/specify/ess-domain/src/entity.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/system.rs
- confidence: inferred
  path: crates/specify/ess-domain/tests/unique_tuple.rs
- confidence: cited
  path: docs/design/cross-record-and-stored-field-guards.md
- confidence: inferred
  path: docs/design/outcome-shapes.md
revision: 11
---
## Need

A row's natural key may be a tuple of fields, and one member of it may come from the caller rather
than the input. Two accounts may hold the same application id, but one account may not hold it twice.
ESS cannot state this. An identity is one field (`crates/specify/ess-domain/src/entity.rs:991`,
`pub identity: Field`), and `instance:` takes one input name (`crates/specify/ess-domain/src/command.rs:8065`).
So the model either makes the application id the identity, which claims a stronger uniqueness than
the system has, or states no uniqueness at all.

Minimal reproduction (0.92.0, `ess/36`). Actor `example.g.User` declares
`attributes: [{name: account_id, type: String}]`:

```yaml
  - name: example.g.Bind
    input: [{name: app_id, type: String}]
    outcomes:
      - name: bound
        creates: example.g.Grant
        instance: {account_id: {caller: account_id}, app_id: input.app_id}
```

```text
domains/g.yaml: invalid type: map, expected a string
```

A struct identity sent as input is refused differently: ESS-COMMAND-001, ``acts on the instance named by `key`, which is no field of an emitted event``. It would also make the caller the authority over the account.

The nearest statement today has two problems. With a synthetic `grant_id`, a row-set duplicate
refusal over `{all: [account_id == caller.account_id, app_id == input.app_id]}` is refused with
ESS-COMMAND-003 (`caller` in a row-set `where:`, beyond10x/ess#636). With the account as input, it
validates, but synthesis cannot arrange a row through that creator (ESS-SYNTH-001 "`taken` is not
decidedly passed over: its row set is True" for the update that needs the row).

Decided by `architecture-decision-record:a-compound-key-is-a-uniqueness-constraint-first`
(accepted 2026-10-23): a compound key is first a uniqueness constraint beside a one-field identity.

## Acceptance

- At the new source format, an entity may declare `unique: [account_id, app_id]` over stored
  fields, and `ess specify validate` exits 0. Each of these is refused by name: a name that is no
  stored field, one field listed twice, the identity listed, and a field of a type without equality
  (`List`, `Map`, `Json`). Below the new format the key is refused as `unsupported_format_version`
  naming it.
- A creating command of such an entity may declare `existing_instance: true` with an error. That
  branch answers a create whose `sets:` would write a tuple a row already holds, whether a member
  comes from `input.` or `{caller: …}`. A creating command with no such branch is refused as
  `non_exhaustive_branches` naming the tuple.
- An `updates:` branch that writes a member of the tuple needs the same answer, or is refused as
  `non_exhaustive_branches`.
- The IR carries `unique` on the entity. An entity without it keeps its IR bytes.
- `docs/design/cross-record-and-stored-field-guards.md` replaces "composite keys … stay out of
  scope" with the constraint, citing the record. `website/docs/reference/spec-versions.md` names the
  key.

## Scope

- cited: crates/specify/ess-domain/src/entity.rs
- cited: crates/specify/ess-domain/src/command.rs
- cited: docs/design/cross-record-and-stored-field-guards.md
- inferred: docs/design/outcome-shapes.md
- inferred: crates/specify/ess-domain/src/system.rs
- inferred: crates/specify/ess-compiler/src/ir.rs
- inferred: crates/specify/ess-domain/tests/unique_tuple.rs
- inferred: website/docs/reference/spec-versions.md
- inferred: schemas/generated/ess.schema.json
confidence: medium

## Fit review

1. **Need.** A row is unique on a tuple of fields, one of them written from the caller. Minimal
   reproduction above (triage run `as-written`, `struct-identity`, `row-set-unique`). The
   requester's proposed syntax: `instance: {account_id: {caller: account_id}, app_id: input.app_id}`
   over a struct identity. An earlier report against the same story asked for a composite identity
   published by the creating event (ESS-SYNTH-001, "a struct held by reference has no member
   projection").
2. **Class.** Gap. "Range overlap, composite keys and the concurrency argument stay out of scope"
   (`docs/design/cross-record-and-stored-field-guards.md:616-617`).
3. **Already expressible?** Partly. A row-set duplicate refusal over input-only members validates
   (`repro/tuple-idiom-input`), but it is per command, not per entity, so another writer can break
   it. No row can be arranged through it for a later command (ESS-SYNTH-001, triage run). `unique:` removes that guard from the creator. With a caller member it does not
   validate (beyond10x/ess#636).
4. **Fit.** The accepted record weighs a composite identity (A), a record-typed identity (B) and a
   uniqueness constraint (C). It chooses C first, because C reuses the duplicate-identity refusal
   and keeps one identity field everywhere
   (`architecture-decision-record:a-compound-key-is-a-uniqueness-constraint-first`,
   § Recommendation). The answer reuses `existing_instance:`, the duplicate-create marker
   (`docs/design/outcome-shapes.md:257-264`), and adds no second spelling. The caller member needs
   no construct of its own: `{caller: …}` already writes a stored field (`docs/design/caller-values.md:44`).
   Addressing an existing row by the tuple is a selection (G2, `docs/design/generic-language-features.md`
   § G2: `instances: {where: p}` is `each r in rows(E where p)`), not an `instance:` map. Reading
   the caller there is beyond10x/ess#636's column.
5. **Second adopter.** A feed item unique on (container, item id), and a container id unique only
   within its connection (the record's own context).
6. **Cost.**
   - One source format (the next one open) and one new key, `unique:`.
   - No new diagnostic codes.
   - An IR field, left out when absent.
   - `ess verify diff` needs `unique-added` (breaking for callers) and `unique-removed` (breaking
     for readers).
   - Synthesis and targets are in the stories under Decisions.
7. **Alternatives.** The record weighs A, B and C. As requested, `instance:` as a map over a struct
   identity is B plus a caller member. B is refused for now by the record, because every consumer
   of an identity value would need a structured key. Changing nothing leaves the model claiming a
   uniqueness the system does not have.

## Decisions

accept, redesigned. Following the accepted record, the compound key is `unique: [fields]` beside a
one-field identity, answered by `existing_instance:`. A member may be written from the caller as
any stored field is. An existing row is addressed by a selection on the tuple, not by an
`instance:` map. Suggested new title: "An entity may declare a unique tuple of fields beside its
identity". Follow-ups:

- `story:synthesis-witnesses-a-unique-tuple-collision`
- `story:generated-targets-enforce-a-unique-tuple`
- `story:a-row-set-refusal-beside-a-set-effect-on-its-entity-is-witnessed`
- the caller column of beyond10x/ess#636

## Source

github:beyond10x/ess#638
