# Family F expressions (source `ess/22`)

Status: coordinator correction after two independent design reviews. Review2's remaining receipt
boundary is corrected below, but this last correction has not been independently re-reviewed;
its exact seam must pass independent implementation review before A3 closes. This is not an
implementation, acceptance result, or shipped-support claim. The initial proposal was based
on `d35eafecf8b4ec5ff26ced16de969d4435baa7ee`; the coordinator rechecked execution/history seams at
`d75824465`. The immutable initial review is
`review-result:expression-family-source22-20261003-r1`.

## Boundary

This page defines the expression slices that remain beside the already approved row-set design:

- A1: a bare right-hand root is a typed fact, and a plain input guard gains the `input` namespace;
- A2: one constant offset on a right-hand Integer or Timestamp fact;
- A3: current time in stored-subject and related-row command predicates;
- A4: dotted input paths in `sets:`, event/error `payload:`, and an `else:` input fallback;
- C: `distinct: {in, as, by}` and String `.utf8_bytes`;
- #200: typed `{param: name}` and `{input: name}` operands for `starts_with`, `ends_with`, and
  `contains`.

The source allocation is `ess/22`. `ess/21` belongs to one-time responses. This proposal does not
redesign `when_related: {entity, where, exists|count|forall}` or filtered related values. Their
selection snapshot, cardinality, three-valued membership, subject borrowing, precedence, scope,
and target rules remain exactly those in `docs/design/filtered-related-reads.md:7-191`. Family F
expressions are checked inside that page's `where` and `forall` by the one checker described here.

General infix arithmetic, a left arithmetic expression, field-minus-field, Duration ordering,
calendar units, days, zones, case-insensitive prefix/suffix/substring operators, free row counts,
and a second row-set evaluator are out of scope. `a - b <= k` is written `a <= b + k`. A duration of
24 hours is `24h`; it is elapsed seconds, not a calendar day. This preserves #244's explicit calendar
and timezone deferral (`.engineering/planning/story/feature-request-244.md:51-69`).

## Compatibility and the two-stage representation

The current parser has only `Operand::Fact(FactPath)` and `Operand::Literal(FactValue)` and treats an
unquoted dotted word, or an in-scope binder, as a fact; every other right-hand word becomes a literal
(`crates/specify/ess-primitives/src/predicate.rs:331-385`). The type checker then refuses a text
literal which names an environment root (`crates/specify/ess-domain/src/expression.rs:981-1034`).
`Predicate` is the compiled IR payload at every condition site (`crates/specify/ess-compiler/src/ir.rs:583-667`)
and serializes through `to_node`, not through a second wire type
(`crates/specify/ess-primitives/src/predicate.rs:1688-1776,2038-2048`). These facts rule out resolving
bare words inside the syntax parser and rule out leaving an ambiguous bare word in a persisted new
predicate.

Use two stages:

1. **Lexical predicate.** Introduce a separate closed `LexicalPredicate` / `LexicalOperand` syntax
   tree. Only `LexicalOperand` has `UnquotedText(String)` for an unquoted, undotted right-hand text
   which is neither an in-scope binder nor a Boolean/Number. The lexical tree implements neither
   `Serialize`, persisted `to_node`, evaluation, nor IR digest input. It is never a variant of the
   existing resolved `Predicate` / `Operand`. Quoted compact text and a map scalar containing
   explicit quote characters retain `Literal(Text)`. This keeps the information that
   `FactValue::parse_literal` currently discards (`crates/specify/ess-primitives/src/facts.rs:735-766`).
   Diagnostic/source rendering may reproduce its written spelling, but cannot produce a resolved
   predicate or a persisted artifact. No evaluator receives an unresolved lexical word.
2. **Typed resolution.** The shared domain checker resolves the lexical tree against the source
   format, owner environment, left operand type, and binder stack, returning the existing resolved
   `Predicate` with the new closed resolved variants described below. IR condition fields,
   serializers, digest builders and suite predicate fields accept only this resolved type. The
   compiler repeats semantic validation for direct domain assembly instead of trusting a prior
   source parse. A shared generic traversal is acceptable; a single enum exposing lexical variants
   to persistence is not. No parallel untyped JSON predicate bag is introduced. Both domain and
   compiler keep using the shared environment/adapters in
   `crates/specify/ess-domain/src/expression.rs:774-888` and
   `crates/specify/ess-compiler/src/expression.rs:132-195`.

For a fact whose path has one segment, canonical output uses an explicit operand mapping because
`a == b` cannot be reparsed without an environment:

```yaml
a: {eq: {fact: b}}
```

Dotted facts keep their existing compact bytes. An offset has one canonical closed mapping:

```yaml
upper:      {eq:  {offset: {fact: lower, add: 5}}}
promoted_at: {lte: {offset: {fact: previous_at, subtract: 24h}}}
```

Exactly one of `add` and `subtract` is required; no extra keys are admitted. This is the serialization
of typed operands, not a free expression object. The source reader may accept these canonical forms
under `ess/22` so `Predicate::to_node` round-trips; authored compact spellings remain
`upper == lower + 5` and `promoted_at <= previous_at - 24h`.

Compile-fail/API controls prove that a `LexicalPredicate` cannot be assigned to an IR condition,
serialized, digested or written as a suite predicate. Direct assembly, ordinary source compilation,
canonical reader round-trips and old-source controls exercise the same resolver. The canonical
resolved reader accepts explicit tagged facts/selectors; it never reinterprets a scalar text using
runtime data or a guessed environment.

Below `ess/22`, lexical `UnquotedText` is converted to the old `Literal(Text)` before the existing checker
runs. Existing valid literals, enum variants, dotted fact operands, and the already shipped bare
binder fix retain their meanings and canonical bytes. New canonical operand mappings, `distinct`,
derived `.utf8_bytes`, dotted value reads, and typed string operands are refused in old sources.
A declared field actually named `utf8_bytes` retains its old meaning. Tests must
pin the old reader/refusal for each new shape and old canonical IR/suite bytes for unaffected
fixtures. An old binary still refuses `ess/22` at the format header before interpreting any word;
the supported source list is currently through 21
(`crates/specify/ess-domain/src/system.rs:52-60`).

## A1: bare fact operands and the input namespace

In source 22, an unquoted undotted right-hand word resolves in this order:

1. an in-scope quantifier binder is a fact (the existing #289 behavior, including nearest-binder
   shadowing);
2. when the left operand is enum-backed and the word is one of its variants, it remains the literal
   variant, preserving an admitted source such as `state == Open` even if another root is named
   `Open`;
3. an exact observable root of the predicate's environment is a fact;
4. otherwise it remains the text literal it meant before.

A quoted word is a literal and keeps the existing refusal when it merely disguises a declared root.
Booleans and numbers remain typed literals. A root named `now` wins as a fact; only the absence of
such a root allows the existing Timestamp/current-time reading. A1 changes only the right-hand side;
the compact grammar still requires a fact path on the left
(`crates/specify/ess-primitives/src/predicate.rs:1579-1625`).

Every comparison uses the existing representation compatibility policy: Bool/Bool, Number/Number,
or Text/Text for equality, and Number/Number or orderable Text/Text for ordering. Nominal wrappers do
not need to match, conversions are not applied, and enum literal membership remains checked
(`docs/design/review-expression-typechecking.md:48-64`). Missing either fact yields `Unknown` under
Kleene logic; it never becomes a literal because the row happened not to publish it.

A plain command-input `when:` also admits `input.<path>` over that same command input. The reserved
namespace is active only when the command does not declare a root field named `input`; a declared
`input` struct keeps its existing field meaning, matching the stored-guard precedent
(`docs/design/value-expressions.md:173-193`). In stored-subject and related-row environments,
`input.<path>` continues to name command input explicitly. Bare stored names name the candidate row;
view parameters remain `param.<name>`, never bare.

