---
format: aep.planning-md/3
id: story:caller-attributes-are-readable-in-a-when-related-predicate
kind: story
status: draft
title: Caller attributes are readable in every row-reading guard (related predicate, all:, row-set where/forall, set-effect where)
summary: 'Adopter report G1: A when_related predicate cannot read a caller attribute (ess#636).'
tags:
- ess-0.71.0
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#636
- provider: github
  reference: beyond10x/ess#638
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-synthesis
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:ess-37-and-conformance-64-65-are-open-for-the-epics-constructs
scope:
- confidence: cited
  path: crates/specify/ess-domain/src/command/caller_value.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command/related_guard.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command/row_set.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command/set_effects.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/system.rs
- confidence: inferred
  path: crates/specify/ess-domain/tests/caller_values.rs
- confidence: cited
  path: docs/design/caller-values.md
- confidence: cited
  path: docs/design/language-grids.md
- confidence: inferred
  path: schemas/generated/ess.schema.json
- confidence: inferred
  path: website/docs/guides/specify/values-and-views.md
- confidence: inferred
  path: website/docs/reference/spec-versions.md
revision: 8
---
## Need

A command cannot state "the related row exists, but it belongs to another caller's account". A
`when_related` predicate reads the related row's stored fields, `input.<field>` and `state`, but not
`caller.<attribute>`, although `when:`, `when_subject` and `sets:` read it from `ess/16`. The same
column of grid R is closed in the two other positions that read rows: a row-set `where:`/`forall:`
and a set effect's `instances:`/`affects:` `where:`. Minimal reproduction (0.92.0, `ess/36`), entity
`example.g.Parent {parent_id, account_id: String}`, actor `example.g.User` with
`attributes: [{name: account_id, type: String}]`:

```yaml
  - name: example.g.Bind
    input: [{name: parent_id, type: example.g.ParentId}]
    outcomes:
      - name: missing
        when_related: {via: input.parent_id, exists: false}
        error: example.g.ParentNotFound
      - name: elsewhere
        when_related: {via: input.parent_id, predicate: account_id != caller.account_id}
        error: example.g.ParentNotFound
      - name: bound
        creates: example.g.Grant
        instance: grant_id
        sets: {parent_id: input.parent_id, account_id: {caller: account_id}}
```

```text
error[ESS-COMMAND-003]: `caller.account_id` reads `caller`, which is not a declared observable root; `when_related` reads the stored fields of `example.g.Parent`, and `caller` is not one
```

The row-set form `when_related: {entity: example.g.Parent, where: account_id == caller.account_id, exists: false}`
and `instances: {where: account_id == caller.account_id}` are refused with the same ESS-COMMAND-003.

## Acceptance

- `ess specify validate` on the reproduction above at the new source format exits 0; at `ess/36` it
  exits 1 with `unsupported_format_version` naming the new format (no longer ESS-COMMAND-003).
- At the new format, `caller.account_id` in a row-set `where:` and `forall:`, in each element of
  `when_related: {all: [...]}`, and in `instances: {where: …}` / `affects: [{where: …}]` validates;
  below it each is refused as `unsupported_format_version` naming the new format.
- A caller attribute that some actor whose `may` names the command does not declare is refused in
  each of those positions with the code `when_subject` gives today (`conflicting_declaration` or
  `undeclared_reference`), its hint naming the actor; an ordering or a literal against a caller
  attribute stays `unobservable_fact`, as in `when:`.
- The compiled IR of the reproduction marks `elsewhere` as decided by the caller, so `ess generate
  openapi` answers it `403` and lists `account_id` under `x-ess-caller`.
- `docs/design/language-grids.md` grid R shows the caller column `yes` (`==`/`!=` against a field)
  for `when_subject:`, `when_related:` predicate, row-set `where:`/`forall:` and set-effect `where:`;
  `docs/design/caller-values.md`, `website/docs/reference/spec-versions.md` and
  `website/docs/guides/specify/values-and-views.md` state the new positions.

## Scope

- cited: docs/design/language-grids.md
- cited: docs/design/caller-values.md
- cited: crates/specify/ess-domain/src/command/caller_value.rs
- cited: crates/specify/ess-domain/src/command/related_guard.rs
- cited: crates/specify/ess-domain/src/command/row_set.rs
- cited: crates/specify/ess-domain/src/command/set_effects.rs
- inferred: crates/specify/ess-domain/src/system.rs
- inferred: crates/specify/ess-domain/tests/caller_values.rs
- inferred: website/docs/reference/spec-versions.md
- inferred: website/docs/guides/specify/values-and-views.md
- inferred: schemas/generated/ess.schema.json
confidence: medium

