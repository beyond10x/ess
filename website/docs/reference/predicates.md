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
| a command outcome's `when_subject: {predicate: …}` (`ess/9`) | the declared stored fields of the entity the command addresses, read just before the command selects a branch; from `ess/15` also the command's input, as `input.<field>` | Not `state`. The input only through the `input.` prefix; see [comparing with the input](#comparing-a-stored-field-with-the-input). Conjunctive with `when`. A refusal may carry it without naming a subject; it reads the one its sibling branches name. |
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

`.count` on a `String`, or on a newtype of one at any depth, is the text's length in Unicode scalar
values (`ess/11`). `é` written as one character counts 1, and `e` followed by a combining accent
counts 2: this is not a grapheme count. Synthesis witnesses a length guard with a text one character
either side of the literal, drawn from the type's `alphabet:` when it declares one, up to 1024
characters; past that the guard is refused as `ESS-SYNTH-018`. A length is not asserted on a view
row in this suite format, so an invariant that reads one is held where values are built.

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
  [order two instants](../guides/write-a-specification.md#order-two-instants);
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
with no leading zero; there are no days, so write `24h`. Anywhere else — an invariant, a view
filter, a selection, a `when_subject:` predicate over stored fields — the operand is refused,
because none of those is the guard over a request's input read while it is handled. `==` and `!=`
against `now` are refused too: an instant is ordered against the current time, never equated with
it. Below `ess/16` the guard is refused as
`unsupported_format_version`. Over a `String`, `now` is still the text `now`.

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

## Comparing a stored field with the input

From `ess/15`, a `when_subject` predicate may compare a stored field of the addressed entity with a
field of the command's input, named with the `input.` prefix: `recording_id != input.recording_id`
ignores a stop for a recording other than the one the call holds. The two sides must have
comparable types, and the input field must be declared by the command. A bare word on the right of
a comparison is a literal, as everywhere, so write the input operand on the right. Under an earlier
header it is refused as `unsupported_format_version`.

The finite prover cannot enumerate two facts against each other, so a command with such a guard
needs a default branch, even over an enum. Synthesis arranges the row and sends the input once
equal to the stored value and once different, one scenario per branch. An entity that declares a
stored field named `input` keeps reading `input.<member>` as that field.

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
| `all`/`any` over many fields (`all: [any: [a > 10, b > 10], c > 10]`) | within two limits. Synthesis first tries up to 64 candidates in a fixed order. If none fits, it solves the guard from its own literals, one field at a time, or one group at a time for fields compared with each other, and tries up to 64 more. A guard past either limit is refused with `ESS-SYNTH-003`. First, each goal is broken down at most 64 times, and each `any` is tried first child first, so a guard that needs many disjunctions to take a later child can be refused. Second, a field compared only with other fields gets its base value, 0 and -1, so a strict chain over four such fields is refused. A value none of the literals leads to, such as `amount > 0.1 and amount < 0.2`, is refused too. |

A refusal with a `when:` over the input is taken before any accepting branch whose guard it
overlaps. Synthesis holds a target to that twice. An accepting branch's witness refutes every such
refusal, with or without a default. And each such refusal is sent again at every overlap point, and
the refusal is required there. For `closed: open == false` and `id-required: ticket_id == ""`,
the `id-required` scenario also sends `{ticket_id: "", open: false}`. This holds for the `when:`
beside a `when_subject:`, sent for a ticket the stored guard admits (or for no ticket, where the
refusal reads the identity itself), and for an external branch's
`when:`, sent without asking the provider. A branch every input of which such a refusal claims is
refused with `ESS-SYNTH-003`, naming that refusal.

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