Required controls include same-identity input equality and inequality (#225), Integer/Timestamp
sibling fields, outer and inner binders, enum-variant/root collision, root named `now`, an absent
Optional operand, a missing root which stays literal, a quoted-root refusal, and both declaration
orders. A target hard-coding the right side as text, comparing the left with itself, or reading the
wrong namespace must fail independently. Synthesis grounds fact/fact guards with equal and unequal
values; its current further-witness mechanism for fact comparisons is the starting seam
(`docs/design/value-expressions.md:180-190`).

## A2: one constant offset

Add the typed resolved operand:

```rust
Operand::Offset(OffsetOperand {
    base: FactPath,
    direction: Add | Subtract,
    magnitude: Integer(Number) | ElapsedSeconds { seconds: i64, written_unit: S | M | H },
})
```

`Operand::Offset` is legal only on the right side. Its base is one resolved fact. Existing `now`,
`now - 60s`, and `now + 5m` remain the existing current-time literal representation and bytes; both
forms use one shared checked arithmetic helper after typing. No nested offset is representable.

- An Integer base takes one unsigned whole decimal magnitude with no sign or fraction. The explicit
  `add`/`subtract` supplies the sign. Zero is legal. The magnitude must fit `i64::MAX`; evaluation
  compares the mathematical sum/difference, even when that intermediate is outside `i64`. The full
  range from `i64::MIN - i64::MAX` to `i64::MAX + i64::MAX` fits signed 128-bit arithmetic. Rust uses
  `i128`, Go uses `math/big.Int`, and TypeScript uses `bigint`, or a proved equivalent exact carrier;
  no intermediate is converted through binary64 or back into a stored Integer. This is a comparison
  expression, not an assignment of an out-of-domain value. The left operand and base both resolve to Integer through
  Optional/newtype wrappers; Decimal and Binary64 are refused for this slice.
- A Timestamp base takes one whole nonnegative `s`, `m`, or `h` magnitude, with the current-time
  grammar and `CurrentTime::MAX_OFFSET_SECONDS` bound. Addition/subtraction means elapsed UTC
  seconds. There are no `d`, months, calendars, or zones. The left operand and base must both resolve
  to Timestamp through Optional/newtype wrappers.
- Integer offsets admit all six comparison operators. Timestamp fact offsets admit all six using
  instant equality/order. A current-time base retains the established restriction to `<`, `<=`,
  `>`, `>=`; `== now` and `!= now` stay refused
  (`docs/design/current-time-guards.md:20-46`).

If either fact is unobserved, an Optional on its path is absent, a Timestamp text is not a valid
instant, or a required clock is not supplied, the leaf is `Unknown`. An Integer intermediate outside
the stored domain is still an exact, orderable mathematical integer: it is not wrapped, saturated,
rounded or turned into `Unknown`. Timestamp arithmetic which cannot produce a
representable RFC 3339 instant is `Unknown` with the evaluator's existing ill-defined-comparison
note; it does not wrap or compare spellings. Negation preserves `Unknown`.

Synthesis tries the exact boundary and one unit either side for Integer, and one second either side
for Timestamp. It composes the base and left candidates rather than adding a general equation
solver. An unwitnessable conjunction retains the existing named no-witness result. Controls include
`upper == lower + 5`, `upper != lower + 5`, ordering at/beside the boundary, subtraction below zero,
Timestamp offsets in each unit, Optional absence, newtypes, malformed/double offsets, a Decimal
refusal, and mutants which ignore the sign, unit, base fact, or use field-minus-field.

Extreme controls must include `i64::MAX < i64::MAX + 1` and
`i64::MIN > i64::MIN - 1`, both True; equality against those out-of-domain intermediates is False.
Also compare an in-range opposite-end operand against the maximal admitted offset on each side.
Paired faults wrap, clamp to the endpoint, round through binary64, or return Unknown. Execute these
controls in each evaluator and actual generated Rust/Go and browser lane that accepts A2. Decimal
and Binary64 operands remain refused rather than being silently coerced into this carrier.

## A3: current time over stored and related rows

Source 22 enables the existing current-time operand in these command-time predicate environments:

- `when_subject.predicate` over the existing addressed subject;
- the stored predicate of identity-addressed `when_related`;
- the approved row-set selector's `where` and its `forall`, for both the row-set guard and the shared
  filtered-related value selector.

The optional ordinary input predicate beside those conditions already has current-time admission.
A command decision supplies one instant to every one of its predicates and every selector value
read. All stored and related facts still come from the one immutable pre-outcome snapshot required
by `filtered-related-reads.md:15-18`. `now` remains excluded from entity/type invariants, view
filters, ordinary selections, authored `satisfies`, set-effect filters, and payload-value arithmetic.
This slice does not promote `now` into a stored fact.

The primitive evaluator continues to receive the clock explicitly through `FactSource::now` /
`WithNow`; a missing clock yields `Unknown` (`docs/design/current-time-guards.md:48-73`). Synthesis
uses the fixed reference instant and the existing one-second margins. To arrange a stored Timestamp,
it must trace the field to a creator input and carry the chosen `ScenarioValue::NowOffset` through
that creator's `sets:` into the observed row. The existing runner fixes equal offsets once per
scenario, so arrangement and later value assertions refer to the same stored timestamp; this does
not freeze the tested command's current-time clock at setup time
(`docs/design/current-time-guards.md:75-127`). An implementation-generated Timestamp, a path inside
a value which cannot carry `now_offset`, a conversion that loses the instant, or a creator route
which cannot preserve the chosen value gets a named no-witness refusal identifying the stored path;
no clock value is fabricated.

Entity Runtime retains `CurrentTimeUnsupported` until it has clock authority
(`crates/generate/ess-entity-runtime/src/lib.rs:438,593`). Native and history execution are required
parts of A3 acceptance. The occurrence clock below supplies their missing authority; the current
blanket interpreter refusal is not an acceptable final implementation. This requirement also covers
ordinary input predicates in the same command so all decision sites agree.

### One observed decision instant per command occurrence

The command edge freezes one valid UTC instant immediately before deciding the outcome against the
pre-outcome snapshot. That instant belongs to this invocation, not to the scenario, the arrangement
creator, a row, a query, a retry origin or a history search branch. Every input, subject,
identity-related, row-set `where`/`forall` predicate and filtered value read within the invocation
uses that exact immutable value. No row iteration, branch retry or predicate evaluation calls a
clock again. A fresh retry that actually makes a new decision gets its own reading; returning a
retained idempotent result does not pretend to have made the original decision again.

Native execution gains an explicit typed entrypoint accepting an optional parsed UTC decision
instant, threading it through the private `interpret/execute/history.rs::Context` and every facts
adapter using `WithNow`. Existing clock-free entrypoints delegate with no clock. The native target
can be constructed with a caller-owned command-clock provider; the target calls it once at this
edge. The default has no provider and reads no host clock. Tests inject a scripted provider and
assert its read count. An unavailable instant produces Unknown only if evaluation actually needs
it; an earlier definitive input, identity or held-state refusal still wins. Remove blanket
pre-selection rejection of an otherwise executable command solely because a later branch reads
`now`. No Unknown predicate selects a default branch or licenses effects.

Generated Rust/Go source22 behavior needing this capability receives an explicit typed command-clock
capability in its context. Generated selection freezes the result once and reuses the same primitive
instant comparisons as native execution. The browser host supplies its explicit clock to actual
generated behavior. Missing capability yields a named unsupported result when the decision needs a
clock-dependent leaf, before any effect; earlier definitive refusals remain executable. An
adapter cannot silently choose host time, a fixed reference instant or the runner's arrangement
instant. Clock-free generated interfaces/bytes remain unchanged. A still-owed method is not proof
that generated current-time selection works: acceptance must compile and execute the produced
selector with both a healthy clock and the listed faulty variants.

### Recorded history format 2

`history.rs` currently admits only `ess-history/1`, whose `invoked_at` and `returned_at` are arbitrary
monotonic ordering coordinates, not UTC. They must never be converted into `now`. Allocate
`ess-history/2` for an optional operation field `decision_time: Timestamp`, recording the actual
frozen selection instant. The existing Operation entity in `models/concurrent-history/` owns this
field; regenerate its schema and Go/TypeScript writers through their owning generators. This is
separate from nominal clock-reading evidence for observed payload members and does not reinterpret
`observe_clock_reading` as a current-time source.

### Typed decision receipt from execution to recording

Do not add an unversioned field to `SemanticCommandResult` or fetch time from a target's shared
"last command" cell after execution. Add a separate in-process typed completion value:

```rust
struct RecordedCommandCompletion {
    answer: Result<SemanticCommandResult, TargetError>,
    decision_time: Option<DecisionInstant>, // parsed Timestamp captured by this command edge
}
```

This receipt is not a new public serialized result envelope. Only the history2 operation field
persists its clock fact. `DecisionInstant` wraps the already validated UTC Timestamp representation
and retains its precision; arbitrary strings and monotonic counter coordinates cannot construct it
without validation. An absent instant means no observed decision time, not an inferred epoch.

Add `ConformanceTarget::execute_command_recorded(request) -> RecordedCommandCompletion` with a
compatibility default that calls existing `execute_command(request)` exactly once and wraps its
answer with `decision_time: None`, including errors. An actual A3 adapter overrides the recorded
method and invokes the same command core as its ordinary method. That core returns its frozen
instant alongside the result; discarding the receipt for an ordinary caller cannot cause another
execution. The recorded method must not implement itself as execute-then-read-clock. The receipt
survives any post-decision transport/observation error, so `Err` may legitimately carry `Some(time)`;
an error before the command reaches the decision edge carries no invented time.

Keep existing `Interleaved::complete(pending)` source-compatible and add
`complete_recorded(pending) -> RecordedCommandCompletion`, whose default wraps `complete` once with
no time. `Atomic` overrides this method to call the target's recorded command method exactly once.
An A3-capable interleaved adapter carries the actual decision receipt in that operation's `Pending`
state until completion, including errors. It does not recover it from whichever operation happened
to finish last. The owning `InFlight.index` associates that receipt with the reserved Operation,
preserving invocation order even when completions arrive out of order.

`Recording::complete` consumes this receipt and writes its optional instant into the reserved
operation **before** branching on `answer`. `Ok` still writes Returned, outcome and the existing
monotonic return coordinate. `Err` still writes Indeterminate with no returned_at/outcome, but keeps
any captured decision_time. This is metadata-preserving error handling, not a fabricated successful
answer. `record`, injected/fault-session recording and subsequent Go/TypeScript explorer recorders
must use the same typed boundary. No recorder reads a UTC clock or normalizes an omitted receipt
into evidence. The existing clock-free defaults continue to produce unchanged history1 bytes.

Retained idempotent-result delivery with no new decision returns no fresh decision_time. The
original operation keeps its own captured instant, and the retry's `retry_of` remains the identity
link; neither the adapter nor recorder copies the original instant into a supposedly new decision.
If a retry actually re-enters command decision, its own receipt carries the one reading from that
decision. Existing retained-result semantic checking remains the authority on whether replay was
correct; the time receipt cannot make a duplicate effect legitimate.

Required controls record actual native and generated adapter execution through `Atomic` and a
non-atomic `Interleaved` fixture. Cover a returned result, failure before decision, failure after
decision, out-of-order completion of two distinct clock readings, missing evidence, and retained
retry. Require the exact same captured value in the operation and a provider read count of one
(zero for the clock-free retained delivery). Reread-at-completion, recorder-wall substitution,
wrong-Pending association, dropped error metadata and copied retry time are separate faults. The
indeterminate test includes a later observed effect/read that requires that operation to have taken
effect, so omitting it from a legal history search cannot accidentally hide the time corruption.
Check both semantic result/error truth and history verdict; do not accept a clock receipt alone as
proof that the command's branch or effects were right.

The format2 reader validates each present value as the existing Timestamp UTC-instant domain,
rejects explicit null, duplicate fields and unknown fields, and retains full supported precision.
The operation's existing unique identity scopes its reading; a recorder obtains it from the actual
command decision rather than the request's expected outcome or the recorder's wall clock. A
Returned operation may omit it only as missing evidence, never as an implicit zero/reference time.
An Indeterminate operation may have a reading if decision actually began, but no reading is guessed
from completion. Matching outcome alone cannot authenticate a fabricated clock: actual adapter
controls must corrupt the observation at its recording seam and demonstrate the verdict changes.

History exploration passes the recorded instant unchanged on every alternative/replay of that
operation. A missing reading at a necessary time leaf leaves the history alternative Unknown,
preserving the existing complete-domain and unresolved-alternative rules; it never proves a
violation or linearizability merely by enumerating guessed instants. Recorded clock facts cannot
make unobserved stored values known. Negative controls use a complete, decisive history so a
violating trace cannot escape through unrelated unresolved values or scheduling alternatives.

Format1 documents retain their exact bytes and meaning. The writer selects format2 iff any operation
records `decision_time`; forcing format1 with the new field is refused. A retained format1 reader
must refuse format2 before checking operations, and the current reader must reject format1 relabels
containing the new field. A format1 history with no time authority stays Unknown where time is
necessary. Controls round-trip format2 through Rust/Go/TypeScript, preserve untouched format1
fixtures, and reject malformed instants and unknown future majors. No suite-major allocation is
borrowed for this independent history envelope.

Controls use independently arranged subject and related instants, `now`, plus/minus s/m/h, each
boundary side, a generated stored instant refusal, a wrong related row, a target using setup time
instead of selection time, and a target comparing timestamp bytes. The precedence controls from the
approved row-set page remain mandatory; no time predicate may run before an earlier input/existence
refusal.

Use a scripted setup time T0 and later command time T1, with a deadline strictly between them, so
reusing setup time changes the outcome. A provider returning a different instant on every read must
still be called exactly once per executed decision. Use multiple rows whose deadlines straddle T1
to kill per-row rereads, wrong-row lookup, and post-effect snapshot reads. Native execution and
recorded format2 histories both exercise subject, identity-related, `where` and `forall`, all offset
units, missing-clock Unknown, and definitive earlier refusal without any clock. Old-format refusal
and actual generated/browser execution are required; no required A3 lane is satisfied by skipping.

## A4: dotted input value paths

Represent an input read as a resolved path, not a top-level name:

```rust
ResolvedInputPath {
    written: String,              // canonical declared segments joined by `.`
    segments: Vec<String>,        // typed traversal, not a wire alias
    type_ref: ResolvedTypeRef,    // terminal declared type
    optional: bool,               // any Optional traversed
}
```

`PayloadSource::InputField` and `ResolvedPayloadValue::InputField` retain their existing serialized
`field` key; a one-segment value emits identical bytes. The in-memory field becomes the resolved
path above (custom serialization writes `written` plus the existing `type_ref`).
`InputOrGenerated` likewise resolves its primary input path. Replace its literal-only fallback with
an untagged typed `ResolvedFallback`: a literal serializes as the existing `otherwise: "value"`, an
input fallback serializes as `otherwise: {input: <resolved path>}`. Generated fallback remains the
absence of `otherwise`. This keeps old IR bytes and gives a new input fallback its type and path.

Paths use declared names. Newtypes and Optional unwrap transparently; struct members consume one
segment. A primitive, enum, List, Map, Union, or Json cannot be traversed. The terminal may itself be
scalar or aggregate and must be assignable to the target, or covered by the existing declared
conversion rule. No ordinal/map-key selector is added.

For `input.a.b`, an absent Optional anywhere on the route yields absence. Copying that source to an
Optional target preserves absence; it cannot satisfy a required target. For
`{input: optional.path, else: input.defaults.value}`, any absent Optional on the primary route selects
the fallback. An input fallback must be statically required along its complete route and assignable
to the target; a second optional fallback is refused because `else:` promises a value. The two paths
are read from the immutable invocation input. There is no partial struct construction, same-named
top-level fallback, zero/default value, or mutation before all required reads succeed.

The current parser already stores `input.a.b` in the `field: String` slot and fails later because
resolution searches only a top-level name (`crates/specify/ess-domain/src/command.rs:924-1054`,
`crates/specify/ess-compiler/src/resolve.rs:2567-2608`). The explicit input and fallback readers also
currently enforce one segment/literal-only (`crates/specify/ess-domain/src/command.rs:1422-1425,1483-1493,1566-1587`). Those are the source22 gates to replace. Runtime/native and generated targets
must use structural reach, replacing direct `input.get(field)` calls
(`crates/verify/ess-conformance/src/interpret/execute.rs:1535-1565` and the Rust/Go emitters at
`crates/generate/ess-synth/src/rust/behaviour.rs:1594,1639`,
`crates/generate/ess-synth/src/go/behaviour.rs:2257,2332`).

Synthesis exercises each source twice where fallback is present: primary present, then primary
absent with the required fallback present, retaining every other branch-selecting input as E4
requires (`docs/design/value-expressions.md:90-125`). It also covers absent Optional parents, nested
newtypes/structs, two siblings of the same type, same-named top-level decoys, sets, event payloads,
error payloads, nested target leaves, declaration order, and conversions. Mutants which read the
root object, first child, same-named top-level field, primary after absence, or literal spelling of
the fallback must each fail.

## `distinct`

Add one predicate variant:

```rust
Predicate::Distinct(Distinct {
    over: FactPath,
    bind: String,
    key: Option<FactPath>,
    key_kind: DistinctKeyKind, // Boolean | Integer | Decimal | String | Uuid | Timestamp | Enum
})
```

Its authored source form is:

```yaml
distinct: {in: files, as: file, by: file.path}
```

`by` is optional. With no `by`, the List element itself must resolve to one admitted scalar. With
`by`, the path must be rooted at `as`, must consume at least one member, and must resolve to one
scalar. The admitted equality domains are Boolean, Integer, Decimal, String, Uuid, Timestamp, and
enums through newtypes, matching `count_distinct`'s established scalar domain and equality
(`docs/design/aggregate-views.md:171-190`). Whole struct/list/map/union/Json equality is excluded;
a struct list uses `by`. Maps are excluded because distinctness is defined over an ordered List,
not the map-value enumeration policy of quantifiers.

The lexical/source form omits `key_kind`; the shared checker resolves and persists that closed enum
alongside `in`, `as`, and optional `by`. The canonical resolved form includes `key_kind`, and the
canonical reader requires it rather than inferring Timestamp from string spelling. Direct typed
assembly rechecks the tag against the declared key type; source compilation rejects a supplied
canonical tag that disagrees with the resolved type. Suite readers require the tag and its new
format, so an untyped old document cannot silently gain instant equality. An observed key outside
its declared kind is Unknown, not silently coerced.

A present empty or one-element list is `True`. For two or more elements, equal known keys make the
result `False`; if every key is known and pairwise unequal it is `True`; otherwise it is `Unknown`
unless a known duplicate already makes it `False`. An absent Optional list is `Unknown`, not empty.
An absent Optional element/key is unknown and is neither skipped nor treated as one common null.
Number equality is exact numeric equality; Timestamp equality is instant equality; String/Uuid/enum
are exact text equality. Negation is Kleene.

Evaluation needs no aggregate fact value: `FactSource::cardinality(over)` gives the size and each key
is read at `over.<index>` or by rebinding the `as` prefix, the same mechanism current quantifiers use
(`crates/specify/ess-primitives/src/predicate.rs:616-670`). `visit_free_paths` records `over` and any
free reads, while the key under `as` is bound. The shared checker reuses its List/binder resolver;
`Access.collection` remains explicit.

Witnessing uses two deliberate ladders. A duplicate case repeats element zero, which the existing
list builder already does for count-built lists (`crates/verify/ess-conformance/src/witness.rs:3466-3508`). A distinct case builds element one at `Distinction::further(1)` and varies the selected key
while satisfying the element type's invariants. Empty/one-element inputs remain controls, but a
decisive positive test also requires `list.count > 1` so vacuity cannot pass. A finite/singleton key
domain which cannot supply enough unequal values yields the existing named no-witness refusal; the
validator does not pretend to prove cardinality satisfiability.

Controls cover scalar lists, struct lists by a nested member, newtypes/enums/Timestamp/Decimal,
duplicate first/last, Optional list absence, Optional key absence, reordered distinct values, and
mutants which compare identities/indices, skip missing keys, compare only adjacent pairs, or always
accept lists of length two.

Equality controls include two lexically different spellings of the same instant
(`2020-01-01T00:00:00Z` and `2019-12-31T19:00:00-05:00`), and Decimal `1` versus `1.0`; each pair is a
duplicate. Integer `9007199254740992` and `9007199254740993` are distinct and must never share a
binary64 carrier. The same timestamp spellings in a String list stay distinct. Use three Boolean
keys with required length three to prove that a distinct witness does not exist; a length-two
`[false,true]` case must have one. A singleton enum with required length two produces the named
no-witness refusal, while a two-variant enum must produce a healthy two-key witness. Both finite
cases preserve the element invariants and must not substitute an empty/one-item list.

Every applicable native, generated Rust/Go, suite Rust/Go/TypeScript and browser lane executes the
same exactness vectors. Paired faulty variants use lexical equality for Decimal/Timestamp, numeric
rounding for large Integers, instant equality for String, invented fresh Boolean/enum values, or
vacuous shortened witnesses. Runtime faults fail a predicate/outcome observation; witness faults
fail synthesis admission/coverage rather than being mislabeled as target failures. An unwitnessable
positive branch is not a skipped passing scenario and cannot close the unit.

## String `.utf8_bytes`

`.utf8_bytes` is a terminal selector on String and newtypes/Optional wrappers around String, beside
the existing `.count`. It resolves to Integer and returns the exact length of the observed UTF-8 byte
sequence. It is not valid on Bytes, Timestamp, Uuid, enums, collections, or another Integer selector;
no continuation after it is legal.

Persist a closed resolved selector `Utf8Bytes { parent: FactPath }`, distinct from the ordinary
declared-field selector `Field(FactPath)`; transient `Access.text_utf8_bytes` alone is insufficient.
Comparison left operands and resolved fact operands carry this selector type. A plain Field
serializes exactly as the previous FactPath; the lexical tree alone carries the unresolved spelling.
The shared resolver first follows declared struct members. Thus a declared member literally named
`utf8_bytes` stays an ordinary field, with its declared type and old bytes. Only a terminal segment
after a String resolves to `Utf8Bytes`. Primitive evaluators never derive selector kind by suffix.

For a derived selector the canonical operand is `{utf8_bytes: <parent>}`. A comparison with a
derived left selector uses the closed canonical form
`{compare: {left: {utf8_bytes: label}, op: lt, right: 4}}`; a derived right selector uses the same
tagged operand. Require exactly `left`, one of the six comparison `op` values, and `right`, with
no extra keys. Source22 may read this canonical form and recheck its parent type; older source and
suite readers refuse it. Plain field comparisons retain their existing canonical forms. This is
one resolved comparison and one evaluator, not an independent comparison language.

For `Utf8Bytes`, an explicit full-path observation for `parent.utf8_bytes` still wins; otherwise read
the parent text and return `FactValue::count(text.len())`. An explicitly present invalid observation
is Unknown, with no fallback to the parent. A `Field` selector performs only a declared-field read.
This preserves the existing direct-binding rule for derived leaves
(`crates/specify/ess-primitives/src/facts.rs:1131-1159`) while persisting the distinction needed by
format admission. A field merely named `utf8_bytes` must not select suite40 or be refused in39.
Absent/unobserved/`null` text yields no fact and comparisons are `Unknown`. Empty text yields zero.
No Unicode normalization occurs.

Synthesis uses the existing 1024 count-witness cap and named above-cap refusal, but resizes by bytes
without emitting invalid UTF-8: ASCII supplies exact byte lengths; multibyte fixtures prove the
selector itself with `é` (2 bytes, 1 scalar) and U+1F600 (4 bytes, 1 scalar). Required controls make
`.count` and `.utf8_bytes` disagree and kill scalar-count, UTF-16-unit, grapheme-count, and code-point
mutants. Generated Rust uses `str::len`, Go `len(string)`, and TypeScript `TextEncoder().encode(text).length`
or an exactly equivalent UTF-8 encoder.

Compatibility controls separately cover a declared struct member named `utf8_bytes`, the derived
selector on a String, and an explicit full-path binding overriding parent-derived length. Relabel
the tagged selector as suite39 and require admission refusal; relabel the ordinary declared field
as39 and retain its previous admitted meaning. Round-trip both under40/41 without changing selector
identity. A suffix-guessing reader, an untagged derived serializer, and a parent-first reader are
separate faulty controls.

## Typed text operands (#200)

Change `TextMatch.value` from `FactValue` to an untagged typed operand:

```rust
enum TextOperand {
    Literal(FactValue),
    Fact { namespace: Param | Input, name: String, path: FactPath },
}
```

Literal scalar serialization is unchanged. New authored/canonical forms are:

```yaml
filter: {note: {contains: {param: query}}}
when:  {phone: {starts_with: {input: prefix}}}
```

Exactly one key and one declared top-level name are admitted. This story does not add dotted typed
text operands; use a top-level String/newtype String parameter or input. `param` is legal in a view
filter; `input` is legal in command input, stored-subject, identity-related, and approved row-set
predicate environments. Invariants and authored `satisfies` have neither namespace. Both sides must
resolve to String or a newtype of String; Optional is admitted and absence yields `Unknown`. An
observed non-text value in an unchecked predicate yields `False`, preserving the existing primitive
table. Evaluation remains byte-wise and case-sensitive.

Bare `param.query` and `input.prefix` remain literals; they are never reinterpreted. Below source22,
every previously admitted one retains that literal meaning. At source22, when the namespace and
following name are declared in that predicate environment, the literal spelling is refused with a
repair to `{param: query}` or `{input: prefix}`, as #200 requires
(`.engineering/planning/story/feature-request-200.md:44-80`). An undeclared namespace-looking literal
continues to be literal; this unit does not broaden the accepted defect fix. There are no ignore-case
prefix/suffix/contains forms.

Witnesses choose one RHS value and left texts which match and differ at the deciding byte for each
operator, plus Optional absence. View synthesis uses the same parameter value for query and row
arrangement; guard synthesis sends the input. Controls kill treating the mapping as its literal JSON,
using the wrong parameter/input, fixing the RHS at generation time, reversing operands, or turning
absence into empty text. Entity Runtime can lower a typed RHS as the existing reference operand in
`Condition::{StartsWith,EndsWith,Contains}`; its current literal-only seam is
`crates/generate/ess-entity-runtime/src/lib.rs:3687-3701`.

## Suite formats and old readers

Suite majors 36–39 are already reserved by coordinated work. Allocate **40 ordinary / 41 coverage**
for Family F predicates which persist new reader semantics:

- `Operand::Offset`;
- a one-segment RHS `Operand::Fact` emitted through `{fact: ...}`;
- `Predicate::Distinct`;
- the tagged `Utf8Bytes` selector (not an ordinary member with that spelling);
- a nonliteral `TextOperand`.

A1 fact/fact predicates with only already unambiguous dotted operands need no new major, but using the
new bare-fact canonical shape selects 40/41. A3 guards are decided by target execution and existing
`now_offset` scenario values already require 26/27; A4 uses existing execute/event/row value shapes.
Neither selects 40/41 unless another persisted Family F predicate occurs. A suite with none of the
listed forms keeps its prior minimum version and bytes.

Add one construct-owned format module with `used_by`, ordinary/coverage selection, and admission
before predicate execution. Rust, generated Go, and generated TypeScript must all admit through 41,
refuse a listed form below 40, and refuse an unknown future major. The current closed lists end at 35
(`crates/verify/ess-conformance/src/scenario.rs:387-395`,
`crates/verify/ess-conformance/src/go/mod.rs:359-389`,
`crates/verify/ess-conformance/src/ts/mod.rs:67-75`). Tests relabel every new predicate as 39 and
prove refusal before evaluation; a 40 ordinary and 41 coverage suite then executes it. No reader may
report `skipped` for an unknown expression and count that as success.

## Target and projection obligations

| Lane | Required Family F disposition and decisive control |
|---|---|
| Shared Rust predicate evaluator | Execute A1/A2, distinct, byte length, and typed text operands with the exact Truth tables above; fixed `WithNow` cases execute A3. Each mutant changes one operator/path/sign/presence rule. |
| Domain/compiler | One source22 gate, one shared typed checker, resolved predicates/value paths only, and exact old-source/old-byte controls. Direct compile and assembly agree. |
| Synthesis | Arrange both branches and all dual-presence cases; return existing named no-witness outside the bounded search. Healthy plus wrong-fact, wrong-sign, duplicate, scalar-count, and literal-RHS targets must separate. |
| Native interpreter/history | Execute every admitted form, including A3 with the explicit occurrence clock and history2 contract above; preserve Unknown/complete-domain authority and earlier definitive refusals when the clock is absent. |
| Generated Rust and Go applications | Emit, compile, and run A1/A2/A3, distinct, byte length, typed text operands, and A4 through actual behavior/storage seams, with one explicit clock read per decision. Patch actual emitted seams for each faulty control. |
| Rust/Go/TypeScript suite runtimes | Admit and execute every emitted 40/41 predicate/observation with zero required skips; malformed and relabelled-old documents refuse. Go/TS predicate evaluators currently own the comparable operator switches at `crates/verify/ess-conformance/src/go/predicate.go:167-174,382-396,691-745` and `crates/verify/ess-conformance/src/ts/predicate.ts:251-350,544-605`. |
| Live browser/WASM | Run the generated Rust behavior for supported generated constructs in the actual browser host. Browser replay alone is not target evidence. |
| Entity Runtime | Lower A1 and typed text facts where entity-core already has reference operands. Keep named `CurrentTimeUnsupported`; add named offset/distinct/UTF-8-byte/value-path limitations wherever entity-core lacks the required operator or atomic value reach. Never lower them as literals. |
| Diff | Treat resolved predicate/operand/selector/value-path changes as behavior changes. `written_condition` is the existing condition seam (`crates/verify/ess-diff/src/diff.rs:967-1015,1516-1552`). |
| Docs/schema/OpenAPI/AsyncAPI | Schema/docs describe exact source22 forms and refusals. Command/view wire schemas stay driven by declarations; expressions do not add request fields or endpoints. Generated docs render typed facts/offsets rather than their canonical mapping as prose. |
| Explorer/authored scenarios/selection plans | Execute admitted 40/41 forms when their environment supplies all reads; otherwise return a named unsupported/capability refusal. A skipped expression is not acceptance. |

Every supported target gets its own healthy and faulty result. A golden file, compiler acceptance, or a
Rust evaluator result does not cover generated Go, generated TypeScript suite execution, Entity
Runtime, or the browser.

## Required source seams

Implementation should begin at these inspected seams rather than add parallel evaluators:

- parse/evaluate/render/walk: `crates/specify/ess-primitives/src/predicate.rs:331-400,719-900,1011-1070,1210-1261,1397-1489,1688-1835,1950-1988`;
- derived String selectors and exact facts: `crates/specify/ess-primitives/src/facts.rs:660-755,1131-1159`;
- one semantic checker, Optional/path metadata: `crates/specify/ess-domain/src/expression.rs:57-209,620-765,802-888,981-1133,1383-1435`;
- value-source parse/type/resolve: `crates/specify/ess-domain/src/command.rs:924-1118,1422-1590,3570-3615`, `crates/specify/ess-compiler/src/resolve.rs:2567-2608,2723-2818,4861-4916`, `crates/specify/ess-compiler/src/ir.rs:979-1188`;
- typed projection/witness: `crates/verify/ess-conformance/src/input.rs:1338-1405`, `crates/verify/ess-conformance/src/witness.rs:262-327,2066-2079,3160-3185,3466-3555`;
- runtime value reading: `crates/verify/ess-conformance/src/interpret/execute.rs:1448-1570`;
- generated predicate/value emitters: `crates/generate/ess-synth/src/rust/invariant.rs:276-291`, `crates/generate/ess-synth/src/rust/selection.rs:141`, `crates/generate/ess-synth/src/go/selection.rs:174`, and the Rust/Go behavior locations cited above;
- suite runtime predicate readers: Go/TypeScript paths in the target table;
- Entity Runtime lowering: `crates/generate/ess-entity-runtime/src/lib.rs:3508-3565,3618-3738`.

## Acceptance set

The minimum named matrix is:

1. A1 self-edge equal/different; sibling numeric and timestamp comparisons; explicit `input` namespace;
   nested binder; enum/root and `now`/root precedence; Optional absence; old literal bytes.
2. A2 Integer equality/order on boundary and both sides, including exact out-of-domain intermediates
   at both i64 extrema; Timestamp s/m/h; add/subtract; invalid/double offset; Decimal and left-arithmetic
   refusals; wrap, clamp, binary64-rounding and accidental-Unknown faults.
3. A3 subject and identity-related stored instants; row-set `where` and `forall`; pre-outcome snapshot;
   one command instant; creator-input `now_offset`; generated instant refusal; precedence controls;
   setup-versus-decision and per-row clock faults; native/history2 actual execution and history1
   byte/refusal compatibility.
4. A4 direct/fallback reads through required and Optional structs in sets/event/error payloads and
   nested targets; primary present/absent; decoy siblings/top-level names; old-source refusal.
5. Distinct scalar and struct-key lists; 0/1/2+ values; known duplicate domination; Optional
   list/key Unknown; nonadjacent duplicate; invariant and view `satisfies` persistence; typed timestamp
   versus String equality, Decimal lexical differences, Integers above2^53 and finite Boolean/enum
   witness exhaustion with their paired faults.
6. UTF-8 byte lengths for ASCII, composed/decomposed text, U+1F600, empty and absent; `.count`
   disagreement; cap refusal; invariant/view persistence; tagged selector versus declared same-name
   member, full-path direct binding, suite39/40 identity/refusal controls.
7. #200 view parameter and every admitted input guard site for all three operators; Optional RHS;
   wrong namespace/type/site; source21 literal preservation; source22 declared-literal repair.
8. Suite 39 relabel refusals, 40/41 healthy and faulty execution in Rust/Go/TypeScript, and unchanged
   suite selection/bytes where no new persisted predicate occurs.

For each family, declaration order and unrelated decoys are passing metamorphic controls. Each faulty
target must fail for the intended outcome/event/row/predicate mismatch, not at setup, admission, or an
unrelated obligation.

## Coordinator allocations and sequencing

Root owns the combined source22 schema, documentation and projection update. The #282 admission
candidate introduces source22 once; subsequent Family F slices add their construct-specific gates
under that allocation. No unit takes another source version merely to avoid composition tests.
Root reserves suite40/41 for the persisted resolved expression vocabulary and history2 for observed
decision-time records. They follow existing suite36/37 binding and38/39 aggregate reservations.
An allocation change requires another explicit coordinator decision and compatibility review;
implementors do not pick a different pair or reuse an existing one.

Implement A1/A2/A4, then the occurrence-clock plus A3, distinct/UTF-8 and row-set composition, with
typed text operands last as the runbook specifies. Shared predicate/IR/runtime work integrates
serially. Required observation/history model/schema/writer updates accompany A3 before it closes;
later filtered-related acceptance cannot claim current-time support until this evidence is green.

This coordinator correction binds the occurrence-clock, typed completion receipt and history2
contract as part of A3. Both full design-review passes are exhausted. The receipt correction remains
an explicit independent implementation-review obligation; neither a design edit nor the previous
named refusal is behavior acceptance.

## Final review decisions (2026-10-04)

A third, final independent review of this page against the integration branch at `26ad39057`
returned needs-revision with three blockers and sixteen further findings, each with a proposed rule.
The bundle coordinator adopted the rules below; they bind the implementation units and supersede any
earlier sentence they contradict.

1. **Collection and text-length reads in suites.** Under suite 40/41, `projection_target` admits
   collection reads and text-length reads in `satisfies`; `row_facts` binds sequence elements and
   `.count` the way the Go runtime already does, and `.count` is lifted together with `.utf8_bytes`.
   Rust, Go and TypeScript runners then agree on `distinct` and `.utf8_bytes` invariants over view rows.
2. **Timestamp fact-versus-fact comparisons.** The checker tags a resolved `Compare` whose two operands
   resolve to Timestamp (`{compare: {left, op, right, as: timestamp}}`); a tagged comparison selects
   suite 40/41 and every suite runtime compares the operands as instants. Synthesis refuses to emit
   `satisfies` for an untagged Timestamp fact-versus-fact comparison with a named refusal.
3. **#200 on the generated lane.** #200 includes generated view-parameter application: generated Rust
   and Go decode the query string through the existing view-query decoder and apply `{param: name}`
   filters, so a generated view query with `params:` stops being an obligation for the admitted
   operators.
4. **Offset parsing.** Rule 3a: an unquoted right side `<binder | root | dotted path> ws? (+|-) ws?
   <magnitude>` whose base resolves is an `Offset`; a type mismatch is `type_mismatch`, an unresolved
   base stays a literal. Integer magnitudes follow the current-time digit rules (no leading zero).
5. **Binder bytes.** One-segment facts that resolve to an in-scope binder keep the compact form and do
   not select 40/41; only root facts are written `{fact:}`. A root shadowed by a binder is refused at
   source.
6. **`input.` in a plain `when:`.** Resolution rewrites `input.<path>` to the root path in the plain
   command-input environment; a one-segment result is written `{fact: x}`; no evaluator changes.
7. **Diff.** Conditions are rendered from canonical `to_node`, so a literal that becomes a fact, an
   offset or a derived selector is reported as a behaviour change; a control proves it.
8. **Dotted identity and reference sources.** A dotted path supplies an identity or a reference address
   only when its whole route is required; existence and precedence read it structurally, and their
   controls repeat with a nested source.
9. **#225 arrangement.** Equal case: both inputs name one arranged instance. Unequal case: a second
   instance is arranged, or synthesis refuses with a named ESS-SYNTH reason. Identity tokens admit only
   `==` and `!=`.
10. **Suite format ordering.** 40/41 are cumulative over 36–39 and each construct keeps its lower-bound
    gate. Until the 38/39 readers land, a 40 reader refuses 38/39 vocabulary as unsupported; the final
    candidate admits 36–41 together.
11. **`.utf8_bytes` position.** A separate operand variant (`Operand::Derived(Utf8Bytes(path))`), legal
    only in `Compare` operands; every other position refuses it.
12. **ESS-VIEW-003 hint.** The hint names `{param: <name>}` for text operators; a control asserts it.
13. **#228 controls.** Named scenarios `unique_absent_accepts`, `unique_present_refuses`,
    `second_create_refuses`, `decoy_per_conjunct_accepts` and `composes_with_existing_instance`; an
    Optional key needs a documented `defined(k)` conjunct, tested.
14. **Instants in Go and TypeScript.** Both implement the grammar of Rust's `Rfc3339Instant` (up to
    nine fraction digits, no leap second, lower-case `t`/`z`, ± offset) and share one vector file with
    Rust.
