# Typed literals in `sets:` and an unknown instance's declared answer

Status: implemented (beyond10x/ess#113). No source format, suite format or diff format moves.

## 1. A literal may be written as the YAML scalar it means

### Before

A payload or `sets:` source was read as text or as a mapping. `items: 0` and `paused: false` were
refused by the reader (`invalid type: integer`) before any rule saw them, with no hint, while
`items: '0'` and `paused: 'false'` compiled to `{"kind":"literal","value":"0"}` and were typed
against the target by `literal_representation`.

### Rule

An unquoted YAML boolean, integer or decimal in a `payload:` or `sets:` source position is read as
a **typed scalar** (`PayloadSource::Scalar { value, scalar }`), where `value` is the canonical text
of the scalar and `scalar` is `boolean`, `integer` or `decimal` (`kind` is already the serde tag of
`PayloadSource`). It is checked against the target by the
same function the quoted literal is, and one rule decides both:

| written | target is `Boolean` | target is `Integer` | target is `Decimal` | target is text or an enum |
|---|---|---|---|---|
| `false` | admitted, as `'false'` | refused: `` `true` or `false` `` hint | refused, as `'false'` is | refused: **quote it** |
| `0` | refused, as `'0'` is | admitted, as `'0'` | refused, as `'0'` is | refused: **quote it** |
| `1.5` | refused, as `'1.5'` is | refused, as `'1.5'` is | refused, as `'1.5'` is | refused: **quote it** |

"Refused, as the quoted form is" means the quoted form's own refusal and hint. "Quote it" is a
`type_mismatch` whose hint spells the repair: `quote it: items: '0'`. An integer beyond `i64` is
refused as its quoted spelling is.

An admitted typed scalar compiles to `ResolvedPayloadValue::Literal { value }` with the canonical
text, which is exactly what the quoted form compiles to. The IR of every model that writes the
quoted form is unchanged, and so is every model that could not be written before.

### Decimal is read and never admitted

Admitting `1.5` over a `Decimal` would compile to something the quoted form does not: the quoted
form is refused today (`primitive_literal` claims `Boolean` and `Integer` only, because those are
the spellings `ess-conformance`'s setup reader sends). Admitting a decimal literal is a change to
that reader too, and is not part of this rule. The decimal is **read** so its refusal is a typed
diagnostic with a repair instead of a parser error.

### Positions

| position | literal form | treated |
|---|---|---|
| `sets:` | `RawPayloadSource` | as above |
| `payload:` | `RawPayloadSource` | as above; one reader for both |
| `when_subject: {field, equals}` | an enum variant name | unchanged: it names a variant, never a number or a boolean, and a typed scalar there would be refused by the enum check anyway |
| predicates (`when:`, `when_subject: {predicate}`, invariants, filters) | already typed (`quantity: {gte: 5}`) | unchanged |
| fixtures | no source-format position; authored scenarios and suites are JSON and already typed | unchanged |

## 2. An unknown instance is answered with the command's `wrong_state` outcome

### Before

A `moves:` or `updates:` outcome takes `instance: <field>` from input. Nothing said what a command
answers when that identity names no record. The suite never tested it, so implementations
differed: the reference `billing` and `oracle-fixture` targets answered `undeclared`.

### Rule (issue option 2, decided)

> A command whose input selects a `moves:` or `updates:` branch, and whose `instance:` names no
> record, is answered with the command's `wrong_state` outcome, when it declares one.

Input guards are decided first. A branch reached by input alone and acting on no instance (a
refusal such as `PayInvoice/rejected`) keeps its answer whatever the instance names. Only the
input that would reach a branch acting on an existing instance meets the rule.

A command without a `wrong_state` outcome has no declared answer. Synthesis records a **coverage
note** for it (`Synthesis::notes`, printed by `ess verify conform synthesize` as `note:`), not a
refusal: nothing the specification states is left unwitnessed, the specification simply states
nothing.

### Witness

One scenario per command that declares `wrong_state` and has a moving or updating branch reached by
input alone. It is keyed by the `wrong_state` outcome's own id, `<command>/outcome/<wrong-state>`,
which no scenario held before: the states that outcome answers in are covered by the illegal-move
family, one scenario per state, and this scenario covers the one case that family cannot arrange.

Steps: execute the command with the input that reaches the moving branch and a **fresh identity**,
then require the `wrong_state` branch, its declared error (where `refuses` holds), and that no
declared event was published. Nothing is arranged.

The fresh identity is the witness of the identity field at a dedicated distinction
(`Distinction::UNKNOWN`, `1 << 20`), which no arrangement reaches: arrangements count further
instances from 1 up to `MAX_CANDIDATES` (64). It is checked against the witness at every one of
those distinctions. Where it equals one — the type has too few values, as a `Boolean` or a
month-wide `Timestamp` witness does — no identity is known to be fresh, and synthesis records
`ESS-SYNTH-001` (`NoWitness`) against the id instead of asserting on an identity another scenario
may have created. Where no input reaches a branch acting on an existing instance, the refusal that
input search returned is recorded against the id.

### Formats

No format moves. The scenario uses the existing `Outcome` id and existing steps, so every reader of
every supported suite format parses and runs it (the Go and TypeScript runners' id grammar already
admits `/outcome/`). The rule is a new obligation on implementations, not a new document shape;
suite content is identified by `spec_digest`, as for every scenario added without a format change
before it. Committed suites and pinned counts move and are listed in the change.

### Reference targets and realizations

`billing` and `oracle-fixture` implement the rule: `IssueInvoice`, `PayInvoice`, `CancelInvoice`
and the oracle's moving commands answer their `wrong-state` branch for an unknown identity, after
the input guards they already decide first. The error carries no `state` field, because an
instance that does not exist is in none; the scenario requires the error by name and no field.

The generated Rust and Go behaviour seams spell that answer with a second variant of the
`wrong_state` outcome that carries none of the error's fields, generated exactly where the
declared variant demands one (`unknown-instance-seams.md`). `gatepass-realization`,
`gatepass-go-realization` and `billing-realization` answer the rule through it.
