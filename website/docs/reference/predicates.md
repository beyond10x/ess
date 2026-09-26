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
| a command outcome's `when_subject: {predicate: …}` (`ess/9`) | the declared stored fields of the entity the command addresses, read just before the command selects a branch | Not the input and not `state`. Conjunctive with `when`. A refusal may carry it without naming a subject; it reads the one its sibling branches name. |
| an entity's `invariants` | the entity's own fields | Checked after every branch that creates or changes the entity. A required field an invariant reads must be set by every `creates:` branch, or declared `Optional<…>`; otherwise validate refuses it with `ESS-COMMAND-018`. |
| a struct type's `invariants` | the struct's own fields | Same grammar, checked against the type. |
| a newtype's `invariants` | the wrapped value, as `value` | For example `value != ""` on a newtype of `String`. |
| a view's `filter` | the source entity's fields and its lifecycle `state` | Selects the rows the view returns. |
| a binding selection's `where` | one list item's fields | A bounded fragment: presence, typed equality and inequality, and `all`/`any`/`not`. See [select ordered records in a binding](../guides/write-a-specification.md#select-ordered-records-in-a-binding). |

Two outcome keys that look like guards are **not** predicates:

- `when_subject: {field: <enum field>, equals: <variant>}` (`ess/6`) is one enum field equal to one
  variant. Its other shape, `when_subject: {predicate: …}` (`ess/9`), is a predicate and is listed
  above. A mapping that writes keys of both shapes is refused while the document is read, and the
  predicate shape under a header older than `ess/9` is refused with `unsupported_format_version`.
- `when_subject_state` is one lifecycle state name.

A binding's `when:` names its cause, such as `periodic:`. It is not a predicate.

A predicate is evaluated in three-valued logic: true, false, or unknown when a fact it reads was
never observed. Only true satisfies a guard. An unobserved collection is unknown, not empty.

## The model the examples use

Every fragment below is placed into this one specification before it is run. A `when:` fragment
replaces the guard of outcome `placed`. An `invariants:` fragment replaces the invariants of
`shop.order.Order`. A `filter:` fragment replaces the filter of `shop.order.OpenOrders`.

```yaml ess-check="model"
format: ess/8
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
| presence, `defined` or `exists` | `defined(coupon)`, `exists(coupon)` | the fact is present. |
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
4. Anything else is text: `channel == Web`.

A right-hand side without a dot is therefore never a field. To compare two fields, put them in one
struct and compare its members, for example `window.ends_at > window.starts_at`. See
[order two instants](../guides/write-a-specification.md#order-two-instants).

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

### Absence is not `null`

An unquoted `null` on the right of `==` or `!=` is refused, with a hint that names `defined(x)` or
`not defined(x)`. Test presence with `defined`. A quoted `"null"` is still the four-character text.

```yaml ess-check="when" ess-expect="refused:ESS-SPEC-017" ess-says="defined(coupon)"
when: coupon == null
```

```yaml ess-check="when" ess-expect="synthesizes"
when: coupon == "null"
```

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
| `exists` | `defined` | `true` or `false` | the fact is present, or absent for `false` |
| `truthy` | | any value | the fact is present and truthy |

The comparison operators also accept their symbols as keys (`"=="`, `"<="`, …). A string operand
follows the compact right-hand-side rule: a bare dotted word is a fact path.

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
collection is a `List` or a `Map`. Over a `Map` the quantifier binds each value, and no key is
reachable. A quantifier is a mapping with exactly three keys:

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

Validate type-checks the path. `.count` on a field that is neither a list nor a map is refused.
So is any other suffix on a list, such as `.length`, and so is a key on a map.

```yaml ess-check="invariants" ess-expect="refused" ess-says="cannot select"
invariants:
  - labels.gold == yes
```

```yaml ess-check="when" ess-expect="refused:ESS-COMMAND-003"
when: sku.count > 0
```

```yaml ess-check="when" ess-expect="refused:ESS-COMMAND-003"
when: tags.length > 0
```

## Ordering

`<`, `<=`, `>`, `>=` and their operator keys order:

- **numbers** by value;
- **`Timestamp`** by the instant it names, so `+01:00` and `Z` spellings compare correctly. See
  [order two instants](../guides/write-a-specification.md#order-two-instants);
- **text**, including newtypes of `String`, byte-wise and lexicographically. `"B" < "a"`, because
  `B` is byte 66 and `a` is byte 97. No locale and no case folding apply.

`Duration` has no ordering. An enum's variants carry no order ESS promises, so test an enum with
membership (`channel: [Web, Store]`), not with `>`.

```yaml ess-check="when" ess-expect="synthesizes"
when: sku < "m"
```

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
| a list element by position (`tags.0`) | yes |
| text ordering | yes, byte-wise |
| `starts_with`, `ends_with`, `contains` | yes. The candidates are the literal, the guard's own literals composed around the field's text, and the literal with one character changed. |

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
knows the entity's lifecycle state. A filter over a value no scenario binds, such as an entity list,
is undecided, and synthesis refuses every scenario that reads the view with `ESS-SYNTH-005`. The
list filters above validate but are refused this way. Filter on `state` wherever the rule allows
it.

```yaml ess-check="filter" ess-expect="unwitnessed:ESS-SYNTH-005" ess-says="tags.count"
filter: tags.count > 0
```

## Limits

Nesting of `all`, `any`, `not`, `none` and quantifiers, in either spelling, stops at 32 levels.
Deeper predicates are refused.