15. **Citations** are refreshed by the units that touch them.
16. **Further lanes.** Mutation gets operators for `Offset` and `Distinct` or an explicit exclusion;
    `ess-ui-check` filters, `ess-gen` types and selection plans refuse a non-literal text operand
    unless a unit implements it.
17. **Non-literal text operands in suites** are dropped from the 40/41 triggers.
18. **Clock-edge faults** are the authority of the scripted-provider controls; history verdicts check
    recorder corruption only.
19. **Byte length edge cases.** The difference from `.count` bound facts is documented, and text with a
    lone surrogate is Unknown in every lane.

Implementation follows the review's unit cut: U1 resolver and A1, U2 offsets, U3 value paths, U4
occurrence clock and history 2 (in parallel with U1–U3), U5 `now` over stored rows, U6 `distinct`,
U7 UTF-8 byte length, U8 row sets and filtered reads, U9 typed text operands; integration in that
order, with U4 before U5.

## U1 implementation seams (2026-10-04)

Refreshed citations for the seams unit U1 (resolver, A1, decision 2 tagging, the /40–/41 pair)
changed. The citations above describe the tree the design was written against and are kept as
that record.

- Lexical capture: `Predicate::from_node_spelled` / `parse_expression_spelled` and `Spelled`
  (`crates/specify/ess-primitives/src/predicate.rs`, `Operand::parse_in` at :392); the lexical
  tree, the per-site capture `Written::from_document` and `resolutions` in
  `crates/specify/ess-domain/src/expression/lexical.rs`; the source document is read beside the
  typed fields in `RawSpecFile::parse` and resolved after assembly in `Specification::resolve_written`
  (`crates/specify/ess-domain/src/spec.rs`).
