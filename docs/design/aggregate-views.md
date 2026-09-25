# Aggregate views: counts, sums and extremes over one entity's rows

Status: proposed (beyond10x/ess#96, `story:aggregate-views`). No source, IR or format change is
made by this page. The implementation unit builds exactly what is written here. Every claim about
current code names `path:line` at base `bb771888907`. Claims not read off the tree are marked
*inferred*.

## Behavior and authority

A call-metrics pipeline writes one row per call session: queue, agent, caller, talk seconds and
wait seconds. Its read API is made of aggregates over those rows: sessions per agent, total talk
time, longest wait, distinct callers. ESS can state the entity and its one-row-per-instance views.
It cannot state the aggregates, so each such endpoint stays `UNMAPPED:` and no conformance suite
checks a number a dashboard shows.

The operator settled four questions on 2026-09-25 (story `Decisions`). This page records them and
does not reopen them:

1. An aggregate is a **view** (`group_by:` plus a field-level `aggregate:`). It is not a new
   declaration kind.
2. There is no time bucketing in this cut.
3. There are no ratios or differences between aggregates. The consumer computes those.
4. **Empty groups are absent** from the result. A view without `group_by` returns **one row**: when
   no row passes the filter, `count` is 0 and `min`/`max` are absent.

## What exists today

| Construct | What it does | Why it does not state an aggregate |
|---|---|---|
| `ViewSpec` (`crates/specify/ess-domain/src/view.rs:293-334`) | `name, source, shape, fields, params, filter, order_by, consistency, naming`. | It has no grouping and no computed field. |
| View field check (`view.rs:506-544`) | Every projected field must be a field of the source (`ESS-VIEW-001`: "promises an observation nothing produces", `view.rs:520-522`), at an assignable type (`ESS-VIEW-002`, `view.rs:534-543`). | A computed field is not a source field. |
| `Field` (`crates/specify/ess-domain/src/types.rs:306-322`) | `name`, `type`, flattened `naming`, `deny_unknown_fields`. | `aggregate:` on a view field is `unknown field aggregate`. |
| `filter:` (`view.rs:552-580`) | A predicate over the source's observable fields and the declared `params`. | Unchanged here. It already runs before any grouping would. |
| Ordered-view arrangement (`crates/verify/ess-conformance/src/synthesize.rs:3067-3200`, `Distinction` at `crates/verify/ess-conformance/src/witness.rs:79-100`) | Arranges further instances through the declared creating outcome so that `Ranked` has two rows. | It asserts a floor and never a ceiling, because §8 allows a target to be shared (`synthesize.rs:146-160`, `3163-3172`). An aggregate is an exact number, so a floor is not enough. See "Scoping" below. |

## Syntax

```yaml
format: ess/9      # e.g. — `ess/S`, the next free source format (see "Formats and versions")
system: metrics
version: v1
domain: metrics.session

events:
  - name: metrics.session.Recorded
    fields: [{name: session_id, type: Uuid}]
  - name: metrics.session.Completed
    fields: [{name: session_id, type: Uuid}]

entities:
  - name: metrics.session.Session
    identity: {name: session_id, type: Uuid}
    fields:
      - {name: queue_id, type: String}
      - {name: agent_id, type: String}
      - {name: caller, type: String}
      - {name: talk_seconds, type: Integer}
      - {name: wait_seconds, type: Integer}
    lifecycle:
      initial: Open
      states: [Open, Completed]
      terminal: [Completed]
      transitions:
        - {name: complete, from: [Open], to: Completed}

commands:
  - name: metrics.session.Record
    input:
      - {name: queue_id, type: String}
      - {name: agent_id, type: String}
      - {name: caller, type: String}
      - {name: talk_seconds, type: Integer}
      - {name: wait_seconds, type: Integer}
    outcomes:
      - name: recorded
        creates: metrics.session.Session
        instance: session_id
        sets:
          queue_id: input.queue_id
          agent_id: input.agent_id
          caller: input.caller
          talk_seconds: input.talk_seconds
          wait_seconds: input.wait_seconds
        emits: [metrics.session.Recorded]
        payload:
          metrics.session.Recorded:
            session_id: {generated: true}
  - name: metrics.session.Complete
    input: [{name: session_id, type: Uuid}]
    outcomes:
      - name: completed
        moves: metrics.session.Session.complete
        instance: session_id
        emits: [metrics.session.Completed]

views:
  - name: metrics.session.TalkTimeByAgent
    source: metrics.session.Session
    consistency: eventual
    filter: state == Completed
    group_by: [agent_id]
    fields:
      - {name: agent_id, type: String}
      - {name: sessions, type: Integer, aggregate: {count: {}}}
      - {name: talk_seconds, type: Integer, aggregate: {sum: talk_seconds}}
      - {name: longest_wait, type: Optional<Integer>, aggregate: {max: wait_seconds}}
      - {name: distinct_callers, type: Integer, aggregate: {count_distinct: caller}}
      - {name: mean_talk, type: Optional<Decimal>, aggregate: {avg: talk_seconds}}

  - name: metrics.session.QueueTotals
    source: metrics.session.Session
    consistency: eventual
    params: [{name: queue_id, type: String}]
    filter: [state == Completed, queue_id == param.queue_id]
    fields:
      - {name: sessions, type: Integer, aggregate: {count: {}}}
      - {name: longest_wait, type: Optional<Integer>, aggregate: {max: wait_seconds}}
```

The issue's own example declares `params: [queue_id]` and a filter that does not read it. Today
that is refused (`view.rs:653-680`: "declares the parameter … and no filter reads it"), and it
stays refused. `QueueTotals` above is the valid form.

- **`group_by:`** is a view-level list of view field names. Absent or `[]` means the view is not
  grouped. `[]` is read as absent and is not written back.
- **`aggregate:`** is a field-level map with **exactly one** key, the function:
  - `{count: {}}` takes no argument. The value must be the empty map, and no other spelling of
    "no argument" is accepted.
  - `{count_distinct: f}`, `{sum: f}`, `{min: f}`, `{max: f}`, `{avg: f}` take `f`, the name of one
    top-level observable field of the source: a declared field, the identity, or `state`.
- A view is an **aggregate view** when at least one field declares `aggregate:`. In an aggregate
  view, every field without `aggregate:` is a **group key** and must be listed in `group_by`. Every
  `group_by` entry must name a field without `aggregate:`.
- A view field's name is its output name. `talk_seconds: {sum: talk_seconds}` is allowed. The filter
  reads the source, so `talk_seconds` in `filter:` is the per-row value and never the sum. There is
  no `having`.

A reader error, not a validation code, handles the shape of the map. `RawAggregate` is an
externally tagged enum with `rename_all = "snake_case"` and variants `Count(Empty)`,
`CountDistinct(String)`, `Sum(String)`, `Min(String)`, `Max(String)`, `Avg(String)`. `Empty` is a
field-less struct with `deny_unknown_fields`. An unknown function, two keys in one map, or an
argument to `count` fails the parse, with serde's message listing the six function names. This
matches how an unknown key fails today (`Field`'s `deny_unknown_fields`, `types.rs:309`).

## Semantics

Evaluation order is fixed: the **filter runs per source row** (with `param.*` bound), the admitted
rows are **partitioned by the tuple of group-key values**, and **each aggregate is computed per
partition**. A partition exists only if at least one admitted row belongs to it (decision 4). An
ungrouped aggregate view has exactly one partition, and that partition exists even when it is
empty (decision 4).

Group keys compare by value equality of their type: text by bytes, numbers by numeric value, enum
by variant name, `Uuid` by its canonical text (`UUID_PATTERN`, `docs/design/review-primitive-semantics.md:39`),
`Boolean` by value.

### Result types

"Unwrapped" means the source field's type after following newtypes **recursively** down to a
primitive or an enum. If an `Optional<…>` appears **anywhere in that chain** — the field itself
`Optional<T>`, or a newtype whose `of:` is `Optional<T>`, at any depth — the field counts as
optional: as an aggregate argument it is V9, as a group key V11 (both `UnsupportedConstruct`), and
never V8/V12. Only a chain with no `Optional` in it reaches the admitted-type tables. The view
field's declared `type:` must be **exactly** the result type in the table. Anything else gets
`ESS-VIEW-002` (below).

| function | admitted input (unwrapped) | result type | over zero rows (ungrouped only) |
|---|---|---|---|
| `count` | none | `Integer` | `0` |
| `count_distinct` | `String`, `Integer`, `Decimal`, `Boolean`, `Uuid`, `Timestamp`, an enum (including `state`) | `Integer` | `0` |
| `sum` | `Integer` | `Integer` | `0` |
| `sum` | `Decimal` | `Decimal` | `0` |
| `min`, `max` | `Integer`, `Decimal`, `String`, `Timestamp` | `Optional<T>`, where `T` is the source field's **declared** type, newtype kept | absent |
| `avg` | `Integer`, `Decimal` | `Optional<Decimal>` | absent |

- **`sum` drops a newtype** and `min`/`max` keep it. A minimum is one of the inputs, so it satisfies
  every invariant the newtype states. A sum does not (a `Seconds` newtype bounded at 86400 has sums
  above that).
- **`count_distinct`** counts distinct values under the equality above. For `Timestamp`, equality
  is the same instant. For `Decimal`, it is numeric (`1.0` and `1` are one value; `Number` holds one
  normalised scale, `crates/specify/ess-primitives/src/facts.rs:46-66`).
- **`min`/`max` ordering** is: numeric for `Integer` and `Decimal`; the instant for `Timestamp`, as
  `evaluate_compare` already orders a declared timestamp (see
  `cross-record-and-stored-field-guards.md`, "a typed fact source"); **byte-wise lexicographic** for
  `String`, the order `story:list-and-text-guards-are-synthesized` fixes for text across the Rust,
  Go and TypeScript lanes. `min`/`max` of a `Timestamp` returns a source row's value as that row
  holds it.
- **`sum` of `Integer`** is the exact sum. If it falls outside `[i64::MIN, i64::MAX]`, the row is not
  representable and the specification makes no claim. Conformance never arranges such a sum (the
  arranged values are below 1000).
- **`sum` of `Decimal`** is the exact decimal sum.
- **`avg` rounding rule.** `avg` is the exact quotient `sum / count` over the partition's values,
  **rounded to 6 fractional digits, ties to even** (half-even). The result is a `Decimal`, and
  trailing zeros carry no meaning (`1.500000` and `1.5` are one `Number`). The scale is fixed so
  that it does not depend on the data, which means every implementation spells the same value. 6
  covers micro-units. Half-even is the IEEE 754 decimal default and the default midpoint rule of
  the common Rust decimal type (*inferred*, not read in this tree). Over zero rows `avg` is absent:
  there is no quotient.
- **Absent** means an `Optional` with no value. The generated encoders omit such a key
  (`crates/generate/ess-synth/src/rust/wire.rs:5-12`). A conformance target reports it in the
  semantic row as `null`, as it already must for a `{cleared: true}` field
  (`synthesize.rs:3246`, asserted as `Node::Null`).

**Decided here, beyond the four operator decisions.** The story's acceptance lists `avg → Decimal`.
This page makes it **`Optional<Decimal>`**, for the reason `min`/`max` are `Optional<T>`: the
ungrouped zero-row partition has no average, and a type that changed with `group_by` would be two
rules. `sum` over zero rows is `0`, and `count_distinct` over zero rows is `0`. The story states
only `count 0` and `min`/`max` absent. The rest follows from the additive identity and from "no
value".

## Declaration and validation

The rules run in **two places**, because `Aggregation` exists only when some field declares
`aggregate:` (see "Domain"), so a check whose subject is "`group_by` without an aggregate" cannot
run on a `ViewSpec`:

- **V5 and V13 run in `TryFrom<RawViewSpec>`** (`view.rs:724-741`), while the raw `group_by` is
  still in hand and before it is discarded. They push into the same `ValidationErrors` that
  `validate_shape` returns there (`view.rs:739`, `validate_shape` `view.rs:416-467`), so they are
  accumulated with the shape errors and never stop at the first one. V3 (duplicate key) runs there
  too, for the same reason: a duplicated key is a property of the written list.
- **Every other rule runs in `ViewSpec::validate`** (`view.rs:474-585`), which gains an aggregate
  pass over `aggregation`. Its errors accumulate into the `errors` it already collects
  (`view.rs:479`, returned at `:584`).

The existing source-field check (`view.rs:514-532`) stays for group keys and every field of a
non-aggregate view, and is **skipped for aggregate fields**. Codes are `ess-domain`
`ValidationCode`s. The compiler maps them to
`ESS-VIEW-<class>` through `family_of` (`crates/specify/ess-compiler/src/resolve.rs:767-787`, `view`
→ `VIEW`) and `class_of` (`resolve.rs:793-837`), so the stable code is fixed by the pair below.

| # | written | `ValidationCode` | location | stable code |
|---|---|---|---|---|
| V1 | a `group_by` entry that is not a field of the view | `UndeclaredReference` | `view.<name>.group_by[<i>]` | `ESS-VIEW-001` |
| V2 | a `group_by` entry naming a field that declares `aggregate:` | `ConflictingDeclaration` | `view.<name>.group_by[<i>]` | `ESS-VIEW-004` |
| V3 | a `group_by` entry repeated | `DuplicateDeclaration` | `view.<name>.group_by[<i>]` | `ESS-VIEW-006` |
| V4 | in an aggregate view, a field with no `aggregate:` that is not in `group_by`. Message: "`x` is neither an aggregate nor a group key, so a row has no single value for it" | `MissingDeclaration` | `view.<name>.fields[<i>]` | `ESS-VIEW-005` |
| V5 | a non-empty `group_by` with no field declaring `aggregate:` | `MissingDeclaration` | `view.<name>.group_by` | `ESS-VIEW-005` |
| V6 | an aggregate argument that is not an observable field of the source (hint lists them, as `view.rs:525-529` does) | `UndeclaredReference` | `view.<name>.fields[<i>].aggregate` | `ESS-VIEW-001` |
| V7 | an aggregate argument containing a dot (`total.amount`) | `UnsupportedConstruct` | `view.<name>.fields[<i>].aggregate` | `ESS-VIEW-009` |
| V8 | an argument with no `Optional` in its newtype chain whose unwrapped type the function does not admit (table above), including `List`, `Map`, struct, union, `Binary64`, `Duration` and `Bytes` | `TypeMismatch` | `view.<name>.fields[<i>].aggregate` | `ESS-VIEW-002` |
| V9 | an argument with an `Optional<…>` anywhere in its newtype chain (checked before V8) | `UnsupportedConstruct` | `view.<name>.fields[<i>].aggregate` | `ESS-VIEW-009` |
| V10 | a declared `type:` other than the function's result type. Message names the expected type | `TypeMismatch` | `view.<name>.fields[<i>].type` | `ESS-VIEW-002` |
| V11 | a group key with an `Optional<…>` anywhere in its newtype chain (checked before V12), or whose unwrapped type is `Timestamp` (decision 2: time bucketing is not in this cut) | `UnsupportedConstruct` | `view.<name>.group_by[<i>]` | `ESS-VIEW-009` |
| V12 | a group key with no `Optional` in its chain whose unwrapped type is not an equality type (`List`, `Map`, struct, union, `Binary64`, `Duration`, `Bytes`) | `TypeMismatch` | `view.<name>.group_by[<i>]` | `ESS-VIEW-002` |
| V13 | a non-empty `group_by` beside `shape:` | `ConflictingDeclaration` | `view.<name>.group_by` | `ESS-VIEW-004` |
| V14 | `order_by:` on an aggregate view | `UnsupportedConstruct` | `view.<name>.order_by` | `ESS-VIEW-009` |
| V15 | a view with an `Aggregation` (any `aggregate:`) in a document below `ess/S`; a `group_by` with no aggregate is V5 at any version. Message: "aggregate views require specification format ess/S", with the number D2 took | `UnsupportedFormatVersion` | `view.<name>.group_by` if non-empty, else `view.<name>.fields` | `ESS-VIEW-009` |

Why each rule is shaped this way:

- **V9 and V11** refuse `Optional` rather than defining how absent values are skipped. Witness rule
  1 fills every optional (`witness.rs:17-19`), so no arrangement could show that an absent value is
  skipped. `story:optional-guards-mean-what-they-say` is changing that rule in this wave, and
  admitting optional inputs is a later step on top of it.
- **V13**: a `shape:` names a struct whose fields cannot carry `aggregate:`, so an aggregate view
  declares its fields inline.
- **V14**: ranking aggregate rows ("top agents") is a window in all but name, and it is out of
  scope below.
- **V15** sits beside the existing gates in `primitive_admission::specification`
  (`crates/specify/ess-domain/src/primitive_admission.rs:219-235`, the view loop), in the shape of
  the ess/3–ess/7 gates in that file (`primitive_admission.rs:89-146`).

The filter check (`view.rs:552-580`) and the params check (`view.rs:653-680`) are unchanged.
`consistency` is unchanged: an aggregate view is `expect`ed or `eventually`ed like any other.

## Domain and IR

### Domain (`ess-domain`)

```rust
/// One aggregate function, as written.
#[serde(rename_all = "snake_case")]
pub enum AggregateFunction { Count, CountDistinct, Sum, Min, Max, Avg }

/// One aggregate field's computation. `input` is `None` exactly for `Count`.
pub struct Aggregate { pub function: AggregateFunction, pub input: Option<String> }

/// What makes a view an aggregate view. Present iff at least one field declares `aggregate:`.
pub struct Aggregation {
    /// Group-key view field names, in declaration order. Empty: one row.
    pub group_by: Vec<String>,
    /// Aggregate view field name → its computation.
    pub functions: BTreeMap<String, Aggregate>,
}
```

- `ViewSpec` (`view.rs:293-334`) gains `pub aggregation: Option<Aggregation>`. `ViewSpec::fields`
  stays `Vec<Field>`, with aggregate fields listed among the others under their declared types.
  Every existing reader of `fields` therefore keeps working unchanged. `ViewSpec::is_aggregate()`
  returns `self.aggregation.is_some()`.
- `RawViewSpec` (`view.rs:694-722`) gains
  `#[serde(default, skip_serializing_if = "Vec::is_empty")] group_by: Vec<String>`, and its
  `fields` becomes `Vec<RawViewField>`. `RawViewField` has `Field`'s three members (`name`, `type`,
  flattened `naming`, same attributes as `types.rs:311-321`) plus
  `#[serde(default, skip_serializing_if = "Option::is_none")] aggregate: Option<RawAggregate>`.
  It is a sibling struct and not `Field` flattened, so `Field`'s `deny_unknown_fields` keeps
  applying at every other position.
- `TryFrom<RawViewSpec>` (`view.rs:724-741`) builds `Aggregation`. `From<ViewSpec>`
  (`view.rs:743-757`) folds it back. A view with no aggregation serializes to the same keys in the
  same order as today, so **its bytes are unchanged**.

### IR (`ess-compiler`)

```rust
pub struct ResolvedAggregate {
    pub function: AggregateFunction,
    /// The source field read, resolved. Absent for `count`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<ResolvedField>,
}

pub struct ResolvedAggregation {
    pub group_by: Vec<String>,
    pub functions: BTreeMap<String, ResolvedAggregate>,
}
```

- `ResolvedView` (`crates/specify/ess-compiler/src/ir.rs:1082-1134`) gains
  `#[serde(skip_serializing_if = "Option::is_none")] pub aggregation: Option<ResolvedAggregation>`,
  declared **after `filter`**, plus `ResolvedView::is_aggregate()`. The resolver fills it where it
  builds `ResolvedView` (`resolve.rs:2465-2478`). `input` is resolved from the source entity's
  observable fields through `self.fields(…)`, as the projected fields are (`resolve.rs:2426-2428`).
- **No `ess-ir/2`.** The field is additive and omitted for every existing view, which is the
  precedent of `typed-response-outcome-payloads.md:25` and `retained-command-results.md:173`. IR bytes
  and provenance body digests (`crates/generate/ess-gen/src/provenance.rs:385`) of every model
  without an aggregate view are unchanged.
- `EssIr::projections()` (`ir.rs:1801-1807`) keeps returning every view. Row-level consumers skip
  aggregate views explicitly (enumerated under Conformance), and one check covers the class.

## Formats and versions

**This page fixes no absolute version number.** The construct takes the **next free** source format,
suite format pair, diff format and synthesis refusal codes **at implementation time**, after units
B1 (`story:string-prefix-suffix-substring-operators`, #95) and C (`story:stored-field-guards`, #75)
have merged into the integration branch. D2 records the numbers it took in its report and in the
CHANGELOG entry. The page writes them as placeholders:

| placeholder | meaning | e.g., if B1 and C had taken nothing beyond C's `ess/8` |
|---|---|---|
| `ess/S` | source format | `ess/9` |
| `ess-conformance/Q`, `/Q+1` | ordinary / coverage suite pair | `/14`, `/15` |
| `ess-diff/D` | semantic diff format | `ess-diff/7` |
| `ESS-SYNTH-U`, `ESS-SYNTH-W` | `AggregateUnscoped`, `AggregateUnwitnessed` | `016`, `017` |

| format | at base `bb771888907` | this construct | old reader |
|---|---|---|---|
| source | `ess/1`–`ess/7` supported (`crates/specify/ess-domain/src/system.rs:53`); `ess/8` claimed by C (`cross-record-and-stored-field-guards.md:316-318`) | **`ess/S`**: `FormatVersion::V<S>`, `SUPPORTED_FORMATS` gains `S`, V15 gates the construct | refuses the header version; a build that predates `ess/S` reading `aggregate:` under an older header fails `unknown field aggregate`, as today |
| document schema | `schemas/generated/ess.schema.json` (`RawViewSpec` at `:2093`) | regenerate with `cargo xtask schema`; `projection-check` is the only thing that sees staleness (`AGENTS.md`, "A change to `RawSpecFile`…") | — |
| IR | no `ess-ir/2` | additive, omitted when `None` | — |
| suite | ordinary `ess-conformance/12`, coverage `/13` newest (`crates/verify/ess-conformance/src/scenario.rs:155-168`, `:368`) | **`/Q` and `/Q+1`**: new `ScenarioId` variant and new refusal causes; admission gates below | Rust: closed parse and the admission gate; Go: version switch (`crates/verify/ess-conformance/src/go/runtime.go:3417-3436`); TypeScript admits up to `/11` (`crates/verify/ess-conformance/src/ts/runtime.ts:4205-4215`) and refuses before callbacks |
| semantic diff | `ess-diff/6` newest (`SUPPORTED_DELTA_FORMATS`, `crates/verify/ess-diff/src/delta.rs:13`) | **`ess-diff/D`** for the two new `ViewChange`s | refuses via `minimum_format` (`crates/verify/ess-diff/src/delta.rs:282`) |
| synthesis refusal codes | `ESS-SYNTH-001`…`015` (`synthesize.rs:502-525`) | **`ESS-SYNTH-U`, `ESS-SYNTH-W`**, gated in all three coverage lanes (below) | — |

Tests name the constants, not literals, wherever they can.

**Suite admission gates.** An aggregate scenario, or a `U`/`W` refusal, in a document labelled with
an older suite version is refused, in the shape of the retained-result gate:

- **Rust:** `aggregate::admit_suite`, a copy of `replay::admit_suite` (`crates/verify/ess-conformance/src/replay.rs:517-529`:
  "retained results require suite/12 or /13", code `UnsupportedVocabulary`), refusing any
  `ScenarioId::Aggregate` when `suite_version.major() < Q`. Called beside it in
  `crates/verify/ess-conformance/src/admission.rs:464`.
- **Go:** `scenarioIdentity` (`runtime.go:3357-3378`) admits `<view>/aggregate` syntactically. In
  `admitSuiteDocument` (`runtime.go:3386`), the scenario loop (`runtime.go:3459-3462`) runs after
  `major` is known (`runtime.go:3417-3436`). It refuses an `…/aggregate` id when `major < Q`.
- **TypeScript:** refuses `/Q` and `/Q+1` as whole envelopes (its table ends at `/11`), so it needs
  no per-scenario gate.

**Coverage refusal codes.** Coverage admission accepts only `ESS-SYNTH-001`…`015` in every lane,
so a refusal-only suite carrying `U`/`W` would be rejected, or would be written at an old coverage
version. Each lane moves exactly the way `015` did:

| lane | code range and effect | version gate |
|---|---|---|
| Rust | `coverage.rs:549-553`: range `1..=15` extends to `W`; `U` and `W` join `5 \| 11 \| 12 \| 14` as `CheckNotEmitted` | `coverage.rs:392-395`: `U`/`W` refusals require `/Q+1`, as `015` requires `/7`. `coverage_build.rs:478-486` (`coverage_version`): an aggregate scenario **or** a `U`/`W` refusal selects `/Q+1` first, as `coverage_build.rs:484` does for `015`. `generated_effect` (`coverage_build.rs:393-398`) maps both causes to `CheckNotEmitted` |
| Go | `runtime.go:304-313`: loop bound `15` → `W`; `U` and `W` added to the `check_not_emitted` list | `runtime.go:3504-3510`: `Q+1` joins the coverage-major list, and `U`/`W` rows are refused when `major < Q+1`, beside the `015`/`major < 7` rule; the coverage-major lists at `runtime.go:80` and `:3438-3440` gain `Q+1` too |
| TypeScript | `runtime.ts:985-995`: same two changes | `runtime.ts:4289`: same rule as Go |

**Version tables and pages that must list the new versions.** The `ess-xtask` docs lane reads each
family's `SUPPORTED_*` constant (`crates/edge/ess-xtask/src/docs.rs:47-64`). Growing a constant
without a row in `FORMAT_RELEASES` (`docs.rs:89`) makes that lane refuse, so D2 adds
`("ess", S, None)`, `("ess-conformance", Q, None)`, `("ess-conformance", Q+1, None)` and
`("ess-diff", D, None)`. The published pages gain the same versions:
`website/docs/reference/formats.md` (source row beside `:105`, diff row beside `:257`, suite row
beside `:269`) and `website/docs/reference/spec-versions.md` (the newest-version examples at
`:9-10` and `:25`, and the diff table at `:73`).

**Tests whose version literals become supported.** Each uses a number as "a version this build
cannot read". When D2's numbers are added, each literal moves to the first number after D2's:

| test | literal at base | family |
|---|---|---|
| `crates/specify/ess-domain/src/spec.rs:1746` | `ess/8` (C moves it first, per its page) | source |
| `crates/specify/ess-domain/src/system.rs:1721` (`a_specification_reports_every_problem_in_one_run`) | `ess/9` | source |
| `crates/specify/ess-domain/src/system.rs:1254` and `:1263-1275` | `ess/8` (C moves them — *inferred*) | source |
| `crates/verify/ess-conformance/src/scenario.rs:2677` | `ess-conformance/14` | suite |
| `crates/verify/ess-conformance/tests/count_reports.rs:460` | `ess-conformance/14` | suite |
| `crates/verify/ess-diff/tests/canonical.rs:1074` | `ess-diff/7` | diff |

The six rows are the literals known at base `bb771888907`. A literal at a
number D2 does not take stays untouched. **This class can be checked by machine:** D2 runs
`grep -rnE '"?(ess/|ess-conformance/|ess-diff/)[0-9]+' crates` for its own `S`, `Q`, `Q+1` and
`D` and fixes every hit that asserts "unsupported". The table is a starting point, not the bound.

## Projections

Each consumer of `ResolvedView` / `ViewSpec` renders the construct, refuses it, or is unchanged for
a stated reason. The list was enumerated by grepping for `ResolvedView`, `.views()` and `.views`
under `crates/`. `crates/edge/ess-xtask/src/consumer_coverage/*` is parked (`AGENTS.md`, "Consumer
coverage is opt-in") and is left out.

| consumer | today | change |
|---|---|---|
| OpenAPI response description (`crates/generate/ess-gen/src/openapi.rs:517-540`, called at `:507`) | "Every row of `V`, a projection of `E`", the filter, the consistency | **render**: after the filter sentence, "Grouped by `k1`, `k2`; one row per group holding at least one instance" or "Always exactly one row", then one clause per aggregate field (`sessions` = count of instances; `mean_talk` = average of `talk_seconds`, rounded to 6 places half-even) |
| OpenAPI row/response schema (`openapi.rs:745-757`, `:819-838`; row message `crates/generate/ess-gen/src/types.rs:859-866`) | built from `view.fields` | unchanged. Aggregate fields carry their declared result types, so `Optional<…>` stays non-required and `Decimal` publishes as today |
| HTTP routes and params (`crates/generate/ess-gen/src/http.rs:199-215`) | one `GET` per view | unchanged |
| docs (`crates/generate/ess-gen/src/docs.rs:919-966`, `filter_sentence` `:2156-2170`, `field_bullet` `:2249`) | filter, fields, order, consistency | **render**: a grouping sentence after `filter_sentence` (same two wordings as OpenAPI), and aggregate field bullets read "`sessions`: Integer, the number of instances in the group". `order_sentence` (`:993`) is unchanged, because V14 refuses `order_by` |
| docs entity section (`docs.rs:800`, `projections()`) | lists an entity's views | unchanged: an aggregate view is a view of that entity |
| native plan (`crates/generate/ess-synth/src/plan.rs:735-770`, `view_contract`) | the `ViewQuery` obligation's contract text | **render**: `view_contract` appends ", grouped by `k…`" or ", one row", and "computing `f = fn(input)`" per aggregate. The Rust and Go obligation seams (`crates/generate/ess-synth/src/rust/obligation.rs:202`, `crates/generate/ess-synth/src/go/obligation.rs:263-275`) quote that contract and need no edit |
| generated Rust row type (`crates/generate/ess-synth/src/rust/items.rs:334-359`) | doc comment plus struct | **render**: doc comment gains the grouping line. The struct is unchanged |
| generated Go row type (`crates/generate/ess-synth/src/go/items.rs:466-485`) | same | **render**: same line |
| Rust/Go ports, encoders, handlers (`crates/generate/ess-synth/src/rust/port.rs:308`, `crates/generate/ess-synth/src/go/port.rs:266`, `rust/wire.rs:302-310`, `crates/generate/ess-synth/src/go/http.rs:382-390`, `:1134`, `crates/generate/ess-synth/src/rust/http.rs:799`) | one owed query returning `Vec<Row>` | unchanged: the owed query computes the aggregate |
| Go refusals, Rust feasibility (`crates/generate/ess-synth/src/go/refusal.rs:152-158`, `crates/generate/ess-synth/src/rust/feasibility.rs:417`) | refuse unrepresentable field types | unchanged: result types are ordinary types |
| web catalog/page (`crates/generate/ess-synth/src/web/catalog.rs:262-282`, `crates/generate/ess-synth/src/web/page.rs:418`) | a card per view showing filter and fields | **render**: catalog gains `"group_by"` and each field's `"aggregate"` (`"sum(talk_seconds)"`). The page note appends "grouped by …" / "one row" |
| web bridge (`crates/generate/ess-synth/src/web/bridge.rs:490`) | serves rows from the owed query | unchanged |
| clap tree (`crates/generate/ess-synth/src/clap/tree.rs:181-192`) | one subcommand per view | unchanged |
| TypeScript | no emitter produces view types in this tree. `TypeScript` appears only under `crates/generate/schema-contract` (import normalization) and the conformance runner (`crates/verify/ess-conformance/src/ts/`) | nothing to render. The runner's refusal is under Formats |
| Entity Runtime lowering (`crates/generate/ess-entity-runtime/src/lib.rs:442-458`) | lowers entities and commands only. `lib.rs` never names a view (grep: zero matches) | unchanged. A view, aggregate or not, stays a host `ViewQuery` obligation carried by the service contract (`crates/specify/ess-service-contract/src/lib.rs:311-330`) |
| service contract, composition, CLI contract (`ess-service-contract/src/lib.rs:166-172`, `crates/specify/ess-composition/src/lib.rs:1321-1347`, `crates/specify/ess-cli-contract/src/resolve.rs:187-191`) | collect view field types and shapes | unchanged |
| dependency graph (`crates/specify/ess-compiler/src/graph.rs:703-728`) | `Projects` edge to the source; field type edges | unchanged: the aggregate reads the source the `Projects` edge already names |
| Binary64 audit (`crates/specify/ess-compiler/src/binary64.rs:82-85`), wire uniqueness (`crates/specify/ess-domain/src/wire.rs:48-61`) | walk view fields | unchanged: V8/V12 keep `Binary64` out |
| semantic diff (`crates/verify/ess-diff/src/diff.rs:1488-1586`, `ViewChange` `change.rs:2367`, `kind` `:2502`) | domain, source, fields, params, ranking, filter, consistency, naming | **render**: `ViewChange::GroupingChanged { before: Vec<String>, after: Vec<String> }` (`grouping-changed`) and `ViewChange::FieldAggregateChanged { field, before: Option<String>, after: Option<String> }` (`field-aggregate-changed`, `member()` = `field`). The rendering is `"sum(talk_seconds)"` / `"count()"`, and `None` means "not an aggregate". Both are emitted after `FilterChanged`, compared over the union of field names. Both have `minimum_format` `D` and relation `Changed` |
| conformance catalog (`crates/verify/ess-conformance/src/web.rs:183-193`) | name, filter, fields, params | **render**: `"group_by"` and `"aggregates"` (same rendering as the diff) |
| authored assertions (`crates/verify/ess-conformance/src/authored.rs:2006-2015`) | an author asserts rows of a declared view | unchanged: a person may assert aggregate rows |

## Conformance

### Scoping: why an exact number needs rows only this scenario made

§8 lets a target isolate scenarios by "generated IDs" alone
(`docs/design/ess-closed-loop-execution-conformance-design-v0.1.md:383-406`). Synthesis already
treats a target as possibly shared, and for that reason writes floors and never ceilings
(`synthesize.rs:3163-3172`). A count is a ceiling and a floor at once. A row made by another
scenario and landing in the same group turns `3` into `4`. So the scenario gives its groups key
values that no other scenario of the suite produces:

- A key is **scopable** when its unwrapped type is `String` or `Uuid` and the creating branch's
  `sets:` fills it from an input field (`ResolvedPayloadValue::InputField`, as `settled` reads it at
  `synthesize.rs:3260-3265`).
- The scoped value for group `g` is the text `"<view>/<g>"` for `String` and
  `uuid("<view>/<g>")` for `Uuid`. `uuid` is the FNV-derived builder at `witness.rs:690-697`, seeded
  with that text.
- **Why no other scenario reaches these values.** Other scenarios get their values from two
  sources:
  - the type-derived witness, which writes a field's own fact path, `path` or `path-n`
    (`witness.rs:663-665`), and never contains `/`;
  - **the model's own literals**, which rule 3 tries as alternatives (`witness.rs:24-27`) and which
    `sets:` and payload literals write directly. These can be any text.

  So synthesis checks every text literal the model writes (guards, `sets:`, payloads, view filters)
  against each scoped value it is about to use. On a match it appends `#2`, `#3`, … until the value
  is no literal of the model. A `Uuid` scoped value is a 48-bit digest (`witness.rs:697`). Its
  distinctness from other witness digests is not checked, and a collision is *inferred* negligible,
  not excluded.
- An aggregate view is **scoped** when at least one group key is scopable, or when its filter has a
  top-level conjunct `f == param.p` with `f` scopable. In the second case the scenario binds `p` to a
  scoped value `"<view>/in"` and gives every row the filter **admits** `f` = that value.
- **Unscoped** views (for example, grouped only by an enum with no scoping parameter) get no
  scenario and a new refusal: **`RefusalCause::AggregateUnscoped { view }`, `ESS-SYNTH-U`**. Hint:
  "group by, or filter by a parameter over, a `String` or `Uuid` field the creating command sets
  from its input". An exact aggregate over rows the scenario did not make is a claim about the
  target's other users, and this crate does not make those (`synthesize.rs:146-160`).
- Runs of one suite repeat these values, and the runner's correlation ids repeat too
  (`crates/verify/ess-conformance/src/runner.rs:170-216`). A target that keeps rows from an earlier
  run must isolate runs as §8 already requires. The aggregate scenario is where a target that does
  not will fail visibly.

A parameter read other than by one top-level equality conjunct gets
**`RefusalCause::AggregateUnwitnessed { view, reason }`, `ESS-SYNTH-W`**. The same cause covers
any arrangement below that the model cannot produce. Both causes map to `Effect::CheckNotEmitted`
(the coverage gates are under "Formats and versions").

### Arrangement: N rows through the creating command

One scenario per aggregate view: **`ScenarioId::Aggregate { view: ViewRef }`, rendered
`<view>/aggregate`**. Keyword `aggregate` joins the closed set (`scenario.rs:671-676`, `parse`
`:684`). The Go and TypeScript `scenarioIdentity` validators (`runtime.go:3357-3378`,
`runtime.ts:4165-4198`) admit `len == 2 && parts[1] == "aggregate"` with a qualified name first.
The version gates are under "Formats and versions".

Rows are created in this order. Each row runs the declared creating outcome, then
`expect_outcome`, then `capture_instance r<n>`. It is then advanced by the declared moves to a
state where the view's filter is **admitted** or **refuted**, as the row's column says.

| row | group | filter | value rule |
|---|---|---|---|
| a1 … a*m* | A | admitted | input `i` at A-row `j` (below) |
| b | B | admitted | input `i` = ladder `100·i + 85` |
| x | A | refuted | ladder `100·i + 97` |
| c | C | refuted | ladder `100·i + 93` |
| bₖ | Bₖ (below), one per group key after the first | admitted | ladder `100·i + 87` |

Which rows exist:

- **Grouped:** every row.
- **Ungrouped:** A's *m* rows and x only. There is no B, C or bₖ, because every admitted row is in
  the one group and the A pattern below is computed over exactly *m* rows.
- **No filter:** x and c are omitted.

#### Inputs and the A pattern

- **Inputs.** The view's **inputs** are the distinct aggregate arguments, in field declaration
  order, **excluding** any field that is a group key or the scoping field `f` (see "One field, two
  roles"). Number them `i = 0 … I−1`.
- **Group size.** *m* is the smallest integer ≥ max(3, I+2) with a prime factor other than 2 and
  5, so a mean over *m* rows can repeat forever at the 6th decimal. That gives 3 for I ≤ 1, 6 for
  2 ≤ I ≤ 4, 7 for I = 5, and 9 for I = 6 or 7. For I > 7 the view gets `ESS-SYNTH-W`.
- **Values.** Input `i` at A-row `j` (`j = 0 … m−1`) takes ladder ordinal `100·i + 1 + t(t+1)`, with
  `t = max(0, j − i − 1)`. In A, input `i` then has exactly **`dᵢ = m − 1 − i` distinct values**.
  These counts are pairwise different, they are all ≥ 2, and none equals *m*. Every group key and
  every other field the rows set holds one value across A, and the row identities give *m*. So
  `count_distinct` over the wrong input, over a key, over the identity, or "counting rows" each
  gives a different number from the declared one. Each input has its own duplicate pattern: the
  first `i+2` rows share its lowest value.
- **Non-terminating mean.** If an `avg` input's A mean `sumᵢ / m` would end within 6
  decimals, the last A-row's value for that input is raised by 1. A mean terminates only if
  `sumᵢ ≡ 0` modulo every prime factor of *m* other than 2 and 5. Raising the sum by 1 makes it
  `≡ 1` modulo such a factor, so the mean no longer terminates. The last
  row's value stays distinct from the others, because `i ≤ I−1 ≤ m−3` means that row has `t ≥ 1`.
  All A values stay below `100·i + 85`, so b, x, c and bₖ never repeat an A value (`t ≤ 7` gives at
  most `57`).
- **The ladder (type, ordinal `n`):** `Integer` `n`; `Decimal` `n`; `String`
  `"<view>/<path>/<n:03>"` (zero-padded, so byte order is numeric order); `Timestamp`
  `2026-01-01T00:00:00Z` plus `n` minutes, written with `Z`; enum variant `n mod len`; `Boolean`
  `n` odd; `Uuid` `uuid("<view>#<path>#<n>")`. On an enum or `Boolean` input the distinct counts
  collapse to the domain's size, and the input-distinguishing guarantee above holds only for types
  with at least `m` values. This is type-bounded, and the page states it rather than refusing it.

#### Group tuples

Each group has a **tuple** of group-key values. Tuples are assigned in the order A, B, C, B₁, B₂, …
and each one **must differ from every tuple assigned before it**:

- **A scoped key** takes `"<view>/<g>"` for group `g`. Such values are always distinct.
- **Any other key** walks its type's **value sequence**, `v₀, v₁, …`. That is the ladder at
  ordinals 0, 1, 2, … with repeats dropped: `Boolean` is `false, true`; an *n*-variant enum is its
  *n* variants in declaration order; `String`, `Uuid`, `Integer` and `Decimal` are unbounded.
- **A, B and C** start at `v₀`, `v₁` and `v₂` in every non-scoped key. A value past the end of a
  finite sequence wraps.
- **Bₖ** is B's tuple with key `k` replaced by **the first value in `k`'s sequence that makes the
  tuple differ from every tuple already assigned**. So Bₖ always differs from B **in `k` itself**,
  whatever `k`'s type:
  - `Boolean`: the negation of B's value;
  - a two-variant enum: the other variant;
  - an *n*-variant enum, `String`, `Uuid`, `Integer`: the first unused value.
  An implementation that ignores `k` merges Bₖ into B.
- **When a group cannot be distinguished:**
  - If `k`'s sequence has no such value (a one-variant enum, or every value already used), no bₖ
    row is arranged for `k`. Ignoring `k` is then unobservable, because every row holds the same
    value in `k`.
  - If C's tuple cannot differ from A's or B's (for example, one `Boolean` key and scoping by
    parameter), c is still arranged but lands in that group, and no `Excludes` is emitted for C.
  - The **Not killed** list below names both cases.

Example with two keys: `agent_id` (scoped) and `channel` (an enum `Voice | Chat`).
- A = (`…/A`, `Voice`); B = (`…/B`, `Chat`); C = (`…/C`, `Voice`).
- B₂ = (`…/B`, `Voice`). `Voice` is the first value of `channel` that makes the tuple new, and it
  differs from B's `Chat`.

#### One field, two roles

A field that is both a group key (or the scoping field `f`) and an aggregate argument takes the
**key's value**. The key value wins over the filter goal, and the filter goal wins over the ladder.

- Such a field is not numbered among the inputs.
- Its aggregate follows from the tuple. For example, a field
  `{name: agents, type: Integer, aggregate: {count_distinct: agent_id}}` in `TalkTimeByAgent` is
  `1` in every group, and `{count_distinct: queue_id}` in `QueueTotals` is `1` in the one row.
- If the filter can only be admitted or refuted by changing a key-held value, the goal search uses
  another conjunct. If none exists, the view gets `ESS-SYNTH-W`, naming the field.

#### Refuting a parameter-scoped filter

When the filter is scoped by `f == param.p`, the refuted row x must still be arranged. The goal
search tries to refute **another conjunct** first. `QueueTotals` leaves x in `Open`, which refutes
`state == Completed`. Only when no other conjunct can be refuted does x take
**`f = "<view>/out"`**, a second scoped value the query never binds.

In both cases the main read's single row excludes x. An implementation that ignores the filter
counts x there, either by state or by admitting another parameter's row, and fails `Contains`. The
common `QueueTotals` shape therefore needs no `ESS-SYNTH-W`. A filter made of nothing but
`f == param.p` is witnessed through `f = "<view>/out"`.

#### Reaching the values

This reuses the goal-directed arrangement of `cross-record-and-stored-field-guards.md`
("Conformance: arranging a row to a goal", parts 1–3).

- **The goal** is the row's key values, input values and filter truth, under the priority above.
- **Inputs** are chosen through the creating branch's `sets:` mappings, subject to that branch's
  own guards still selecting it.
- **The filter** is decided per row by the existing three-valued
  `shows(view, state, settled, params)` (`synthesize.rs:3577`).
- **States** are reached as `arrange_beside` reaches an admitted state (`synthesize.rs:3461-3481`),
  generalized to "admitted" or "refuted".
- **Failure:** a field the search cannot set, or a filter truth it cannot reach, gets
  `ESS-SYNTH-W`, which names the field or the row.

**Expected values** are computed by one function in `ess-conformance` (a new `aggregate` module)
over the rows' **determined** values (the `settled` map, `synthesize.rs:3235-3275`), under the
Semantics section. The pattern is the starting point, and expected numbers are always read off what
was arranged. Where the filter overrides an input's value, the guarantees above hold for the
inputs it did not touch.

### Observation

After the last row, the view is read once in the block its consistency decides, through the
existing `require` (`synthesize.rs:3403-3438`). The query uses the scoped parameter binding where
there is one. The block holds:

- **For every group tuple with at least one admitted row:** `Contains` (`scenario.rs:2137-2140`)
  with every group key and **every aggregate field** at its expected value. `min`, `max` or `avg`
  with no value is `Node::Null`, and `avg` is a `Node::Number` holding the rounded decimal.
- **For every group tuple whose rows are all refuted:** `Excludes` (`scenario.rs:2145-2148`) on its
  group keys (decision 4).
- **Ungrouped views:**
  - `Contains` of the aggregate fields and `Counts { at_least: 1, at_most: 1 }`
    (`scenario.rs:2206-2213`). A ceiling is sound here because "one row" is the view's own
    definition, not a claim about other users.
  - A second read follows with the parameter bound to the scoped value `"<view>/empty"`. It must
    hold `Contains { count: 0, sum: 0, count_distinct: 0, min/max/avg: null }` (whichever the view
    declares) and `Counts {1, 1}` (decision 4).

The runner is unchanged. `Contains` and `Excludes` match by `Node` equality (`runner.rs:2221-2225`,
`2291-2305`). `Decimal` values reach the runner as numbers, as `Decimal` input fields already do
in view assertions (`synthesize.rs:3260-3265`). The schema-string versus node-number disagreement
is the existing one (`review-primitive-semantics.md:42-45`). `select_fresh_format`
(`scenario.rs:155-168`) tests `crate::aggregate::used_by` first and selects `/Q`.

**The example.** `TalkTimeByAgent` has inputs `talk_seconds` (i = 0), `wait_seconds` (1) and
`caller` (2). So I = 3 and *m* = 6.

| field | A-rows a1…a6 | A result | b (group B) |
|---|---|---|---|
| `talk_seconds` | 1, 1, 3, 7, 13, 21 | sum 46; avg 46/6 = **7.666667**; 5 distinct | 85 |
| `wait_seconds` | 101, 101, 101, 103, 107, 113 | max **113**; 4 distinct | 185 |
| `caller` | `…/201` ×4, `…/203`, `…/207` | count_distinct **3** | `…/285` |

- **A row:** `{agent_id: "metrics.session.TalkTimeByAgent/A", sessions: 6, talk_seconds: 46,
  longest_wait: 113, distinct_callers: 3, mean_talk: 7.666667}`.
- **B row:** `{…/B, sessions: 1, talk_seconds: 85, longest_wait: 185, distinct_callers: 1,
  mean_talk: 85}`.
- **x** (A, left `Open`) carries 97, 197 and `…/297`. **c** (C, left `Open`) carries 93, 193 and
  `…/293`.
- 46 mod 3 = 1, so no adjustment was needed.

### The mutants it must kill

The unit's test runs the synthesized suite through `Runner` against an in-process `ConformanceTarget`
written in the test, following the precedent at `crates/verify/ess-conformance/tests/elapsed.rs:85`
and `:316`.

- The target is independent of the `aggregate` module, because a target computing through it would
  share its defects. It is switched by a `Mutant` enum.
- It is also run through the Go lane as `crates/verify/ess-conformance/tests/subject_history.rs:120-190`
  does (correct target green, one mutant red).
- The TypeScript lane is shown to refuse the suite version before any callback.
- The fixture adds a second, two-key view (`agent_id` scoped, `channel: Voice | Chat`), so that the
  Bₖ rule is exercised on a two-value type.

| mutant | caught by (example numbers) |
|---|---|
| ignores the filter | A: `sessions` 7, `talk_seconds` 143 with x; C's `Excludes` |
| ignores a group key (single-key view) | A's and B's `Contains` (one merged row of 7 sessions) |
| ignores the second key `channel` (two-key view) | B's and B₂'s `Contains` (merged into one row) |
| drops one admitted row (first or last created) | A's `sessions` 5, or B's row missing |
| emits empty groups (`count 0` for C) | C's `Excludes` |
| `count_distinct` counts rows | A: 6 against 3 |
| `count_distinct` over another input or a key | A: 5 (`talk_seconds`), 4 (`wait_seconds`) or 1 (`agent_id`) against 3 |
| `min`/`max` swapped | A: 101 against 113 |
| `sum`/`max`/`avg` over the wrong input | A: e.g. `sum(wait_seconds)` = 626 against 46 |
| `avg` truncates instead of rounding | A: `7.666666` against `7.666667` |
| ungrouped view returns no row when empty | `Counts {1, 1}` on the empty read |
| ungrouped view reports `min` 0 when empty | `min: null` on the empty read |

**Not killed:**

- **Half-even against half-away-from-zero.** No tie is arranged: a tie at 6 places needs a group of
  2⁷ or more rows.
- **`Excludes` on an `eventual` view.** An implementation that projects the c row after A's and B's
  rows are current passes C's `Excludes` on the read that satisfies the rest. The exclusion is
  decisive for `read_your_writes` views.
- **The type-bounded cases in "Group tuples":** a key with a single possible value, and a C that
  cannot differ from A or B.
- **Enum or `Boolean` inputs** with fewer values than *m*.

### Row-level consumers skip aggregate views

Every synthesis path that assumes one view row per entity row skips `is_aggregate()` views:

- `view_expectations` (`synthesize.rs:3067-3080`);
- invariant reads via `projections()` (`synthesize.rs:4075`, `holds_after` `:4106`);
- value-object positions (`synthesize.rs:4296-4302`);
- the subject-state observation view (`synthesize.rs:2655-2665`);
- the subject-fact views (`crates/verify/ess-conformance/src/synthesize/subject_fact.rs:121-130` and
  `:397-434`, which feeds `SubjectShape::of`, `crates/verify/ess-conformance/src/subject.rs:40`).

An aggregate view grouped by the identity and `state` would otherwise pass every one of those
pickers' field checks. **The class is closed by a check, not by this list:** a test synthesizes a
model whose aggregate view groups by the identity and `state`, and asserts that no scenario other
than `<view>/aggregate` names that view in any step.

## Deciding checks for the implementation unit

1. **Validation:** one case per V1–V15 (V3, V5 and V13 raised from `TryFrom<RawViewSpec>` together
   with any shape error, proving accumulation), each asserting the `ValidationCode`, the location and, after
   compiling, the `ESS-VIEW-00N` code.
2. **Format:** a document one below `ess/S` with `group_by` is refused with `ESS-VIEW-009`. Every existing
   fixture's source, IR and suite bytes are unchanged. `ess.schema.json` is regenerated.
3. **Semantics:** the `aggregate` module against the table: half-even at 6 places (`0.0000005` →
   `0.000000`, `0.0000015` → `0.000002`), zero-row values, `Decimal` `count_distinct` equality.
4. **Synthesis:** the example yields `metrics.session.TalkTimeByAgent/aggregate` with the steps
   above and the values in "Observation". `QueueTotals` yields both reads. An enum-only-keyed view
   yields `ESS-SYNTH-U`.
5. **Mutants:** every row of the table, Rust runner. Correct target and one mutant, Go lane.
   Refusal before callbacks, TypeScript lane.
6. **Class check:** the row-consumer test above.
7. **Projections:** OpenAPI description, docs, plan contract, Rust and Go doc lines, web and
   conformance catalogs, and the two diff variants at `ess-diff/D`.
8. **Admission gates:** a `/Q−1` suite carrying an `…/aggregate` scenario is refused by the Rust
   admission (`UnsupportedVocabulary`) and by the Go admission; a coverage document with a `U`/`W`
   refusal below `/Q+1` is refused in Rust, Go and TypeScript; a refusal-only suite with `U`/`W`
   is written at `/Q+1`.
9. **Version tables:** `FORMAT_RELEASES` rows for `S`, `Q`, `Q+1`, `D`, and the `formats.md` /
   `spec-versions.md` entries, so the `ess-xtask` docs lane passes; every "unsupported version"
   literal the grep in "Formats and versions" finds is moved.
10. **Group tuples:** the two-key fixture view yields a B₂ row whose `channel` differs from B's, and
    a `Boolean`-keyed variant yields B₂ with the negated value; a one-variant-enum key yields no
    bₖ row.
11. **Two roles and parameter refutation:** `count_distinct` over the group key is 1 per group;
    a view whose filter is only `f == param.p` arranges x with `f = "<view>/out"` and no
    `ESS-SYNTH-W`.

## Out of scope

- **Time buckets** (decision 2), including `Timestamp` group keys (V11).
- **Ratios and differences** between aggregates (decision 3).
- **`having`**: a filter over aggregate results. The filter reads source rows only.
- **Joins**: aggregates over more than one entity. `source:` stays one entity.
- **Windows and ranking**: running totals, top-N, and `order_by` on an aggregate view (V14).
- `Optional` inputs and group keys (V9, V11). `Duration`, `Bytes` and `Binary64` inputs (V8).
  Struct paths (V7). Percentiles, median, and distinct over more than one field.

## What was rejected

- **A new declaration kind (`summaries:`)** was rejected by decision 1. It would duplicate
  `source`, `filter`, `params`, `consistency` and every projection of a view.
- **Floors instead of scoping.** `count >= 3` survives a shared target, but it does not catch a
  mutant that ignores the filter (a count of 4 still passes). The story requires catching that.
- **Deriving the field type** instead of requiring `type:`. The view is the contract a consumer
  reads, and a written type makes it readable without the table. V10 keeps the two from drifting.
- **`avg` as `Binary64`** (the issue's alternative). A float mean differs between summation orders,
  so two correct implementations would disagree, and conformance has no `Binary64` witness to
  arrange one with (`crates/verify/ess-conformance/src/witness.rs:646`).