## Fit review

1. **Need.** A refusal for a related row that exists but is owned by another caller's account
   cannot be stated; the adopter falls back to an unchecked refusal or to no refusal at all. Minimal
   reproduction above, output in the triage run (`ess specify validate --path as-written`, exit 1,
   ESS-COMMAND-003). The requester's proposed syntax: `predicate: account_id != caller.account_id`
   under `via: input.parent_id`.
2. **Class.** Gap. `caller.<attribute>` is admitted only in `when:` and `when_subject:`
   (`crates/specify/ess-domain/src/command/caller_value.rs:352-411` walks those two conditions and
   no other); the related-row check builds its environment with no caller
   (`crates/specify/ess-domain/src/command/related_guard.rs:1781-1792`), so the expression checker
   reports `caller` as an undeclared root (`crates/specify/ess-domain/src/expression.rs:984`).
3. **Already expressible?** No. `when_subject` reads the caller only over the addressed subject
   (`docs/design/caller-values.md:44`), and the row here is another entity's; the `when-subject-sibling`
   repro validates only because it guards the Parent itself, which a command creating a Grant does
   not address. An idiom moving the account check into an `external:` branch would take the checked
   figure out of the specification (`docs/design/generic-language-features.md:588-592`, policy 1).
4. **Fit.** Grid R, column "caller": the target closes it for every command position
   (`docs/design/language-grids.md:53-54`), and the G1 delivery plan names "the caller and subject
   columns closed" (`docs/design/generic-language-features.md:671`). The cells today are `I don't
   know` / `no` for `when_related:` predicate, row-set `where:` and `instances:`/`affects:` `where:`
   (`docs/design/language-grids.md:25-27`); the reproduction shows all three refused. So the whole
   column opens, not the `when_related` cell alone (closing rule, `generic-language-features.md:211-216`).
   Vocabulary: the same `caller.<attribute>` operand and the same `==`/`!=`-against-a-field rule
   (`caller-values.md:54-62`); no new key. Composes: the interpreter already evaluates a related
   predicate with the caller in hand (`crates/verify/ess-conformance/src/interpret/execute/subject.rs:31-47, 89-100`);
   generated Rust and Go already read a related predicate in an environment with `caller.`
   (`crates/generate/ess-synth/src/determined.rs:242-245, 848-852`); synthesis and the row-set
   interpreter do not (follow-up stories). Entity Runtime keeps `CallerUnsupported` by name
   (`caller-values.md:88`).
5. **Second adopter.** A ticketing service: `example.ticket.Assign` names a `Queue` by input and is
   refused where `queue.team_id != caller.team_id`; or a row-set refusal "no open session of the
   caller's own device", `when_related: {entity: example.session.Session, where: device_id == caller.device_id and state == Open, exists: false}`.
6. **Cost.** One source format (the next one open, `ess/37` if no other decision allocates it
   first); no new key, no new diagnostic code, no IR shape change (guards already carry `caller.`
   paths and `decided_by_caller` exists, `caller-values.md:74-79`); no suite format (the step's
   `caller` exists from suite/26, `caller-values.md:104-106`). `ess verify diff` reports a changed
   predicate as `outcome-condition-changed` (`generic-language-features.md:625-627`).
7. **Alternatives.** (a) Change nothing: the adopter's refusal stays unstated and unchecked;
   rejected, it is a gap. (b) Open only the `when_related` predicate cell, as requested: refused by
   the closing rule, and the row-set and set-effect cells would be the next request. (c) Open the
   caller column in every row-reading guard position with the existing operand rule: chosen.
   The requester's spelling is taken unchanged inside (c).

## Decisions

accept, redesigned: the requester's spelling is admitted as written, and the decision opens the
whole caller column of grid R for the positions that read rows (related-row predicate, `all:`
elements, row-set `where:`/`forall:`, set-effect `where:`), with the `==`/`!=`-against-a-field rule
of `ess/16`, under the next source format. Synthesis and the targets follow in
`story:a-caller-read-in-a-row-reading-guard-is-witnessed-by-two-callers` and
`story:generated-targets-answer-a-caller-read-in-a-row-reading-guard`.

## Source

github:beyond10x/ess#636
