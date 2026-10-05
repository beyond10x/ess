# Aggregate group selection and isolated observations

Status: accepted as the binding design for beyond10x/ess#361 and beyond10x/ess#362 (2026-10-05);
implemented for binding-irrelevant aggregate views (see "Implementation status"). No source syntax,
suite step vocabulary, runner, IR or format change. This page extends
[`aggregate-views.md`](aggregate-views.md), "Conformance".

## Problem and authority

An aggregate view whose filter compares a parameter with a group key (`team == param.team`,
`group_by: [team]`) validates, and so does one whose key is copied from an addressed related row
(`{related: {via: input.depot_id, field: team}}`). Synthesis refused both with `ESS-SYNTH-017`: the
parameter reads a group key. The copied-key control without the parameter synthesizes. A view grouped
by the lifecycle state alone (`group_by: [state]`) validated and was refused with `ESS-SYNTH-016`.
The planner refused a parameter on a group key and every grouped view no scoped key isolates.

Deleting either refusal alone is not a fix. The arrangement overwrote every scope field with one
`in` value, and the observation used one admission decision and one parameter binding for the whole
scenario: group distinctions would collapse, or the wrong query would reuse another query's
admission.

Fresh suites declare `scenario_initial_state: empty` (suite/34 and /35). That authorizes exact
observations over all rows created in the scenario; old suites have no such authority. Synthesis
used to select that provenance only at its end, after aggregate planning. Fresh provenance is now
established before the aggregate family runs (the final format selection is retained), and the
planner reads the actual typed `Empty` authority at its boundary. Isolation is never inferred from
an absent parameter or a version number. A planner run without that authority keeps the old
scoped/shared-target behaviour. Admitted stored historical bytes are never resynthesized or
rewritten.

## Separate selection from scoping

Each admitted top-level `field == param.name` equality is an explicit parameter binding. The
current one-read/equality profile and its diagnostics stay for expressions outside it. Two roles are
distinguished:

- a **non-group scope field**, whose existing `in`/`out` scoped value still controls cross-query
  selection;
- a **group selector**, whose query argument is taken from an actual arranged group key and never
  overwrites the row's tuple.

Parameter and field types are the source-resolved ones, including nominal and `Optional` wrappers.
Values come through the existing key ladder, related arrangement and actual settled value, never
through string conversion or a second related-row evaluator. Ordinary `String`/`Uuid` keys, walked
numeric, `Boolean`, `Timestamp` and enum keys, fixed keys and state keys share this binding
mechanism where the admitted types permit the equality. Captured identities use
`ScenarioValue::Instance` and opaque fact tokens only inside synthesis; those tokens are never
emitted to targets as literals. An unknown typed value is a named failed witness, not an invented
value.

Group selectors are not proof of isolation by themselves: a finite `Boolean`, state or enum key may
collide with another scenario. Exactness under fresh `Empty` authority solves this; legacy
shared-target mode keeps its own scoping proof. `Optional` absent group values follow the
predicate's actual null/missing comparison semantics. Where equality cannot select the absent
group, nothing pretends it can: every source-reachable present selection is retained, and the
`Optional` absent group is tested in views that admit it. An absent parameter spelling obeys its
field presence policy.

## Arrange once, evaluate each query independently

The group tuple pattern and aggregate value discrimination are unchanged. Direct and related copied
keys stay distinct throughout arrangement. Every owner and related prelude runs before the first
affected row, so first/last related-row mutants stay observable.

Arrangement goals are separated from query-specific filter results. Each planned row is arranged by
lifecycle and non-selector filter goals through the existing driver, advance and `shows` authority.
Group-selector conjuncts are bound to the actual reached row's selected key, so creating another
group does not require it to pass the primary group's parameter. Source predicates are never
mutated and residual filter truth is never discarded. Every query later evaluates the full original
predicate against every actual reached row with its own parameters.

A refuted-row pattern used only to witness the group-parameter comparison is supplied by an
actually distinct group; no impossible lifecycle refutation of a tautological residual predicate is
forced. Where a residual state or input filter exists, an independently refuted row is retained
wherever source reachability permits. The non-group scope `out` fallback changes only its actual
non-group scope fields; it never overwrites group keys or related values. A required
residual-filter witness that cannot be arranged keeps its precise reason, and no aggregate number
is guessed.

After arrangement, the distinct selectable parameter vectors are enumerated from the actual group
tuples in deterministic tuple order. Full filter truth is evaluated per row and parameter vector
through the typed `shows` seam over the actual reached state and settled values. Unknown truth is a
failed observation, not `False`. Admitted rows are grouped by their actual full key tuples, and
each aggregate is computed with the existing `evaluate`/`evaluate_skipping_absent` and exact
`Number`; there is no second arithmetic.

At least two distinct selections are observed where the reachable domain has them, over
differently sized and value-patterned groups. A valid nonmatching selection is added where the
admitted parameter domain provides one, proven unequal to every arranged key; an invalid enum or
state is never invented, and no fresh value is claimed in a singleton domain. Several parameters
select the existing joint tuples plus useful valid unmatched combinations, capped deterministically
rather than enumerated as a Cartesian product. Where only one tuple is possible, the ignored-selector
mutant is recorded as observationally equivalent rather than a second value fabricated.

