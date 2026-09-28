# Set effects over filtered instances (`instances:`, `affects:`, ess/16)

Status: implemented, first cut (beyond10x/ess#167, #175; `story:set-effects-over-filtered-instances`).
Both constructs are admitted under source format `ess/16`. Below it `instances:`, `affects:` and a
`{count: changed}` beside `instances:` are refused with `unsupported_format_version` at the key
written, before the outcome is converted, so no shape rule of the construct and no cascade
(`missing_declaration`, `empty_declaration`) is reported beside it; a `{count: changed}` on any other
outcome stays the nested mapping it was.

## The gap

An outcome that `moves:` or `updates:` needed `instance:` naming one input of the entity's identity
type. A command ending every open session of a team (#167), or putting every other participant of a
room on hold beside its subject (#175), could declare its summary event and nothing about the rows,
so no scenario checked that the matching rows changed, that the others did not, or that a reported
count was right.

## The constructs

**Set subject (#167).** A `moves:` or `updates:` outcome declares `instances: {where: <predicate>}`
instead of `instance:`. The predicate is the stored-field grammar of `when_subject:`
(`command/subject_fact.rs`) over the entity's fields, with `input.<field>` operands. A `moves:`
skips a selected row resting outside the transition's `from` states rather than refusing; zero
selected rows is an accepted answer. The outcome's own `sets:` applies to every changed row. The
model keeps `subject: None` beside `SetEffects::instances`, so no single-instance reader mistakes the
branch for one.

**Count.** `{count: changed}` in `payload:` is the number of rows the outcome changed. It is
admitted only as a whole `payload:` field of an outcome with `instances:`, only into an `Integer`
field, and refused elsewhere by name (`unsupported_construct`, or `type_mismatch` for the type).

**Secondary effect (#175).** An outcome with one existing subject (`moves:` or `updates:` with
`instance:`) declares `affects:`, a list of `{entity, where, sets}`. `where` reads the entity's
fields, `input.<field>` and `subject.<field>`, the subject's stored value before the outcome. Where
`entity` is the subject's entity the subject itself is excluded. Selection is by filter only; a
relation named in place of a filter is not part of this cut.

`sets:` of either takes a literal, `input.<field>`, `{input: …, else: …}`, `{generated: true}` or
`{cleared: true}`. `{subject: …}`, `{related: …}`, `{increment: …}` and `{caller: …}` read one row or
the caller and are refused by name under a set effect. A `moves:` inside `affects:` is refused by
name.

Refused besides: `instance:` with `instances:` (`conflicting_declaration`); `instances:` on
`creates:` (`conflicting_declaration`) or on `deletes:`/`preserves:` (`unsupported_construct`);
`instances:` with no verb (`missing_declaration`); `instances:` on a refusal
(`refusal_mutated_state`) or under a condition other than `when:` or the default
(`unsupported_construct`); `affects:` beside `instances:` or on a creation, deletion or preservation
(`unsupported_construct`) or with no subject (`missing_declaration`); a `sets:` entry writing the
entity's identity under `instances:` or in an `affects:` entry (`conflicting_declaration`: every
selected row would hold one identity); an undeclared entity or
transition (`undeclared_reference`); a filter reading an undeclared field or input.

A set move counts as the cause of its transition for the lifecycle check. It is no driver of an
arrangement: a state reachable only through a set move cannot be arranged for another scenario.

## Conformance

The `instances:` branch gets its own `/outcome/` scenario. Synthesis chooses the input that reaches
the branch, writes its values into the filter, and arranges through the declared creations and
moves three rows the filter selects, and rows it leaves out: for a conjunction (`all:`), one per
conjunct, that conjunct false and every other true, so a target dropping any one conjunct changes a
row it must leave; for any other filter (`any:`, `not`, a single comparison), one row making the
whole filter false. Each is in a state the move starts from and has every `sets:` field holding a
value the effect would change. For a `moves:` one selected row rests outside the `from` states,
preferring a state other than the move's arrival. After the command the scenario requires the
outcome, each event with `{count: changed}` equal to three — not two, so a target reporting a
constant is caught — and reads every row back: the changed ones in the arrival state with what
`sets:` wrote, the others as arranged.

The same scenario then sends the command again, under a further witness input the filter selects
none of the rows by (the zero-match call): it must be accepted with the same outcome, `{count:
changed}` must be 0, and every row must read as the first call left it. Where no witness the search
offers leaves every row out, the zero-match call is left out and the rest of the scenario stands.

`affects:` appends a segment to the branch's own scenario: a subject under a fresh name, three rows
each entry's filter selects (with the subject's values written in, so a `subject.<field>` conjunct
is a conjunct like any other) and the rows it leaves out as above, the command, then the subject as
the branch leaves it and the entry's rows changed or as they were.

Rows are read from a row-level view publishing the entity's identity that is unfiltered,
unparameterised, unpaged and `read_your_writes`, and that publishes — at the entity's own types —
the state a set move leaves and every field the effect's `sets:` write. A set `updates:` writing no
field changes nothing a read can see. Where no such view exists the scenario is refused by name
(`NoWitness`) rather than filed: a row read back where the change cannot be seen is no observation,
and a scenario built on it would pass a target changing the wrong rows. Every step used — `capture_instance`,
`execute_command`, `expect_event`, `expect_view` `contains` — already exists, so a suite carrying a set
effect keeps the format its other steps select and no new suite major is taken; Go and TypeScript
runtimes need nothing new.

The count and the unchanged rows are claims about every stored row: they hold on a target no other
scenario writes to at the same time, which §8 already requires of a shared target.

## Targets

Entity Runtime refuses both with `SetEffectUnsupported`: an entity-core operation acts on the one
instance its request names. Rust, Go, Web and Clap synthesis refuse both by name as
`MissingRepresentation`. OpenAPI and AsyncAPI change nothing on the wire; the generated
documentation says which rows a branch changes. `ess-diff/9` adds `outcome-set-effect-changed`,
carrying one line per construct on each side.

## Out of scope

Atomicity and partial failure (an implementation stopping at the first error), `affects:` across a
relation by name, and the order in which rows change.