- Resolver and checker: `resolve_lexical` (:818), `read_input_namespace`, the instant tagging and
  `check_predicate` (:1045) in `crates/specify/ess-domain/src/expression.rs`; the format gate and
  shadowing rule beside `Checker::compare` (:1550); the quoted-root refusal in `text_literal`
  (:1258); the identity-token ordering refusal `identity_orderings` in `lexical.rs`.
- Canonical forms and evaluation: `CompareKind` and `Predicate::Compare { kind }`, `to_node`
  (:2002), `Display` through `InScope` (:2364), the tagged reader `tagged_compare` and the instant
  branch of `evaluate_compare` (:943) in `predicate.rs`.
- Suite format: `crates/verify/ess-conformance/src/expression_format.rs`; the registered majors
  `SUPPORTED_SUITE_FORMATS` (`scenario.rs:401`); the Go emitter cap (`go/mod.rs:370`) and runtime
  `suiteMajor`/`suiteMajorsNotRead`, `admitFactOperand`, `predicateUsesFactOperand`
  (`go/runtime.go`); the TypeScript `SUITE_MAJORS`, `admitFactOperand`, `predicateUsesFactOperand`
  (`ts/runtime.ts`) and emitter cap (`ts/mod.rs:66`); readers `parseFactOperand` and
  `parseTaggedCompare` in `go/predicate.go` (:313, :412) and `ts/predicate.ts` (:548, :822); the
  shared vectors `crates/specify/ess-primitives/tests/vectors/root-fact-operand.json` and
  `rfc3339-instants.json`.
