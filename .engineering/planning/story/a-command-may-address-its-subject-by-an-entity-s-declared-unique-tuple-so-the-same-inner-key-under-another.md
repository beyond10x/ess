---
format: aep.planning-md/3
id: story:a-command-may-address-its-subject-by-an-entity-s-declared-unique-tuple-so-the-same-inner-key-under-another
kind: story
status: draft
title: A command may address its subject by an entity's declared unique tuple, so the same inner key under another parent reads as unknown
summary: 'instance: addresses a row by its identity only, so a command cannot address it by an entity''s declared unique tuple and the same inner key under another parent cannot read as unknown (gap report, 2026-11-24)'
tags:
- external-request
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:an-identity-may-be-a-composite-of-fields
- informed_by: architecture-decision-record:a-compound-key-is-a-uniqueness-constraint-first
- informed_by: story:an-update-addresses-its-record-by-an-optional-identity-or-a-unique-field
revision: 4
---
## Need

A record is unique only within its parent: the same `order_id` under another `workspace_id` is a
different row. The caller of an update names the row by both values, and a call that names a
workspace with no such order must read as "no such order", not as a hit on another workspace's row.

Reproduction (brand-free, ESS example form):

```yaml
entities:
  - name: Order
    identity: id
    unique: [workspace_id, order_id]
    fields: [{name: workspace_id, type: WorkspaceId}, {name: order_id, type: OrderId}, {name: status, type: Status}]
commands:
  - name: confirm
    inputs: [{name: workspace_id, type: WorkspaceId}, {name: order_id, type: OrderId}]
    outcomes:
      - {name: confirmed, updates: Order, instance: ???, sets: {status: confirmed}}
      - {name: unknown-order, unknown_instance: true, error: no-such-order}
```

What exists, read at `origin/main` (not repeated here):

- `instance:` names one top-level input, `instance: Option<String>`
  (`crates/specify/ess-domain/src/command.rs:7556`), and the compiled instance is the supplied or the
  observed identity (`ResolvedInstance::Supplied | Observed`, `crates/specify/ess-compiler/src/ir.rs:828`).
  An entity's identity is one `Field` (`crates/specify/ess-domain/src/entity.rs:991`).
- The accepted ADR `architecture-decision-record:a-compound-key-is-a-uniqueness-constraint-first`
  chose option C, a `unique: [fields]` constraint beside a one-field identity, "C first, then B if
  relations on compound keys are still needed". Options A (composite identity) and B (record-typed
  identity) were not chosen. The active story `story:an-identity-may-be-a-composite-of-fields`
  declares and checks the constraint on create and on an update that writes a member. Neither the ADR
  nor that story lets a command address its subject by the tuple.
- The draft `story:an-update-addresses-its-record-by-an-optional-identity-or-a-unique-field` addresses
  a subject by one unique field. This story is the same construct for a tuple.
- Today's workaround is the cost in the report: identity is `order_id`, `workspace_id` is a stored
  field, and every command carries an `other-workspace` outcome that reads the found row's
  `workspace_id`. That outcome is a subject predicate choosing between a move and an update, which
  `ess generate synthesize --target rust` leaves an obligation. It also moves a figure the reader sees
  (the tuple is the key) into a guard on every command.

Related markers that this removes or that other stories already carry:

- `when_related:` comparing a related row's `workspace_id` with the command's own: grid R reads an
  input and the related row's own fields in `when_related:` today (`docs/design/language-grids.md:25`),
  so `workspace_id == input.workspace_id` is expressible. Once the subject is addressed by the tuple,
  the command's own value is the input. Reading the addressed row's field there is the draft
  `story:a-when-related-predicate-reads-the-addressed-row-as-subject-field-and-subject-state-so-one-branch-is-selected`.

## Outcome

An update, delete or read-only command may address its subject by the entity's declared `unique`
group, one input per member. A call whose tuple matches no row answers `unknown_instance:`. The
tuple is the row's address everywhere an identity is: synthesis, the interpreter, generated Rust and
Go, OpenAPI.

## Acceptance

Format (D77): lands in the source format that `story:an-identity-may-be-a-composite-of-fields` lands
in (`ess/38` while it is open and unreleased); below it every new key is refused as
`unsupported_format_version` naming it.

