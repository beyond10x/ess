# Read-API view idioms

Status: decision record for requests declined with an idiom (beyond10x/ess#439, #441, #442, #443,
#444, #446, #447). Each section names what was asked, the construct ESS already has that states
it, and the shape to start from if the request is reopened. No format changes here, except the
last section, "An identity no row carries", which adds the view key `absent:` in `ess/25`
(beyond10x/ess#533).

Every idiom has a validated model in `docs/design/read-api-view-idioms.example/`, one file and one
domain per idiom (`system.yaml` carries the header). The held-row idiom of #442 is a conformance
fixture instead, because its claim is decided by running the suite.
`crates/verify/ess-conformance/tests/read_api_view_idioms.rs` holds each section to its phrases,
validates and synthesizes the example, and pins every refusal quoted below to today's message.

```console
ess specify validate --path docs/design/read-api-view-idioms.example
ess verify conform synthesize --path docs/design/read-api-view-idioms.example --out suite.json
ess verify conform run --target interpreted --report-format 2 \
  --path docs/design/read-api-view-idioms.example \
  --scenarios docs/design/read-api-view-idioms.scenarios/latest-tie.yaml
```

Synthesis refuses three views of the example, each on purpose: `idioms.fold.NonZeroTotalByLabel` and
`idioms.fold.NonZeroTotalBySession` (`ESS-SYNTH-017`, see the per-key total section) and
`idioms.ratio.LongCalls` (`ESS-SYNTH-017`: its parameter is read other than by `field ==
param.name`).

## Derived values over aggregates are the consumer's

Asked (#441): a view field computed from two measures of the same row, such as a ratio with a
stated rounding and a value for a zero denominator, or a difference, and a filter that scales a
parameter (`duration_sum > param.greater_than * 1000`).

Decision 3 of 2026-09-25 (`docs/design/aggregate-views.md`, decisions; "Out of scope") stands:
there are no ratios or differences between aggregates. The idiom is to return both operands as
conditional measures in one row, and the consumer derives the ratio or difference with its own
rounding and its own rule for a zero denominator. `idioms.ratio.QueueOutcomes` returns `queued =
count` and `abandoned = count where abandoned == true`, one row per queue. The suite checks both
numbers exactly, and the abandonment rate is `abandoned / queued` in the client.

Refused today, and kept refused:

- `aggregate: {divide: […]}`: "unknown variant `divide`, expected one of `count`, `count_distinct`,
  `sum`, `min`, `max`, `avg`".
- A field with neither `aggregate:` nor a group key: "promises an observation nothing produces"
  (`ESS-VIEW-001`) and "is neither an aggregate nor a group key" (`ESS-VIEW-005`).

A parameter is declared in the stored unit, and the adapter converts the wire value before the read.
`idioms.ratio.LongCalls` declares `limit_ms` and filters `duration_ms > param.limit_ms + 1000`: one
constant offset is all a comparison adds (`website/docs/reference/predicates.md`, "one constant
offset"). A scale is not an expression: `param.limit_s * 1000` is read as a text literal and refused
as `type_mismatch`.

If decision 3 is reopened, start from a `ratio:` aggregate: `sum` or `count` over `sum` or `count`,
`Decimal`, half-even at 6 digits like `avg`, absent where the denominator is zero. It needs `ess/23`,
a suite pair carrying computed values, a diff classification and every target.

## The latest earlier row is a held row

Asked (#442): an end record derives its duration from the latest earlier matching begin record:
carried where the end has one, derived otherwise as `end.at - begin.at` clamped at 0, dropped where
no begin matches. The proposal was a value reading the latest or earliest row a filter selects,
ordered by a field.

A row-set read chooses no row by order: "There is no implicit latest row"
(`docs/design/filtered-related-reads.md`), and a latest-row implementation is one of its named
failing controls. That rule stands, so no row order is chosen here either. The idiom makes the latest
begin a row the specification holds:

- The begin is held as one row per correlation key, written by a create-or-update pair
  (`unknown_instance: true` beside `updates:`). The row's identity is that key: the caller composes
  it from the correlation fields, one key per (customer, session), so a later begin of the pair
  addresses the held row and replaces it. The composition is the caller's contract, and the model
  does not check it: a begin sent under a second key for the same pair is a second row, and the
  end refuses the two as ambiguous rather than choose one.
- The end reads that row through a row-set selector over the correlation fields, with each count its
  own branch: `count: {gt: 1}` refuses as ambiguous, `count: {eq: 0}` drops the end, a `forall`
  over `began_at >= input.at` clamps the duration to 0, and the default branch copies `began_at` with
  `{related: {entity, where, field}}`.
- An end that carries its duration (`when: defined(duration)`) is its own first branch.

The model is `crates/verify/ess-conformance/tests/fixtures/latest-correlated-record.yaml`, domain
`idioms.correlated`. It synthesizes seven scenarios with no refusal and passes on the interpreter; a
target that keeps the first begin of a key fails `Begin/outcome/superseded`.

The derived branch stores both instants, and `ended_at - began_at` is the consumer's: arithmetic
beyond one constant offset is not in the value language (`docs/design/value-expressions.md`, as
#441 keeps for views). "Latest earlier" read as the latest begin whose instant is not after the
end's, with begins arriving out of order, is a point-in-time lookup this idiom does not serve. If
that need is confirmed, start from `{related: {entity, where, field, latest_by: <field>}}` with ties
broken by identity, designed together with #446's `arg_max` as one order-dependent pick.

## A total maintained per key is a grouped view

Asked (#443): per (customer, session, label), the total duration of the end rows, kept current as
rows arrive, with no total where it would be zero; proposed as a view with `materialized: true` and
an update rule, or a create-or-update effect keyed by a natural key.

A total over stored rows is an observation, and ESS states an observation as a view
(`docs/design/aggregate-views.md`: an aggregate is a view, not a new declaration kind). Whether a
target keeps it materialized is how it serves the view, not a domain fact. The idiom, in
`idioms.fold`:

- The rows are the entity `idioms.fold.SessionEnd`, with the invariant `duration >= 0`.
- A grouped `sum` view per key: `group_by` `[customer, session, label]` (`TotalByLabel`) or
  `[customer, session]` (`TotalBySession`), each `{sum: duration}`.
- `filter: duration > 0` on the same views (`NonZeroTotalByLabel`, `NonZeroTotalBySession`). With
  every duration non-negative a group sums to 0 exactly when each of its rows is 0, so the filter
  admits none of its rows and the group is absent: a zero total is absent. No `having` is needed.
- The total needs no invariant of its own: a sum of non-negative rows is non-negative, and the
  suite checks the exact sum.

Synthesis writes the group scenarios of the two unfiltered views. It refuses the two filtered ones
(`ESS-SYNTH-017`: the arranger refutes a filter only through lifecycle states, not a value), so an
authored scenario in the test holds the claim: durations 0 and 0 under one key and 0 and 5 under
another leave exactly one row, `duration_sum` 5. A target that keeps the zero-total group fails it.
The arranger gap is follow-up work, not part of this decision.

A stored per-key record keyed by identity is the existing create-or-update pair. It cannot add an
input amount, because `{increment:}` takes a literal; a recompute from all rows is the view.

## A unit is a newtype

Asked (#444): a `unit:` key on numeric fields, shown by projections and checked where units mix.

A unit is a newtype, a type declared with `kind: newtype`: `{name: Millis, kind: newtype, of:
Integer}`, and `Seconds` likewise, with fields typed by them (`idioms.units`). The name reaches JSON
Schema as the schema title and as `x-ess-name`, with `x-ess-kind: newtype`, and a field typed by it
is a `$ref` to it. A value of one unit is not assignable to a field of another: setting a `Seconds`
field from a `Millis` input is refused as `type_mismatch`, "no conversion is declared". A field's
prose, "in milliseconds", goes in its `summary:`, which the JSON Schema projection carries as its
description. `idioms.units.CallDurations` projects a `Millis` and a `Seconds` field side by side.

A `unit:` key would be a second spelling of what the newtype name says, and the two could disagree.
Two limits remain, and the test holds both:

- Two units still compare: comparisons across differently named numeric newtypes are admitted, so
  `talk_ms > wait_s` validates. Same-representation operands compare by design
  (`docs/design/review-expression-typechecking.md`); checking units there would change that rule
  for every newtype.
- `sum` returns the bare primitive: a `sum` declared `Millis` is refused, "whose result type is
  Integer". `min` and `max` keep the newtype.

If an adopter needs prose on the type itself, the next step is a `summary:` on newtypes, which no
newtype accepts today.

## An order-dependent value needs a declared order

Asked (#446): per group, a value of the last row in query order (`first`/`last`), and groups nested
inside an outer group.

"Last in query order" is not a fact a view states: an unordered view names no particular rows. The
idioms, in `idioms.latest`:

- The latest or earliest instant per group is `max` or `min`: `idioms.latest.LatestAt` is
  `{max: at}`, which orders `Timestamp` values by instant.
- The latest row of one group is a parameterised row view with a declared `order_by` and `paging:`,
  read with size 1: `idioms.latest.LatestEvent`, `filter: outer == param.outer`, `order_by: [at desc,
  event_id asc]`. The identity is the tie rule. Synthesis arranges several rows at distinct instants
  and asserts the order and the pages; it arranges no tie, so the authored scenario
  `docs/design/read-api-view-idioms.scenarios/latest-tie.yaml` records two events of one group at
  one instant and requires them in ascending identity order. A target breaking ties the other way
  fails it.
- Nested groups are a flat `group_by: [outer, inner]` the consumer nests
  (`idioms.latest.ByOuterInner`).

Refused today, and kept refused: `{last: …}` ("unknown variant `last`, expected one of `count`,
`count_distinct`, `sum`, `min`, `max`, `avg`"), and `order_by:` on an aggregate view (`ESS-VIEW-009`,
"ranking aggregate rows is a window, which is not in this cut").

An order-dependent aggregate needs a declared order and a tie rule on every target, and synthesis
needs rows that tie. Both are out of scope with windows and ranking (`docs/design/aggregate-views.md`,
"Out of scope"). If reopened, start from `arg_max`, written `arg_max: {field, by}`, with ties
broken by identity, designed together with #442's ordered pick.

## A view has one source

Asked (#447): a view over a declared relation that projects fields of both ends, and computed label
columns.

`source:` stays one entity (`docs/design/aggregate-views.md`, "Out of scope"). Which idiom fits
depends on the fact wanted (`idioms.joined`):

- The value when the row was written: copy it at write with the value source that opens
  `{related: {via`, here `{related: {via: input.agent_id, field: label}}`, then project or group by
  the copy (`idioms.joined.CallsByAgentLabel`). Synthesis arranges the referenced row between decoys
  and reads the copy back, so a target that reads another row fails.
- The current value of the other row: two views, `idioms.joined.Agents` and `idioms.joined.Calls`,
  joined by the consumer on `agent_id`.
- A label of an enum value: the variant's `display:`. Any other label is presentation, and the
  consumer's.

Refused today, and kept refused: a view field naming another entity's field (`agent.label`: "invalid
field name identifier … contains '.'") and a field the source does not have (`ESS-VIEW-001`).

If a live join is needed, start from `field: {related:` on a view field, in full `field: {related:
{via: <reference field>, field: <field>}}`: the existing value source read at query time, one hop,
absent where the reference is.

## A clock-relative window is resolved by the caller

Asked (#439): a read that counts rows whose instant falls in a range named relative to the request
time (`TODAY`, `LAST_7_DAYS`, …) in a named zone, the same over the last N minutes, and one value
that is the time a row has spent in its current state, measured at read time.

There is no window parameter kind, no named zone and no clock in a view: the caller resolves the
range. It turns `TODAY` in its zone, or the last N minutes against its own clock, into two
instants and sends them as two `Timestamp` parameters. The view compares each row's instant with
them, `from` inclusive and `to` exclusive: `filter: [started_at >= param.from, started_at <
param.to]` (`idioms.window.CallsInRange`). ESS needs no zone data and reads no clock for it; that the resolver
puts `TODAY` in the right zone, across a daylight-saving change too, is the adopter's to test.

Synthesis witnesses the idiom on an aggregate view under the suite's empty initial state. A
top-level ordering of a `Timestamp` field against a `Timestamp` parameter is a range selector. Rows
are arranged a second either side of each bound and on it: `param.from` is sent as
`2020-01-01T10:00:00Z`, `param.to` as `2020-01-01T12:00:00Z`, and rows land at 09:59:59, 10:00:00,
11:59:59 and 12:00:00 UTC. One further row inside is spelled at an offset under which its written
text sorts outside (`2020-01-01T08:00:00-03:00`), so a target comparing text, ignoring a bound or
making `to` inclusive counts another number. Each row outside the window holds every other
conjunct of the filter, so the bound alone refutes it. A range parameter read any other way, a bound on a
group key and two bounds on one side keep the `ESS-SYNTH-017` refusal.

Refused today, and kept refused: `started_at >= now - 1h` in a view filter (`type_mismatch`, naming
the guard positions where `now` is admitted) and a calendar `window:` in a view filter
(`type_mismatch`). A named zone is refused everywhere (`docs/design/calendar-window-guards.md`).

Out of scope: the running duration of a row in its current state at read time. It needs a computed
view field and a read-time clock, neither of which a view has (#441 above;
`docs/design/current-time-guards.md`). Return the instant the row entered its state and let the
consumer subtract it from its own clock.

If reopened, start from a `window:` parameter kind resolved against a request clock the suite can
set, with a pinned zone-data dependency; it needs `ess/23`, a suite step that fixes the target's
clock and zone data in every lane.

## An identity no row carries

Asked (#533): a per-record status read answers a default record for an identity no row carries
(`200`, every flag false, only the identity set), where it used to answer an error. A view lists the
rows its source holds, so before `ess/25` an empty answer was the only answer for a missing
identity, and a suite read nothing for one: a target answering the default record and a target
answering no row both passed. Commands have `unknown_instance:`; views had no counterpart.

From `ess/25` a view read by identity declares its answer for an identity no row carries with
`absent:`. This cut admits one form, a default row whose fields are literals:

```yaml
views:
  - name: demo.status.FlagById
    source: demo.status.Flag
    params: [{name: id, type: demo.status.FlagId}]
    filter: id == param.id
    fields: [{name: id, type: demo.status.FlagId}, {name: paused, type: Boolean}]
    absent:
      row: {paused: false}
```

A read of `FlagById` for an `id` no `Flag` carries answers one row: `id` the requested value,
`paused` false. A read for a stored `id` answers that row, as before.

Where it is admitted, and what it refuses:

- The view's filter is the one comparison `<identity field> == param.<name>`, in either operand
  order, and nothing else: the view selects its source entity by identity, the condition
  `body: single` reads in `ess-http/1` (`docs/design/http-presentation-binding.md`, ESS-HTTP-031).
  A filter with further conjuncts is refused, because an identity some row carries and the filter
  refutes is a second kind of "no row" the default would have to tell apart; admitting it later is
  additive. Refused as `conflicting_declaration` at `view.<name>.absent`, naming the source entity
  and its identity field.
- The identity parameter is not `Optional`: the default row holds the identity the read requested,
  and a read that leaves an `Optional` parameter out requests none (`conflicting_declaration` at
  `view.<name>.absent`). Admitting it, with no row as the answer, would be additive.
- Not beside an aggregate (`group_by:` or an `aggregate:` field) or `paging:`: a default row is one
  projected row, and an aggregate's row or a page is not one. Refused as `conflicting_declaration`.
- `row:` names projected fields of the view. Each value is a literal, typed by the rule `sets:`
  types a literal by: text, a variant of an enum, `true` or `false`, a whole number or a decimal,
  checked against the view field's type (`type_mismatch`), inside every alphabet and invariant of
  each newtype that type is written through, as an input's `example:` is. The same holds for every
  other literal position (`sets:`, `payload:`, a binding's `mapping:`, an enum variant's
  `attributes:`). A name the view does not project is `undeclared_reference`.
- The identity field is never in `row:`: its value is the requested parameter
  (`conflicting_declaration`). Every other projected field that is not `Optional` is in `row:`
  (`missing_declaration`); an `Optional` field `row:` leaves out is absent in the default row.
- `absent:` without `row:` and any other key are parse refusals naming the key. An `error:` answer
  (the error a read of an unknown identity answers) is not in this cut: a view read answers rows
  only, so a suite expectation of an error needs a suite vocabulary a view read does not have. It is
  the additive next step, `absent: {error: <Error>}`, designed with that suite format.
- Below `ess/25`, `absent:` is refused as `unsupported_format_version`, naming `ess/25`.

The IR carries `absent` on the view, `{param, row}`, `row` holding each literal's canonical text,
an `Integer` or `Decimal` by value (`1.50` is `1.5`, so a respelling is no change),
and omits it where the view declares none, so every other model keeps its IR bytes, its suite and
its generated artifacts.

The interpreter answers the default row where no stored row of the source carries the requested
identity. Synthesis reads it: in every scenario that shows a stored row of such a view, after that
row's own assertions, it reads the view again for an identity no scenario stores (the witness one
distinction past the unknown-instance witness) and requires `Contains` the identity, where the view
projects it, every literal of `row:`, and `null` in every other field. A target that answers no row, or the default row with a
flag set, fails that read; a target that answers the default row for a stored identity fails the
read before it. No suite format moves: the read and the expectation are today's `query_view` and
`contains`. An `Optional` field `row:` leaves out is asserted `null`, as a projection writes an
`Optional` holding nothing, so a target that fills it fails the read.

Generated view queries keep such a view an obligation, naming `absent:`: a view with an equality
parameter was one already. OpenAPI states the default row in the view operation's description.
`ess verify diff` reports the key gained, lost or changed as `absent-changed` on the view.

The HTTP binding keeps its bytes and its rules. `body: single` with `empty: {status: <4xx>}` stays
the HTTP answer for a view that holds no row, and stays required beside `single`. On a view that
declares `absent:` an unknown identity holds the default row, so the served answer is `200` with that
row and `empty` is not reached by a read of an unknown identity. Dropping `empty` beside `absent:`,
or refusing the pair, is a later ess-http change; neither is made here, so either stays open.
