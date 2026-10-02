<!--
  generated from gatepass v1
  model digest 7d021b6ebe1c4715096f165d6564389be0f46311f67d791ed748f627314d611c
  contract digest 2668f3034afb388a33d7add462e15a830b6010fbfe83101f1dd2526fa18d52ed
  do not edit: regenerate with `ess synthesize --target go`
-->
# Target notes — go

For gatepass v1. The `PLAN.md` beside this file is language-neutral and **byte-identical in every target's tree**; this document is what *this* target could not carry across it. Regenerate with `ess synthesize --target go`.

7 weakening(s), 0 target refusal(s). A weakening is emitted code that holds less than the first target's; a target refusal is a capability the plan marks generated and this language cannot represent — a fact about the language, never about the specification.

## Weakened — emitted, with less than the first target holds

| the guarantee | what this target provides | capabilities affected |
| --- | --- | --- |
| handling a closed set of variants is exhaustive: a `match` that forgets one does not compile | the set stays closed — an undeclared variant cannot implement the sealed interface's unexported marker from another package — but a `switch` over it is not checked, so a consumer that forgets a variant compiles and falls through. Go has no exhaustiveness check and none can be emitted; every generated sealed interface says so in its own doc comment | domain type, entity lifecycle, command contract, binding delivery, component port, component transport |
| a value of a generated type exists only where a generated constructor or transition produced one | Go gives every type a zero value that no constructor has to produce, so `Email{}`, an invoice resting in a state nothing moved it to, and a nil variant of a sealed interface are all spellable from any package. The unexported field stops a *populated* value being forged; nothing in the language stops the empty one existing | domain type, entity lifecycle, command contract, event type, error type, view type |
| refining a runtime state into the typed lifecycle is total: every declared state has an arm and no other state can reach it | the snapshot's state field is a sealed interface, whose zero value is nil and names no declared state — the previous row's weakening, reaching this one. Refinement therefore answers `(value, ok)`, and a caller that ignores the second result gets the interface's own zero value | entity lifecycle |
| every generated type compares by value | Go defines `==` only for comparable types, so a generated type carrying a list, a map or bytes cannot be compared at all — and no deep comparison is emitted in its place, because a hand-written equality is behaviour, and behaviour is not synthesised | domain type, entity lifecycle, command contract, event type, error type, view type |
| a JSON object leaves this system with its members in the order the specification declares them | the served bodies are built as `map[string]any` and written by `encoding/json`, which sorts a map's keys — so a body's members come out alphabetical here and in declaration order in the first target. The two are the same *value*, no published contract states an order, and every consumer that parses rather than greps is unaffected; what is lost is the ability to compare two applications' bodies byte for byte, which is why the gate compares them as values. Emitting a writer that kept the order would mean emitting a second JSON writer beside the standard library's | component transport |
| a command behaviour the specification fully determines is generated, over storage and context ports | this target does not generate command behaviour yet: each one the plan marks generated keeps its behaviour seam here, owed, with the same contract and a stub refusing it, exactly as an obligation | command behaviour |
| a view query the specification fully determines is generated, over a storage port that lists an entity's rows | this target does not generate view queries yet: each one the plan marks generated keeps its query seam here, owed, with the same contract and a stub refusing it, exactly as an obligation | view query |

## Refused by this target — planned, not emitted

| capability | source | why |
| --- | --- | --- |