- At the new format, `instance: {unique: [workspace_id, order_id]}` on a command whose inputs carry
  those two names validates and `ess specify validate` exits 0.
- Each of these is refused by name: a list that is not the entity's declared `unique` group, a member
  with no input of that name, an input whose type differs from the member's, and a command that names
  both `instance: <input>` and the tuple form.
- A call that sends an existing `(workspace_id, order_id)` takes the accepting branch. A call that
  sends the same `order_id` under another `workspace_id`, or one that sends a tuple no row holds, takes
  `unknown_instance:`. Synthesis (`ess verify conform synthesize`) witnesses both calls against two
  rows that differ in one member.
- `ess generate synthesize --target rust` and `--target go` on the reproduction report `0 obligation(s)`
  for `confirm`, and the suite passes against each generated server.
- `ess verify diff` rates a change of the addressed group as a breaking change; an entity without the
  new key keeps its IR bytes.

## Scope

- cited: `crates/specify/ess-domain/src/command.rs` (`instance`, 7556), `entity.rs` (`identity`, 991),
  `crates/specify/ess-compiler/src/ir.rs` (`ResolvedInstance`, 828), `crates/generate/ess-synth`
  (storage lookup and `determined.rs`), `crates/generate/ess-gen/src/openapi.rs` (the route path),
  `docs/design/cross-record-and-stored-field-guards.md` (617-621, 636),
  `website/docs/reference/spec-versions.md`.
- Size: L, about 700 changed lines; the HTTP route shape for a tuple is decision AC9-D2.
- Depends on `story:an-identity-may-be-a-composite-of-fields`.

## Fit review

1. **Need.** A natural key of two fields must address a row, so a wrong parent reads as unknown.
   Reproduction above. The reporter's syntax (labelled theirs): a row addressed by two top-level input
   fields.
2. **Class: gap.** The constraint exists (accepted ADR, active story); the addressing does not.
3. **Already expressible?** Partly, and the idiom fails the fix-by-idiom test. A synthetic `order_key`
   input the caller computes from the pair moves the tuple out of the specification. The
   `other-workspace` outcome keeps the figure but costs an obligation per command in Rust synthesis
   and a branch per command. `hex(sha256(tuple))` (draft `story:a-command-guard-compares-a-value-with-the-digest-of-several-named-inputs-hex-sha256-tuple-and-the-interpreted`)
   compares; it does not address.
4. **Fit.** Grid row: the subject-addressing row (`instance:`), siblings `updates:`, deletes, read-only
   commands and `instances:`. The tuple form reuses `unique:` vocabulary and the `unknown_instance:`
   answer. Every target resolves a tuple to one row (the storage port gains one lookup by group) or
   refuses by name. The route of a bound command carries both members (AC9-D2).
5. **Second adopter.** A `Chapter` unique within a `book_id` by `number`; a `Seat` unique within a
   `flight_id` by `label`.
6. **Cost.** One key shape, no new identity type, no change to relations (still one `via:`), one
   storage lookup, one diff classification.
7. **Alternatives.** Change nothing (the idiom above); ADR option B (record-typed identity) and option A
   (composite identity), both larger and not chosen; a narrow form for one `unique` field only (the
   draft above). The tuple form wins because it is the same construct as the one-field draft and keeps
   ADR option C's one-field identity.

## Decisions

AC9-D1 (ARCHITECTURE): how a command addresses a row by a tuple. A: by the declared `unique` group. B: record-typed identity (ADR option B). C: composite identity (ADR option A). D: decline, with the synthetic-key idiom. Open, with the owner (decided by nobody yet, 2026-11-24): the default A is not taken, and the story stays a draft until the owner answers.

AC9-D2: the HTTP route of a tuple-addressed command. A: both members as path segments or inputs, as the binding declares. B: a binding may carry one member in the path and one in the query. Decided: A, 2026-11-24 (the coordinator, under the owner's delegation). It applies to the route of a tuple-addressed command, which exists only once AC9-D1 is answered.

- AC9-D1: A, taken by the operator session on 2026-11-25 on the owner's word of 04:03 CET; reversible until a release carries it

- A release that carries this key names this pick in its pull-request line; the operator session merges that release only after the owner has read the list.