- Synthesis: `RefusalCause::InstantComparisonUntagged` (`ESS-SYNTH-021`) and `untagged_instants`,
  and the shared-witness instance binding in `supply` (`crates/verify/ess-conformance/src/synthesize.rs`).
- Generated guards: `Kind::Instant` in `crates/generate/ess-synth/src/determined.rs`;
  `compare_instants` / `compareInstants` in `rust/behaviour.rs` and `go/behaviour.rs`.
- Diff: `written_condition` (:1007) and `written_invariants` (:370) in
  `crates/verify/ess-diff/src/diff.rs`.

## U2 implementation seams (2026-10-05)

Refreshed citations for the seams unit U2 (A2, one constant offset) changed.

- Operand and canonical form: `Operand::Offset`, `OffsetOperand` (`spellings`, `integer_at`,
  `instant_at`, `value_at`, `to_node`), `OffsetMagnitude` and `OffsetDirection`; the reader arm in
  `Operand::fact_mapping`; `Spelled::offsets`; `Predicate::evaluate_offset` and
  `Predicate::reads_offset` (`crates/specify/ess-primitives/src/predicate.rs`). The shared
  elapsed-time grammar and arithmetic are `ElapsedUnit::parse_magnitude` and
  `Rfc3339Instant::plus_elapsed`, which `CurrentTime` now uses (`crates/specify/ess-primitives/src/time.rs`).
