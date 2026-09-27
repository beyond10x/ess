# Value expressions in `sets:`, `payload:` and subject guards

Status: E1–E5 implemented in source format `ess/14` (beyond10x/ess#133, #134, #135, #136, #137).
E1 (#135) needs no format version; E2–E5 are refused below `ess/14` with
`unsupported_format_version`. E6 (#157) and E7 (#140) are implemented in source format `ess/15` and
refused below it with `unsupported_format_version`; E7 carries suite formats `ess-conformance/20`
and `/21`. E4's literal fallback (#163) is implemented in source format `ess/16` and refused below
it with `unsupported_format_version`. E5's per-leaf comparison (#179) carries suite formats
`ess-conformance/26` and `/27`.

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
implementation. The Go and TypeScript runtimes refuse both by version. A suite without a partly
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

## Not in this design

- Arbitrary arithmetic, string functions or conditionals beyond E4's one fallback.
- A source read from a row other than the subject.
- Per-leaf struct comparison in suites (E5).
