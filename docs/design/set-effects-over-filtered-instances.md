# Set effects over filtered instances (`instances:`, `affects:`, ess/16)

Status: implemented, first cut (beyond10x/ess#167, #175; `story:set-effects-over-filtered-instances`).
Both constructs are admitted under source format `ess/16`. Below it `instances:`, `affects:` and a
`{count: changed}` beside `instances:` are refused with `unsupported_format_version` at the key
written, before the outcome is converted, so no shape rule of the construct and no cascade
(`missing_declaration`, `empty_declaration`) is reported beside it; a `{count: changed}` on any other
outcome stays the nested mapping it was. From `ess/23` both also delete the rows they select
(beyond10x/ess#452, below), and an `affects:` entry may write one row per element of an input list
(beyond10x/ess#459, below).

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

**Selection.** Only a filter that holds selects a row, under `instances:` and `affects:` alike (one
selection in the interpreter). A filter left unknown because a field it reads is absent — an
`Optional<…>` field of the row or of the subject holding nothing, or an Optional input left out —
does not hold, so the row is not selected and the outcome still answers. A filter left unknown by a
required field never written, or by a value no history recorded, stays undecidable (beyond10x/ess#229,
adversary pass 1; this reaches `ess/16` documents too, which were answered as undecidable before).

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
the caller and are refused by name under a set effect. A `moves:` inside `affects:` is admitted from
`ess/22` (below).

Refused besides: `instance:` with `instances:` (`conflicting_declaration`); `instances:` on
`creates:` (`conflicting_declaration`) or on `preserves:` (`unsupported_construct`), and on
`deletes:` below `ess/23` (below); `instances:` with no verb (`missing_declaration`); `instances:` on
a refusal (`refusal_mutated_state`) or under a condition other than `when:` or the default
(`unsupported_construct`); `affects:` beside `instances:` or on a creation or preservation
(`unsupported_construct`), on a deletion below `ess/23`, or with no subject (`missing_declaration`); a `sets:` entry writing the
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

## Related-record moves (`ess/22`, beyond10x/ess#229)

An `affects:` entry may declare `moves: <Entity>.<transition>` (`story:related-record-effects`):
deactivating a user ends that user's live sessions. The move names a transition of the entry's own
`entity` (another entity's is `conflicting_declaration`, an undeclared one `undeclared_reference`),
and `sets:` beside it is optional. Its semantics are those of an `instances:` move: rows are selected
from the store before the outcome; a selected row resting in the transition's `from` states takes it
and comes to hold what `sets:` writes; one resting elsewhere is skipped, not refused; zero rows is an
accepted answer. The move counts as the cause of its transition for the lifecycle check, and is no
driver of an arrangement. The compiled entry carries the transition as `moves`, left out of the
document where the entry only sets fields, so a model without the form keeps its bytes.

Under `ess/16` to `ess/21` the move is refused at `affects[<n>].moves`, before conversion, with
the code those formats always gave it (`unsupported_construct`) and a message naming `ess/22`. From
`ess/22` a move naming another entity's transition (`conflicting_declaration`) or no entity
(`missing_declaration`) is refused at the same key in the same pass. Either way the move is taken off
the entry and the rest kept, so the branch converts and the refusal comes alone: no
`empty_declaration`, `non_exhaustive_branches`, `unreachable_branch` or uncaused-transition cascade.

An outcome takes one transition per entity in `affects:`: a second entry declaring `moves:` over an
entity an earlier entry already moves is `conflicting_declaration` at its `moves`, since a row both
select would have to take two and no order between them is defined. A moving and a setting entry, or
any number of setting entries, over one entity are admitted, and apply to a row in the order written.

The `affects:` segment witnesses a moving entry as the `instances:` scenario witnesses a move: its
changed rows rest in a `from` state other than the arrival wherever the arranging commands reach
one (a row already resting where the move arrives reads the same whether or not it was taken), one
row the filter selects rests outside them (another state than the arrival preferred), and after the
command the changed rows are read in the arrival state, the others as they were. The view that reads
them back must publish the state. Where no row can show its `sets:` changing, the state alone
separates them. Where only the arrival state itself is reached, the move is seen through what
`sets:` writes, and an entry writing nothing is refused by name (`NoWitness`). No witness row of
`instances:` or `affects:` is arranged through an act taking a branch with a set effect
(`instances:` or `affects:`), whose effect would reach rows the arrangement does not account for
before the command is witnessed; another path is taken, or the optional row skipped outside the move
is left out as where no arrangement reaches it. Sending the command under test is otherwise allowed:
a branch of it that changes only the row it names (a creation under another caller, beyond10x/ess#287)
is folded into that row like any other act.

Every row the segment arranges is read back as every entry, in the order written, leaves it: an
entry over the row's entity whose filter selects the row as arranged writes its `sets:` and, where
it moves and the row rests in its `from` states, takes its move. So one row two entries select gets
one expectation, the one the interpreter answers; for an entry no other entry touches the reads are
the ones written before. Where an entry cannot tell whether it selects a row another entry arranged,
the scenario is refused by name. `ess-diff/9`'s
`outcome-set-effect-changed` line for the entry names the move; the generated documentation says
which rows move where. A transition only such an entry takes is performed by its branch in the
mutation audit's component scoping.

Entity Runtime refuses the moving form by name as it refuses every `affects:` (below). The Rust,
Go, Web and Clap targets emit the command as an owed behaviour whose contract names the rows it
moves (below, "Targets"), so the effect stays an obligation the conformance suite checks.

## Deleting the selected rows (ess/23, beyond10x/ess#452)

Two refusals of the first cut are lifted from `ess/23`; no keyword is added.

**Bulk removal.** `deletes:` with `instances:` removes every stored row of the entity the filter
selects, the same selection `moves:` and `updates:` make: "revoke every token of this user". Zero
selected rows is an accepted answer. `{count: changed}` in `payload:` is the number of rows removed.
`sets:` beside it is refused (`conflicting_declaration`), as on a single `deletes:`: a removed row
holds nothing to set.

**Removal beside a subject.** An `affects:` entry may declare `deletes: <Entity>`, spelled like the
entry's `moves: <Entity>.<transition>`: every row of the entry's entity its filter selects is
removed, the subject itself excepted where the entity is the subject's. It names the entry's own
`entity`; another entity is `conflicting_declaration` at the entry, naming both. `sets:` beside it,
or `moves:` in the same entry, is `conflicting_declaration`. A deleting entry beside any other entry
over the same entity is `conflicting_declaration` at the second of them, as two moving entries are;
over different entities the pair is admitted, and an entry whose `deletes:` was refused and taken
off changes nothing and is beside nothing. A deleting entry over an entity another domain owns than
the command's is `unsupported_construct` at the entry, naming the binding idiom (below). And
`affects:` is admitted beside a `deletes:` subject, which exists before the outcome, so
`subject.<field>` reads it as it was: "delete the user and its tokens".

Below `ess/23` each keeps the refusal it had, naming `ess/23`: `instances:` on `deletes:` and
`affects:` beside `deletes:` are `unsupported_construct`, and the entry's `deletes:` key is
`unsupported_format_version`. The refused key is taken off before the outcome is converted, so the
refusal comes alone, with no `empty_declaration` ("declares no outcomes") beside it; an entry keeps
its place, so every later entry's refusal names the position written. A bulk deletion that the
conversion refuses first under every format — beside `instance:`, or with a filter that does not
parse — keeps that refusal. Below `ess/16` `instances:` is refused with its `deletes:` taken off.
A refusal of another verb names `deletes:` as admitted only in an `ess/23` source.

The rows are witnessed as other set effects are (above): three rows the filter selects, one per
conjunct left out, the command, then each removed row read absent from every immediate view of its
entity with `deletes:`'s absence check (`expect_subject_absent`, `ess-conformance/22`) and each
other row read as arranged; the bulk form then sends the command again under an input selecting
nothing, with a count of 0 and every row as the first call left it. No suite step is new. The
interpreter removes the rows. Entity Runtime refuses both with `SetEffectUnsupported`; Rust, Go,
Web and Clap owe the command, as for every set effect (below, "Targets"). `ess-diff`'s
`outcome-set-effect-changed` line names the deletion, and the generated documentation says which
rows are removed.

**Removal in other domains** is not an `affects:` entry, and a deleting entry over another
domain's entity is refused: a domain boundary is where bindings belong. The deleting command emits one event, and the cascade is one binding per receiving domain
on that event, each with its own `delivery:` and `on_failure:`; each receiving command removes its
own rows with the bulk form. ESS states no order between bindings of one event, and atomicity
across them stays out of scope (below).

## One row per element of an input list (ess/23, beyond10x/ess#459)

An `affects:` entry may write one row per element of a list the command's input carries, rather
than select stored rows: a scheduled fetch sees N documents and records each as its own row, keyed
by the document.

```yaml
affects:
  - entity: demo.feed.SeenDocument
    each: {in: input.applied, as: doc}
    instance: doc.document_id
    sets: {source_id: input.source_id, content_hash: doc.content_hash, revision: doc.revision}
```

**Semantics.** `each: {in: input.<list>, as: <name>}` walks a `List` of a struct in the command's
input, in order. For each element, `instance: <name>.<member>` reads an identity of the entry's
entity, and the row it names is updated if held and created in `initial` if not: the existence
pair's create-or-update (beyond10x/ess#164), applied per element. An update changes only the fields
the entry writes; every other field of the held row is carried. `sets:` writes each such row. A
source there may read `<name>.<member>`, a member of the element at the field's own type or the
`Optional` of it, beside the sources every `affects:` entry takes; `{subject: …}`, `{related: …}`,
`{increment: …}` and `{caller: …}` stay refused as on every set effect, and the subject's identity
is read from the input that names it. An element is read one member deep and not inside a nested
mapping. Where the entry is over the subject's own entity, an element naming the subject is
skipped: the subject itself is excepted, as under every `affects:` entry, and holds what its own
branch writes. An empty list writes nothing and is an accepted answer. A row the list does not name
is not removed: replace-a-set, removing the rows a list no longer names, is not in this cut, and
composes later with the deleting entry above.

**Duplicate identities.** Two elements naming one identity would leave a row that depends on their
order. The entry is admitted only where a declared `distinct:` holds that member distinct across
the list: `{distinct: {in: <list>, as: x, by: x.<member>}}` conjoined in the branch's own `when:`,
or negated in the plain `when:` of a refusal of the command (`when: {not: {distinct: …}}`). A
refusal also guarded by the stored subject (`when_subject:`) or a related row (`when_related:`)
refuses a repeated member only where that guard holds too, so it does not count. Without one,
validate refuses with `missing_declaration` naming the list and the member.

**Refusals.** `each:` beside `where:`, `moves:` or `deletes:` in one entry is
`conflicting_declaration`: the entry creates or updates the rows its elements name and selects,
moves or removes no other. So are `instance:` without `each:` and an element read writing the
identity, which comes from `instance:` alone; `each:` without `instance:`, or an entry with neither
`where:` nor `each:`, is `missing_declaration`. `in:` other than `input.<field>` is
`unsupported_construct`, an undeclared input or member `undeclared_reference`, and an input not
holding a `List` of a struct, an `instance:` member not of the entity's identity type, or an
element read not of its field's type `type_mismatch`.
`affects:` sits beside one subject (`moves:` or `updates:` with `instance:`), so a command with no
subject of its own is refused with `missing_declaration` naming the `each:` entry; admitting
`each:` as an outcome's own set subject is a labelled follow-up, not this cut. A required field an
entity invariant reads must be written by the entry, as by any creation (`ESS-COMMAND-018`). Below
`ess/23` `each:` is refused with `unsupported_format_version` naming `ess/23`, at the key and before
conversion; the entry keeps its place and changes nothing, so the refusal comes alone. An entry
with `instance:` and no `each:`, or with no `where:`, is refused before conversion too, alone, and
below `ess/23` the refusal names `ess/23` and says to declare it.

**Atomicity.** A refused command changes nothing, the rule every outcome keeps, so a refused run
writes none of the rows. Partial failure inside an implementation stays out of scope, as for every
set effect (below).

**Conformance.** The entry appends to the branch's own scenario. A second subject is arranged and
the command is sent for it with two elements, the row to be held and a decoy, so the rows exist
without any other command creating the entity: the existence pair arranges its updating sibling
through the same command. Where another command updates a row of the entity by a supplied identity
and writes a field the entry does not write, it is sent for the held row, so the row holds a value
an update carries and a replacement loses. The command is then sent for a third subject with an
empty list, requiring the accepting branch, where its guard admits one. The command under test is
then sent with two elements, one naming the held row with other values and one naming an identity
no row holds; the held row's two elements are built at adjacent witness distinctions, which differ
for every scalar kind, a `Boolean` and an enum of an even number of variants included. Afterwards
every view publishing the entity's identity and every field the entry writes, immediate,
unfiltered, unparameterised and unpaged, must read the held row with the element's values and the
carried field, the new row with its element's values, both in `initial` where the view publishes
the state, and the decoy as the first call left it; and a snapshot by each element's identity
selects exactly one row, the existence pair's one-row-per-identity check, so a target adding a
second row for a held identity fails. A target that only creates, only updates, replaces a held
row, or refuses an empty list fails the scenario. Each refusal scenario of the command then reads
the entry's entity absent, by `deletes:`'s absence check, for every identity the refused call's
list names and no earlier accepted call sent, so a target that writes the rows and then refuses
fails it. Where no view publishes those, the scenario is refused by name (`NoWitness`), as is an
entry over an entity another entry of the outcome also writes, and a branch with more than one
`each:` entry. Every step used (`execute_command`, `expect_outcome`, `query_view`, `expect_view`
`contains`, `snapshot_subject`, `expect_subject_absent`) already exists, so no conformance major and
no new suite step is taken: a suite carrying the entry keeps the format its other steps select.

**Targets.** The interpreter writes the rows after the subject, element by element in list order,
skipping an element that names the subject.
Entity Runtime refuses the entry with `SetEffectUnsupported`; Rust, Go, Web and Clap synthesis owe
the command, as for every `affects:` (below, "Targets"). `ess-diff`'s `outcome-set-effect-changed` line
names the entry and its list, and the generated documentation says it writes one row per element of
the list.

## One event per changed row (ess/25, beyond10x/ess#531)

Logging a team out moves every selected seat to `Closed`; for each seat that was `Busy` the service
also publishes `Left {user_id, call_ref}` for that seat. An entry of `emits:` may be published once
per row the outcome changed, with a payload read from that row:

```yaml
- name: closed-all
  moves: demo.desk.Seat.close
  instances: {where: team == input.team}
  emits:
    - demo.desk.TeamClosed
    - {event: demo.desk.Left, each: changed, when_state: Busy}
  payload:
    demo.desk.TeamClosed: {team: input.team, closed: {count: changed}}
    demo.desk.Left: {user_id: {changed: user_id}, call_ref: {changed: call_ref}, was: {changed: state}}
```

**The key.** An `emits:` entry is the event's name, as before, or the mapping `{event: <name>, each:
changed}`. `each:` takes the one word `changed`. The event is still the outcome's: it is named in
the command's events, a binding on it is caused by the command, and its `payload:` block is keyed by
its name like any other.

**The state filter.** `when_state:` names one state of the set subject's entity, or a list of them,
spelled as `when_subject_state:` spells them. The event is published only for a changed row that
held one of those states **before** the outcome. Without the key every changed row publishes it. A
state the entity does not declare is `undeclared_reference`; under a `moves:` a state outside the
transition's `from` is `conflicting_declaration`, since a row resting there is skipped and never
changed; an empty list or a state named twice is `conflicting_declaration`.

**Which rows.** The rows the outcome changed, the rows `{count: changed}` counts: under a `moves:`
the selected rows resting in the transition's `from` states, under an `updates:` every selected row,
under a `deletes:` every removed row. `{count: changed}` keeps counting all of them, whatever a
state filter leaves out.

**The value source.** In the payload block of such an event, `{changed: <field>}` reads a field of
the row the occurrence is for — its identity or any declared field — and `{changed: state}` the
lifecycle state it held, both **as the row was immediately before the outcome**, the reading
`{subject: …}` and `{subject: state}` make of one subject; a removed row is read as it was before
removal. The field's type must be the event field's, or `Optional` into `Optional`, as for
`{subject: …}` (`type_mismatch` otherwise); `{changed: state}` is the entity's `State` type. Every
other source an event payload takes is admitted beside it and is the same in every occurrence.
`{changed: …}` is admitted only as a whole payload field of an event the outcome publishes per
changed row: on any other event, in `sets:`, in an error's payload or inside a nested mapping it is
`unsupported_construct`, and a field the entity does not hold is `undeclared_reference`.

**Order.** The outcome's events are published in the order `emits:` lists them; a per-row entry
contributes, at its place, one occurrence per row, in ascending order of the rows' identities (the
order in which rows change stays out of scope, so the identity is the one order every target can
state). A conformance suite does not assert that order: the claims of one event after one command
are met as a set from suite `/44` (`event_multiplicity`), so a target publishing the same
occurrences in another order passes.

**Refusals.** Below `ess/25` the mapping entry is `unsupported_format_version` at
`outcomes.<name>.emits[<n>]`, naming `ess/25`, and `{changed: …}` reads as the nested mapping it
always was. The entry is admitted only on an outcome with `instances:`: on any other outcome — one
subject, `affects:`, or none — it is `unsupported_construct` at the entry. Either refusal is made
before the outcome is converted, the entry read as the plain event it names and the `{changed: …}`
fields of its payload block read as `{generated: true}`, as a refused `{count: changed}` is, so the
refusal comes alone. An event listed per changed row may be listed once in its outcome's `emits:`
(`conflicting_declaration` at the second entry): its one payload block cannot say which listing a
field is for. A per-row event over an `affects:` entry's rows is not in this cut.

**Format consequences.** A new key shape under `emits:` and a new value source, both in source
format `ess/25`. The compiled set subject carries the per-row events as `events` (each the event
and, where written, `when_state`), and the payload values `changed_field` and `changed_state`; a
model using neither keeps its IR bytes and digest. No conformance step and no suite major is new:
a suite claiming one event more than once after one command is `/44` (or `/45`) by the existing
rule.

**Conformance.** The `instances:` scenario arranges its three changed rows so that, where the
arranging commands reach them, two rest in a state the filter admits and one in a state the outcome
changes from and the filter leaves out. With several filtered per-row events each row rests where it
meets the most of those wants still open across them, so two filters admitting no state in common
are each witnessed on a changed row they publish for and one they do not. After the command it claims
each per-row event once for each changed row its filter admits, with every value the scenario
determines: the row's identity as the instance captured for it, its fields as arranged, its state
before the move. Each arranged row is a further instance, so its fields carry their own values as far
as their type and the filter allow. An `Optional` field the arrangement left absent (cleared) is
claimed absent, left out or `null`. A target omitting one of those rows fails a claim
(`ESS-CF-EVENT`). A target reading another row's values, or a value the outcome wrote, fails
`ESS-CF-PAYLOAD`. The zero-match call claims the event not at all (`expect_no_event`). Then, where
the search finds an input selecting none of the rows written so far, the scenario arranges, under
it, the rows the filter selects and no event is due for — one resting in a state the outcome changes
from and every state filter leaves out, and under a `moves:` one resting outside the `from` states —
sends the command, requires `{count: changed}` equal to the rows it changed and `expect_no_event`
for the per-row event, and reads the rows back. Each filtered event that segment did not witness on
a row it leaves out gets one more segment of its own, under a further such input: one changed row
its filter leaves out, the other per-row events claimed for it where theirs publish, and
`expect_no_event` for this one. A target publishing an event for a row its filter leaves out fails
there. Where no such row is reached the segment is left out, as the zero-match call is. Where no
changed row can rest in a state a filter admits, the scenario is refused by name (`NoWitness`).

**Targets.** The interpreter publishes the events. Entity Runtime refuses the outcome with
`SetEffectUnsupported`, and Rust, Go, Web and Clap synthesis as `MissingRepresentation`, as for
every set effect. The generated explorers leave the outcome out by name: their reference model
expects each listed event once. `ess-diff`'s `outcome-set-effect-changed` line for `instances:`
names each per-row event and its state filter.

## Targets

Entity Runtime refuses both with `SetEffectUnsupported`: an entity-core operation acts on the one
instance its request names. Rust, Go, Web and Clap synthesis emit the workspace and keep the command
an owed behaviour, its obligation naming `instances:` or `affects:` and its contract naming the rows
each set effect changes, its move or removal and the fields it writes. Until beyond10x/ess#547 they
refused both by name as `MissingRepresentation`; that refusal predated owed behaviour, and the
generated seam already takes only the input, so nothing in it confines the behaviour to one row
(`docs/design/set-effect-identity-filters.md`, D4). OpenAPI and AsyncAPI change nothing on the wire; the generated
documentation says which rows a branch changes. `ess-diff/9` adds `outcome-set-effect-changed`,
carrying one line per construct on each side.

The generated explorers' reference model (Go `Explore`, TypeScript `explore`) applies a `moves:` or
`updates:` under `instances:` as above: the selection read before any row changes, a selected row
outside the `from` states skipped, `sets:` on every changed row (beyond10x/ess#538,
[the model runner](mutation-audit-and-model-runner.md), "Selected rows"). A bulk `deletes:` is left
out of exploration by name, and so is an outcome carrying an `affects:` entry, which the model does
not apply; modelling `affects:` is later work.

## Out of scope

Atomicity and partial failure (an implementation stopping at the first error), `affects:` across a
relation by name, the order in which rows change, and cross-domain cascade (one binding per
receiving domain answers it, beyond10x/ess#452).
