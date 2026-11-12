# An accepted input names the rows a `references` relation points at

Status: implemented (story `accepted-input-arranges-referenced-list-rows`); the synthesis module
is `ess-conformance/src/synthesize/referenced_input.rs`. This page opens no format of its own. No
new keyword, no new `ValidationCode`, no IR shape change, no suite or `ess-diff` version: the
scenario uses existing steps. It sits beside `related-rows-quantifier.md`, which arranges the
rows a `when_related:` guard reads, and `generic-language-features.md` § G2, which declares the
`references` relation.

## The gap

An entity declares `relations: [{name: sources, kind: references, target: Source, cardinality:
many, via: sources}]`, its field `sources` is a `List` of the target's identity, and a creating
command writes it from its input: `sets: {sources: input.sources}`. Nothing in the model
guards the list. Synthesis built the accepted scenario from the witness search, which refutes
sibling guards only, so `sources` held a fresh identity that no step of the scenario created. A
target that resolves the references it stores refused it, and the scenario the suite calls accepted
failed. The same held for a plain field under `cardinality: one`.

A model that names a relation implies the reference names a row. The suite should not send a
dangling reference on the branch it calls accepted.

## The rule

On a run of a branch that accepts, an input copied unchanged into a field carrying a `references`
relation names rows the scenario arranged before the command, each created through the referenced
entity's creating route (`arrange_first`, as for related rows), and an unrouted list is sent
empty, but only where nothing in the model reads the field:

| relation | a route creates the target | no route creates it |
|---|---|---|
| `cardinality: many`, a `List` | exactly as many created rows as the witness list held, eight at most (a longer list is a refusal naming the relation) | the empty list, which names no row |
| `cardinality: one` | the created row's identity | no scenario: a refusal naming `<entity>.<relation>` |

The rule fails safe relative to the witness, and is an allow-list: an input field `x` is unread
only when both hold. Every use of `input.x` anywhere in the resolved command is exactly the claimed
branch's `sets: {f: input.x}` into the field `f` that carries the relation; every branch of the
command counts, and every part of it (guards in all forms, sets, payload, response, replays,
affects, instances, filters, emits, external eligibility), found by writing the resolved command
out and reading every name in it, so no reader is listed by name. And nothing reads the stored
`f`: no invariant of the entity, and no branch of any command acting on the entity (a guard, a copy
of `f` into another field or an event, an expression). Anything else counts as read, and so does
anything in doubt. A read field is sent the witness value unchanged, as before this unit: the
scenario is neither arranged nor refused for it, and the decisions the witness already made hold. The
witness already satisfies every guard and invariant over the input, so no length is chosen again.
A plain reference has no empty value, so an unrouted `one` is refused by name, read or not,
where it is arranged.

A referenced row rests in the state a creating branch leaves it in (`into:`, else the lifecycle's
initial state), so a route is no less a route for creating straight into a later state.

A referenced row whose creating branch writes a reference of its own from its input is created
naming a row arranged first, under the same allow-list, applied to the creating command's own input
field and the stored field it writes: a field the creating command uses elsewhere, or the created
entity reads, keeps its witness value. The row a command names and three rows
chained below it make four rows in all. The fourth is created plain, so its own references stay
what the witness chose, as do those of a creating branch another family arranges (a related guard,
a held-state guard, an owner). Where no route creates a chained row's own `many` list it is sent
empty; an unrouted `one` there gives up the chain and refuses the outer scenario by name.

Left as before: a refusal and a replay (they are sent the input that selects them), a field
something reads, a field another arrangement already named, a relation held to the entity itself,
and an optional input the witness sent absent. A command guarded by a related row
(`when_related:`), by a list of references or by a row set keeps its own arrangement of the rows
its guard reads, including the creating half of a create-or-update, which leaves that row
unarranged on purpose; none of them is changed here.

## Rows and distinctions

Every run of a scenario arranges its own rows: the further runs of a branch (an Optional left out,
a union variant, an expression point) each take a block of distinctions of their own, past every
block the other arrangements number from, so no run reuses a row and no identity is created twice.

## Not covered

A row arranged for another command's scenario (a move, an update, a view read) through the
creating command still sends the witness input, and so does a command a related guard, a list of
references or a row set arranges, and so does a reference field a guard or an invariant reads:
its scenario sends the witness identity, which a target resolving references does not accept. All
are recorded as backlog; their cases stay in `tests/referenced_list_input.rs`, ignored by slug.

## Byte pins

Only a model that writes an input into a field carrying a `references` relation moves: its accepted
scenarios gain the creating steps ahead of the command. No example under `examples/` declares one.
