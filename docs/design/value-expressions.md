# Value expressions in `sets:`, `payload:` and subject guards

Status: E1–E5 implemented in source format `ess/14` (beyond10x/ess#133, #134, #135, #136, #137).
E1 (#135) needs no format version; E2–E5 are refused below `ess/14` with
`unsupported_format_version`. E6 (#157) and E7 (#140) are implemented in source format `ess/15` and
refused below it with `unsupported_format_version`; E7 carries suite formats `ess-conformance/20`
and `/21`. E4's literal fallback (#163) is implemented in source format `ess/16` and refused below
it with `unsupported_format_version`. E5's per-leaf comparison (#179) carries suite formats
`ess-conformance/26` and `/27`. E8 (#166) is implemented in source format `ess/16` and refused
below it with `unsupported_format_version`; it needs no suite format.

## Behavior and authority

Seven issues filed on 2026-09-27 from retrofits of existing services onto 0.35.0 (`ess/13`) share
one cause. A payload or `sets:` value could be `input.<field>`, a literal, `{response: …}`,
`{generated: true}` or `{cleared: true}`, and a `when_subject` predicate could compare a stored
field with a literal only. The implementations being specified do more:

| Issue | What the implementation does | What the specification had to say instead |
|---|---|---|
| #133 | emits the addressed row's previous value; sets a field from stored state | `{generated: true}`, which checks presence and type only; `subject.note` compiled silently to the text `"subject.note"` |
| #134 | adds one to `retry_count` | nothing (preservation then asserts the old value) or no view over the field |
| #135 | starts `score` at `0.0` | an input, which changes what the command accepts |
| #136 | emits `ref: {id: <input>, label: <minted>}` | `ref: {generated: true}`, so the copied `id` is never checked |
| #137 | emits the caller's `seq`, or mints one when none is given | `{generated: true}`, or a duplicated outcome per presence |
| #140 | upper-cases an input before looking it up | a case-sensitive guard that refuses valid values or admits invalid ones |
| #157 | ignores a stop for a recording other than the row's current one | nothing; the target declines those scenarios |

## Constructs

### E1 — a Decimal literal (#135)

A literal over a target that is `Decimal` underneath is admitted, quoted (`'0.25'`) or unquoted
(`0.25`, `0`). One grammar, `ess_primitives::facts::Number::decimal_literal`, is used by the
validator and by the synthesizer: an optional `-`, digits without a leading zero, optionally a
point and digits. `+1`, `01`, `.5`, `5.`, `1e3` and a literal with more places than its binary64
keeps (`0.30000000000000001`) are refused. No format version: a document that validated before
means the same.

A payload literal is now compared as the value its target type reads it as. Before, every payload
literal was asserted as its text, so `retries: 0` over an `Integer` required the string `"0"`.

### E2 — `{subject: <field>}` (#133)

The addressed entity's field, as it was immediately before this outcome. Admitted in `payload:`
and in `sets:` of an outcome that acts on an **existing** subject (`moves:` or `updates:`). Refused
on `creates:` (there is no row before) and on an outcome with no subject. The identity field is a
legal source. The field's type must be assignable to the target's, or a declared conversion must
cover the pair, exactly as for `input.<field>`.

Synthesis asserts the value where the arrangement determined it: the arranged row's settled value
for that field. Where the arrangement did not determine it, the payload field is covered by the
shape (presence and type) and the `sets:` target stops being a claim about the row.

A text literal spelled `subject.<field name>` is refused with `misspelled_reference` in every
format, naming `{subject: <field>}`. It was the silent miscompile #133 reports; `event.<field>`
is refused the same way already.

### E3 — `{increment: <number>}` and `{generated: true}` in `sets:` (#134)

`retries: {increment: 1}` sets the field to its value before the outcome plus the number. The
number is a non-zero integer for an `Integer` target and a decimal literal (E1 grammar) for a
`Decimal` target; a negative number decrements. Admitted in `sets:` of an existing subject only,
over a required (non-`Optional`) numeric target. Synthesis asserts `before + n` where the
arrangement determined `before`.

For a nested `sets:` mapping, the previous location is the full sequence of target fields:
`packet: {amount: {increment: 1}}` reads `packet.amount`, never a top-level `amount`.
Each read uses the immutable pre-outcome snapshot, including when two nested structs share a
leaf name. The recursive resolved payload already carries this ancestry; no path member or new
source, IR, or suite format is introduced by preserving it through evaluation and generation.
Unlike increment, `{subject: amount}` still explicitly names a top-level subject field.

Required struct parents and present Optional struct parents can supply the previous value.
An absent parent supplies no value: native execution cannot publish the transition and generated
behavior returns its existing unmet-obligation result before storage mutation. No zero, new parent,
or same-named top-level fallback is inferred. Optional numeric leaves remain refused, as do nested
mappings through List, Map, Enum, or Union parents.

Native execution, history checking, and synthesis retain exact Integer and Decimal arithmetic.
An unknown history leaf transfers only when its complete domain proves the exact arithmetic and
target constraints; an unresolved parent remains undetermined. Generated Rust and Go retain their
Integer-only increment support. Recursive target planning must mark a nested Decimal increment
as an obligation rather than emit unsupported arithmetic. Existing generated-target obligations
for newtypes over structs remain explicit.

`{generated: true}` is admitted in `sets:`: the implementation decides the new value. Synthesis
makes no claim about the field after the outcome, so preservation no longer asserts the old value.

### E4 — `{input: <field>, else: {generated: true}}` (#137) and `else: <literal>` (#163, `ess/16`)

The caller's value when the optional input is present, otherwise a value the implementation mints.
The input must be `Optional<T>` with `T` assignable to the target. Admitted in `payload:` and
`sets:`. Synthesis asserts the supplied value when the scenario sends one, and the shape only when
it omits the input.

From `ess/16` the fallback may instead be a literal: `tier: {input: tier, else: Standard}`,
`rank: {input: rank, else: 3}`. The literal is held to the rule a literal written in the target's
place is — a variant of the enum the target is, text for a String-backed target, an unquoted
boolean, whole number or decimal over the primitive it spells, `'0'` rather than `0` over a
`String` — and `subject.<field>` after `else:` is refused as the misspelling E2 describes. In a
payload it also gets the bare payload literal's `misspelled_reference` checks: `else: rank` names
an input without its prefix, and `else: inptu.rank` or `else: event.rank` misspells one. `else:`
admits nothing else: `input.<field>`, `{subject: …}`, `{cleared: true}`, a nested mapping or a
second `{input: …}` are refused where they are written. Below `ess/16` a literal fallback is
refused with `unsupported_format_version`; `{generated: true}` keeps its `ess/14` meaning and its
IR bytes. The IR carries the literal as `otherwise` on the same `input_or_generated` source, left
out when the fallback is generated.

Synthesis asserts both halves in the outcome's scenario. Its first invocation sends the witness
input as before, so the sent value is asserted and an implementation that always stores the
literal fails. After everything that invocation asserts, the branch runs again on a further
instance, with every optional input that the outcome reads only through literal fallbacks left
out, and asserts the literal, read as the target's type, on the event and the row. An input is
kept where anything else reads it (a plain `input.<field>`, a generated fallback, the subject's
identity, a fixture), and where the branch is no longer selected without it, one field at a time.
An implementation storing a different default fails that second invocation. A branch selected by
the held state or the stored row, a replayed branch and a state refusal get only the first
invocation: their arrangement searches fix the plain witness, and arranging it twice would repeat
its identities.

A refusal of the literal reads as the refusal of the same literal written bare in that place: same
code, path, owner and reason. Only the hint differs, and only where the bare repair is one `else:`
refuses: a misspelled reference is repaired with a literal or `else: {generated: true}`, and a
quoted spelling keeps its wrapper (`label: {input: label, else: '0'}`).

### E5 — nested mappings for a struct target (#136)

A mapping whose keys are the fields of a struct-typed target, one source per leaf:

```yaml
payload:
  demo.orders.Tagged:
    ref:
      id: input.order_id
      label: {generated: true}
```

A mapping is read as a source when every key is a source keyword (`response`, `generated`,
`cleared`, `subject`, `increment`, `input`, `else`), and as a nested mapping otherwise. The
target must be a struct (through newtypes and `Optional`); every declared struct field must be
given a source, as `ess/4` requires of every emitted field. Nesting depth is bounded by the type.
Synthesis asserts the whole struct where every leaf is determined, as one value under the field's
name. Where one leaf is not — `rank: {generated: true}` beside four `input.` leaves
(beyond10x/ess#179) — each determined leaf is asserted on its own, and the undetermined one stays
covered by the shape for presence and type.

**Representation: dotted keys.** A determined leaf is written as its own entry under its dotted
path, `lead.number`, in the event payload and in the view row expectation — not as a partial nested
map. The Rust runner already walks dotted paths for the payload shape (`reach_into`), whose leaves
are keyed by the same paths, so the comparison is that walk applied to value keys; a field name
cannot contain a dot, so an older suite's keys read exactly as before. A partial map would instead
have turned map equality into a subset test, which a `Json` value (compared structurally) must not
get. A dotted payload key must name a leaf of the step's own shape, or the suite is refused.

A leaf of a nested struct is walked to its scalar leaves, and so is a struct value a leaf reads whole (`place: input.place`), by its declared fields; a part the value does not carry (an absent `Optional` struct) is left unasserted. The row entries come from the branch's
own `sets:` where the view projects the field at the entity's type; the fields the arrangement
settled are not given dotted entries, so a later act's claims about the row stay by field name.

A view row carries no shape, so the undetermined leaves a nested `sets:` mapping writes are claimed
separately: for each one whose declared type is never absent (not `Optional`, not `Json`), the row
expectation is followed by an `excludes` of the subject's identity with that dotted path `null`. The
runner reads a dotted path that finds nothing as `null`, so a row that never writes `lead.rank`
fails. Nothing is claimed when the view does not project the identity. **The row type of a
generated leaf is not asserted; the payload shape is**: no view expectation can say "holds an
`Integer`", and adding one would be a new step kind.

Suites carrying such a leaf take `ess-conformance/26` (ordinary) and `/27` (coverage), the round-3
pair: an older reader would look for a top-level field named `lead.number` and fail a conforming
implementation. The Go and TypeScript runtimes execute both (beyond10x/ess#188). A suite without a partly
determined struct keeps its earlier format and bytes.

### E6 — `input.<field>` in a `when_subject` predicate (#157, `ess/15`)

A comparison in `when_subject: {predicate: …}` may name a field of the command's input as an
operand with the `input.` prefix: `recording_id != input.recording_id`. The two operands must have
comparable types, by the rules any comparison is checked by; an input field the command does not
declare is `undeclared_reference`. A bare word on the right of a comparison is a literal, as
everywhere, so the input operand is written on the right (or both sides are dotted paths).
Synthesis witnesses the guard both ways by choosing the input from the arranged row's value: equal,
and different.

- The finite partition declines a guard comparing two facts, so a command with one needs a genuine
  default, as for any open guard — even over an enum.
- A stored field named `input` keeps being read as that field; the namespace applies only where
  the entity declares none, so no document that validated before `ess/15` changes meaning.
- Synthesis grounds each such comparison on the arranged row: the stored side becomes the value
  the row holds and is handed to the candidate search as a literal. Because a text has no
  neighbour the search could offer, a command with such a comparison also tries the candidates of
  a further witness distinction, whose base values differ from the ones the row was built from.
  Commands without one keep their candidates and their suites.
- A guard reading the input is decided at synthesis and never reaches a suite, so no suite format
  changes. Entity Runtime lowers `input.<field>` to `$args.input.<field>`.

### E7 — case-insensitive comparison (#140, `ess/15`)

Map operators `equals_ignore_case` (a text literal) and `in_ignore_case` (a list of text literals),
over `String` and its newtypes. Case folding is ASCII only: `A`–`Z` fold to `a`–`z`, every other
byte compares as itself, so the three evaluators (Rust, Go, TypeScript) agree by construction.
Entity Runtime has no such operator; lowering refuses it (`CaseFoldUnsupported`), as it refuses
text orderings.

- The predicate is its own variant, `Predicate::FoldMatch`, not a string operator: the two arrived
  in different formats, and a selection plan (`ess/3`) keeps refusing it by the rule it refuses
  every predicate outside its bounded contract.
- A list under `equals_ignore_case` and a scalar under `in_ignore_case` are refused by the reader;
  a non-text literal is `type_mismatch`, an empty list `empty_declaration`, a bare word naming a
  declared field the #74 refusal.
- The finite partition declines a fold, so a command with one needs a default.
- Synthesis witnesses the guarded branch on the literal in the other ASCII case, which only a
  folding implementation accepts. The refuting branch has two witnesses. First, ahead of any base
  text, the literal with one character changed (equal under folding to no literal the guards
  name, so `in_ignore_case: [web, xeb]` is refuted by `yeb`, not by another member), which a target
  comparing only lengths or the first byte accepts. Then, as a further row of the default's
  scenario where the literal has one, the literal in a case only Unicode folding equates with it —
  each non-ASCII letter in its other case (`café` to `CAFÉ`), or else its first `k` as U+212A
  KELVIN SIGN or first `s` as U+017F LONG S — which a target using Unicode case folding
  (`strings.EqualFold`, `toLowerCase`) accepts.
- A fold in a command guard is decided at synthesis. A suite carrying one in a view's `satisfies`
  expectation or an observed selection plan takes `ess-conformance/20` (ordinary) or `/21`
  (coverage); each implies every major below it. Rust, Go and TypeScript admit those majors;
  browser replay refuses them by version, and `infra-spec/1` refuses the operators.

### E8 — `{related: {via: <field>, field: <field>}}` (#166, `ess/16`)

A field of the row another row references, as it is when the outcome runs. Dispatching a shipment
emits the region stored on the shipment's customer:

```yaml
payload:
  demo.shipping.ShipmentDispatched:
    shipment_id: input.shipment_id
    region: {related: {via: customer_id, field: region}}
```

- **`via`** is a field of the subject or of the input (`input.customer_id`, on any outcome). A
  subject field is read as it was before the outcome, so on `moves:` and `updates:` as for E2. On
  `creates:` there is no row before, and a subject field is admitted only where the branch sets it
  from its input unchanged (`sets: {customer_id: input.customer_id}`): it then holds that input,
  and the read is the input's. Any other subject field on `creates:` is `conflicting_declaration`.
  The type of `via` must be an entity's identity type itself: `Optional<…>`, `List<…>` and
  `Map<…>` are refused with `type_mismatch`, because the source reads one row that is always
  there. A missing referenced row is the implementation's refusal or a declared outcome, not
  something this source decides.
- **Which entity.** The relation the subject field carries names it: a `references` relation of
  cardinality `one` the subject declares on it, or an `owns` relation another entity declares over
  the subject through it (the owner; a field carries one relation, so an owned subject cannot also
  declare a `references`). An input read is settled by the relation on the subject field the
  branch sets from that input. Otherwise the one entity whose identity is that type names it. A
  type that is no entity's identity is `type_mismatch`. A type that several entities are identified
  by, with no relation to say which, is `conflicting_declaration`, and every hint names a remedy
  that validates on that branch:
  - a subject field: declare the `references` relation on it, or the owner's `owns`;
  - an input, on a branch that sets a subject field from it: declare the `references` relation on
    that field. On `creates:` the hint says there is no row before, which is why the field the
    branch sets is the one to use;
  - an input, on a branch that sets none: set a subject field from the input and declare the
    relation on it;
  - an input, on a branch with no subject: give the entities distinct identity types.

  The rule is `ess_domain::command::related_value::referenced_entity` (with `input_carrier`),
  called by the validator and the compiler alike.
- **`field`** is a field of that entity (its identity included), assignable to the target or
  covered by a declared conversion, exactly as for `input.<field>`. `via` and `field` each name one
  field; a dotted name is `undeclared_reference`.
- One hop. Admitted in `payload:`, in `sets:` and as a leaf of a nested mapping.

**Spelling.** `related:` holds a mapping of its own, and `{related: …}` is written alone. The
issue's flat `{via, entity, field}` would have made `via` and `field` source keywords, so a struct
with a field called `field` could no longer be filled by a nested mapping. The entity is derived
from the type or the relation rather than written a second time.

`related` is **not** a source keyword. The reader recognises one exact shape — a mapping whose only
key is `related`, holding exactly `via` and `field` as text — and reads every other mapping under a
`related` key as a nested mapping, as before. The reader cannot know the format (the header may be
in another file), so it recognises the shape in every format, and the assembled specification
reads it back as a nested mapping below `ess/16` (`related_value::read_below_ess_16`): a struct
field `related` whose fields `via` and `field` take the texts written, each read as a bare text is.
So no `ess/14` or `ess/15` document changes meaning. That mapping over a target that is not a
struct is refused below `ess/16` with `unsupported_format_version`. From `ess/16`, a mapping keyed
`related` that is not the exact shape, over a target that is not a struct, is `type_mismatch`, with
a hint giving the shape.

**IR.** `related_field` with `via` (`{from: subject | input, field, type_ref}`), `entity`, `field`
and `type_ref`. A document without the source keeps its IR bytes.

**Synthesis.** For each `via` a branch reads, the arrangement creates three rows of the referenced
entity where its lifecycle starts, ahead of everything else: a decoy, the referenced row, and a
second decoy.
- **Distinctions.** The referenced row is at a multiple of 27720 (= lcm(1..=12)) and the decoys
  are one below and one above it. A decoy is one away from the referenced row, and the witness
  cycles enum variants by distinction, so every decoy field of an enum with two or more variants
  holds another variant; a text or number differs by construction.
- **Pointing the subject at the referenced row.**
  - An input `via`, or a `creates:` subject field set from one, is bound to the referenced row, as
    an owner's input is. Where it is already bound to the owner the arrangement created for an
    owned `creates:` subject, that owner is the referenced row. It sits at the plain witness, so
    its decoys, one away from a multiple of 27720, differ from it for every enum of up to twelve
    variants. The decoys go before the owner and right after its capture.
  - A subject field of an existing subject is pointed at the referenced row by rewriting the
    subject's creating act, through that branch's `sets: <via>: input.<field>` (the link
    `arrange_owner` reads), and only where no later act rewrote the field.
- **The value at the branch.** Where the document declares an `updates:` branch of the referenced
  entity that writes a read field from its input unchanged, that branch is run on the referenced
  row after everything else in the arrangement and just before the branch under test. Its input is
  chosen to write a value neither the referenced row nor a decoy held, where one of the first
  twelve further witnesses does, else one that at least differs from the referenced row's. The
  value it wrote is the one asserted. So an implementation that copied the value when the subject
  was created publishes the old one and fails. Branches chosen by the row's stored fields, or
  decided by an injected external outcome, are not used. Where the document declares no such
  branch, the value at the branch is the value at creation, and a copy made at creation cannot be
  told apart from a read at the branch.
- **What is asserted.** The referenced row's value is asserted on the event and, for `sets:`, on
  the row. An implementation reading a decoy, the row created first or the row created last fails.
- **Where nothing is asserted.** If the rows cannot be arranged or the link is not declared, the
  source is undetermined: the payload shape covers it, and the row gets no claim. A branch reached
  through the stored-row search (a `when_subject` predicate) is arranged by that search and does
  not get the rows.
- **Where the value is kept.** It is carried in the arrangement's settled fields under a key that
  is neither a field name nor a fact path, so no view row, filter or lookup by field name reads it.

No new step, step field or value kind, so no suite format: a suite asserting a related value is a
literal in an `expect_event` payload or a view row, which every runtime already compares.
Entity Runtime refuses the source with `ValueExpressionUnsupported`, as it does every value
expression: entity-core has no join. `ess-gen` documentation describes it as it describes every
other source (`demo.shipping.Customer.region of the row subject.customer_id names`). `ess-synth`
renders no payload source except a response field. Neither has anything to refuse.

### E9 — `{caller: <attribute>}` and `caller.<attribute>` (#168, `ess/16`)

A value read from the authenticated caller, and a guard operand comparing it with an input or a
stored field. Its binding design is [`caller-values.md`](caller-values.md): actor `attributes:`,
the every-actor rule, the `==`/`!=`-against-a-field restriction, the suite/26 `caller` field on
command steps, and synthesis under two caller assignments.

## Not in this design

- Arbitrary arithmetic, string functions or conditionals beyond E4's one fallback.
- A source read through more than one reference, or through an `Optional` or list reference (E8).
- A guard reading a related row (for example, refusing when the referenced customer is `Closed`).
- Per-leaf struct comparison in suites (E5).
