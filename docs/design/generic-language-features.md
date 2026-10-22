# Generic language features: one expression language, queries over rows, aggregates over expressions, one precedence rule

Status: accepted, 2026-10-14 (proposed 2026-10-13). Binding; no code, IR, suite or source format changes
with this page. It supersedes, for the constructs it names, the scope cuts in
[`value-expressions.md`](value-expressions.md) ("Not in this design"),
[`expression-family-source22.md`](expression-family-source22.md) ("Boundary"),
[`aggregate-views.md`](aggregate-views.md) ("Out of scope") and the "kept refused" answers of
[`read-api-view-idioms.md`](read-api-view-idioms.md). Every claim about current behaviour cites
this tree (`0e862220b`) or says "I don't know". Paths are relative to the repository root;
`dom` is `crates/specify/ess-domain/src`, `prim` is `crates/specify/ess-primitives/src`, `conf` is
`crates/verify/ess-conformance/src`, `synth` is `crates/generate/ess-synth/src`, `comp` is
`crates/specify/ess-compiler/src`.

## Context

The evidence is [`docs/analysis/2026-10-13-language-request-history.md`](../analysis/2026-10-13-language-request-history.md)
(132 language requests, 2026-09-25 to 2026-10-13):

- **One-to-one shipping.** 59 of 81 shipped language features (73%) are a construct shaped like
  the request: a new key, a named operation, or an existing operand admitted in one more position.
  18 (22%) are general, 7 of those relaxations or literals (analysis § 3). The share was 22% both
  before and after the fit review of 2026-09-30.
- **Recurrence.** 69 of 132 (52%) ask for a capability an earlier construct already had in another
  position, for another type, or as another operation (analysis § 4). Every recurrence chain is a
  row of a capability grid closed one cell at a time: `now` (`when:` in `ess/16`, two more guard
  positions in `ess/22`, declined in view filters, asked again), a value read from somewhere (six
  mapping forms, the sixth open), text operations (five follow-ups to the first three).
- **Declines that came back.** 6 of 15 declines with an idiom came back within 7 days, 4 of them
  reopened by another adopter for one reason: "the idiom moves the figure out of the
  specification, so conformance never checks the number the reader sees" (analysis § 5). The
  idioms page says it of itself: "the abandonment rate is `abandoned / queued` in the client"
  (`docs/design/read-api-view-idioms.md:35-37`).
- **Format churn.** 23 source formats after `ess/1` in 39 days; 9 of them added one narrow key or
  operand for one request (analysis § 6). `ess/25` is open in this tree and carries three unrelated
  constructs (`dom/system.rs:249-257`, `release: None`).
- **Four generic features absorb the open work.** Of the 35 open or moved language requests, a
  typed expression language (G1), queries over rows (G2), aggregates over expressions (G3) and one
  precedence rule (G4) absorb 27 (77%); of the latest adopter's 14 language requests, 13
  (analysis § 9).

The cut that made G1 look expensive was synthesis: "It composes the base and left candidates rather
than adding a general equation solver" (`docs/design/expression-family-source22.md:183-185`). That
cost is real for guards and absent for values, and this page states it per operation family rather
than avoiding the family.

## Decision

1. **G1.** ESS has one typed expression language. One grammar, one checker, one evaluator, used in
   every position that reads a value. The mapping value forms become spellings of it.
2. **G2.** Rows are queried with one construct: a selection of an entity's rows, optionally ordered
   with a declared tie rule, consumed by `exists`, `count`, `forall`, `one`, `first` and `each`,
   and navigated through declared references. Every row-set and related-row construct shipped so
   far is a spelling of it.
3. **G3.** An aggregate measure takes an expression over the row, a group key is an expression, and
   arithmetic over measures is a view field expression with a declared rounding and a declared
   empty and zero-denominator answer.
4. **G4.** One precedence rule decides which branch answers first, for every combination of guard
   kinds, and validation stops refusing combinations because "which of the two answers first is
   not stated".
5. **Grids.** What can be read where, and which operation applies to which type, are two grids
   with today's state recorded below. A decision opens or closes a whole row or a whole column,
   never one cell.
6. **Policy.** No decline whose idiom moves a checked figure out of the specification; one source
   format per design decision; the general share of shipped language features is counted in every
   release.

## G1. The expression language

### Types

The checker's types are today's scalar and composite types, unchanged in name and wire form:
`Integer`, `Decimal`, `Binary64`, `Timestamp`, `Duration`, `String`, `Boolean`, `Uuid`, `Bytes`,
enums, identities, newtypes, `Optional<T>`, `List<T>`, `Map<K, V>`, structs and `Json`. Today the
checker collapses them into three scalar kinds — Bool, Number (Integer, Decimal, Binary64) and Text
(String, Timestamp, Duration, Uuid, Bytes, enums) — and `Json` is no scalar
(`dom/expression.rs:36-47`); Optional and newtype wrappers are unwrapped transparently
(`dom/expression.rs:844-852`). The decision replaces the three kinds with the declared types
themselves, so that `Timestamp - Timestamp` can be `Duration` and an enum can be ordered by its
declaration. Newtypes keep today's rule: same-representation operands compare
(`docs/design/review-expression-typechecking.md`; `read-api-view-idioms.md:135-138`); arithmetic
between two different numeric newtypes yields the bare primitive, as `sum` does today
(`read-api-view-idioms.md:139-140`).

`Binary64` takes part in comparisons only, as today; it is excluded from arithmetic because two
correct implementations disagree on summation order (`aggregate-views.md:1047-1050`).

### Operations

