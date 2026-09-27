# Value expressions in `sets:`, `payload:` and subject guards

Status: E1–E5 implemented in source format `ess/14` (beyond10x/ess#133, #134, #135, #136, #137).
E1 (#135) needs no format version; E2–E5 are refused below `ess/14` with
`unsupported_format_version`. E6 (#157) and E7 (#140) are accepted and not implemented: each needs
a suite format version and Go and TypeScript evaluator changes, and they ship in a later format.

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

### E4 — `{input: <field>, else: {generated: true}}` (#137)

The caller's value when the optional input is present, otherwise a value the implementation mints.
The input must be `Optional<T>` with `T` assignable to the target. Admitted in `payload:` and
`sets:`. `else` admits `{generated: true}` only in this cut. Synthesis asserts the supplied value
when the scenario sends one, and the shape only when it omits the input.

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
Synthesis asserts the whole struct where every leaf is determined; otherwise the field is covered
by the shape. Per-leaf comparison would change the suite's payload key semantics in three runners
and is left for a later suite format.

### E6 — `input.<field>` in a `when_subject` predicate (#157, not implemented)

A comparison in `when_subject: {predicate: …}` may name a field of the command's input as an
operand with the `input.` prefix: `recording_id != input.recording_id`. The two operands must have
comparable types. Synthesis witnesses the guard both ways by choosing the input from the arranged
row's value: equal, and different.

### E7 — case-insensitive comparison (#140, not implemented)

Map operators `equals_ignore_case` (a text literal) and `in_ignore_case` (a list of text literals),
over `String` and its newtypes. Case folding is ASCII only: `A`–`Z` fold to `a`–`z`, every other
byte compares as itself, so the three evaluators (Rust, Go, TypeScript) agree by construction.
Entity Runtime has no such operator; lowering refuses it, as it refuses text orderings.

## Not in this design

- Arbitrary arithmetic, string functions or conditionals beyond E4's one fallback.
- A source read from a row other than the subject.
- Per-leaf struct comparison in suites (E5).
