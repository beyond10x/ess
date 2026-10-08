---
title: Predicates
sidebar_position: 4
description: Every predicate form ESS accepts, where each one is accepted, and which ones synthesis can build a scenario for.
---

# Predicates

A predicate is a condition over facts. ESS reads the same grammar in every place it takes one, in
two spellings: a **compact** string (`quantity > 0`) and a **structured** mapping (`all:`, `any:`,
`quantity: {gte: 5}`). Both spellings build the same kind of predicate, and they may be mixed at any
depth. They read a bare dotted word differently in one place; see
[equality shorthand](#the-equality-shorthand-reads-a-literal).

Every YAML example on this page is run by a test. The test runs `ess specify validate` on each one.
It also runs `ess verify conform synthesize` wherever the page makes a claim about synthesis. A
page that says a form works when it does not fails the gate. An example that describes a release
not yet made is marked as pending in the page source. The test still runs it, expects it to
disagree with the page today, and fails once it agrees, so the marker cannot outlive the change.

## Where a predicate is accepted

| Place | What the predicate reads | Notes |
|---|---|---|
| a command outcome's `when` | the command's input fields | A branch without `when` is the default. |
| a command outcome's `when_subject: {predicate: …}` (`ess/9`) | the declared stored fields of the entity the command addresses, read just before the command selects a branch; from `ess/15` also the command's input, as `input.<field>` | `state` from `ess/18`, the held lifecycle state; not before. The input only through the `input.` prefix; see [comparing with the input](#comparing-a-stored-field-with-the-input). Conjunctive with `when`. A refusal may carry it without naming a subject; it reads the one its sibling branches name. |
| a command outcome's `when_related: {via: input.<field>, predicate: …}` (`ess/18`) | the declared stored fields of the row of another entity whose identity `input.<field>` carries, read just before the command selects a branch, and the command's input as `input.<field>` | `state` from `ess/20`, the related row's held lifecycle state; not before. From `ess/22` `input.<field>` may be `Optional<…>`: checked only when present, and an absent reference selects no `when_related` branch; and `via` may be a bare stored field of the addressed subject, read as it was before the branch (see [a stored reference](#a-stored-reference)). Keyed by that entity's identity only, one hop; from `ess/22` a lookup by any other field is the row-set form below. A missing row makes the predicate unknown, so it selects only the sibling `when_related: {via: …, exists: false}` branch, which the command must declare. Any branch may carry it, a `creates:` or a refusal naming no subject included; conjunctive with `when` except on the `exists: false` branch, which answers a missing row before any other; never beside a `when_subject*` guard. See [a guard over another entity's row](#a-guard-over-another-entitys-row). |
| a command outcome's `when_related: {entity, where, …}` (`ess/22`) | the rows of `entity` that `where` selects: each candidate row's declared fields, identity and held lifecycle state `state` bare, the command's input as `input.<field>`, the addressed subject as it was before the branch as `subject.<field>`, and `now` as the decision's one instant; `forall` reads the same over each selected row | The rows are the store just before the branch is selected; no row the outcome writes is one of them. `exists` is a Boolean, `count` one comparison (`eq`, `ne`, `lt`, `lte`, `gt`, `gte`) with a whole number, `forall` a second predicate, true of no rows. Conjunctive with `when`; never beside `via` or a `when_subject*` guard. See [a guard over the rows a selector selects](#a-guard-over-the-rows-a-selector-selects). |
| an entity's `invariants` | the entity's own fields and its lifecycle `state` | Checked after every branch that creates or changes the entity. A required field an invariant reads must be set by every `creates:` branch, or declared `Optional<…>`; otherwise validate refuses it with `ESS-COMMAND-018`. |
| a struct type's `invariants` | the struct's own fields | Same grammar, checked against the type. |
| a newtype's `invariants` | the wrapped value, as `value` | For example `value != ""` on a newtype of `String`. |
| a view's `filter` | the source entity's fields and its lifecycle `state` | Selects the rows the view returns. |
| a binding selection's `where` | one list item's fields | A bounded fragment: presence, typed equality and inequality, and `all`/`any`/`not`. See [select ordered records in a binding](../guides/specify/binding-context.md#select-ordered-records-in-a-binding). |

Two outcome keys that look like guards are **not** predicates:

- `when_subject: {field: <enum field>, equals: <variant>}` (`ess/6`) is one enum field equal to one
  variant. Its other shape, `when_subject: {predicate: …}` (`ess/9`), is a predicate and is listed
  above. A mapping that writes keys of both shapes is refused while the document is read, and the
  predicate shape under a header older than `ess/9` is refused with `unsupported_format_version`.
- `when_subject_state` is one lifecycle state name, or from `ess/18` a list of them.

A binding's `when:` names its cause, such as `periodic:`. It is not a predicate.

A predicate is evaluated in three-valued logic: true, false, or unknown when a fact it reads was
never observed. Only true satisfies a guard. An unobserved collection is unknown, not empty.

## The model the examples use

Every fragment below is placed into this one specification before it is run. A `when:` fragment
replaces the guard of outcome `placed`. An `invariants:` fragment replaces the invariants of
`shop.order.Order`. A `filter:` fragment replaces the filter of `shop.order.OpenOrders`.

```yaml ess-check="model"
format: ess/15
system: shop
version: v1
domain: shop.order
types:
  - name: shop.order.Channel
    kind: enum
    variants: [Web, Store, Phone]
  - name: shop.order.Sku
    kind: newtype
    of: String
entities:
  - name: shop.order.Order
    identity: {name: order_id, type: Uuid}
    fields:
      - {name: channel, type: shop.order.Channel}
      - {name: quantity, type: Integer}
      - {name: sku, type: shop.order.Sku}
      - {name: note, type: Optional<String>}
      - {name: tags, type: List<String>}
      - {name: labels, type: "Map<String, String>"}
    invariants:
      - quantity >= 1
    lifecycle:
      initial: Open
      states: [Open, Closed]
      terminal: [Closed]
      transitions:
        - {name: close, from: [Open], to: Closed}
errors:
  - name: shop.order.Refused
    summary: The order was not placed.
  - name: shop.order.NotOpen
    summary: The order is not open.
commands:
  - name: shop.order.PlaceOrder
    input:
      - {name: channel, type: shop.order.Channel}
      - {name: quantity, type: Integer}
      - {name: sku, type: shop.order.Sku}
      - {name: coupon, type: Optional<String>}
      - {name: tags, type: List<String>}
      - {name: gift, type: Boolean}
      - {name: labels, type: "Map<String, String>"}
    outcomes:
      - name: placed
        when: quantity > 0
        creates: shop.order.Order
        instance: order_id
        emits: [shop.order.OrderPlaced]
        payload:
          shop.order.OrderPlaced:
            order_id: {generated: true}
        sets:
          channel: input.channel
          quantity: input.quantity
          sku: input.sku
          tags: input.tags
          labels: input.labels
      - name: refused
        error: shop.order.Refused
  - name: shop.order.CloseOrder
    input:
      - {name: order_id, type: Uuid}
    outcomes:
      - name: closed
        moves: shop.order.Order.close
        instance: order_id
        emits: [shop.order.OrderClosed]
        payload:
          shop.order.OrderClosed:
            order_id: input.order_id
      - name: wrong-state
        wrong_state: true
        error: shop.order.NotOpen
events:
  - name: shop.order.OrderPlaced
    fields:
      - {name: order_id, type: Uuid}
  - name: shop.order.OrderClosed
    fields:
      - {name: order_id, type: Uuid}
views:
  - name: shop.order.OpenOrders
    source: shop.order.Order
    consistency: read_your_writes
    filter: state == Open
    fields:
      - {name: order_id, type: Uuid}
      - {name: channel, type: shop.order.Channel}
  - name: shop.order.OrderById
    source: shop.order.Order
    consistency: read_your_writes
    fields:
      - {name: order_id, type: Uuid}
      - {name: channel, type: shop.order.Channel}
      - {name: quantity, type: Integer}
      - {name: sku, type: shop.order.Sku}
      - {name: note, type: Optional<String>}
      - {name: tags, type: List<String>}
      - {name: labels, type: "Map<String, String>"}
      - {name: state, type: shop.order.Order.State}
```

## Compact forms

A compact predicate is one string. It holds a single comparison, a bare path, a presence check or a
`not` in front of one of those.

| Form | Example | Holds when |
|---|---|---|
| comparison | `quantity > 0` | the operator holds. The operators are `==`, `!=`, `<`, `<=`, `>`, `>=`. |
| bare path | `gift` | the fact is present and truthy. |
| presence, `defined` or `exists` | `defined(coupon)`, `exists(coupon)` | the fact is present. From `ess/16` the fact may be an `Optional` struct, list or map; see [presence of an aggregate](#presence-of-an-optional-struct-list-or-map). |
| absence, `missing` | `missing(coupon)` | the fact is absent. It is the same as `not defined(coupon)`. |
| negation | `not gift`, `not defined(coupon)` | the rest of the string does not hold. |
| constant | `always`, `true`, `never`, `false` | always, or never. |

```yaml ess-check="when" ess-expect="synthesizes"
when: channel == Web
```

```yaml ess-check="when" ess-expect="synthesizes"
when: gift
```

```yaml ess-check="when" ess-expect="synthesizes"
when: not gift
```

The right-hand side of a comparison is read in this order:

1. A quoted value is text: `sku == "A.1"`.
2. `true` and `false` are booleans. A number is a number.
3. A bare word containing a dot is a fact path.
4. Inside a [quantifier](#quantifiers), a bare word that is exactly the name of a binder in scope is
   that binder: `t != l`.
5. Anything else is text: `channel == Web`.

Through `ess/21` a right-hand side without a dot is therefore never a field (from `ess/22` see
[a bare word names a field](#from-ess22-a-bare-word-names-a-field)). To compare two fields, put them in one
struct and compare its members, for example `window.ends_at > window.starts_at`. See
[order two instants](../guides/specify/fields-and-invariants.md#order-two-instants).

An equality whose text literal is the name of a declared field is refused rather than read as
text. The refusal says the literal is the text and not the field. Quoting the word does not change
this, and neither does the `eq` or shorthand spelling.

```yaml ess-check="when" ess-expect="refused:ESS-COMMAND-002" ess-says="not the field"
when: sku == gift
```

```yaml ess-check="when" ess-expect="refused:ESS-COMMAND-002" ess-says="not the field"
when: sku == "gift"
```

A bare dotted word is a fact path, so `sku == A.1` reads a path `A.1` that the input does not
declare. Quote a literal that contains a dot.

```yaml ess-check="when" ess-expect="refused:ESS-COMMAND-003"
when: sku == A.1
```

```yaml ess-check="when" ess-expect="synthesizes"
when: sku == "A.1"
```

Comparisons are type-checked against the declared fields. A literal the field's type cannot hold is
refused, and so is an enum variant the enum does not declare.

```yaml ess-check="when" ess-expect="refused:ESS-COMMAND-002"
when: gift == yes
```

```yaml ess-check="when" ess-expect="refused:ESS-COMMAND-001" ess-says="does not declare as an enum variant"
when: channel == Post
```

A path the predicate's place does not provide is refused. A `when` reads only the command's input.

```yaml ess-check="when" ess-expect="refused:ESS-COMMAND-003"
when: colour == Web
```

The compact form has no `&&`, `||` or `in [...]`. Write a conjunction, disjunction or membership in
the structured form below.

```yaml ess-check="when" ess-expect="refused" ess-says="a predicate is either a comparison"
when: channel in [Web, Store]
```

An unquoted `&&` or `||` after a comparison is refused rather than read as part of one text
literal. Quote the whole literal if the text really contains it.

```yaml ess-check="when" ess-expect="refused" ess-says="are not part of the compact form"
when: sku == A1 && gift
```

```yaml ess-check="when" ess-expect="synthesizes"
when: sku == "A1 && gift"
```

### From `ess/22`: a bare word names a field

From `ess/22` (beyond10x/ess#225, #233), an unquoted word without a dot on the right of a comparison
is decided against the declarations of the place it is written in, in this order:

1. the name of a binder in scope is that binder, as before;
2. where the left side is an enum that declares the word as a variant, it is that variant, so
   `state == Open` keeps its meaning beside a field named `Open`;
3. the name of a field the place reads — an input in a `when:`, a stored field in a
   `when_subject:` or `when_related:` predicate, an `instances:` or `affects:` filter, an entity's,
   struct's or newtype's invariant or a view's `filter:` — is that field: `task_id != depends_on`
   compares two inputs, `leased <= capacity` two stored fields;
4. anything else is the text it always was.

A quoted word is text in every format, and one naming a field is still refused, now with the
repair to write it unquoted. A field named `now` is that field; only where none is declared does
`now` read the current time. A binder named like a field of the place is refused where a bare word
would read it: rename the binder. A plain `when:` also reads `input.<field>` as that input, unless
the command declares an input named `input`, which keeps being read as itself.

A one-segment field on the right is written back as the explicit operand `{fact: depends_on}`
(`task_id: {eq: {fact: depends_on}}`), because the compact `task_id == depends_on` reads as text
in every earlier format. A dotted path and a binder keep their compact spelling. The explicit
operand is an `ess/22` form: under `ess/21` and earlier it is refused as
`unsupported_format_version`, and the bare word keeps its old meaning and its old refusal.

A comparison of two `Timestamp` fields is tagged to compare the instants they name, never their
spellings: `valid_until > valid_from` is written back as
`{compare: {left: valid_until, op: gt, right: {fact: valid_from}, as: timestamp}}`. The tag is
read only between two `Timestamp` facts, and only from `ess/22`. A `compare` mapping without
`left` is still a constraint on a field named `compare`.

Two inputs whose type is an entity's identity compare only by `==` and `!=`: ordering identities
names nothing a caller supplied, and is refused as `type_mismatch`. Synthesis sends the equal case
as one arranged instance named twice and the unequal case as two arranged instances.

A suite carrying the explicit operand or the tag is written as `ess-conformance/40` (ordinary) or
`/41` (coverage), which the Rust, Go and TypeScript runners read; a suite labelled with an earlier
number is refused before any step runs. A suite whose comparisons need neither keeps its number.

### From `ess/22`: one constant offset

From `ess/22` (beyond10x/ess#233, #244), the right side of a comparison may be one fact moved by
one constant: `upper <= lower + 5`, `expires_at <= issued_at - 24h`. An unquoted right side written
`<fact> + <magnitude>` or `<fact> - <magnitude>`, with or without spaces around the sign, is an
offset where the fact names a binder in scope, a field of the place or a dotted path through
either; where it names nothing it stays the text it always was.

- Between two `Integer` facts (through newtypes and `Optional`) the magnitude is a whole number
  from `0` to `9223372036854775807`, with no sign, fraction or leading zero. The comparison uses the
  exact sum: `upper < lower + 1` holds for `9223372036854775807` on both sides, and nothing wraps,
  saturates or rounds. `Decimal` and `Binary64` are refused.
- Between two `Timestamp` facts the magnitude is a whole number of `s`, `m` or `h` under the
  current-time bound, and the base moves by that many elapsed UTC seconds. A day is `24h`; there are
  no days, months, calendars or zones.

All six operators are admitted for both. A fact not observed, an absent `Optional`, a text that
names no instant, or a moved instant past what an RFC 3339 `date-time` spells makes the comparison
unknown. A base of another type is refused as `type_mismatch`. Against an `Integer` or a `Timestamp`,
so is a right side spelled as an offset of a field whose magnitude does not read — `lower + 05`,
`lower + 5 + 3`, `issued_at - 1d` — or that is quoted; against a `String` such a spelling is the text
it always was, so `mode == read-only` beside a field named `read` still compares with `read-only`.
`now - 60s` keeps reading the current time, with its `<`, `<=`, `>`, `>=` restriction, wherever no
field is named `now`.

An offset is written back as one closed mapping, `upper: {lte: {offset: {fact: lower, add: 5}}}` or
`expires_at: {lte: {offset: {fact: issued_at, subtract: 24h}}}`, with exactly `fact` and one of
`add` and `subtract`. It is an `ess/22` form, refused below it, and a suite carrying it is
`ess-conformance/40` or `/41`. Generated Rust and Go behaviour decides such a guard; a view filter
with an offset stays owed, an entity invariant with one refuses the generated target by name, and
Entity Runtime refuses it as `OffsetUnsupported`.

### From `ess/22`: distinct list members

From `ess/22` (beyond10x/ess#237), `distinct: {in: <list>, as: <name>, by: <name>.<member>}` holds
when no two elements of a `List` share a key: the element itself, or, with `by`, the one member of
it that `by` names under the binder, such as `{distinct: {in: files, as: file, by: file.path}}`.
`by` is optional for a list of scalars and required for a list of structs. The key resolves,
through newtypes and `Optional`, to a `Boolean`, `Integer`, `Decimal`, `String`, `Uuid`, `Timestamp`
or enum; a struct, list, map, union, `Json`, `Binary64`, `Duration` or `Bytes` key is refused as
`type_mismatch`, and so is a `Map` or a scalar in `in`.

Keys compare by their type: numbers exactly, so `1` and `1.0` are one `Decimal` key and
`9007199254740992` and `9007199254740993` two `Integer` keys; a `Timestamp` by the instant it names,
so `2020-01-01T00:00:00Z` and `2019-12-31T19:00:00-05:00` are one key; text, a `Uuid` and an enum
exactly. An empty or one-element list holds. For two or more, two known equal keys anywhere make
it false; every key known and pairwise unequal makes it true; anything else is unknown. An absent
`Optional` list is unknown, not empty, and an absent key is neither skipped nor one shared null.
A key outside its type — a fraction under `Integer`, a text no `date-time` spells — is unknown.
Negation keeps unknown.

The source leaves the key's type out; the canonical form writes it, as
`{distinct: {in: files, as: file, by: file.path, kind: string}}`, where `kind` is one of `boolean`,
`integer`, `decimal`, `string`, `uuid`, `timestamp` and `enum`. A source that writes `kind` must
write the one its declarations give. A mapping under `distinct` without `as` is a constraint on a
fact named `distinct`, as it always was. It is an `ess/22` form, refused below it, and a suite
carrying it is `ess-conformance/40` or `/41`, whose readers require `kind` and read a view row's
lists element by element. Generated Rust and Go behaviour owes a guard or a view filter reading it
by name, an entity invariant with it refuses the generated target by name, and Entity Runtime
refuses it as `DistinctUnsupported`.

### From `ess/22`: the UTF-8 byte length of a text

From `ess/22` (beyond10x/ess#233), `.utf8_bytes` after a `String` — or a newtype or `Optional` of
one — is the number of bytes the text's UTF-8 encoding takes, an `Integer`: `label.utf8_bytes <=
255` holds a column limit that `.count`, which counts Unicode scalar values, cannot. `é` is two
bytes and one scalar, `€` three bytes, U+1F600 four bytes, one scalar and two UTF-16 code units; `e`
followed by a combining accent is three bytes. No Unicode normalization occurs, and empty text is
zero. A guard `when: label.utf8_bytes > 255`, an invariant `title.utf8_bytes <= 64` and a
comparison of two byte lengths, `label.utf8_bytes != code.utf8_bytes`, are all admitted.

It is an operand of a comparison, on either side, and nothing else: `defined(label.utf8_bytes)`, a
bare `label.utf8_bytes`, `any_of`, a text operator and a quantifier over it are refused as
`type_mismatch`. It is not defined on `Bytes`, `Timestamp`, `Uuid`, an enum, a number or a
collection, and nothing may follow it. A struct member that is itself named `utf8_bytes` is that
member, in every format. Below `ess/22` the selector is refused naming `ess/22`.

An absent `Optional`, an unobserved text and a value that is no text make the comparison unknown.
A text holding a lone UTF-16 surrogate is no Unicode text, and its byte length is unknown in every
runner: a Rust string cannot hold one, the Go runner refuses a string that is not UTF-8, and the
TypeScript runner refuses a lone surrogate rather than measure the three bytes of the U+FFFD an
encoder would write in its place. A generated Rust or Go server answers `400` to a request body
that escapes one.

An observation bound at the full path `<text>.utf8_bytes` wins over the text, as one bound at
`<text>.count` does for `.count`. The two differ in what such an observation may be: any value
bound at `.count` is compared as it is, while only a whole number from zero is a byte length —
anything else bound at `.utf8_bytes` makes the comparison unknown, and the text is not read instead.

The byte length is written back as its own operand, `{utf8_bytes: label}`: on the right as
`limit: {gte: {utf8_bytes: label}}`, and on the left in the closed form
`{compare: {left: {utf8_bytes: label}, op: lte, right: 255}}`, with exactly `left`, `op` and `right`. A suite carrying
it is `ess-conformance/40` or `/41`; a member named `utf8_bytes` selects nothing. In such a
`satisfies`, a text's `.count` is asserted on view rows beside it. Synthesis witnesses each bound at
the length and one byte either side, as text led by a wide character where the alphabet admits one,
so an implementation counting scalars, UTF-16 units or graphemes fails a scenario; past 1024 bytes
the guard is refused as `ESS-SYNTH-018`. Generated Rust and Go behaviour and invariant checks
measure with `str::len` and `len` of a valid UTF-8 `string`; a view filter with a byte length stays
owed, and Entity Runtime refuses it as `Utf8BytesUnsupported`.

### Absence is not `null`

An unquoted `null` on the right of `==` or `!=` is refused, with a hint that names `defined(x)` or
`not defined(x)`. Test presence with `defined`. A quoted `"null"` is still the four-character text.

```yaml ess-check="when" ess-expect="refused:ESS-SPEC-017" ess-says="defined(coupon)"
when: coupon == null
```

```yaml ess-check="when" ess-expect="synthesizes"
when: coupon == "null"
```

### Presence of an Optional struct, list or map

From `ess/16`, `defined(x)` and `missing(x)` accept any `Optional<T>`, including a struct, a list
or a map `T`. Presence belongs to the `Optional`, not to what it holds:

| The value at `x` | `defined(x)` |
|---|---|
| a struct, list or map, including an empty one | `true` |
| left out, or `null` | `false` |

This lets an invariant say when an optional field must be gone. For an entity whose
`metrics: Optional<demo.queue.Metrics>` is held only while it is `Paused`, the invariant is
`any: [state == Paused, {not: "defined(metrics)"}]`. Synthesis checks it after every branch that
changes the entity. If an outcome leaves `Paused` without clearing `metrics`, that check fails.

Below `ess/16`, `defined()` over an `Optional` aggregate is refused with
`unsupported_format_version`. `defined()` over an aggregate that is not `Optional` is always
present, so it is refused as a type mismatch in every format. A bare path over any aggregate is
refused in the same way.

## Structured forms

| Key | Aliases | Holds when |
|---|---|---|
| `all` | `and`, `all_of` | every child holds |
| `any` | `or` | at least one child holds |
| `not` | | its one child does not hold |
| `none` | `none_of_these` | no child holds |

`all`, `any` and `none` take a list, or a single child. A bare list is an implicit `all`. A mapping
with several keys is also an `all` of its entries.

```yaml ess-check="when" ess-expect="synthesizes"
when:
  any:
    - channel == Web
    - channel == Store
```

```yaml ess-check="when" ess-expect="synthesizes"
when:
  all:
    - channel == Web
    - quantity: {gte: 5, lte: 50}
```

```yaml ess-check="when" ess-expect="synthesizes"
when:
  none:
    - channel == Phone
    - gift
```

```yaml ess-check="when" ess-expect="synthesizes"
when:
  not: channel == Phone
```

```yaml ess-check="when" ess-expect="synthesizes"
when:
  - channel == Web
  - gift
```

There is no `implies`. Write "if A then B" as `any: [{not: A}, B]`:

```yaml ess-check="invariants" ess-expect="synthesizes"
invariants:
  - any:
      - not: channel == Phone
      - quantity <= 5
```

An entity invariant also reads the held lifecycle state as `state`. This one says a closed order
still has a positive quantity; the same form states which value a stored field holds in each state
(see [a view field derived from the lifecycle state](../guides/specify/values-and-views.md#a-view-field-derived-from-the-lifecycle-state)):

```yaml ess-check="invariants" ess-expect="synthesizes"
invariants:
  - {any: [state != Closed, quantity > 0]}
```

## Map operators

A fact path used as a key constrains that fact. The value is one of three things:

- a scalar, which means equality: `channel: Web`, `gift: true`, `quantity: 3`;
- a list, which means membership: `channel: [Web, Store]`;
- a mapping of operators. Several operators in one mapping must all hold.

| Operator | Aliases | Operand | Holds when |
|---|---|---|---|
| `eq` | `equals` | a scalar | the fact equals the operand |
| `ne` | `not_equals` | a scalar | the fact differs from the operand |
| `lt` | | a scalar | the fact is less |
| `lte` | `le` | a scalar | the fact is less or equal |
| `gt` | | a scalar | the fact is greater |
| `gte` | `ge` | a scalar | the fact is greater or equal |
| `any_of` | `in`, `one_of` | a list or one value | the fact is one of the values |
| `none_of` | `not_in` | a list or one value | the fact is none of the values |
| `exists` | `defined` | `true` or `false` | the fact is present, or absent for `false`. From `ess/16` also over an `Optional` struct, list or map |
| `truthy` | | any value | the fact is present and truthy |

The comparison operators also accept their symbols as keys (`"=="`, `"<="`, …). A string operand
of a comparison operator follows the compact right-hand-side rule: a bare dotted word is a fact
path. A comparison has no arithmetic beyond one constant offset, so `duration_ms > param.limit_s *
1000` compares with the text `param.limit_s * 1000` and is refused as `type_mismatch`, naming the
offset form and a parameter declared in the unit the field is stored in.

`any_of`, `none_of`, their aliases and `in_ignore_case` hold literal values only
(beyond10x/ess#438). An operand that is one dotted word naming a view parameter or a command input,
`queue_id: {in: param.queues}`, would be the text it spells, so it is refused as `type_mismatch` in
every format and against every field type. Membership in a list the caller sends is a quantifier,
`exists: {in: param.queues, as: q, that: queue_id == q}` (`not:` around it for `none_of`); a
parameter holding one value is compared with `==` or `!=`. The refusal names the form that fits
what the word names.

### The equality shorthand reads a literal

The scalar shorthand `path: value` is the one place a bare dotted word is **not** a path. It is
always a literal. So `{sku: A.1}` compares `sku` with the text `A.1`, while `{sku: {eq: A.1}}` and
`sku == A.1` both read a path `A.1` and are refused here.

```yaml ess-check="when" ess-expect="synthesizes"
when:
  sku: A.1
```

```yaml ess-check="when" ess-expect="refused:ESS-COMMAND-003"
when:
  sku: {eq: A.1}
```

```yaml ess-check="when" ess-expect="synthesizes"
when:
  channel: [Web, Store]
```

```yaml ess-check="when" ess-expect="synthesizes"
when:
  channel: {in: [Web, Store]}
```

```yaml ess-check="when" ess-expect="synthesizes"
when:
  channel: {any_of: [Web, Store]}
```

```yaml ess-check="when" ess-expect="synthesizes"
when:
  channel: {one_of: [Web, Store]}
```

```yaml ess-check="when" ess-expect="synthesizes"
when:
  channel: {not_in: [Phone]}
```

```yaml ess-check="when" ess-expect="synthesizes"
when:
  channel: {none_of: [Phone]}
```

```yaml ess-check="when" ess-expect="synthesizes"
when:
  quantity: {eq: 3}
```

```yaml ess-check="when" ess-expect="synthesizes"
when:
  quantity: {ne: 3}
```

```yaml ess-check="when" ess-expect="synthesizes"
when:
  quantity: {lt: 10}
```

```yaml ess-check="when" ess-expect="synthesizes"
when:
  quantity: {gt: 3}
```

```yaml ess-check="when" ess-expect="synthesizes"
when:
  gift: {truthy: true}
```

```yaml ess-check="filter" ess-expect="synthesizes"
filter:
  note: {exists: true}
```

```yaml ess-check="filter" ess-expect="synthesizes"
filter: defined(note)
```

```yaml ess-check="filter" ess-expect="synthesizes"
filter:
  note: {defined: true}
```

An unknown operator is refused, and the refusal lists the operators the parser knows.

## Quantifiers

A claim about every element of a collection, or about some element of it, needs a quantifier. A
collection is a `List` or a `Map`. Over a `Map` the quantifier binds each **value**, never a key
and never a key/value entry, and no key is reachable: `exists: {in: redirect_uris, as: r, that: r
== input.application}` over a `Map<String, String>` asks whether some value of the map equals
`input.application`, and `r` has the map's value type. Validation types `r` that way, the
conformance evaluator walks the values in key order, and Entity Runtime folds the same values in
its canonical key order. A quantifier is a mapping with exactly three keys:

| Key | Value |
|---|---|
| `in` | the list or map, as a fact path |
| `as` | a one-segment name the body binds each element to |
| `that` | a predicate over that name and any other fact |

`forall` holds when every element satisfies `that`. `exists` holds when at least one element
does. Quantifiers have no compact spelling and may nest.

```yaml ess-check="filter" ess-expect="valid"
filter:
  exists:
    in: tags
    as: t
    that: t == vip
```

```yaml ess-check="invariants" ess-expect="valid"
invariants:
  - forall:
      in: tags
      as: t
      that: t != ""
```

```yaml ess-check="invariants" ess-expect="valid"
invariants:
  - forall:
      in: labels
      as: label
      that: label != ""
```

A bare word on the right of a comparison that names a binder in scope, the quantifier's own or an
outer one, reads that binder. Here no tag may equal any label value:

```yaml ess-check="invariants" ess-expect="valid"
invariants:
  - forall:
      in: tags
      as: t
      that:
        forall: {in: labels, as: l, that: t != l}
```

The equality shorthand and a quoted word are always text, so either one naming a binder in scope is
refused, as a literal naming a field is.

```yaml ess-check="invariants" ess-expect="refused" ess-says="not the binder"
invariants:
  - forall:
      in: tags
      as: t
      that:
        forall: {in: labels, as: l, that: {t: l}}
```

## `.count` and ordinals

A list or a map publishes its size as `<path>.count`, which compares like any number.

```yaml ess-check="filter" ess-expect="valid"
filter: tags.count > 0
```

```yaml ess-check="invariants" ess-expect="valid"
invariants:
  - labels.count >= 0
```

One element of a list is reachable by its position, counted from `0`: `tags.0` is the first
element. A map has no ordinals.

```yaml ess-check="when" ess-expect="synthesizes"
when: tags.0 == vip
```

`.count` on a `String`, or on a newtype of one at any depth, is the text's length in Unicode scalar
values (`ess/11`). `é` written as one character counts 1, and `e` followed by a combining accent
counts 2: this is not a grapheme count. Synthesis witnesses a length guard with a text one character
either side of the literal, drawn from the type's `alphabet:` when it declares one, up to 1024
characters; past that the guard is refused as `ESS-SYNTH-018`. A length is not asserted on a view
row in this suite format, so an invariant that reads one is held where values are built — unless
the same invariant reads an `ess/22` form, such as a [UTF-8 byte
length](#from-ess22-the-utf-8-byte-length-of-a-text), whose suite format carries both. For the
number of bytes a text takes, use `.utf8_bytes`.

```yaml ess-check="when" ess-expect="synthesizes"
when: sku.count > 0
```

Validate type-checks the path. `.count` on a field that is not a list, a map or a String is refused.
So is any other suffix on a list, such as `.length`, and so is a key on a map.

```yaml ess-check="invariants" ess-expect="refused" ess-says="cannot select"
invariants:
  - labels.gold == yes
```

```yaml ess-check="when" ess-expect="refused:ESS-COMMAND-003"
when: quantity.count > 0
```

```yaml ess-check="when" ess-expect="refused:ESS-COMMAND-003"
when: tags.length > 0
```

## Ordering

`<`, `<=`, `>`, `>=` and their operator keys order:

- **numbers** by value;
- **`Timestamp`** by the instant it names, so `+01:00` and `Z` spellings compare correctly. See
  [order two instants](../guides/specify/fields-and-invariants.md#order-two-instants);
- **text**, including newtypes of `String`, byte-wise and lexicographically. `"B" < "a"`, because
  `B` is byte 66 and `a` is byte 97. No locale and no case folding apply.

`Duration` has no ordering. An enum's variants carry no order ESS promises, so test an enum with
membership (`channel: [Web, Store]`), not with `>`.

```yaml ess-check="when" ess-expect="synthesizes"
when: sku < "m"
```

### A `Timestamp` against the current time

From `format: ess/16`, a command outcome's `when:` may order a `Timestamp` input against `now`, the
moment the implementation handles the request, moved by a whole number of seconds, minutes or hours.
That includes the `when:` beside `when_subject_state:`, `when_state_changes:`, `when_subject:` or an
external cause:

```text
when: starts_at < now - 60s
when: expires_at > now + 5m
when: {starts_at: {ge: now - 1h}}
```

`now` goes on the right of `<`, `<=`, `>` or `>=`. The offset is written `<n>s`, `<n>m` or `<n>h`
with no leading zero; there are no days, so write `24h`. From `format: ess/22` a `when_subject:`
predicate and the predicate of an identity-addressed `when_related:` may order a stored
`Timestamp` against `now` too:

```text
when_subject: {predicate: expires_at >= now - 1h}
when_related: {via: input.member_id, predicate: banned_until > now + 30s}
```

One command decision reads one instant: its input guard and every row it reads are decided with
it, over the rows as they were before the outcome. Below `ess/22` such a stored ordering is refused
as `unsupported_format_version`, naming `ess/22`. Anywhere else — an invariant, a view filter, a
selection, a set-effect filter — the operand is refused, because none of those is read while a
request is handled. A view over the last N minutes, or over a range named in a zone, takes the two
instants its caller resolved as `Timestamp` parameters instead, `filter: [started_at >= param.from,
started_at < param.to]` (beyond10x/ess#439). `==` and `!=` against `now` are refused too: an
instant is ordered against the current time, never equated with it. Below `ess/16` the guard is
refused as `unsupported_format_version`. Over a `String`, `now` is still the text `now`.

A generated suite witnesses such a guard a second either side of its boundary and never on it:
`starts_at < now - 60s` is sent `now - 61s` requiring the refusal and `now - 59s` requiring the
other branch. The suite carries each value as a `now_offset`, which the runner turns into an
instant from the wall clock it is given when it sends the command, so a target that handles the
request within a second decides it as required. Entity Runtime has no clock operand and refuses the
guard (`CurrentTimeUnsupported`). A value chosen from a fixed instant the same field is also
ordered against is sent as that instant. Synthesis refuses to witness `now` against a `Timestamp`
inside a structure or a list element (`exists: {in: starts, as: s, that: s > now}`): a `now_offset`
replaces a whole input field. It also refuses a field ordered against `now` and against a fixed
instant between `2019-12-30T23:59:59Z`, the instant it decides values at, and
`2026-09-27T00:00:00Z`: that instant lies on the other side of `now` at every run. See
`docs/design/current-time-guards.md`.

A stored instant is arranged through the input of the command that writes it: the value chosen for
the row is sent to that command as a `now_offset` and travels through its `sets:` into the row,
and the row read back is required to hold it. A stored instant no such input carries — one the
implementation generates, a literal, a converted value, a member inside a structure — is refused
by name rather than decided at the reference instant. A target supplies the decision's instant
from its own clock: the interpreter reads a command clock it is handed once per decision, and a
generated Rust or Go behaviour reads its context's command clock once per decision. With no clock,
a decision that needs one is refused naming the command clock, and every answer decided before it
stands. See `docs/design/expression-family-source22.md`, A3.

### A calendar window

From `format: ess/22` (beyond10x/ess#244), a command guard may hold an instant to a weekly window:
listed weekdays, a time of day from `from` up to `to`, at UTC or a fixed offset. It is admitted
where `now` is — a command outcome's `when:`, its `when_subject:` predicate and an
identity-addressed `when_related:` predicate — in the structured form only:

```text
when:
  window: {at: now, days: [mon, tue, wed, thu], from: "08:00", to: "16:00", offset: "+01:00"}
when_subject:
  predicate:
    window: {at: ready_at, days: [fri], from: "22:00", to: "02:00", offset: Z}
```

`at` is `now`, the decision's one instant, or a `Timestamp` field the site reads. `days` lists at
least one of `mon`, `tue`, `wed`, `thu`, `fri`, `sat` and `sun`, each once. `from` and `to` are quoted
`"HH:MM"`: `from` is inclusive, `to` exclusive, and `24:00` is the end of the day (`00:00` as `to` is
refused naming it). A window whose `from` is after its `to` crosses midnight and belongs to the day
it opens: `days: [fri], from: "22:00", to: "02:00"` holds Friday 22:00 to Saturday 02:00, and not
Friday 01:00. `offset` is `Z` or `±HH:MM`, at most 14 hours either way; `+00:00` is written back as
`Z` and `-00:00` is refused.

A named time zone — `offset: Europe/Berlin`, `offset: UTC`, a `zone:` key — is refused at source,
naming the fixed-offset spelling. A window is evaluated with the same integer arithmetic in Rust, Go
and TypeScript and needs no zone data, so it does **not** follow daylight saving: `08:00 to 16:00 at
+01:00` is 07:00 to 15:00 UTC all year. The instant is compared, never its spelling:
`2020-01-06T02:30:00-05:00` is 08:30 at `+01:00`.

The fact unobserved, an absent `Optional`, text that names no instant and `now` read without a clock
make the window unknown. Below `ess/22` it is refused, naming `ess/22`; an invariant, a view filter,
a selection and a set-effect filter refuse it as `type_mismatch`, and `at: now` where a field or
binder is also named `now` is refused. `window:` without `at` is a constraint on a fact named
`window`, as before.

A generated suite witnesses a window over an input — in a `when:`, or read as `input.<field>` in a
stored row's predicate, at any depth of the guard — a second either side of every `from` and `to` on
the week of Monday 2020-01-06. Each further row spells its instant at an offset under which the
instant's written clock, read as UTC, falls on the other side, so a target comparing spellings fails.
A window over a stored instant is arranged through the input that writes it, on a row for each of
its deciding instants: `from` and `to` on a listed day, a second before `from`, the last second
before `to`, and `from` on each unlisted day. A window in an authored `satisfies:` is refused by
name. A window over `now` is refused by name in synthesis: the target decides
it by its own clock, and no suite step sets that clock. The interpreter decides it at the command
clock it is handed. Generated Rust and Go behaviour leaves a command with a window owed, naming the
window, and Entity Runtime refuses it as `CalendarWindowUnsupported`. See
`docs/design/calendar-window-guards.md`.

## String operators

`starts_with`, `ends_with` and `contains` test a text fact against a literal. They need
`format: ess/8`. An older document that uses one is refused as `unsupported_format_version`, in
every predicate position. They are map-form only: there is no compact form, no alias and no `not_`
spelling, so negate one with `not:`. Several in one mapping are conjoined.

They apply to `String` and to newtypes of `String` at any depth, including through `Optional`.
Any other type is refused, enums, `Uuid`, `Timestamp` and `Duration` included. Matching is
byte-wise and case-sensitive, with no Unicode normalisation: `ABC` does not begin with `a`, and a
decomposed `é` does not begin with a composed one. An unobserved fact is unknown, so neither a
guarded branch nor its negation is taken.

```yaml ess-check="when" ess-expect="synthesizes"
when:
  all:
    - sku: {starts_with: "SKU-"}
    - not: {sku: {contains: "test"}}
    - sku: {ends_with: "0"}
```

```yaml ess-check="when" ess-expect="synthesizes"
when:
  sku: {starts_with: "A", ends_with: "0"}
```

```yaml ess-check="when" ess-expect="synthesizes"
when:
  all:
    - sku: {contains: "RE"}
    - not: {sku: {starts_with: "RE"}}
```

```yaml ess-check="when" ess-expect="synthesizes"
when:
  exists:
    in: tags
    as: t
    that:
      t: {starts_with: "vip"}
```

The operand is the text it spells. It is never a number, a Boolean or a fact path, and quote
characters inside it are part of it: `{starts_with: "+44"}` tests for a leading `+44`, and
`{starts_with: '"x"'}` for a leading `"`. Unquoted, YAML reads `+44` as the number 44, which is
refused.

```yaml ess-check="when" ess-expect="refused:ESS-COMMAND-002" ess-says="quote the literal exactly as written"
when:
  sku: {starts_with: +44}
```

```yaml ess-check="when" ess-expect="refused:ESS-COMMAND-002" ess-says="does not admit"
when:
  channel: {starts_with: "W"}
```

A literal that names a field of the same owner is refused, as for `==`: a string operator compares
with a literal only. The empty literal is refused too, because it holds for every text and its
negation for none. Write `defined(x)` if presence is meant.

```yaml ess-check="when" ess-expect="refused:ESS-COMMAND-002" ess-says="compares with a literal only"
when:
  sku: {contains: gift}
```

```yaml ess-check="when" ess-expect="refused:ESS-COMMAND-007" ess-says="defined(sku)"
when:
  sku: {ends_with: ""}
```

A guard and its negation are still not a complete set of branches. The prover that lets enum
branches go without a default does not read text, so a command whose outcomes are all guarded by
string operators needs a default branch.

The same operators work in every predicate position: invariants, view filters and binding
selections. In a selection the literal is at most 4096 bytes.

```yaml ess-check="filter" ess-expect="valid"
filter:
  sku: {starts_with: "SKU-"}
```

```yaml ess-check="invariants" ess-expect="valid"
invariants:
  - quantity >= 1
  - not: {sku: {contains: " "}}
```

## Case-insensitive operators

`equals_ignore_case` tests a text fact against one text literal, and `in_ignore_case` against a
list of them, ignoring ASCII case. They need `format: ess/15`; an older document that uses one is
refused as `unsupported_format_version`, in every predicate position. Like the string operators
they are map-form only, with no compact form and no negated spelling: negate one with `not:`.

They apply to `String` and to newtypes of `String` at any depth, including through `Optional`.
Folding is ASCII only: `A` to `Z` fold to `a` to `z`, and every other character compares as
itself. So `WEB` equals `web`, but `É` does not equal `é`, `STRASSE` does not equal `straße`, and
the Kelvin sign does not equal `k`. Rust, Go and TypeScript all answer this way. An unobserved fact
is unknown.

```yaml ess-check="when" ess-expect="synthesizes"
when:
  sku: {equals_ignore_case: "sku-1"}
```

```yaml ess-check="when" ess-expect="synthesizes"
when:
  not: {sku: {in_ignore_case: ["a-1", "b-2"]}}
```

Synthesis sends the guarded branch the literal with its ASCII letters in the other case, `SKU-1`
for the first example, which only an implementation that ignores case accepts. It sends the other
side the literal with one character changed, `xku-1`, which folds to nothing the guard names, and,
in a further row, `ſku-1` with the long s U+017F in place of the `s`: ASCII folding refutes it,
and an implementation that folds Unicode case, as `toLowerCase` or `strings.EqualFold` do,
accepts it and fails the scenario. A literal with a non-ASCII letter gets that letter in its other
case instead (`café` becomes `CAFÉ`).

The operand is the text it spells. A number, a Boolean or a literal naming a field of the same
owner is refused. `equals_ignore_case` takes one literal and `in_ignore_case` a list; the other
shape is refused, and so is an empty list, which holds for no text.

```yaml ess-check="when" ess-expect="refused:ESS-COMMAND-002" ess-says="quote the literal exactly as written"
when:
  sku: {equals_ignore_case: 44}
```

```yaml ess-check="when" ess-expect="refused:ESS-COMMAND-002" ess-says="does not admit"
when:
  channel: {equals_ignore_case: "web"}
```

```yaml ess-check="when" ess-expect="refused:ESS-COMMAND-007" ess-says="lists no literal"
when:
  sku: {in_ignore_case: []}
```

As with the string operators, the prover that lets enum branches go without a default does not read
text, so a command guarded only by these operators needs a default branch. They work in
invariants and view filters too. A binding selection refuses them, and so does Entity Runtime
lowering (`CaseFoldUnsupported`), which has no condition that folds case.

```yaml ess-check="filter" ess-expect="valid"
filter:
  sku: {in_ignore_case: ["SKU-1", "SKU-2"]}
```

A generated suite carries one of these operators only in a view expectation. Such a suite takes
`ess-conformance/20`, or `/21` with coverage; a guard over command input is decided when the suite
is generated and does not change its format.

## Text shapes without patterns

ESS has no pattern predicate: there is no regular-expression operator, and `matches:` is refused
with the list of operators there are. A shape a value must have is stated with the constructs
above, and where it is a property of the value rather than of one guard, on its type:

- a fixed beginning, end or part: `starts_with`, `ends_with`, `contains`;
- a length: `.count`, in Unicode scalar values;
- the characters a value may hold: `alphabet:` on a `String` newtype;
- a fixed beginning every value of a type carries: `prefix:` on a newtype;
- a closed set of values: `any_of`, or `in_ignore_case` ignoring case.

"Starts with `GB-` and is nine characters long" is one guard, and synthesis writes a text that
holds and one that does not:

```yaml ess-check="when" ess-expect="synthesizes"
when:
  all:
    - sku: {starts_with: "GB-"}
    - sku.count == 9
```

```yaml ess-check="when" ess-expect="refused" ess-says="starts_with, ends_with, contains, equals_ignore_case, in_ignore_case"
when:
  sku: {matches: "^GB-[0-9]{6}$"}
```

What these do not state is a position-dependent character class, such as "six digits after the
prefix". A pattern predicate would need a portable dialect with a parser of ESS's own, a
translation for every runner, and a matching text and a near miss for every pattern. A regular
expression the system stores and evaluates as data, as an operator of its stored rules, is the
system's evaluator, not a guard of the specification:
[the stored-rules boundary](https://github.com/beyond10x/ess/blob/main/docs/design/stored-rules-boundary.md)
says how such a verdict is specified.

## Comparing a stored field with the input

From `ess/15`, a `when_subject` predicate may compare a stored field of the addressed entity with a
field of the command's input, named with the `input.` prefix: `recording_id != input.recording_id`
ignores a stop for a recording other than the one the call holds. The two sides must have
comparable types, and the input field must be declared by the command. A bare word on the right of
a comparison is a literal, as everywhere, so write the input operand on the right. Under an earlier
header it is refused as `unsupported_format_version`.

The finite prover cannot enumerate two facts against each other, so a command with such a guard
needs a default branch, even over an enum. Synthesis arranges the row and sends the input once
equal to the stored value and once different, one scenario per branch. The input side may name a
member of a struct input, such as `input.publication.expected_fence`. A wrong-state scenario sends
the input that misses such a guard on the row it arranges.

Where the stored side is a quantifier over a stored list or map, the stored value is each element
the arranged row holds, a map's values: the input is sent once equal to an element the row holds
and once equal to none, so `exists: {in: redirect_uris, as: r, that: r == input.application}` is
witnessed on both sides and a target that ignores the entries, reads the map's keys, or reads only
whether it is empty fails. The row is arranged with several entries where the collection is written
from an input: where one element decides the quantifier (an `exists` that holds, a `forall` that
fails), that element is neither the first nor the last in key order, so a target reading only one
value, or `forall` for `exists`, fails; where the whole collection decides it, the row holds two or
more elements. Where no arrangement reaches that shape, the row with one entry is used. An entity
that declares a stored field named `input` keeps reading `input.<member>` as that field.

## A guard over another entity's row

From `ess/18`, a branch may be guarded by one row of another entity: the row whose identity an input
field carries. `when_related: {via: input.tenant, exists: false}` is taken when no row carries
`input.tenant`; `when_related: {via: input.tenant, predicate: redirect_client != input.client}` is
taken when the row exists and the predicate over its stored fields holds. `input.tenant` must be an
input typed as exactly one entity's identity; from `ess/22` it may be `Optional<…>` of that
identity, and the guard is then checked only when present (see
[an Optional reference](#an-optional-reference)). The guard composes with `when:` and with any
subject a branch names, a `creates:` included, and a refusal may carry it without naming one. A
command declares at most one `exists: false` branch over a row, and one wherever a predicate reads
that row, because a missing row selects no predicate and never the default. Through `ess/21` a
command reads one related row; from `ess/22` it may read several (see
[several related rows](#several-related-rows)).

From `ess/20`, the predicate may also read the related row's held lifecycle state as `state`, as a
`when_subject` predicate reads the addressed subject's from `ess/18`: a release published only for
a candidate in state `Accepted` refuses with `when_related: {via: input.candidate, predicate: state
!= Accepted}` beside a default that publishes (beyond10x/ess#229). `state` is typed by the related
entity's lifecycle, so a state it does not declare is refused, and it enters the same related-row
× input partition as the stored fields. Under `ess/18` and `ess/19` it is refused as
`unsupported_format_version`, naming `ess/20`. Synthesis drives the related row along its
lifecycle to a state on each side of the predicate: the refusal is sent for a candidate still
`Proposed` between two `Accepted` decoys, the default for an `Accepted` one between two `Proposed`
decoys, so a target that checks only that the row exists, or reads another row's state, fails.
Under `ess/20`, where accepting a candidate itself reads a related candidate — a parent that must
not be accepted yet — synthesis arranges that parent one level deep, fresh in its initial state,
and goes no deeper; a branch only a deeper arrangement could reach is refused with
`ESS-SYNTH-003`, saying so. A document under an earlier header synthesizes the suite it did:
there such a move is not arranged, and its branches are refused as before.

A missing row is answered by the `exists: false` branch before any other branch, whatever the
input: an `exists: false` branch therefore carries no `when:`, and an accepting `when:` branch may
overlap it. The one answer before it is `existing_instance:`, which may sit in the same command: an
identity a record already carries is refused as taken before the related row is read.

Synthesis sends the `exists: false` branch an identity no row carries, beside two rows of the entity
that carry others and with an input an accepting `when:` branch would take. It sends every other
branch the identity of a row it creates between two decoys, arranged so that each predicate is
witnessed true in its own scenario and false in the default's, including a predicate over a stored
field alone. A predicate with two or more `all`/`any` children is sent once more per child, on a
row where that child alone decides it, so a target that drops one conjunct or one disjunct fails;
where no row can isolate a child, synthesis reports `ESS-SYNTH-003` for the branch. Beside
`existing_instance:` it sends a taken identity naming a related row that does not exist, and
requires the `existing_instance:` refusal. Under an earlier header the key is refused as `unsupported_format_version`.

### An Optional reference

From `ess/22` (beyond10x/ess#304), `via: input.<field>` may name an input declared
`Optional<…>` of the other entity's identity, and the guard is checked only when present. An absent
reference reads no row and selects no `when_related` branch: it is not a missing row, so
`exists: false` does not answer it. Selection carries on without the related row: the addressed
row's existence and held state answer as before, then the branches that read no related row — an
input-guarded `when:` branch or the default. Those branches must answer the absent case exactly
once, or validate refuses the command with `non_exhaustive_branches` naming the absent reference. A
present reference is read as a required one is: an identity no row carries still selects
`exists: false` before any other branch, and a present row's predicate refusal still follows the
held state where the command declares `wrong_state:`. The declared `Optional<…>` type is kept in
the compiled model. Under `ess/21` and earlier an Optional reference is refused as
`unsupported_format_version`, naming `ess/22`. Generated documentation and OpenAPI say the
reference is checked only when present.

Synthesis witnesses the absent case on the scenario of the branch it selects, on a further
instance sent without the reference, between two rows of the related entity that a predicate
refusal would select; a target that reads absence as a missing row, or reads some row of the
entity, fails it. Where an unknown addressed identity is answered by `wrong_state`, synthesis sends
it without the reference. The present cases are witnessed as for a required reference.

### Several related rows

From `ess/22` (beyond10x/ess#283), a command may guard on more than one related row, each named
by an input of its own, required or `Optional<…>`:

```text
- name: no-such-switch
  when_related: {via: input.switch, exists: false}
  error: demo.run.NoSuchSwitch
- name: switch-paused
  when_related: {via: input.switch, predicate: state == Paused}
  error: demo.run.SwitchIsPaused
- name: no-such-capability
  when_related: {via: input.capability, exists: false}
  error: demo.run.NoSuchCapability
- name: capability-revoked
  when_related: {via: input.capability, predicate: state == Revoked}
  error: demo.run.CapabilityIsRevoked
- name: started
  creates: demo.run.Run
  instance: run_id
```

Each row is checked as a lone row is: its predicates against its own entity, at most one
`exists: false` branch over it, and one wherever a predicate reads it. Declaration order decides
between rows. A missing row answers first: the rows are read in the order their `exists: false`
branches are declared, and the first missing one answers, before any present row's predicate. Then,
after the addressed row's existence and held state, the first declared predicate refusal whose
predicate holds answers, before every accepting branch: a paused switch under a revoked capability
is refused as `switch-paused`. Two refusals over one row that both hold, or two accepting branches
over different rows, are still `conflicting_declaration`. Validation covers every row's fields, and
an Optional row's absence, crossed with the input, up to 64 joint cases. Past that cap, or over a
domain it cannot enumerate, a command needs a default; beside one, two accepting branches over
different rows that can both hold are still refused as `conflicting_declaration` rather than
admitted unchecked. A stored-field `via` stays the command's only
row. Under `ess/21` and earlier a second row is refused as `unsupported_format_version`, naming
`ess/22`.

Synthesis witnesses each branch with every other row present and arranged so that nothing the
order answers first is selected there — for a predicate refusal, no earlier-declared refusal over
that row; for `exists: false`, any row — and sends each refusal the overlaps the order decides:
both rows missing for the first declared `exists: false`, an earlier Optional reference left out
beside a missing row, a refusing row beside a missing one, and two refusing rows for the first
declared refusal. A target that reads the rows in another order, or ignores one, fails.

### A stored reference

From `ess/22` (beyond10x/ess#304), `via` may name a stored field of the subject the command
addresses, written bare: `via: blocked_by`. The field is read as the subject held it just before
the branch, and is typed as the other entity's identity or `Optional<…>` of it:

```text
- name: blocker-missing
  when_related: {via: blocked_by, exists: false}
  error: demo.tasks.BlockerMissing
- name: blocked
  when_related: {via: blocked_by, predicate: state != Done}
  error: demo.tasks.Blocked
- {name: wrong-state, wrong_state: true, error: demo.tasks.TaskStateConflict}
- name: completed
  moves: demo.tasks.Task.complete
  instance: task_id
```

A task is completed only once the task its stored `blocked_by` names is `Done`. The stored field is
read after the input-guarded refusals, the addressed row's existence and its held state, and
before every accepting branch: an unknown task or a task already `Done` is answered first, then an
identity no task carries selects `blocker-missing`, then a stored task that is not `Done` selects
`blocked`. An absent `blocked_by` reads no row and selects no `when_related` branch, as an absent
Optional input does. `wrong_state:` may sit beside it, unless an accepting `when_related` branch
moves the task: that is refused as `conflicting_declaration`, as for an input reference. The guard
is refused on a command that creates its subject, since no row holds the field before the branch,
on a command that addresses no existing subject through its input, and on a field the subject does not store. Under `ess/21` and
earlier it is refused as `unsupported_format_version`, naming `ess/22`.

Synthesis arranges the related row, then rewrites the act that created the subject so that the
stored field names it, and sends the command naming only the subject. A stored task that is not
`Done` sits between two `Done` decoys, a `Done` one between two open decoys, so a target that reads
another task's state, the subject's own state, or refuses whenever a blocker is stored fails. A
stored identity no row carries is witnessed only where the act that writes the field stores it
unchecked. Where every writer refuses such an identity, synthesis reports `ESS-SYNTH-003` for the
`exists: false` branch, naming it unreachable. Generated Rust, Go, Web and clap targets keep the
command a hand-written obligation, and their contract states this order.

## A guard over the rows a selector selects

From `ess/22` (beyond10x/ess#228, beyond10x/ess#299), a branch may be guarded by the rows of an
entity that a predicate selects, rather than by one row an input identity names:

```text
- name: claims-taken
  when_related:
    entity: demo.binding.Identity
    where: {all: [tenant_id == input.tenant_id, sub == input.sub, defined(org_id), org_id == input.org_id]}
    exists: true
  error: demo.binding.ClaimsAlreadyExists
- name: bound
  creates: demo.binding.Identity
  instance: user_id
- {name: already-bound, existing_instance: true, error: demo.binding.IdentityAlreadyExists}
```

No two users of one tenant carry the same claims: a second bind with claims another user carries is
refused, and `existing_instance:` still answers first for the same user. `where` reads each
candidate row's declared fields, identity and held lifecycle state `state` bare, the input under
`input.`, and on a command whose branches address one existing subject through the input, that
subject as it was before the branch under `subject.`; a `subject.` read on a command that only
creates is refused. `now` is the decision's one instant, as in every other guard of the command.
The test is exactly one of `exists: true|false`, `count: {<op>: <n>}` with `eq`, `ne`, `lt`, `lte`,
`gt` or `gte` and a nonnegative whole number, or `forall: <predicate>`, which every selected row
satisfies and which is true of no rows. `via` beside `entity`, `where` without `entity`, no test,
two tests, and `where: always` are refused where the guard is written; under `ess/21` and earlier
the form is refused as `unsupported_format_version`, naming `ess/22`.

The rows are the store just before the branch is selected: a row the outcome creates or changes is
never one of its own candidates, and no order among the rows decides anything. A row whose
membership is unknown is kept as a possible member, never dropped: an `Optional<…>` key compared
without a `defined()` conjunct leaves an absent row's membership unknown, and then the decision is
undetermined, so write `defined(org_id)` beside `org_id == input.org_id` where `org_id` may be
absent. A row-set refusal answers after the input-guarded refusals, `existing_instance:`, and the
addressed row's existence and held state, before every accepting branch; a row-set branch that
accepts is taken in declaration order among the accepting branches; the default only where every
guard before it is decidedly false. In this release a command reads one kind of related row: a
row-set guard beside an identity-addressed `when_related:` or a `when_subject*` guard is refused as
`unsupported_construct`. Without a default, the count tests over one selector must answer every
number of rows, or validate refuses the command as `non_exhaustive_branches`; a branch every count
of which an earlier branch over the same rows answers is `unreachable_branch`.

A value may read one field of the one row a selector selects, in `sets:` and `payload:` (see
[the related value sources](../guides/specify/values-and-views.md)):

```text
- name: ambiguous
  when_related:
    entity: demo.jobs.Attempt
    where: {all: [worker_id == input.worker_id, batch_id == input.batch_id]}
    count: {gt: 1}
  error: demo.jobs.Ambiguous
- name: started
  when_related:
    entity: demo.jobs.Attempt
    where: {all: [worker_id == input.worker_id, batch_id == input.batch_id]}
    count: {eq: 0}
  creates: demo.jobs.Attempt
  instance: attempt_id
  sets: {worker_id: input.worker_id, batch_id: input.batch_id, delay: 0}
- name: retried
  creates: demo.jobs.Attempt
  instance: attempt_id
  sets:
    worker_id: input.worker_id
    batch_id: input.batch_id
    delay:
      related:
        entity: demo.jobs.Attempt
        where: {all: [worker_id == input.worker_id, batch_id == input.batch_id]}
        field: delay
```

Exactly one selected row supplies the value, read at the field's declared type, `Optional<…>`
included; none or several supply no value and no transition, and no first or latest row is chosen,
so a command declares the branches for those counts itself, as above.

Synthesis arranges the rows through the declared creating commands: one decoy per conjunct of the
selector, refuting that conjunct alone, then as many rows as the branch needs — one, two, none or
three, in that order — and, for a `forall`, one of them refuting it where the branch needs it false.
Where a row is selected, the decoys are arranged again after it, so the selected row is neither the
oldest nor the newest record holding a value the selector compares, and a target that keeps one
record per such value fails. A copied value differs from every decoy's and from zero. The branch the rows decide is checked again
over every step of the scenario, a row arranged through the command under test included, and
every other scenario sending a row-set command keeps only the branch its rows decide. A selector
needs an equality between a `String` or `Uuid` field and the input or the subject, so a scenario
counts only its own rows. From `ess/23` a `String` or `Uuid` member of a struct identity counts as
such a field: `at.region == input.place.region` beside `at.shelf == input.place.shelf` selects the
record the place names, its decoys are arranged under other identities, and no scenario creates an
identity a row already carries. Compare the members one by one: `at == input.place` compares two
structs and is refused as `ESS-COMMAND-002`. Without such an equality, and on a command reading
two selectors, synthesis reports
`ESS-SYNTH-001` naming the branch. Generated Rust and Go keep such a command a hand-written
obligation, naming the row set; Entity Runtime refuses it as `RowSetUnsupported`.

## What synthesis can witness

`ess verify conform synthesize` builds one scenario for every branch of every command. It fills
every input field with a value of the field's type. For a branch's guard it tries the literals the
guard itself writes, one on each side. When no candidate reaches a branch, synthesis does not guess.
It reports a refusal naming the scenario it could not build, and `synthesize` still exits `0`.

| Guard over command input | Both branches synthesized? |
|---|---|
| comparison, membership, `all`/`any`/`not`/`none` over enums, numbers, booleans and text equality | yes |
| a bare path over a `Boolean` | yes |
| a bare path over text (`when: sku`) | no. The falsy side needs an empty text, which is not a literal of the guard. `ESS-SYNTH-003` |
| `defined(x)`, `not defined(x)`, `x: {exists: …}` over an `Optional` | yes. One candidate omits the field. |
| `.count`, `exists`, `forall` over an input list | yes. The list is built from the guard's own literal: a one-element list satisfies the guard, and `[]` or a list of other text refutes it. |
| `.count`, `exists`, `forall` over an input map | yes. The map holds one entry, and its value is tried at the guard's own literal as a list element is. |
| a list element by position (`tags.0`) | yes |
| text ordering | yes, byte-wise |
| `==` or `!=` between two input fields, at any depth (`owner == ticket.owner`) | yes. Each side is also tried at the other's value. |
| `starts_with`, `ends_with`, `contains` | yes. The candidates are the literal, the guard's own literals composed around the field's text, and the literal with one character changed. |
| `all`/`any` over many fields (`all: [any: [a > 10, b > 10], c > 10]`) | within two limits. Synthesis first tries up to 64 candidates in a fixed order. If none fits, it solves the guard from its own literals, one field at a time, or one group at a time for fields compared with each other, and tries up to 64 more. A guard past either limit is refused with `ESS-SYNTH-003`. First, each goal is broken down at most 64 times, and each `any` is tried first child first, so a guard that needs many disjunctions to take a later child can be refused. Second, a field compared only with other fields gets its base value, 0 and -1, so a strict chain over four such fields is refused. Where none of those fits either, a `Decimal` is also tried at the exact midpoint of every two adjacent literals it is compared with, so `amount > 0.1 and amount < 0.2` is met by `0.15`. |

Every `all`/`any` with two or more children is witnessed once per child, at any depth. An `any`
(or an `all` under `not`) sends its branch one more input per child, where that child alone holds;
an `all` (or an `any` under `not`) sends the default branch one more input per child, where that
child alone fails. Below the top level, the input must also be one where that connective decides
the whole guard. So a target that writes `or` for a nested `and`, such as
`any: [all: [a, b], all: [a, c]]`, fails a scenario.

Every input also satisfies the invariants over it. That covers the invariants of each struct the
input holds, and those of each entity a branch copies the input into. For
`sets: {fingerprint: input.fingerprint}` beside the entity invariant
`fingerprint.version == "v1"`, every input is sent with `fingerprint.version` at `"v1"`. The
invariants' own literals are tried first. For a comparison of two fields, each field is also tried
at the other's value and, for an order between numbers, one either side of it. A list whose
`.count` an invariant reads is tried at the lengths the invariant names. Fields no invariant
reads keep their own values.

Each input is held to the invariants of the branches it can reach, not of every branch at once.
Two branches may copy one input into two entities whose invariants disagree, and each branch gets
an input its own entity accepts. When a guard moves a field that an invariant ties to another,
such as `origin == "eu"` beside `origin == route`, the other field is moved with it. An input that
still breaks an invariant is never sent. A branch whose entity invariants no input of this bounded
search meets is refused with `ESS-SYNTH-003`, and the refusal names each invariant.

A refusal with a `when:` over the input is taken before any accepting branch whose guard it
overlaps. Synthesis holds a target to that twice. An accepting branch's witness refutes every such
refusal, with or without a default. And each such refusal is sent again at every overlap point, and
the refusal is required there. For `closed: open == false` and `id-required: ticket_id == ""`,
the `id-required` scenario also sends `{ticket_id: "", open: false}`. This holds for the `when:`
beside a `when_subject:`, sent for a ticket the stored guard admits (or for no ticket, where the
refusal reads the identity itself), and for an external branch's
`when:`, sent without asking the provider. A branch is refused with `ESS-SYNTH-003`, naming that
refusal, when every candidate the search tries for it is claimed by the refusal. The search is
bounded, so an input it does not try may still reach the branch.

Two accepting branches with a `when:` over the input may overlap too. The first declared whose guard
holds answers. For `small: amount < 100` written before `flagged: amount > 50`, the input
`{amount: 75}` takes `small`. Synthesis holds a target to that the same two ways. A later branch's
witness refutes every accepting branch declared before it, beside a default. And the first
branch's scenario also sends an input in each overlap and requires that branch there, so a target
answering `flagged` for `75` fails. An external branch takes the same place in that order: an
accepting branch declared before it answers an input its guard claims, whatever the provider says,
so its scenario is sent an input outside that guard. A branch is refused with `ESS-SYNTH-003`,
naming the earlier branch, when every candidate the search tries for it is claimed by an earlier
one.

An overlap is searched over the guards' own literals, and for a `Decimal` also at the midpoint of
every two adjacent ones, so `11 < amount < 12` is sent `11.5`. An overlap the scenario of the branch
taken first does not send is listed as a note naming both branches, unless the candidates cover
every region the literals divide the input into and none lies in both guards. That happens where
no candidate reaches it, or where the branch is arranged over a stored row, a replay or a preserved
subject, or in a held state another branch also claims.

```yaml ess-check="when" ess-expect="unwitnessed:ESS-SYNTH-003"
when: sku
```

```yaml ess-check="when" ess-expect="synthesizes"
when: defined(coupon)
```

```yaml ess-check="when" ess-expect="synthesizes"
when: not defined(coupon)
```

```yaml ess-check="when" ess-expect="synthesizes"
when:
  coupon: {exists: false}
```

```yaml ess-check="when" ess-expect="synthesizes"
when: tags.count >= 1
```

```yaml ess-check="when" ess-expect="synthesizes"
when:
  exists:
    in: tags
    as: t
    that: t == vip
```

```yaml ess-check="when" ess-expect="synthesizes"
when:
  forall:
    in: tags
    as: t
    that: t != spam
```

An entity invariant is checked by reading it off a view that publishes every field it reads. That
check reads scalar view fields only. An invariant over a list (`.count`, `forall`, `exists`)
therefore validates, but synthesis refuses the scenarios that would check it with
`ESS-SYNTH-011`. The same happens to any invariant whose fields no view publishes.

```yaml ess-check="invariants" ess-expect="unwitnessed:ESS-SYNTH-011"
invariants:
  - tags.count >= 0
```

A view `filter` decides which rows a scenario expects to read. After a command, a scenario always
knows the entity's lifecycle state, and every field the command set from values it sent, read at
the field's declared type. A list set from the input is one of them: `tags: input.tags` sends a
list, so `tags.count` and a quantifier over `tags` are decided, and the scenario asserts that the
view shows the row or leaves it out.

```yaml ess-check="filter" ess-expect="synthesizes"
filter: tags.count > 0
```

A filter over a field the creating command sets, such as `source == "web"`, is decided from the
value the scenario sent. Where the scenario's own row does not meet it, and no row the default
input creates ever does, the view is asserted over a row that does as well: synthesis creates one
more instance with an input chosen toward the filter and expects the view to contain it, beside
excluding the scenario's own row. A filter using `equals_ignore_case` or `in_ignore_case`, negated
or not, gets one more instance in every scenario that reads the view, whatever its own row holds:
one carrying the literal in its other ASCII case (`WEB` for `web`), asserted as the filter decides
it. The view is expected to contain it under `equals_ignore_case` and to exclude it under
`not: {equals_ignore_case: …}`, so an implementation that compares bytes fails either way. Where no
creating command can produce such a row, or the view does not project the entity's identity and
so cannot tell the rows apart, the scenario keeps what it asserts about its own row alone, and does
not assert that such a view holds no rows once another row it admits has been created.

A filter that compares a value no scenario binds, such as `note`, which no command sets, is
undecided, and synthesis refuses every scenario that reads the view with `ESS-SYNTH-005`. Filter on
`state` wherever the rule allows it.

```yaml ess-check="filter" ess-expect="unwitnessed:ESS-SYNTH-005" ess-says="note == hello"
filter: note == hello
```

## Limits

Nesting of `all`, `any`, `not`, `none` and quantifiers, in either spelling, stops at 32 levels.
Deeper predicates are refused.