| family | forms | result |
|---|---|---|
| comparison | `==` `!=` on every scalar; `<` `<=` `>` `>=` on every ordered type: Integer, Decimal, Binary64, Timestamp (by instant), Duration (by elapsed seconds), String (by UTF-8 bytes), an enum with a declared order | Boolean |
| logic | `and` `or` `not`; `in [..]`, `not in [..]` | Boolean |
| presence | `defined(x)` narrows `x` to its inner type in the branch it guards; `coalesce(x, y)` | Boolean; the inner type |
| conditional | `if c then a else b`, both arms one type | that type |
| arithmetic | `+` `-` on Integer and Decimal; `*` on Integer and Decimal; `/` on Integer and Decimal; unary `-`; `round(x, digits, half_even \| half_up \| down)`; `abs`, `min(a, b)`, `max(a, b)` | Integer for `+ - *` over Integers; Decimal otherwise; `/` is always Decimal |
| time | `Timestamp - Timestamp`; `Timestamp ± Duration`; `Duration ± Duration`; `Duration * Integer`; literals `60s`, `5m`, `1h`; `now` | Duration; Timestamp; Duration; Duration |
| text | `len(s)` (characters), `utf8_bytes(s)`, `lower(s)`, `upper(s)`, `trim(s)`, `starts_with(s, t)`, `ends_with(s, t)`, `contains(s, t)`, `s + t` (concatenation) | Integer; Integer; String; String; String; Boolean; Boolean; Boolean; String |
| collection | `count(xs)`, `exists(x in xs: p)`, `forall(x in xs: p)`, `distinct(xs)` | Integer; Boolean; Boolean; Boolean |
| rows | G2 below | — |
| aggregate | G3 below, in measures only | — |

Rules every form follows:

- **Exact numbers.** Integer arithmetic is exact over the mathematical integers inside a bound of
  signed 128 bits; a result outside it is `Unknown`, never wrapped, saturated or rounded. Decimal
  arithmetic is exact; `/` divides to 6 fractional digits half-even, as `avg` does today
  (`conf/aggregate.rs:92-150`), unless wrapped in `round(…)` with its own digits and mode. A zero
  divisor makes the result absent: `/` has type `Optional<Decimal>`, and the author writes
  `coalesce(a / b, 0)` where absence is not the answer.
- **Unknown.** An unobserved fact, an absent Optional on a path, an invalid instant or an
  out-of-bound intermediate makes the leaf `Unknown`, as the offset design already says for its
  one form (`expression-family-source22.md:176-180`). `Unknown` is never silently false; a view
  observation with an `Unknown` leaf is undetermined, as a measure condition is today
  (`conditional-aggregate-measures.md:47`).
