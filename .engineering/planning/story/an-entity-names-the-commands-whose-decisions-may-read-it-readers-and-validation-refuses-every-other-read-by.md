---
format: aep.planning-md/3
id: story:an-entity-names-the-commands-whose-decisions-may-read-it-readers-and-validation-refuses-every-other-read-by
kind: story
status: draft
title: 'An entity names the commands whose decisions may read it, readers: [...], and validation refuses every other read by name'
tags:
- gap-p2
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:ess-specify-reads-prints-every-entity-each-command-s-decisions-read-by-kind-and-site-from-one-read-analysis
- depends_on: story:the-specification-format-ess-38-is-open-as-the-newest-reads-and-compiles-every-specification-exactly-as-ess
revision: 3
---
## Need

A domain fact about who may *read* an entity in a decision: "advisory notes never decide an
approval", "no command reads the audit log", "only the export command reads a vault row". A
downstream application states the first only by construction: the approving command takes no
advisory input, and no command reads the advisory entity. ESS has no declaration that says it, so
a later command that gains a guard over that entity breaks the claim and validation stays green.

Confirmed at `bcfc51555`: an entity declares a name, an identity, fields, relations, a lifecycle,
invariants, naming and `provided` (`crates/specify/ess-domain/src/entity.rs:984-1015`), and no key
restricts its readers. No rule limits a command's reads to its own domain. The only cross-domain
rule refuses deleting through `affects:` in another domain
(`crates/specify/ess-domain/src/command/set_effects.rs:2655-2674`).

Minimal reproduction, written fresh and not run:

```yaml
format: ess/38
# …
entities:
  - name: demo.review.Note
    identity: {name: note_id, type: demo.review.NoteId}
    fields: [{name: request_id, type: demo.approval.RequestId}, {name: tone, type: demo.review.Tone}]
    readers: []                       # no command's decision reads a note
    lifecycle: {initial: Filed, states: [Filed], terminal: [Filed]}
commands:
  - name: demo.approval.Approve
    input: [{name: request_id, type: demo.approval.RequestId}]
    outcomes:
      - name: flagged                 # this read breaks the claim
        when_related:
          entity: demo.review.Note
          where: {all: [request_id == input.request_id, tone == negative]}
          exists: true
        error: demo.approval.Flagged
      - name: approved
        # …
```

The requester's words: "ESS has no 'never reads' invariant across domains". They proposed no
syntax.

## Outcome

From `ess/39` an entity may declare `readers:`, a list of commands and domains. The list is closed
for decisions. A command not on it, and not of a listed domain, whose outcome reads the entity
outside its own subject (each kind `g16-command-reads` reports, `subject` excepted) is refused at
validation. The refusal names the command, the entity and the site of the read. `readers: []`
means no other command reads the entity. An entity without the key restricts nothing, as today.
The design page says what the check cannot see.

## Acceptance

Format (D77, 2026-11-24): this story's construct lands in `ess/38`, opened by story:the-specification-format-ess-38-is-open-as-the-newest-reads-and-compiles-every-specification-exactly-as-ess; where this text names `ess/37`, `ess/39`, "the new format", "the next source format" or a format still to be decided as the format of this construct, read `ess/38`, and where it names `ess/36` as the format below it, read `ess/37` (`ess/37` shipped empty in 0.93.0); a reproduction model keeps the format it declares.

- Red first: `crates/specify/ess-domain/tests/entity_readers.rs` validates the reproduction and
  expects one refusal, with the message pinned:

  ```text
  [conflicting_declaration] command.demo.approval.Approve.outcomes.flagged.when_related: `demo.approval.Approve` reads `demo.review.Note`, whose `readers:` does not name it (hint: add the command or its domain to `readers:` of `demo.review.Note`, or drop the read)
  ```

  Without `flagged` the specification validates with exit 0. Today `readers` is refused as an
  unknown key.
- One case per read kind of `g16-command-reads` (`related`, `row-set`, `rows-quantifier`,
  `value`, `selection`, and `binding` if `g16-command-reads` D1 is (a)), each refused at its own
  site.
- An outcome whose subject is the entity (creates, moves, updates or deletes it, or guards on it
  with `when_subject*`, `existing_instance:`, `unknown_instance:` or `wrong_state:`) is not a read
  for this rule, and a test pins that.
- A `readers:` entry that names no command and no domain is refused as `undeclared_reference`. An
  entry listed twice is `conflicting_declaration`. Below `ess/39` the key is refused as
  `unsupported_format_version` naming `ess/39`. No new `ValidationCode`.