- Resolver and checker: `resolve_lexical_reading`, `offset_spelled` and `base_resolves` (rule 3a),
  `Checker::offset` and `Checker::malformed_offset`, and `TypeEnvironment::is_integer` in
  `crates/specify/ess-domain/src/expression.rs`; the lexical capture keeps an offset spelling as
  `LexicalOperand::UnquotedText`, and a dotted path with an unspaced `-` (`window.lower-5`) as
  `LexicalOperand::DottedSpelling`, decided through `Spelling` (`expression/lexical.rs`). A stored-row
  guard against an offset of an input is grounded as that input against the held value moved back
  (`ground_leaf`, `synthesize/subject_fact.rs`).
- Suite format: `expression_format::reads`; the Go reader `parseOffsetOperand`, `admitOffsetOperand`
  and `compareOffset` (`go/predicate.go`, `go/runtime.go`) and the TypeScript `parseOffsetOperand`,
  `admitOffsetOperand`, `compareOffset` and `predicateNumbers` (`ts/predicate.ts`, `ts/runtime.ts`);
  shared vectors `crates/specify/ess-primitives/tests/vectors/offset-operand.json`.
- Synthesis: `offset_copies` beside `equality_copies` (`crates/verify/ess-conformance/src/witness.rs`).
- Generated guards: `offset_supported` (`crates/generate/ess-synth/src/determined.rs`), the
  `compare_offset_integers`/`compare_offset_instants` and `compareOffsetIntegers`/`compareOffsetInstants`
  helpers in `rust/behaviour.rs` and `go/behaviour.rs`; generated invariant checks refuse an offset
  by name (`rust/invariant.rs`, `go/invariant.rs`).