- **Absence in a guard.** Where an implementation decides a record whose `Optional` is absent, the
  specification says so with `coalesce(x, default)` as the comparison's operand, never by reading
  `Unknown` as false: `coalesce(is_error, true) == false`, written on the left of a compact
  comparison from `ess/29` (https://github.com/beyond10x/ess/issues/620). Synthesis sends such an
  `x` absent as one of its arms.
- **Absence in a selector.** From `ess/30` the `where:` of a row-set guard, a filtered read and an
  ordered head says the same of a candidate row: `coalesce(enabled, true) == true`, a field of the
  candidate defaulted by a literal and compared with a literal, selects the rows whose `enabled`
  is true or absent (https://github.com/beyond10x/ess/issues/627). A `forall:` body, a set
  effect's filter and every other call in a selector keep their refusal; below `ess/30` the
  refusal is unchanged, its hint naming `ess/30`. Synthesis arranges candidates with the field
  absent and written; generated Rust and Go keep the command an obligation named by its selector,
  and Entity Runtime refuses it as every row set.
- **Time.** Elapsed seconds only. No days, months, calendars or zones; those remain the calendar
  window's (`docs/design/calendar-window-guards.md`). `== now` and `!= now` stay refused
  (`current-time-guards.md:43`).
- **Text.** `lower`, `upper` and case-insensitive comparison map ASCII letters only, as
  `equals_ignore_case` does today (`prim/predicate.rs:536-540`); `trim` removes ASCII space, tab,
  CR and LF. Regular expressions stay out (open question 4).
- **Narrowing.** `defined(x)` in a guard narrows `x` in the payloads and `sets:` of the branch it
  selects (https://github.com/beyond10x/ess/issues/556), as `ESS-COMMAND-002`'s refusal of an
  absent Optional already narrows an input (https://github.com/beyond10x/ess/issues/169).

### Spelling

The compact text form of predicates is the grammar: `when:`, `when_subject:`, `where:`, `filter:`,
`forall:` and `affects:` filters keep accepting what they accept and gain the forms above. A value
position (`sets:`, `payload:`, a response field, a view field, a measure input, a group key) takes
`{expr: "<expression>"}`, because a bare text there is already a literal or an input path. Every
existing mapping form stays valid and lowers to the same expression:

| mapping form today | expression |
|---|---|
| `input.f`, `{input: f}` | `input.f` |
| `{subject: f}`, `{subject: state}` | `subject.f`, `subject.state` |
| `{related: {via: r, field: f}}`, `via: [a, b]` | `input.r.f`, `subject.a.b.f` (G2 navigation) |
| `{related: {entity, where, field}}` | `one(rows(E where p)).f` (G2) |
| `{increment: n}` | `subject.f + n` |
| `{input: f, else: x}` | `coalesce(input.f, x)` |
| `{caller: a}` | `caller.a` |
| `{count: changed}`, `{changed: f}` | `count(changed)`, `changed.f` |
| `{response: …}`, `{generated: true}`, `{cleared: true}` | unchanged: these name what the implementation supplies or removes, not a computation |

A `{generated: true}` value stays the implementation's in the model, and synthesis asserts only its
presence and type. From `ess/30` an authored scenario may state one its own earlier steps
determine (a revision counting its commits), and the suite checks it exactly; below `ess/30` that
act is refused as `ESS-AUTHOR-042` (beyond10x/ess#623, `authored-generated-values.md`).

### Where an expression may appear

Every position in grid R below. The evaluation instant and the facts present are the position's,
not the expression's: a guard is evaluated against the store snapshot immediately before branch
selection, as row sets already are (`docs/design/filtered-related-reads.md:15-19`); a `sets:` value
against the same snapshot and the input; a view field against the row at query time.

### Synthesis cost, per operation family

Synthesis has no solver: it generates bounded candidate ladders and checks them
(`conf/witness.rs:11-13`; at most 64 candidates, `conf/witness.rs:140-147`), and an unsatisfied
search is the named refusal `GuardUnsatisfiable` (`conf/synthesize.rs:740-751`). Every evaluator
calls `Predicate::evaluate` (`prim/predicate.rs:1747-1806`); the Go and TypeScript runtimes
re-implement it (`conf/go/predicate.go:1164`, `conf/ts/predicate.ts:522`).

The decisive split: **a value is evaluated forwards, a guard must be satisfied backwards.** In
`sets:`, payloads, view fields and measure inputs, synthesis arranges the facts and computes the
expected value with the shared evaluator, as it already computes aggregate expectations with
`crate::aggregate::evaluate` (`conf/synthesize/aggregate.rs:1-18`) and value forms with
`expression_value_at` (`conf/synthesize.rs:8396-8490`). No search is added there. Only guards need
a witness for an expression, and only there does the cost grow.

| family | witness generation (guards) | mutation | interpreted target and runners | today's code |
|---|---|---|---|---|
| comparison | unchanged ladder: the literal and one step either side; Duration joins Timestamp's one-second ladder | unchanged: strictness swap, outward step, `==`/`!=` flip | unchanged; Duration compared by elapsed seconds instead of the refusal | ladders `conf/witness.rs:4149-4237`; mutation `conf/mutate.rs:832-939`; evaluation `prim/predicate.rs:1809-1928`; Duration refusal `dom/expression.rs:2322-2337` |
| arithmetic, linear | **isolate one fact**: fix every other fact of the leaf at its ladder values, solve the remaining one exactly for the boundary, try it and one unit either side. This generalises the offset composition, which is the case of two facts and a constant | new class `arith-swap` (`+`↔`-`, `*`↔`/`), `arith-drop` (an operand dropped), and the existing outward step on the constant | exact i128 Integer and i128-scaled Decimal, already the carrier (`prim/facts.rs:46-67`); Go `math/big`, TypeScript `bigint`, as for offsets | offset witness `conf/witness.rs:3239-3360`; offset mutation `conf/mutate.rs:913-918, 941-990`; offset evaluation `prim/predicate.rs:832-840, 1950-1990`; Go `conf/go/predicate.go:1283`; TypeScript `conf/ts/predicate.ts:652-660`; generated Rust `synth/rust/behaviour.rs:655-725` |
| arithmetic, non-linear (`fact * fact`, `fact / fact` in a guard) | ladder only; where it finds no witness, `GuardUnsatisfiable` names the leaf. Admitted, not refused: in values it costs nothing | as linear | as linear | refusal `conf/synthesize.rs:740-751` |
| time | the one-second ladder around each boundary, `now` at the suite's fixed reference instant; `Timestamp - Timestamp` isolates one instant as linear arithmetic does | outward by one second, as for offsets | `instant_at` and `plus_elapsed` | `conf/witness.rs:4211-4277`; `conf/now_offset.rs:62-64`; `prim/predicate.rs:845-855`; `prim/time.rs:817` |
| `now` in views | a **request clock** the suite sets on the view step; today no suite step sets a target's clock for a view, which is why a window over `now` is refused (`conf/now_offset.rs:532-555`) | outward by one second | the runner resolves the step's clock as it resolves `NowOffset` for commands (`conf/runner.rs:3219`) | a suite format pair is the cost |
| text functions | compositional: `trim` pads the witness with whitespace, `lower`/`upper` vary case (the fold refutations already do), `len`/`utf8_bytes` resize at the boundary and one either side | new class `text-fn-drop` (the function removed) | byte and ASCII operations | compositions `conf/witness.rs:2416`; fold refutations `conf/witness.rs:2216-2370`; `.count` `conf/witness.rs:2469-2540`; bytes `conf/witness.rs:2542-2820`; evaluation `prim/predicate.rs:327-333, 621-629` |
| presence, `coalesce`, `if` | one witness per arm: the fact absent and present, the condition true and false, as `{input, else}` already does | arm swap | three-valued | `conf/synthesize.rs:8443-8463` |
| collection | unchanged | unchanged (`distinct` stays out of boundary mutation, `conf/mutate.rs:106-109`) | unchanged | `dom/expression.rs:2470-2471` |

Every generated target implements a new form or refuses it by name, as the fit review's question 4
requires (`.agents/skills/assessing-external-requests/SKILL.md:47-48`). Today the generated Rust
behaviour renders a supported subset and refuses Window, Distinct, Forall and Exists by the
catch-all (`synth/determined.rs:1074-1160`), and row sets (`synth/determined.rs:210-217`,
`519-525`). The web target is the generated Rust behind WebAssembly and has no evaluator of its own
(`synth/web/mod.rs:1-30`). Entity Runtime lowering refuses what it does not lower, by name, as it
does for `before_row` (`docs/design/cross-record-and-stored-field-guards.md:942-944`).

## The two grids

Grid R (what can be read where) and grid O (which operation applies to which type), each with
today's state, the citation of each row and the target, live on one tracked page:
[`language-grids.md`](language-grids.md). A change that opens or refuses a row or a column updates
that page in the same change; this record keeps no copy of its own.

### Closing rule

A request is triaged onto cells of these grids (and of the effect × target grid G2 implies). The
decision opens or refuses the whole row or the whole column the cell sits in. Opening one cell is
refused at review; a refusal of a whole row or column names why, in the design page that decides
it. A row or column this page does not open stays as it is until a design page opens it whole.

## G2. Queries over rows

One construct, `rows(E where p order_by k₁ asc, k₂ desc)`, a selection of entity `E`'s rows from
the immutable pre-selection snapshot, never including rows the outcome itself creates or changes
(`filtered-related-reads.md:15-19`). Without `order_by` it is unordered, as today ("No order among
the rows decides anything", `dom/command/row_set.rs:23`). With `order_by` the order is total:
identity ascending is appended as the last key unless the author writes another, so ties are
always decided ("There is no implicit latest row" stays true: the order is declared).

Consumers:

- `exists(sel)`, `count(sel)`, `forall(r in sel: p)`: Boolean and Integer, as today's row-set test
  (`dom/command/row_set.rs:115-126`).
- `one(sel)`: the one row; zero or several rows supply no value, as today
  (`filtered-related-reads.md:113-127`).
- `first(sel)`: the head of an ordered selection; refused on an unordered one. Absent where empty.
- `each r in sel` (effects and per-row events) and `each x in input.list` (iteration over a list).

Navigation: a path through a declared reference reads the referenced row:
`subject.incident.state`, `input.agent_id.label`, `E[input.id]` for a row by identity. Through an
Optional reference the path is absent where the reference is. Depth is any chain of declared
references (open question 8); today it is two (`dom/command.rs:1861-1863`).

**First cut, `ess/28` (beyond10x/ess#564, beyond10x/ess#557).** Navigation ships one hop, in the
positions a view reads and as a value: a view field `field: {related: {via, field}}` and a view
filter path `<reference>.<field>` read the referenced row when the view is read, and a value source
`{related: {via, field: state}}` reads its lifecycle state. A missing far row reads as absent in a
view, as an absent reference does, so a navigated field is `Optional` and a filter over it shows no
row; a value source keeps its rule (a present reference naming no row is a missing row, answered by
the guard's `exists: false` branch). The value source keeps its two-hop chain; a second hop in a
view or a guard, `E[id]`, and navigation in guard predicates are not in this cut and stay refused
by the paths' own checks. A conjunction over several rows the input names is the guard form
`when_related: {all: [{via, predicate}, …]}`, each row with its own predicate; a comparison across
two rows (`E[input.a].direction == E[input.b].direction`) is one such branch per value until
navigation reaches guard predicates. Aggregate, paged and `absent:` views refuse navigation by name. Each
navigated view is witnessed by a `<view>/reference/<field>` scenario (suite/54 and /55): two
referenced rows, a decoy, an absent reference, and a second read after the named row changes.
The same format quantifies over a list of references (beyond10x/ess#558): `when_related: {via:
input.<list>, exists: false}` over a `List<…>` of one entity's identity is `not forall(id in
input.<list>: exists(E[id]))`, false for an empty list, and `{missing: input.<list>}` reads the
first element naming no row. A predicate over each listed row waits for navigation in guard
predicates and is refused by name.

| existing construct | as G2 |
|---|---|
| `when_related: {via: input.r, exists: false}` / predicate | `not exists(E[input.r])` / a predicate over `E[input.r]` (`dom/command.rs:551-566`) |
| `when_related: {entity, where, exists \| count \| forall}` (`ess/22`) | `exists(rows(E where p))`, `count(…) op n`, `forall(r in …: q)` (`dom/command/row_set.rs:53-126`) |
| `{related: {via, field}}`, `via: [a, b]` | navigation `subject.a.b.field` (`dom/command.rs:1162-1186`) |
| `{related: {entity, where, field}}` | `one(rows(E where p)).field` (`dom/command.rs:1187-1199`) |
| `instances: {where: p}` with `moves:`/`updates:`/`deletes:` | the effect's target is `each r in rows(E where p)` (`dom/command/set_effects.rs:86-89, 154-166`) |
| `affects: [{entity, where, sets}]`, `each: {in, as}` | `each r in rows(E where p)`, `each x in input.list` (`dom/command/set_effects.rs:95-148`) |
| `emits: {event, each: changed}` (`ess/25`) | `each r in changed` (`dom/command/set_effects.rs:33-37, 172-178`) |
| `existing_instance:`, `unknown_instance:` | `exists(subject)`, `not exists(subject)` (G4) |
| `{count: changed}` | `count(changed)` |

What it opens, by request: ordered heads (https://github.com/beyond10x/ess/issues/543,
https://github.com/beyond10x/ess/issues/549), a selector testing each candidate's related rows
(https://github.com/beyond10x/ess/issues/551), identity-addressed and row-set guards on one command
(https://github.com/beyond10x/ess/issues/473), `affects:` by identity
(https://github.com/beyond10x/ess/issues/547), existence of each element of a list of references
(`forall(id in input.ids: exists(E[id]))`, https://github.com/beyond10x/ess/issues/558), a related
row's state as a value (https://github.com/beyond10x/ess/issues/557), a related identity read in a
predicate (https://github.com/beyond10x/ess/issues/505), a view reading a referenced row's current
fields one hop (https://github.com/beyond10x/ess/issues/564,
https://github.com/beyond10x/ess/issues/447), a field from a correlated earlier row
(`first(rows(Begin where key == subject.key and at <= subject.at order_by at desc)).at`,
https://github.com/beyond10x/ess/issues/442), one event per changed row
(https://github.com/beyond10x/ess/issues/531, shipped as `ess/25` and kept as its sugar).

A view's `source:` stays one entity (`dom/view.rs:919-923`); a view reads other entities only by
navigation from its source row, never by a join on arbitrary keys. That keeps the view's row count
the source's, which aggregate scoping depends on (`aggregate-views.md:443-546`).

**Cost.** Synthesis already arranges selected rows against decoys, one per conjunct, with selected
counts `[1, 2, 0, 3]` (`conf/synthesize/row_set.rs:1-31, 60`) and arranges a referenced row between
two decoys (`conf/synthesize/related.rs:1-11`). An ordered selection adds an arrangement of rows
that differ only in the order key, and one tie on it, so a target that orders the other way or
breaks ties otherwise fails; today a tie exists only as an authored scenario
(`read-api-view-idioms.md:155-161`). Mutation gains `order-flip` on selections (it exists for
ranked views, `conf/mutate.rs:124`) and a `first`↔last swap; today a row set's `where` and `forall`
are not mutated at all (`conf/mutate.rs:268-270`). The interpreter evaluates row sets three-valued
(`conf/interpret/execute/row_set.rs:66-139`; `comp/ir.rs:1455-1490`). Generated Rust and Go do not
implement row sets today (`synth/determined.rs:210-217, 519-525`); G2 either implements them or
keeps that refusal by name per construct.

### Enforced references (`ess/30`, decided 2026-10-21)

beyond10x/ess#509 (two adopters, `story:enforced-reference-needs-no-missing-target-branch`): a
command reading the row a stored reference names must declare `when_related: {via: <field>,
exists: false}` (`ESS-COMMAND-005`, `dom/command/related_guard.rs`), while synthesis, given that
branch, reports it unreachable (`ESS-SYNTH-003`, `conf/synthesize/related_guard/stored.rs`)
because every writer of the field refuses an identity no row carries. The adopter can satisfy
validation or synthesis, not both.

**Decision: the integrity is derived, not authored.** Validation accepts the proof the
specification already holds, with no new key. From `ess/30` the branch may be left out where
(1) every branch that creates a row of the subject's entity sets the field, and every branch that
writes it (`sets:` on the addressed row, on `instances:` rows or in an `affects:` entry) writes
`{cleared: true}`, the field's own value, or an input its command refuses first with an unguarded
`exists: false` over the same entity; and (2) no branch deletes a row of the referenced entity
(`deletes:`, `instances:` with `deletes:`, `affects:` with `deletes:`; a binding runs a command, so
the command is named). Where either fails, the refusal stands and its hint names each branch that
defeats the proof; below `ess/30` it stands with a hint naming `ess/30` where the proof holds
(`dom/command/reference_integrity.rs`).

Why derived over an authored integrity key (`integrity: enforced` on the relation):

- **Least surprising.** Both adopters asked for this proof ("validation could accept the same
  proof instead of a new authored key", the second report on beyond10x/ess#509), and it makes
  validation agree with the unreachability synthesis already reports. A key would make the author
  restate what the commands already say, and would still need the same deletion and writer check
  to be honest.
- **Reversible.** No source key, no schema change and no IR change: a model that does not leave the
  branch out keeps its IR bytes and suite. Adding a key later, to name the guarantee or to pin it
  against a future deleting command, extends this rule; removing a key once published would break
  every document that wrote it.
- **The cost, accepted.** The guarantee is non-local: a deleting command added elsewhere refuses the
  reading command. The refusal names the deleting branch, so the cause is one hop away.

Conformance checks the derived integrity (acceptance 3, `conf/synthesize/reference_integrity.rs`).
After every accepting branch that writes a row of the referencing or the referenced entity, the
suite reads the row each arranged reference names through an unfiltered view of the referenced
entity and requires it present. A branch about a referenced row that no arranged reference names
first points a new referencing row at it, through the creating branch that stores the reference
from its input. A target that deletes the row fails
(`conf/tests/enforced_reference.rs`, `a_target_that_deletes_the_referenced_row_fails`). The
reads are filed under the referencing entity's `<entity>/invariant/after/<command>/<outcome>` id,
where they join the reads of any declared invariants. The id says what holds of that entity after
the branch, and reusing it gives the suite no identity an older reader refuses, so the suite major
stays put. A specification that leaves no branch out keeps its suite bytes.

Generated Rust and Go read the row such a command names with no answer for a missing one, before
the first branch that reads it (`synth/determined.rs`, `kept_reference`; `stored_read` in each
target's `behaviour.rs`), and a target that never reads it fails the related refusal's scenario
(`synth/tests/enforced_reference_served.rs`). Synthesis's unreachability proof reads deletions as
well as writers (`conf/synthesize/related_guard/stored.rs`, `unreachable`): with no deleting
branch a present `exists: false` branch stays `ESS-SYNTH-003`, naming the writers; beside one, its
witness creates a referenced row, links the subject to it, and deletes the row through the
deleting command before the branch (`conf/tests/enforced_reference.rs`). Only a branch deleting
its own subject drives that witness; where the row is deleted only through `instances:` or
`affects:`, the arrangement is still refused by name.

## G3. Aggregates over expressions

Today a measure is one of six functions over one top-level field (`dom/view.rs:309-322, 361-372`),
with `skip_absent` (`ess/15`) and `where:` (`ess/22`); a path is refused, "an aggregate reads one
top-level field" (`dom/view.rs:1316-1328`); `order_by:` on an aggregate view is refused as a window
(`dom/view.rs:1439-1450`).

Decision:

- **A measure input is an expression over the source row** and its navigated references:
  `{sum: {expr: "ended_at - started_at"}}`, `{sum: {expr: "duration * level.weight"}}`
  (https://github.com/beyond10x/ess/issues/561). The existing `{sum: field}` is the expression
  `field`.
- **Types by grid O**: `min`/`max` over every ordered type, an enum with a declared order included
  (https://github.com/beyond10x/ess/issues/566); `sum`/`avg` over Integer, Decimal and Duration.
- **A group key is an expression**, including `each(list_field)`, one group per element
  (https://github.com/beyond10x/ess/issues/565). Timestamp group keys stay refused
  (`dom/view.rs:1622-1634`): time buckets need calendars.
- **Arithmetic over measures** is a view field `{expr: "…"}` reading the row's measures and view
  parameters (`1 - down / (param.to - param.from)`), typed by grid O, with `/`'s zero-divisor
  absence and rounding (https://github.com/beyond10x/ess/issues/563,
  https://github.com/beyond10x/ess/issues/441). It is evaluated after grouping, reads no source
  row, and is checked exactly by the suite: the figure stays in the specification.
- **Library functions, not keys**: `arg_max(expr, by)` / `arg_min` with identity tie-break
  (https://github.com/beyond10x/ess/issues/446) and `covered(from, to, within: [a, b])`, the
  measure of the union of intervals clipped to a period
  (https://github.com/beyond10x/ess/issues/562).
- **Ranking**: `order_by:` over group keys and measures, with paging, on an aggregate view. Running
  totals and windows stay out.

| existing construct | as G3 |
|---|---|
| `{count: {}}`, `{count_distinct: f}` | `count()`, `count_distinct(f)` |
| `{sum \| min \| max \| avg: f}` | the same function over the expression `f` |
| `skip_absent: true` | unchanged; applies to the expression's value |
| `where:` on a measure | unchanged (`conditional-aggregate-measures.md:13-31`) |
| `group_by: [f, g]` | expressions `f`, `g` |

**Cost.** Synthesis computes expected measures forwards with `crate::aggregate::evaluate`
(`conf/aggregate.rs:92-167`; `conf/synthesize/aggregate.rs:1-18`), so an expression input changes
the expected value, not the search; the contrast arrangement of conditioned measures
(`conf/synthesize/aggregate/contrast.rs:1-26`) varies every field the expression reads instead of
one. Mutation has no aggregate class today beyond `order-flip` (`conf/mutate.rs:124`); G3 adds
function swap (`sum`↔`avg`, `min`↔`max`) and the arithmetic classes on the input. Decimal stays
exact i128 (`conf/aggregate.rs:215-300`).

## G4. One precedence rule

### What exists

The order is stated once in prose (`docs/design/cross-record-and-stored-field-guards.md:810-841`)
and once in code: `Phase` lists `InputAbsent`, `RelatedRow` (step 1), `InputRefusal` (2),
`Existence` (3), `HeldState` (4), `PresentRelated` (5), `Accepting` (6) and `Default`
(`dom/command/precedence.rs:27-61`); `place` assigns each branch (`precedence.rs:406-477`); `order`
sorts within a phase by rank, then declaration (`precedence.rs:483-499`). Validation, the
interpreter (`conf/interpret/execute.rs:564`), synthesis (`conf/synthesize/precedence.rs:53`), the
generated Rust (`synth/rust/behaviour.rs:1258-1283`) and Go (`synth/go/behaviour.rs:1843-1881`)
and Entity Runtime lowering (`crates/generate/ess-entity-runtime/src/lib.rs:1947`) read it.
`docs/design/selection-plan.md:3-5` still says no consumer reads the plan; that line is stale.

What ESS-COMMAND-004 (`ConflictingDeclaration`, class 4, `comp/resolve.rs:301-302, 1231-1233`)
still refuses, each because the order was not stated for it:

- `when_related:` beside `when_subject`, `when_subject_state`, `when_state_changes`,
  `unknown_instance` or `input_absent`: "which of the two answers first is not stated"
  (`dom/command/related_guard.rs:543-552, 578-596`;
  https://github.com/beyond10x/ess/issues/554);
- `when_related:` beside `wrong_state` outside the `ess/22` composition
  (`related_guard.rs:866-918`; https://github.com/beyond10x/ess/issues/282);
- `before_row: true` beside any `when_related:` (`dom/command.rs:3383-3396`), left undecided by its
  design (`cross-record-and-stored-field-guards.md:927-931`);
- `when_subject` beside a lifecycle guard (`dom/command/subject_fact.rs:212`) and explicit
  subject-state guards beside `wrong_state` (`dom/command/subject_state.rs:172`).

An identity-addressed and a row-set `when_related:` on one command is refused as ESS-COMMAND-009,
"one command reads one kind of related row in this cut" (`dom/command/row_set.rs:295-297,
318-326`; https://github.com/beyond10x/ess/issues/473). An input refusal declared after
`unknown_instance:` is not refused although the two orders disagree: only held-state and
`before_row` declarations are checked (`dom/command.rs:3398-3421, 3432-3557`).

### The rule

**A branch answers in the order of what it must read: the request alone; then whether the
addressed record exists; then what the addressed record holds; then the rows around it; then the
causes outside the specification. Within one step, the first declared whose condition holds.**

| step | answers | today's step |
|---|---|---|
| 0 | `input_absent:` | `InputAbsent` (unchanged) |
| 1 — the request alone | input-guarded refusals; `external:` + `before_row: true`, also beside `when_related:` | 2 |
| 2 — existence of the addressed record | `unknown_instance:` (`not exists(subject)`), `existing_instance:` on the identity a creation takes | 3, and step 1 for `existing_instance:` on a command reading a related row |
| 3 — the held state and subject guards | `when_subject_state:`, `when_state_changes:`, `when_subject:`, `wrong_state:` (last) | 4 |
| 4 — related rows | first existence (`exists: false` over a row the input or a stored field names, in declaration order), then predicate and row-set refusals over present rows, then a row-set upsert's `unknown_instance:` | 1 for input-named rows; 5 for stored references and present rows |
| 5 — acceptance and external causes | accepting `when:`, `external:`, accepting `when_related:` and row-set branches, declaration order | 6 (unchanged) |
| 6 | `otherwise:` | `Default` (unchanged) |

Derivation. Steps 1, 2, 3, 5 and 6 are today's steps 2, 3, 4, 6 and `Default` in today's order:
input refusals are decided before any row is loaded, as Entity Runtime's lowering pins
(`cross-record-and-stored-field-guards.md:849-853`), and `before_row` exists so that a request-only
refusal says nothing about existence (`cross-record-and-stored-field-guards.md:883-889`). The one
move is related-row existence: today it answers first for rows the input names and fifth for rows a
stored field names (`selection-plan.md:23, 27`; `cross-record-and-stored-field-guards.md:816,
831-835`). Under the rule both answer at step 4, after the addressed record, because a related row
is around the addressed record, not before it. That also settles the case the selection-plan page
left to its owner (`selection-plan.md:135-142`).

What changes, from the format that carries G4:

- Every combination ESS-COMMAND-004 refuses above for want of an order is admitted; the rule
  orders it. Overlap within one step is resolved by declaration order, as steps 1 and 5 already
  are (`docs/design/input-guard-overlap-precedence.md:45-49`). ESS-COMMAND-009's one-kind-of-row
  refusal is lifted with G2.
- Validation refuses a declaration order that contradicts the step order wherever two branches'
  guards can both hold, for every pair of steps, generalising the held-state rule
  (`dom/command.rs:3432-3557`) and the `before_row` rule (`dom/command.rs:3398-3413`). A consumer
  reading one declaration-order pass then gives the rule's answer.
- `unknown_instance:` is admitted on a command that only preserves or refuses
  (https://github.com/beyond10x/ess/issues/535) and beside `when_subject:` update selection
  generally (https://github.com/beyond10x/ess/issues/530 is `ess/25`'s special case of it).
- The markers become sugar for G1 predicates at their step: `unknown_instance:` is `not
  exists(subject)` at step 2; `wrong_state:` is "the branch step 5 would select moves from a state
  the row does not hold" at the end of step 3, unchanged in meaning
  (`cross-record-and-stored-field-guards.md:823-825`); `when_subject_state: [A, B]` is
  `subject.state in [A, B]`.

### An external branch narrowed by a subject guard (`ess/29`)

Option 2 of https://github.com/beyond10x/ess/issues/584, `story:external-branch-narrowed-by-subject-guard`.
An external cause that applies only to some configured records is declared once, for those
records: `external:` beside `when_subject: {predicate: P}` (and an optional `when:`), from `ess/29`.
Below it the pair keeps its `conflicting_declaration` ("a subject fact has one selection
authority"), its hint naming `ess/29`.

- **Selection.** The branch is taken where the provider takes it, its input guard holds and `P`
  holds of the addressed row. On a row `P` does not select, the provider is not asked and the
  branch is not taken, whatever it would say. `P` is not a selector beside the others: the joint
  stored-field partition leaves the branch out, as it leaves every external branch out.
- **Place.** It reads the held row, so it is a step-3 branch of the rule, in declaration order
  among `when_subject_state:`, `when_state_changes:` and `when_subject:`, before `wrong_state:`.
  Declared first, it answers a row a later held-state refusal selects too, which is the order the
  downstream service checks in (#584). The held-state declaration rule (#486) refuses neither order
  beside a held-state branch, since both are step 3.
- **Refused.** `before_row: true` beside it (the branch reads the row it would answer before), and
  the `{field, equals}` shape beside `external:` (write it as the predicate).
- **IR.** A new condition, `external_subject` (`cause`, `predicate`, `input` when declared); no
  model below `ess/29` changes its bytes.
- **Consumers.** The interpreter reads `P` over the held row before it reads the provider.
  Synthesis witnesses the branch forced on a row `P` selects, where it can on one a held-state
  branch declared after it selects too, and forces it beside every sibling witnessed on a row `P`
  decides false, where that sibling must still answer. Generated Rust and Go read the row, then
  `P`, then ask the context. Entity Runtime lowers it to `P` over the loaded row, the input guard
  and the host's external evidence, all required.
- **A race that overrides a held-state branch** (beyond10x/ess#623, Ask B). "Commit unless the
  head moved" needs no new key: the race is declared first, `external:` beside
  `when_subject: {predicate: state == Validated}`, above `committed` (`when_subject_state:
  Validated`). Both are step 3, so the race answers where the provider takes it and `committed`
  answers otherwise; synthesis witnesses both with no `ESS-SYNTH-003`/`004`. From `ess/30` the
  race may declare `compensates: true`: the flag changes what its refusal does to the row, not
  what it reads or where it answers, so it still answers at step 3, and its witness reads the row
  back in the compensating state. A plain `external:` declared below `committed` stays
  unreachable, since step 3 answers every validated row first; synthesis refuses it as
  `ESS-SYNTH-003`, naming `committed` and the narrowed race as the repair
  (`tests/external_narrowed_by_subject.rs`, `tests/external_beside_held_guard.rs`).

### Entity Runtime and `when_related:` (decided 2026-10-21)

Every guard pair the rule admits lowers to Entity Runtime or is refused by name (answer 10). A
command with a `when_related:` branch stays refused as `RelatedGuardUnsupported`, at every format,
and the refusal says why. entity-core decides an operation on its arguments and the one row its
request names, and has no lookup of another row by design (`entity-core/src/definition.rs`, "no
lookup"). The one lowering ESS could make on its own, the host reading the other row and binding a
copy as arguments as it binds external evidence, is not taken: that copy is read apart from the
decision that commits, so a guard such as `exists: false` could hold on a row another operation has
already removed, which no ESS target answers. It would also add binding targets and requirements to
the public binding plan, which is a contract change for every host. It lowers when entity-core can
read another entity's row inside the decision (the catalogue's `Needs::EntityCore`); a row set
(`RowSetUnsupported`) waits for the same capability.

What stays: the phase plan as the one executable definition, derived on demand and never serialised
(`selection-plan.md:174-181`); the one test seam `with_phase_order` (`selection-plan.md:183-191`);
the pinned table (`selection-plan.md:199-209`); every answer on every model below the G4 format.
`PrecedencePlan::new` already takes the format (`selection-plan.md:171`), so the plan of an older
document keeps today's phases.

## Policy

1. **No decline whose idiom moves a checked figure out of the specification.** An idiom is a valid
   answer only if the suite still checks the figure the specification's reader sees. A request
   whose only idiom fails that test is either absorbed into a generic feature, or refused as out
   of the language with the reason, and its fit review says which. The fit-review skill's
   question 3 ("If an idiom exists, the answer is the idiom",
   `.agents/skills/assessing-external-requests/SKILL.md:40-41`) gains this condition.
2. **One source format per design decision, not per request.** G4, G1, G2 and G3 each get exactly
   one source format, allocated when its first wave lands and closed at the release after its last.
   A request arriving while a format is open is triaged onto the grids and lands in that format if
   its row or column is the one being opened; otherwise it waits for the next decision. A
   construct for one request does not get a format of its own.
3. **The general share is counted every release.** Each release section of `CHANGELOG.md` that
   changes the source language ends with one line:
   `Language: G general, S one-to-one, P part (general share X%; cumulative Y%).` An `Added` or
   `Changed` entry that admits a new source spelling counts once. It is **general** where it opens
   or closes a whole row or column of grid R or grid O (or of the effect × target grid of G2), or
   relaxes a refusal across every position; **one-to-one** where it opens fewer cells than its row
   or column; **part** where one design opens a row in more than one release. Defects, tooling and
   refusal-only changes are not counted. The baseline is the analysis's: 18 general, 59
   one-to-one, 4 part, 22% (analysis § 3). Each counted entry names its grid row or column in its
   text, so the count can be re-done from the changelog alone.

## Migration and format consequences

- **Source.** Every existing spelling stays valid under every format it is valid under today. From
  each feature's format, the mapping forms and markers listed above lower to the general IR; below
  it, they keep their own IR variants and bytes (AGENTS.md, "Determinism and formats"). Consumers
  read both until a later, separate decision retires the old variants. Nothing is re-lowered in a
  persisted document.
- **IR.** New `Operand` and `Predicate` variants beside today's (`prim/predicate.rs:552-584,
  1278-1367`); `Operand::Derived` (today `utf8_bytes` only) is the seed of the function call
  variant. A new aggregate input variant beside `Aggregate.input` (`dom/view.rs:361-372`). No
  `ess-ir/2`, no property bag (AGENTS.md, "Boundaries").
- **Formats.** Four source formats after `ess/25` closes: G4 first, then G1, G2, G3 (`ess/26`
  onwards, numbers allocated at landing). `ess/25` takes none of this work. Each format needs a
  suite format pair (ordinary and coverage) for what it adds to a step: G1 the computed value
  expectations and the view request clock, G2 ordered-selection expectations, G3 expression
  measures. `ess-diff` reports a changed expression as the existing condition or aggregate change
  kinds (`outcome-condition-changed`, `FieldAggregateChanged`,
  `conditional-aggregate-measures.md:84`); a new diff version only where a new kind is needed.
- **Schema.** Every `RawSpecFile` change regenerates `schemas/generated/ess.schema.json` through
  `cargo xtask schema` (AGENTS.md).
- **Diagnostics.** Refusals for type errors in expressions reuse `TypeMismatch`
  (`prim/error.rs:410`); the refusals the rule removes are removed, not renamed.

## Delivery

Waves of at most five stories, each story with named conformance scenarios as its acceptance. The
first wave starts after the current release (0.60.0, 2026-10-11) and lands into the next one; no
wave lands into `ess/25`.

**Wave 1 — G4 and the policy (format A).**
1. The rule in `precedence::place` and `PrecedencePlan::new` under format A, with the pinned table
   re-written; `cross-record-and-stored-field-guards.md` and `selection-plan.md` updated.
2. Admit every guard combination ESS-COMMAND-004 refuses for want of an order
   (https://github.com/beyond10x/ess/issues/554, https://github.com/beyond10x/ess/issues/282,
   `before_row` beside `when_related:`).
3. Validation of declaration order against step order for every pair of steps.
4. `unknown_instance:` on preserve-or-refuse commands and beside update selection
   (https://github.com/beyond10x/ess/issues/535, https://github.com/beyond10x/ess/issues/530,
   https://github.com/beyond10x/ess/issues/534 residue).
5. The policy: the changelog count line from the next release, the fit-review condition, and the
   grids as a tracked page the fit review cites.

**Wave 2 — G1 values (format B).**
1. The typed expression AST and checker in `ess-primitives`/`ess-domain`, grid R's position table
   as data.
2. Evaluation in the shared evaluator, the Go and TypeScript runtimes and the generated Rust and Go
   behaviour (or named refusals).
3. Expressions in `sets:`, event and error payloads and view fields
   (https://github.com/beyond10x/ess/issues/560, https://github.com/beyond10x/ess/issues/555,
   https://github.com/beyond10x/ess/issues/506, https://github.com/beyond10x/ess/issues/498,
   https://github.com/beyond10x/ess/issues/545).
4. Forward synthesis of value expectations and the `arith-swap`, `arith-drop` mutation classes.
5. Text functions and presence (https://github.com/beyond10x/ess/issues/532,
   https://github.com/beyond10x/ess/issues/556).

**Wave 3 — G1 guards and time (format B, same decision).**
1. Arithmetic in guards with the isolate-one-fact witness.
2. Duration ordering and time arithmetic in every position.
3. `now` in views with a suite request clock (https://github.com/beyond10x/ess/issues/567,
   https://github.com/beyond10x/ess/issues/439).
4. The binding cause `where:` on the full grammar (https://github.com/beyond10x/ess/issues/492).
5. Mapping forms lowered to expressions under format B; the caller and subject columns closed.

**Wave 4 — G2 (format C).**
1. `rows(…)` with `order_by` and identity tie-break; `exists`, `count`, `forall`, `one`, `first`.
2. Navigation through declared references, any depth, and `E[id]`.
3. Effects over selections and by identity; `each` (https://github.com/beyond10x/ess/issues/547,
   https://github.com/beyond10x/ess/issues/558).
4. Ordered heads and candidates tested by their related rows (https://github.com/beyond10x/ess/issues/543,
   https://github.com/beyond10x/ess/issues/549, https://github.com/beyond10x/ess/issues/551,
   https://github.com/beyond10x/ess/issues/473, https://github.com/beyond10x/ess/issues/505,
   https://github.com/beyond10x/ess/issues/557).
5. View fields by navigation (https://github.com/beyond10x/ess/issues/564); ordered-selection
   synthesis and mutation.

**Wave 5 — G3 (format D).**
1. Expression measure inputs (https://github.com/beyond10x/ess/issues/561).
2. `min`/`max` over every ordered type, enum order (https://github.com/beyond10x/ess/issues/566).
3. Expression group keys and per-element groups (https://github.com/beyond10x/ess/issues/565).
4. Arithmetic over measures with rounding and zero-divisor answer
   (https://github.com/beyond10x/ess/issues/563, https://github.com/beyond10x/ess/issues/441).
5. `arg_max`/`arg_min`, `covered`, ranking (https://github.com/beyond10x/ess/issues/446,
   https://github.com/beyond10x/ess/issues/562).

Not absorbed, and decided elsewhere: the 8 open requests of analysis § 9 marked "none"
(https://github.com/beyond10x/ess/issues/493, https://github.com/beyond10x/ess/issues/507,
https://github.com/beyond10x/ess/issues/508, https://github.com/beyond10x/ess/issues/509,
https://github.com/beyond10x/ess/issues/525, https://github.com/beyond10x/ess/issues/533,
https://github.com/beyond10x/ess/issues/542, https://github.com/beyond10x/ess/issues/559).

## Open questions, answered

The record was approved as proposed on 2026-10-14. Its open questions are answered here, and each
answer binds the wave that first needs it.

1. **Input refusals first.** Kept. Request-only refusals answer before existence and the held state,
   as today; moving them would change every validating command's answer and reopen the disclosure
   `before_row` closed. Wave 1 pins it in the precedence table.
2. **A release in the middle of a decision.** A release ships a format with what it holds at that
   commit, and the format is frozen from then on. The rest of the feature moves to the next format
   version and is counted under the same decision, not as a second one: the count measures decisions
   an adopter has to make, and the second format adds constructs without changing the first's
   meaning. A release never ships a format whose meaning a later commit of the same decision
   changes.
3. **`{expr: …}` or bare text.** `{expr: …}`. Bare text would change what an existing spelling
   means.
4. **Regular expressions.** Out. The text shapes without patterns stay the way to check a string's
   form. A request that needs a pattern gets its own design with the witness automaton costed.
5. **Enum order.** Explicit `ordered: true` on the enum. Every existing enum stays unordered, and
   `min`/`max`/ordering over an unordered enum is a validation refusal naming the flag.
6. **Unicode.** ASCII-only case mapping and trimming, as today's ignore-case. A non-ASCII operand is
   compared byte for byte. A Unicode version needs pinned tables in every runner and is a separate
   decision.
7. **Non-linear guards.** Admitted. Synthesis refuses a branch it cannot witness by name
   (`ESS-SYNTH-…`, naming the non-linear term), and an authored scenario can stand in.
8. **Navigation depth.** Any chain of declared references, as written: a path is a finite sequence
   of reference steps spelled in the specification, so a cycle in the reference graph is harmless.
   There is no transitive closure and no recursion.
9. **Cross-row invariants.** A separate design, outside grid R
   (https://github.com/beyond10x/ess/issues/75).
10. **Entity Runtime.** It lowers each form the shared evaluator evaluates, wave by wave; a form it
    does not lower yet is refused by name at lowering, never silently dropped, and the
    lowered-definitions table records which. Each wave's stories name the forms they lower.
11. **Retiring the mapping forms.** Not planned. The mapping forms stay as spellings lowered to
    expressions in every format; retiring one is a format decision of its own, taken only when no
    adopter's specification uses it.
