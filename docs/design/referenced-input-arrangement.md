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
| `cardinality: many`, a `List` | a non-empty list of created rows: as many as the witness list held, one at least, three at most | the empty list, which names no row |
| `cardinality: one` | the created row's identity | no scenario: a refusal naming `<entity>.<relation>` |

A non-empty list is arranged where a route exists, because an empty list would pass without
exercising the reference. A plain reference has no empty value, so the refusal is the only result
the rule admits there.

Left as before: a refusal and a replay (they are sent the input that selects them), a branch whose
own `when:` guard reads the field (its witness list is what selects it), a field another
arrangement already named, a relation held to the entity itself, and an optional input the witness
sent absent. A command guarded by a related row (`when_related:`), by a list of references or by a
row set keeps its own arrangement of the rows its guard reads, including the creating half of a
create-or-update, which leaves that row unarranged on purpose; none of them is changed here.

## Rows and distinctions

Every run of a scenario arranges its own rows: the further runs of a branch (an Optional left out,
a union variant, an expression point) each take a block of distinctions of their own, past every
block the other arrangements number from, so no run reuses a row and no identity is created twice.

## Byte pins

Only a model that writes an input into a field carrying a `references` relation moves: its accepted
scenarios gain the creating steps ahead of the command. No example under `examples/` declares one.