Each query asserts exact aggregate fields for its expected groups, excludes the arranged groups that
do not qualify, and, under `Empty` authority, asserts the exact number of rows, so invented or
unarranged extra groups are detected. A grouped empty result is zero rows; an ungrouped empty result
keeps the existing one-row aggregate semantics. The selected key's `ScenarioValue` is reused in both
the parameters and the expected keys. Scope and group parameters over the same field resolve
consistently; there is no duplicate-map last-write authority.

## State-only and other unscoped groups

With actual `Empty` provenance, `AggregateUnscoped` is lifted for reachable groups and every group's
totals are asserted exactly, `Optional` absent groups included. State keys still require commands
that actually reach the state; there is no direct lifecycle injection. A source that offers a
transition exposes stale grouping. A single-state lifecycle has one honest group and needs no fake
state. Non-additive aggregates become exact under the same authority where the existing pattern can
witness them.

Ungrouped delta behaviour is retained: it already checks valid behaviour. Fresh grouped exactness
and historic scoped/delta assertions are tested separately. Existing identity-group pattern limits
and unrelated unsupported type, filter or creator combinations stay explicit backlog items. No
existing supported case acquires a new fence.

## Bindings and preconditions

Where a view's rows can be changed asynchronously — a binding runs a command whose outcome creates,
moves or changes rows of the view's source entity — an exact observation needs a causal cut after
that binding's effects, and no conformance step provides that authority yet. Such a view keeps a
refusal by name (`ESS-SYNTH-017`, naming the binding, its command and the missing binding cut); no
number is asserted. A declared precondition whose outcome creates or changes rows of the source
entity runs before every scenario's arrangement, so its rows are not ones the arrangement
determines; such a view is likewise refused by name. Unconnected bindings and preconditions do not
refuse anything.

## Required evidence

- Failing assertions over the real current synthesizer requiring aggregate scenarios for a direct
  group parameter, a copied related group parameter and state-only grouping, kept separate from
  source admission.
- Independent targets execute the synthesized suites natively, through the generated Go and
  TypeScript runtimes and through the WASM runner. Healthy targets own state and query evaluation
  and never read expected rows from the suite.
- Faults: ignore or change the parameter; wrong selected group; wrong first/last related row;
  merge/drop groups where observable; count all source rows; stale lifecycle state; retain a
  preceding scenario's rows; extra spurious group; wrong exact sum, count or non-additive result.
  Cold begin/reset, known state transitions and empty/nonmatching queries are exercised.
  Unobservable mutants under singleton or constrained domains are documented, not counted as
  killed.
- Joint group keys, two selectors, mixed non-group scope, `Optional` absent/present, related owner
  identity references, exact values above 2⁵³, filters, transitions and deterministic regeneration.
  Whole report counts are asserted, and no refusal is newly hidden. Frozen old-reader rejection for
  /34 and /35, historical admitted bytes and the existing aggregate, related and `Optional`
  regression tests are preserved.

## Implementation status

Implemented in `crates/verify/ess-conformance/src/synthesize/aggregate.rs`, with two narrow changes
in `synthesize.rs`: the provenance establishment immediately before the aggregate family, and the
`shows` seam binding each literal parameter at the view's declared parameter type, as the
interpreter binds a request's parameters. Before it, a `Boolean` parameter was bound as its
spelling and an integer past 2⁵³ through binary64, so no `Boolean` or large-integer selector could
ever hold.

- **Every value sent is one its type admits.** A walked group key walks only the ladder values
  its declared type admits: a ladder whose first 64 values all pass is kept as it is, otherwise
  it is the admitted values met in the first 4 096 ordinals, and with none the view is refused by
  name (`ESS-SYNTH-017`). Every tuple value is then checked against its key's type, and one that
  fails — a scoped value a newtype's pattern or length refuses — refuses the view by name. A
  nonmatching read sends only a value the selector parameter's declared type admits; with none
  the read is not made. This applies to views that kept their old observation too: a bounded key
  there used to arrange rows a correct target refuses to create.
- **Every row the creating command names exists.** Under an exact observation the related rows the
  creating command copies fields through are arranged even where the view reads nothing copied
  from them; a copied field no arrangement can supply refuses the view by name. (A view keeping
  its old observation still leaves such an input pointing at no row; that is unchanged here.)

- **Group selectors** are admitted under `Empty` authority for every key family the planner already
  arranges. A selector key that may be absent gets no absent group of its own (`N<k>`): equality
  cannot select it. An absent value of another key is arranged and asserted exactly.
- **The unscoped lift** applies to every view `ESS-SYNTH-016` refused: grouped by the lifecycle
  state, an enum, a `Boolean` or another walked key alone, and ungrouped with no `count` or `sum`.
  Under `Empty` authority `ESS-SYNTH-016` is no longer produced; without it, it is unchanged. An
  ungrouped view with a `count` or `sum` keeps its change observation. An exact ungrouped view is
  one row, its aggregates over no admitted row included.
- **A related guard on the creating command** (the #361 comment's shape: a `when_related` refusal
  for a missing related row, a group key copied from that row, a parameter over the reference)
  was the same group-parameter refusal; it is a group selection now. Each candidate row is
  arranged on a related row of its own, so each selection admits one row; rows of one group do
  not share their related row.
- **Exactness** (row counts, exact `Optional` absent groups) applies to the views this design newly
  admits. Views that synthesized before keep their bytes: the committed suites are unchanged.
- A view that would take a new path but is reached by a source-entity binding or precondition is
  refused by name, as above.
- Out of scope: the consumer-side aggregate observation wire adapter, binding causal-cut authority,
  and product-browser execution (a WASM fixture run is not a product-browser run).
