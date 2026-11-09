---
format: aep.planning-md/3
id: story:when-subject-beside-a-row-set-validates-under-the-g2-format
kind: story
status: draft
title: when_subject, when_subject_state, when_state_changes and input_absent beside a row set validate under the next format, ordered by one precedence rule
summary: ESS-COMMAND-009 refuses when_subject beside a row set at every format; the G2 record lifts it in its own format.
tags:
- downstream
- follow-up
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#564
- provider: github
  reference: beyond10x/ess#642
relations:
- informed_by: story:a-guard-on-the-input-addressed-parent-row-holds-on-every-command-of-its-children
- decomposes: epic:generic-row-queries
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: cited
  path: crates/specify/ess-domain/src/command/row_set.rs
revision: 6
---
## Need

A create-or-update must answer `404` for a missing parent before it creates the child, re-enables a
paused child, or does nothing. The parent check is a row-set `when_related` refusal. It is written as
a row set because the child is keyed by the parent's identity type, and `via: input.parent_id` is
then refused as naming the identity of two entities (ESS-COMMAND-004). The re-enable branch reads
the child with `when_subject`. Validation refuses the pair at every format.

Minimal reproduction (0.92.0, `ess/36`). `example.g.Parent` and `example.g.Grant` are both keyed
`parent_id: example.g.ParentId`, and `Grant` stores `enabled: Boolean`:

```yaml
  - name: example.g.Mint
    input: [{name: parent_id, type: example.g.ParentId}]
    outcomes:
      - name: no-parent
        when_related: {entity: example.g.Parent, where: parent_id == input.parent_id, exists: false}
        error: example.g.ParentNotFound
      - name: first
        unknown_instance: true
        creates: example.g.Grant
        instance: parent_id
        sets: {enabled: true}
      - name: again
        when_subject: {predicate: enabled == false}
        updates: example.g.Grant
        instance: parent_id
        sets: {enabled: true}
      - name: already
        updates: example.g.Grant
        instance: parent_id
        sets: {enabled: true}
```

```text
error[ESS-COMMAND-009]: `example.g.Mint` selects on a row set (`when_related: {entity, where, …}`) and on a `when_subject` guard; one command reads one kind of related row in this cut
  help: guard the command on the row set alone, or move the other guard to a command of its own
```

Declaring `again` first gives the same refusal. With an identity-addressed `via:` in place of the row
set (and a distinct identity type for `Grant`), the same four branches validate with `again` declared
first, synthesize 7 scenarios and pass on the interpreted target. So only the row-set cell is closed.

### Second reproduction (beyond10x/ess#564)


A command that addresses a row and reads its related rows through a row-set guard cannot also
refuse on the addressed row's stored fields: "not found when the incident belongs to another
environment" beside "refused when no window of the incident exists". The pair is refused as
ESS-COMMAND-009, "one command reads one kind of related row in this cut"
(`crates/specify/ess-domain/src/command/row_set.rs:39-42`); `ess/32` lifted the identity-addressed
`when_related:` beside a row set and kept this pair refused at every format.