- Mutation: guard-boundary sites read the resolved guard of an `ess/22` source, and
  `offset_step` moves an offset outward (`crates/verify/ess-conformance/src/mutate.rs`).
- Entity Runtime: `LoweringCode::OffsetUnsupported` (`crates/generate/ess-entity-runtime/src/lib.rs`,
  `subset.rs`).

## U3 implementation seams (2026-10-05)

Refreshed citations for the seams unit U3 (A4, dotted input value paths) changed, with the
decisions it took where this page left a choice.

- Representation. The in-memory and IR `field` stays a `String` holding the declared segments
  joined by `.`, so a one-segment read keeps its bytes; `type_ref` is the last segment's type,
  `Optional<…>` of it where an `Optional` before it may leave it absent (the precedent of
  `{related: …}` through an Optional reference). The fallback is
  `ResolvedFallback::{Literal(String), Input { input: ResolvedInputRead }}`, untagged, so a
  literal still serializes as `"otherwise": "<text>"` and an input as
  `"otherwise": {"input": {"field", "type_ref"}}` (`crates/specify/ess-compiler/src/ir.rs`).
- Domain. `crates/specify/ess-domain/src/command/input_path.rs` resolves a path (`resolve`,
  `InputPath`, `Unresolved`) and states its refusals; `CommandSpec::read_input` is the one entry
  every value-source check uses (`command.rs`: `check_payload_entry`, `validate_sets`;
  `command/value_expression.rs`: `check_read`, `check_fallback`, `check_input_fallback`). The
  parser admits `{input: <path>, else: …}` and `else: input.<path>` while
  `ess_primitives::predicate::reads_source22_operands()` holds, so below `ess/22` each form keeps
  the refusal it had; a source with no header is refused at assembly (`input_path::below_ess_22`).
  Decision 8: `identity_paths` refuses a path that may be absent as a creation identity, and
  `related_via` refuses one as the address of another row (`input_path::optional_route`).
- Compiler: `Resolver::input_read` in `crates/specify/ess-compiler/src/resolve.rs`, beside
  `payload_field` and `expression_field`.
- Native interpreter: `Context::get` reads structurally (`ess_compiler::ir::read_input`) in
  `crates/verify/ess-conformance/src/interpret/execute/history.rs`, so payload, `sets:`,
  existence and related reads all reach a member; error payloads read a path or an input fallback
  in `error_value` (`interpret/execute.rs`).
- Synthesis: `supplied_at`, `set_at`, `input_type`, `optional_prefix`, `reads_input_path` and the
  path-aware `without_literal_fallbacks` in `crates/verify/ess-conformance/src/synthesize.rs`;
  the further run that leaves an Optional on a path out captures its own instance. Existence
  (`synthesize/existence.rs`) writes a nested identity with `set_at`. Arrangements that name a
  row through an input — an owner, a related row, a binding's trigger input, an aggregate's row —
  read top-level inputs only and refuse a path by name.
- Generated targets: `input_value` in `crates/generate/ess-synth/src/rust/behaviour.rs` and
  `crates/generate/ess-synth/src/go/behaviour.rs`; `existence_identity` in `determined.rs` keeps a
  command whose existence lookup would read a path an obligation.
- Entity Runtime: `is_input_path` and the `INPUT_PATH` row of the lowerable subset refuse a path by
  name (`crates/generate/ess-entity-runtime/src/{lib,subset}.rs`).