- A domain in `readers:` admits every command declared in that domain.
- The IR carries `readers` on the entity, left out when the key is absent. A model without it
  keeps its IR bytes and digest, and an old-reader test reads an IR without the field.
- `RawSpecFile` gains the key, `cargo xtask schema` regenerates
  `schemas/generated/ess.schema.json`, and `task projection-check` passes.
- `docs/design/declared-readers.md` states the rule and what it cannot see:
  - an `external:` branch, decided outside the model, may consult anything;
  - an input value a caller computed from a view of the entity;
  - implementation code outside the model: the suite does not witness that a read is absent;
  - views: reading is what they are for, and a view grant governs it.
- `website/docs/reference/spec-versions.md` names the key in the `ess/39` row. The entity
  reference lists `readers:`.
- No generated target changes: the claim is about the model. The design page says so, and a test
  asserts that generated Rust for the reproduction without `flagged` is byte-equal with and
  without `readers:`.

## Fit review

Tree `bcfc51555`. Received 2026-11-24 from a downstream application, report item 16.

1. **Need.** Restrict which commands' decisions may read an entity, and refuse every other read.
   Reproduction above. No syntax was proposed.
2. **Class.** *Gap.* The claim can be held only by construction (entity keys at
   `entity.rs:984-1015`).
3. **Already expressible?** No. An invariant holds of one row (`docs/design/language-grids.md:51-52`).
   Cross-row invariants are a separate design (`generic-language-features.md`, open question 9,
   beyond10x/ess#75), and this claim is about the model, not about rows.
4. **Fit.** Outside both grids: it constrains the reads of grid R and opens no cell. It fits "decisions as
   data, with evidence" (O2): a static check over the compiled model, refused by name.
   - Vocabulary: a list of qualified names, like the one an actor's grants hold. A command entry or
     a domain entry resolves as every other reference does.
   - A closed list on the entity, not a deny list on the reader. A command added later is
     refused by default, and the claim stays in one place. The requester's "never reads", per
     command or per domain, is `P2B-D3` (b).
   - Targets: none generate or run anything for it. `ess verify diff` reports a change of the
     claim (`g16-readers-diff`). Synthesis does not witness it in this cut (D3).
5. **Second adopter.** A vault of personal data whose rows only `demo.export.Export` reads
   (`readers: [demo.export.Export]`). An append-only audit log (`readers: []`).
6. **Cost.** One key at `ess/39`, an IR field left out when absent, a schema regeneration, a
   diff kind (next story), and no diagnostic code. Size M.
7. **Considered.** (a) Change nothing: by construction, unchecked. (b) `never_reads:` on a
   command or a domain: a deny list, which misses every command written later. (c) The verb of
   `g16-command-reads` only, with the claim asserted in the application's CI: the claim stays out
   of the specification. (d) Chosen: `readers:` on the entity, with (c)'s analysis behind it.

## Decisions

**Accept, redesigned** (`P2B-D3` (a)). Open questions:

- **D1 (`P2B-D3`; open, with the owner as AQ2). Where does the claim live?** (a) `readers:`, closed, on the entity. (b)
  `never_reads:` on a command or a domain. (c) Only `ess specify reads`, checked by the
  application. (d) Decline. **Recommended: (a).**
- **D2 (open, with the owner as AQ2). Is an outcome acting on the entity a reader?** (a) No: its own subject is exempt. (b) Yes:
  every writer must be listed. **Recommended: (a).** The claim is about other commands' decisions.
  (b) makes every entity with `readers: []` list its own writers.
- **D3 (decided: a, 2026-11-24). Does the suite witness the claim against an implementation?** (a) Not in this cut. A
  later story adds a non-interference pair per command outside the list: two arranged stores that
  differ only in the entity's rows give the same outcome. (b) In this cut. **Recommended: (a).**
  The static check covers the model, and the pair is a synthesis design of its own.

## Scope

- cited: `crates/specify/ess-domain/src/entity.rs` (`EntitySpec` 984-1015, `RawEntity` 2172-2198)
- inferred: `crates/specify/ess-domain/src/command/reads.rs` (from `g16-command-reads`)
- inferred: `crates/specify/ess-compiler/src/ir.rs` (entity field, skipped when empty)
- cited: `schemas/generated/ess.schema.json` (regenerated)
- inferred: `crates/specify/ess-domain/tests/entity_readers.rs` (new)
- inferred: `docs/design/declared-readers.md` (new), `website/docs/reference/spec-versions.md`
- Size: M (about 500 changed lines).

## Depends on

- `g16-command-reads`
- `g15-ess39-opens` (or `ess/38`, if `P2B-D5` (a))

## Source

A downstream application, 2026-11-24, report item 16.