Minimal reproduction (ess 0.92.0, `format: ess/36`; `repro/h-subject-beside-row-set` of the triage
of beyond10x/ess#564):

```yaml
  - name: demo.ops.CloseIncident
    outcomes:
      - {name: unknown-incident, unknown_instance: true, error: demo.ops.NotFound}
      - name: other-environment
        when_subject: {predicate: environment != input.environment}
        error: demo.ops.NotFound
      - name: no-window
        when_related: {entity: demo.ops.Window, where: incident_id == subject.incident_id, exists: false}
        error: demo.ops.WrongState
      - name: closed
        moves: demo.ops.Incident.resolve
        instance: incident_id
      - {name: wrong-state, wrong_state: true, error: demo.ops.WrongState}
```

```text
$ ess specify validate --path h-subject-beside-row-set
error[ESS-COMMAND-009]: `demo.ops.CloseIncident` selects on a row set (`when_related: {entity, where, …}`) and on a `when_subject` guard; one command reads one kind of related row in this cut
help: guard the command on the row set alone, or move the other guard to a command of its own
exit 1
```
## Acceptance

- At the new source format, the reproduction with `again` declared before `no-parent` validates
  (`ess specify validate`, exit 0).
- As declared above (`no-parent` first), it is refused as `conflicting_declaration`
  (ESS-COMMAND-004), because the precedence order answers `again` at step 3 and `no-parent` at
  step 4. The hint says to declare `again` first, which is the message the identity-addressed form
  gives today.
- At the new format, `when_subject_state:`, `when_state_changes:` and `input_absent:` beside a
  row-set guard validate too. Below it each pair keeps ESS-COMMAND-009, its message unchanged and its
  hint naming the new format.
- The precedence plan places the pair by the one rule: step 3 subject guards, then step 4 row-set
  refusals, then a row-set upsert's `unknown_instance:` creation. A unit test in `precedence.rs` pins
  the order for the reproduction.
- With `again` declared first, `ess verify conform run --target interpreted` answers `no-parent` for
  an unknown parent with no grant, and `again` for a paused grant.
- `docs/design/language-grids.md` grid R row `when_related:` drops "one kind of related row per
  command". `website/docs/reference/predicates.md` lists the pair.

Also, from the triage of beyond10x/ess#564 (an addressed row's `when_subject` beside a row set):

At G2's next source format:

- `ess specify validate --path h-subject-beside-row-set` exits 0; the precedence rule answers
  `unknown-incident` (step 2), then `other-environment` and `wrong-state` (step 3), then
  `no-window` (step 4), then `closed`;
- `ess verify conform synthesize` witnesses every branch: `other-environment` on an incident with
  a window in another environment, `no-window` on an incident in the sent environment with no
  window, `closed` with one; `ess verify conform run --target interpreted` passes all, and a target
  reading the windows first fails `other-environment`;
- below that format the pair keeps ESS-COMMAND-009, its message unchanged and its hint naming the
  format; a model without the pair keeps its IR bytes;
- generated Rust and Go keep the command an obligation naming the row set; Entity Runtime refuses
  it by name (`RowSetUnsupported`).
## Scope

- cited: crates/specify/ess-domain/src/command/row_set.rs
- cited: crates/specify/ess-domain/src/command/precedence.rs
- inferred: crates/specify/ess-domain/src/system.rs
- inferred: crates/specify/ess-domain/tests/row_set_beside_subject_guard.rs
- inferred: docs/design/language-grids.md
- inferred: website/docs/reference/predicates.md
confidence: high

## Fit review

1. **Need.** A create-or-update whose existing-row branches read the row, guarded first by a
   row-set refusal over another entity. Reproduction above (triage run: `as-written` and
   `again-first`, exit 1, ESS-COMMAND-009). The requester's proposed syntax is the reproduction as
   written.
2. **Class.** Gap. The refusal is a scope cut, "in this cut"
   (`crates/specify/ess-domain/src/command/row_set.rs:383-384`). `beside` lists the kinds still
   refused (`row_set.rs:314-325`). `ess/32` opened only the identity-addressed kind
   (`row_set.rs:328-334`).
3. **Already expressible?** Only by changing the model: give `Grant` its own identity type and use
   `via:` (`repro/via-sibling` validates, synthesizes and passes). That changes the identity and the
   wire of a 1:1 child keyed by its parent, so it is no idiom for this shape.
4. **Fit.**
   - G4 decides the order: subject guards answer at step 3, related rows and row sets at step 4
     (`docs/design/generic-language-features.md:482-497`).
   - "ESS-COMMAND-009's one-kind-of-row refusal is lifted with G2" (`generic-language-features.md:512-513`).
   - The precedence code already places every such branch
     (`crates/specify/ess-domain/src/command/precedence.rs:515-545`: a row-set upsert's
     `unknown_instance:` after the row-set refusals; subject guards at `HeldState`).
   - The closing rule lifts the whole remaining list of `beside`, not `when_subject` alone
     (`generic-language-features.md:211-216`).
   - The requester's declaration order (row set first) is the one the rule refuses. The refusal names
     the order, as it does for the identity-addressed form today (triage run for the sibling model,
     ESS-COMMAND-004 "declare `again` before `no-parent`").
5. **Second adopter.** A seat reservation is created, resumed or kept, and is refused first when no
   open event matches the input's event code (`where: code == input.code and state == Open`).
6. **Cost.**
   - One source format: the next one open, counted under G2 (`generic-language-features.md`,
     answer 2).
   - No new key and no new diagnostic code.
   - No IR change: the plan is derived, never serialised (`generic-language-features.md:580-581`).
   - Synthesis and generated targets: `story:a-row-set-upserts-creating-branch-is-arranged-past-its-row-set-refusals`.
     Generated Rust and Go keep row sets as named obligations (`generic-language-features.md:299-301`).
7. **Alternatives.** (a) Change nothing: the adopter splits the command, which changes the API.
   (b) Admit the pair with the row set answering first, as the requester declared: rejected, because
   it contradicts G4's step order, under which the addressed record's guards answer before the rows
   around it. (c) Admit every remaining kind under the one rule: chosen.

## Decisions

accept, redesigned. The pair and its siblings (`when_subject_state:`, `when_state_changes:`,
`input_absent:`) validate beside a row set under the next source format, ordered by G4. The
requester's declaration order is refused with a hint to declare the subject guard first. The
outcome the requester expects, a `404` for a missing parent on create, holds: on a row-set upsert the
creating `unknown_instance:` answers after the row-set refusals. The synthesis half is
`story:a-row-set-upserts-creating-branch-is-arranged-past-its-row-set-refusals`.

## Source

github:beyond10x/ess#642, github:beyond10x/ess#564