- Mutation: `sets-retarget` retargets a path to a same-named top-level input
  (`crates/verify/ess-conformance/src/mutate.rs`).

## U6 implementation seams (2026-10-05)

Refreshed citations for the seams unit U6 (`distinct`) changed, and the choices it made inside the
section above.

- Predicate and canonical form: `Predicate::Distinct(Box<Distinct>)`, `Distinct` (`over`, `bind`,
  `key`, `key_kind`), `Distinct::key_path` and `Distinct::evaluate`, `DistinctKeyKind` and its
  canonical spelling `kind: boolean|integer|decimal|string|uuid|timestamp|enum`,
  `Predicate::reads_distinct` and `Predicate::distincts`, the reader `Predicate::distinct`
  (`crates/specify/ess-primitives/src/predicate.rs`). `key_kind` is an `Option`: `None` only as an
  authored source writes it, before resolution; nothing compares an unresolved key and no suite
  reader admits one. A `distinct:` mapping is read as the construct only where it holds `as`, which
  is no operator, so `distinct: {in: [a, b]}` keeps meaning the fact `distinct`; below `ess/22` the
  construct is refused naming the format. Shared vectors
  `crates/specify/ess-primitives/tests/vectors/distinct.json`.
- Resolver and checker: `key_kinds`, `with_key_kind`, `typed_over` and `distinct_key_kind` beside
  `TypeEnvironment::primitive`, and `Checker::distinct`
  (`crates/specify/ess-domain/src/expression.rs`); the compiler's environment answers `primitive`
  (`crates/specify/ess-compiler/src/expression.rs`). A list is required (a `Map` and a scalar are
  refused), the key must resolve to an admitted scalar, a supplied kind must agree, and a direct
  assembly without a kind is refused naming it.
- Suite format: `expression_format::reads`, `binds_sequences` and `unkinded`; the Rust runner binds
  a row's sequences element by element only for a predicate `binds_sequences` answers
  (`runner::row_facts_with_sequences`, decision 1), and synthesis admits those reads
  (`input::sequence_read`). Go `parseDistinct`, `distinct`, `distinctKey` and
  `predicateUsesDistinct`; TypeScript `parseDistinct`, `distinctKey`, `Predicate.distinct` and
  `predicateUsesDistinct`.
- Synthesis: `Choice::Keyed`, `Builder::keyed`, `keyed_lists` and `respelled`
  (`crates/verify/ess-conformance/src/witness.rs`): a command whose guard reads `distinct` is tried
  first with, per list, a three-element duplicate whose third element repeats the first one's key —
  in a `-05:00` spelling for an instant — beside every other such list held distinct, then with every
  list distinct at two elements. The finite-domain controls of the section above — three Boolean
  keys, a singleton enum refused by name — are not implemented by U6.
- Generated lanes: `determined::supported` owes a guard, filter or selection reading `distinct` by
  name; the Rust and Go invariant checks refuse it by name; Entity Runtime refuses it as
  `DistinctUnsupported`; `infra-spec/1` refuses it. Mutation excludes a `distinct` leaf from
  `guard-boundary` (decision 16); the guard around it is still negated.
- Undecidable rows: an absent key is `Unknown` in all three runners, which the Rust runner reports
  as `error` and the Go runner as `failed`. That split is older than this unit and holds for every
  undecidable `satisfies`.

## U7 implementation seams (2026-10-05)

Refreshed citations for the seams unit U7 (String `.utf8_bytes`) changed, and the calls it made where
this page left a choice.

- Operand and canonical form: `Operand::Derived` and `Derived::Utf8Bytes` (`parent`, `observed_at`,
  `value`, `value_with`, `to_node`), the `utf8_bytes` arm of `Operand::fact_mapping`, the untagged
  branch of `Predicate::tagged_compare`, `compare_node`, and `Predicate::reads_utf8_bytes`
  (`crates/specify/ess-primitives/src/predicate.rs`). The derived operand on the right of a fact is
  `path: {<op>: {utf8_bytes: <parent>}}`; every other comparison holding one is the untagged
  `{compare: {left, op, right}}`, which a reader refuses without a derived operand, and `as:
  timestamp` never stands beside one. Shared vectors:
  `crates/specify/ess-primitives/tests/vectors/utf8-bytes.json` (texts as UTF-16 code units, so a
  lone surrogate can be written).
- Resolver and checker: `derive_selectors`/`derived_selector` run after the lexical lowering in
  `resolve_lexical_reading`; `utf8_bytes_path` refuses the path spelling in `resolve`, by format
  below `ess/22` and as `type_mismatch` in every non-operand position; `Checker::derived` types the
  operand (`crates/specify/ess-domain/src/expression.rs`). A caller's byte length is a misplaced
  caller read (`command/caller_value.rs`).
- Decision 19, as implemented: a bound `<parent>.utf8_bytes` wins like a bound `.count`, but only a
  whole number from zero (within `i64`) is a byte length; any other bound value is Unknown with no
  fallback, and a bound `null` is no observation. Lone surrogates: unrepresentable in Rust (a JSON
  escape is refused by the reader), refused by `utf8.ValidString` in the Go runner and the generated
  Go guards and invariants, refused before `TextEncoder` in TypeScript (`utf8Length`).
- Decision 1, as implemented: a text's `.count` is admitted in `satisfies` only in a predicate that
  already needs `/40` (`input::lifted_text_length`, also in authored `satisfies`), so a predicate
  without `/40` vocabulary keeps its refusal and every such suite its bytes.
- Coordinator decision F5 (2026-10-05, after the E-U7 adversary pass): the lift above also moves an
  `ess/22` invariant that pairs a text's `.count` with another `/40` form — an offset, or a tagged
  instant comparison — from no `satisfies` under `/34` to an asserted `satisfies` under `/40`. That
  is accepted as decision 1. Byte identity is owed to models whose predicates hold no `ess/22`
  expression vocabulary: a `.count` alone and a member named `utf8_bytes`, in `ess/21` and `ess/22`,
  keep their bytes (`tests/adversary_e_u7_probe.rs` asserts both sides).
- Decision 19 over the wire: a generated Go server of a model that compares a byte length refuses a
  request body escaping a lone surrogate with `400` (`loneSurrogates` before `readJSON`'s decoder,
  `go/http.rs`), instead of reading U+FFFD; models without a byte length keep their surface bytes.
  The generated Rust reader already refuses one (`rust/json.rs`, `unicode`); both answers are held
  by `ess-synth/tests/expression_utf8_bytes.rs`.
- Witnessing beyond a literal bound: two byte lengths compared with each other are tried as a wide
  text against a narrow one at the deciding lengths (`é`/`ab`, `😀`/`a`, `😀`/`abc`,
  `wide_narrow_pairs`), and each length is also tried packed with the widest scalars
  (`resize_bytes`), so a guard on a text's `.count` and `.utf8_bytes` together is witnessed.
- Follow-ups (pre-existing for `.count`, not fixed here): a view filter is witnessed by one scenario
  on one side only (F4), and a stored-row guard comparing an input length with the row's length is
  never witnessed true (F6).
- Suite format: `expression_format::reads`; Go `parseUtf8BytesOperand`, `utf8BytesOf`,
  `predicateUsesUtf8Bytes`; TypeScript `parseUtf8BytesOperand`, `utf8BytesOf`, `utf8Length`,
  `predicateUsesUtf8Bytes`.
- Synthesis: `byte_lengths`, `text_bytes_ladder`, `resize_bytes` and `wide_texts`
  (`crates/verify/ess-conformance/src/witness.rs`): each bound and one byte either side, tried first
  as text led by U+1F600 or `é`; the cap refusal names `<parent>.utf8_bytes`
  (`synthesize::unsatisfied`).
- Generated guards: `Step::Utf8Bytes` and `resolve_derived` (`crates/generate/ess-synth/src/determined.rs`);
  generated invariants measure with `.as_str().len()` and Go `len` under `utf8.ValidString`
  (`rust/invariant.rs`, `go/invariant.rs`); a view filter stays owed by name.
- Entity Runtime: `LoweringCode::Utf8BytesUnsupported` (`lib.rs`, `subset.rs`).
