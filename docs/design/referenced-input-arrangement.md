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
entity's creating route (`arrange_first`, as for related rows):

| relation | a route creates the target | no route creates it |
|---|---|---|
| `cardinality: many`, a `List` | exactly as many created rows as the witness list held, eight at most (a longer list is a refusal naming the relation) | the empty list, which names no row, where it still selects the branch and keeps the entity's invariants; else a refusal naming `<entity>.<relation>` |
| `cardinality: one` | the created row's identity | no scenario: a refusal naming `<entity>.<relation>` |

The witness already satisfies every guard and invariant over the input, so the arrangement never
chooses a list's length again: a list the witness held empty stays empty, and a list of four stays
four. Where the witness sends a non-empty list the reference is exercised; a model whose guards
admit only the empty list is not forced to send more. A plain reference has no empty value, so
the refusal is the only result the rule admits there.

A referenced row rests in the state a creating branch leaves it in (`into:`, else the lifecycle's
initial state), so a route is no less a route for creating straight into a later state.

A referenced row whose creating branch writes a reference of its own from its input is created
naming a row arranged first: the row a command names and three rows chained below it, four rows
in all. The fourth is created plain, so its own references stay what the witness chose, as do those
of a creating branch another family arranges (a related guard, a held-state guard, an owner).

Left as before: a refusal and a replay (they are sent the input that selects them), a branch whose
own `when:` guard reads the field itself (its witness list is what selects it; a guard reading
only the list's `count` is not left alone, as the arrangement keeps the count), a field another
arrangement already named, a relation held to the entity itself, and an optional input the witness
sent absent. A command guarded by a related row (`when_related:`), by a list of references or by a
row set keeps its own arrangement of the rows its guard reads, including the creating half of a
create-or-update, which leaves that row unarranged on purpose; none of them is changed here.

## Rows and distinctions

Every run of a scenario arranges its own rows: the further runs of a branch (an Optional left out,
a union variant, an expression point) each take a block of distinctions of their own, past every
block the other arrangements number from, so no run reuses a row and no identity is created twice.

## Not covered

A row arranged for another command's scenario (a move, an update, a view read) through the
creating command still sends the witness input, and so does a command a related guard, a list of
references or a row set arranges. Both are recorded as backlog; their cases stay in
`tests/referenced_list_input.rs`, ignored by slug.

## Byte pins

Only a model that writes an input into a field carrying a `references` relation moves: its accepted
scenarios gain the creating steps ahead of the command. No example under `examples/` declares one.
