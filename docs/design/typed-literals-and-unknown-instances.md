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
| `false` | admitted, as `'false'` | refused: `` `true` or `false` `` hint | refused, as `'false'` is | refused: **quote it** where `'false'` is admitted, or where an enum declares the variant in another case (`False`), named in the hint; otherwise as `'false'` is |
| `0` | refused, as `'0'` is | admitted, as `'0'` | refused, as `'0'` is | refused: **quote it** where `'0'` is admitted; otherwise as `'0'` is |
| `1.5` | refused, as `'1.5'` is | refused, as `'1.5'` is | refused, as `'1.5'` is | refused: **quote it** where `'1.5'` is admitted; otherwise as `'1.5'` is |

"Refused, as the quoted form is" means the quoted form's own refusal and hint. "Quote it" is a
`type_mismatch` whose hint spells the repair: `quote it: items: '0'`. Over an enum the quoted form
is often refused too, because YAML reads `True` as the boolean `true` and `'true'` is not a variant
of `[True, False, Unknown]`; where a declared variant equals the boolean ignoring case, the hint
names that variant quoted, `quote it: tests_pass: 'True'`, and the literal is still refused rather
than read as the variant (beyond10x/ess#426). Otherwise the quoted form's own refusal lists the
variants. An integer beyond `i64` is refused as its quoted spelling is.

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
| `mapping:` (a binding) | `AuthoredMappingSource` (`MappingSource::Scalar`) | as above, by the same function (beyond10x/ess#445): `'true'` and `true` fill a `Boolean` input, `3` over text is refused with `quote it`; an admitted constant compiles to `ResolvedMappingValue::Literal`, read against the target type by every target. A target no literal spells keeps the binding's own refusal, which names a field of the event as the source instead of an input |
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

## 2a. A declared not-found outcome comes first (0.35.1)

### Before

Section 2 made `wrong_state` the answer for an unknown instance even where the command declares
its own answer for one. A command such as `Close` may declare `not-found`
(`external: no door carries input.door_id`, `error: DoorNotFound { door_id }`) beside
`wrong-state`. Synthesis still sent a fresh identity and required `wrong-state` /
`DoorStateConflict`, so an implementation answering its own declared outcome failed, and no
declaration could change that without dropping coverage. A door that does not exist is not in the
wrong state.

### Rule

> A command whose input selects a `moves:` or `updates:` branch, and whose `instance:` names no
> record, is answered with the command's **not-found outcome** when it declares exactly one;
> otherwise with its `wrong_state` outcome, when it declares one.

A not-found outcome is read from the declaration, never from a name: an externally decided refusal
(`external:` without an input guard, acting on no instance, replaying nothing) whose declared
error carries a field of the type of an identity the command's acting branches read from input.
The error reports the identity it could not find. A sibling external refusal whose error reports
something else (`Jammed { force }`) is about another input and does not qualify.

Two or more qualifying outcomes leave the answer undeclared: synthesis records
`Note::UnknownInstanceAmbiguous` naming them and assumes neither. Their injected scenarios stay.

### Witness

The same scenario as section 2 — the command, the input reaching the acting branch, a fresh
identity, nothing arranged — requiring the not-found outcome, its error by name, and no declared
event. It is filed under the not-found outcome's own id and **replaces** the scenario that injected
its external cause with `ConfigureExternalOutcome`: sending an identity no record carries is that
cause, witnessed rather than forced. The `wrong_state` id then holds no scenario, as before
section 2; its states remain the illegal-move family's. Where no identity is known to be fresh, the
injected scenario stays and nothing is refused.

### Formats

No format moves, for the reason section 2 gives. A suite for a command declaring both outcomes
loses the `…/outcome/wrong-state` scenario section 2 added and its not-found scenario changes
content under the same id.

## 2b. A read-only command declares its unknown-instance answer (beyond10x/ess#535)

### Before

`unknown_instance:` (ess/15, `outcome-shapes.md`) was admitted only on a command with a `moves:`,
`updates:` or `deletes:` branch reading `instance:` from input. A command whose branches only
preserve the row its input names, or refuse it through `when_subject:`, reads that row just the
same, and its implementation answers an identity no row carries with its own error. Validation
refused the marker there with `unreachable_branch`, and the only form left was an `external:`
refusal that nothing tied to a missing row.

```yaml
- name: demo.desk.Ring
  input: [{name: user_id, type: String}]
  outcomes:
    - {name: no-seat, unknown_instance: true, error: demo.desk.NoSeat}
    - {name: closed, when_subject: {predicate: state == Closed}, error: demo.desk.NoSeat}
    - {name: rung, preserves: demo.desk.Seat, instance: user_id}
```

### Rule

> An `unknown_instance:` branch is reachable on a command with a branch that addresses an existing
> instance from input: a `moves:`, `updates:`, `deletes:` or `preserves:` branch reading
> `instance:` from input.

A `when_subject:` refusal reads the subject a sibling branch addresses, so a command with one has
that sibling, and the sibling decides. A command with no input-addressed branch at all keeps
`unreachable_branch`.

The create-or-update pairing (`outcome-shapes.md`, ess/16) is unchanged below `ess/27`: a creating
`unknown_instance:` still needs a sibling `moves:` or `updates:` on the same entity, so one beside
a `preserves:` branch alone is refused `conflicting_declaration`, the hint naming `ess/27`.

From `ess/27` (the one precedence rule, `generic-language-features.md` § G4) the creation pairs
with a `preserves:` branch too — "ensure the record exists": the creation answers at step 2 for an
identity no record carries, the preserving branch and any `when_subject:` refusal beside it every
stored one. A `deletes:` sibling stays refused in every format: a create-or-delete toggle is not
the construct. Wherever `when_subject_state:` admits an `unknown_instance:` branch, refusing or
creating, `when_subject:` admits it as well; #530's create-or-update is `ess/25`'s special case.
Entity Runtime keeps refusing a creation selected by existence by name
(`ExistenceSelectionUnsupported`).

### Witness

The section 2 scenario, keyed by the declared branch's own id: the command, the input reaching the
preserving branch, a fresh identity, nothing arranged, the declared branch, its error where it
refuses, and no declared event. A `preserves:` branch is tried for that input only after every
moving, updating and deleting branch, and only on a command declaring `unknown_instance:`.

A preserving command that declares no `unknown_instance:` branch takes no not-found or
`wrong_state:` answer from its preserving branch, and its scenarios are unchanged. Declaring the
branch adds its one scenario and changes no other.

The interpreted target and the generated Rust and Go behaviour already answered a missing row on a
preserving branch with the command's declared unknown-instance answer; they pass the scenario
unchanged.

### Formats

No format moves: the key existed, the scenario uses the existing `Outcome` id and steps, and a
document this change admits was refused before.
