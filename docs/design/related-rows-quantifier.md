# A selector tests each candidate's related rows (`ess/27`)

Status: implemented (https://github.com/beyond10x/ess/issues/551); the synthesis module is
`ess-conformance/src/synthesize/row_set/related_rows.rs`. Source format
`ess/27`, unreleased at the time of writing (`FormatVersion::V27`,
`crates/specify/ess-domain/src/system.rs:355`). This page opens no format of its own. No new
keyword, no new `ValidationCode`, no IR shape change, no suite or `ess-diff` version. It builds on
`filtered-related-reads.md` (the selector and its snapshot), `ordered-related-head.md`
(https://github.com/beyond10x/ess/issues/543, the head of an ordered selection),
`published-value-operand.md` (https://github.com/beyond10x/ess/issues/549, which names this gap
under "Not in this design") and `set-effect-identity-filters.md`
(https://github.com/beyond10x/ess/issues/547, whose set-effect filters this construct stays out
of). It extends the predicate quantifier (`forall:`/`exists:` with `in`, `as`, `that`,
`crates/specify/ess-primitives/src/predicate.rs:59-84`).

## The gap

A matching command picks the best slot of a group **among the slots that can match**: slots
where each side holds at least one `Present` membership. The pick is the head of an ordered
selection over `Slot` (#543), and the head of each side is read from the chosen slot (#549). The
outer selector's `where:` needs a test on each **candidate's** related rows. The selector grammar
reads the candidate's own fields bare, `input.`, `subject.` and `now`
(`crates/specify/ess-domain/src/command/row_set.rs:12-16`). Nothing in it ranges over other rows.

Minimal reproduction, the binary built at `3fdd9479e` (`ess 0.59.0`, `format: ess/27`). The
specifications were written for this page and are not committed: entities `demo.slots.Slot
{slot_id, group_id, score}`, `demo.slots.Membership {membership_id, slot_id, side, element_key,
element_score}` (lifecycle `Present → Removed`) and `demo.slots.Pair`; `MatchInGroup` publishes
`PairMade {pair_id, slot_id, left_key, right_key}` with `slot_id` read as
`{related: {entity: Slot, where: …, order_by: [score desc, slot_id], first: slot_id}}`. With
`where: {all: [group_id == input.group_id, state == Open]}` it validates (`demo v1 — 1 file(s),
valid`). The three ways to add "and the slot can match" are refused:

```console
# the requester's spelling: {exists: {entity: demo.slots.Membership, where: {all: [slot_id == candidate.slot_id, side == Left, state == Present]}}}
$ ess specify validate --path v2-issue-exists.yaml
  - [type_mismatch] command.demo.slots.MatchInGroup.outcomes.paired.payload.demo.slots.PairMade.slot_id: the `where:` of `{related: {entity, where, order_by, first}}` is no predicate: cannot parse predicate "entity": a quantifier takes `in`, `as` and `that`, and nothing else

# a counter on the slot, kept by the joining command (`affects: [{entity: Slot, where: slot_id == input.slot_id, sets: {left_waiting: {increment: 1}}}]`)
$ ess specify validate --path v4-counter.yaml
  - [unsupported_construct] command.demo.slots.Join.outcomes.joined.affects: outcome `joined` creates its subject and declares `affects:`; a secondary effect is admitted beside `moves:` and `updates:`, and, from ess/23, `deletes:`, whose subject exists before the outcome

# a quantifier over a declared relation (`Slot` owns `memberships`, many, via `slot_id`)
$ ess specify validate --path v3-relation-quantifier.yaml
  - [unobservable_fact] command.demo.slots.MatchInGroup.outcomes.paired.payload.demo.slots.PairMade.slot_id: `memberships` reads `memberships`, which is not a declared observable root
```

Without a test, a model whose best-scored slot has an empty side can neither publish a pair (the
inner heads select nothing, `ordered-related-head.md`, "No row") nor refuse by name (a guard cannot
read the published head, `published-value-operand.md`, "Where it stands").

The requester's proposals, labelled as theirs: a correlated `exists: {entity: Membership, where:
{slot_id: candidate.slot_id, side: Left, state: Present}}` where `candidate` names the row under
test; or a maintained aggregate on the candidate (a count of related rows) that a selector reads.

## The construct

The quantifier `exists:` takes, as its collection, **the rows of an entity**:

```yaml
format: ess/27
# …
slot_id:
  related:
    entity: demo.slots.Slot
    where:
      all:
        - group_id == input.group_id
        - state == Open
        - exists:
            in: {entity: demo.slots.Membership}
            as: member
            that: {all: [member.slot_id == slot_id, member.side == Left, member.state == Present]}
        - exists:
            in: {entity: demo.slots.Membership}
            as: member
            that: {all: [member.slot_id == slot_id, member.side == Right, member.state == Present]}
    order_by: [score desc, slot_id]
    first: slot_id
```

`exists: {in: {entity: E}, as: b, that: P}` holds of a candidate when **some row of `E`**, bound to
`b`, satisfies `P`. Inside `that:`:

- `b.<field>` reads the related row: its identity, a declared field (and its struct members) or
  `state`, exactly the fields a selector reads of its own candidate (`row_set::candidate_fields`,
  `row_set.rs:240-251`);
- a bare name keeps reading the **candidate**, as everywhere in the selector;
- `input.`, `subject.`, `now` and, in a value-source selector, `payload.<Event>.<field>` (#549) keep
  their meaning.

The rest of the quantifier is unchanged: `as:` is one segment (`predicate.rs:2968-2986`), `that:` is
any predicate, `not:` negates it. The issue's model writes its refusal with the same predicate:

```yaml
- name: no-match
  when_related:
    entity: demo.slots.Slot
    where: {all: [group_id == input.group_id, state == Open, <exists Left>, <exists Right>]}
    exists: false
  error: demo.slots.NoMatchableSlot
```

### Why the quantifier, and why bare stays the candidate

The predicate language already has one construct for "some element of a collection satisfies a
body": `exists: {in, as, that}`, with a named binder so that an inner element and the outer context
can both be read (`predicate.rs:1413-1428`, "Why the binder is part of the value"). Free paths in
the body pass through to the enclosing context (`Element::rebind`, `predicate.rs:1462-1480`). So in
a selector, bare names stay the candidate and the binder names the related row, with no new root
and no new reading of a bare word. Only the collection is new.

The requester's shape reverses that: bare names become the related row, and the candidate moves to
a reserved `candidate.` root. A reserved root collides with a declared field of the same name, the
problem #549 had to resolve for `payload` and records as a live mismatch for `subject`
(`published-value-operand.md`, "The name `payload`"). It also gives `exists:` two shapes inside one
predicate grammar: `{in, as, that}` and `{entity, where}`.

### The collection `{entity: E}`

`in:` takes a fact path, as today, or a mapping with exactly one key, `entity`, holding a qualified
entity name. A mapping under `in:` is refused at every format today, by the predicate parser,
before any format is known (measured: "quantifier `in`: expected a fact path, found a mapping", in a
value selector and in a `when_related:` selector, as `type_mismatch` and `unparsable_predicate`).
No document any `ess` release accepted holds the shape, so recognising it changes no accepted
document's meaning. `entity:` is the key every selector already names its rows by
(`filtered-related-reads.md`).

## Rules

### The rows are tied to the candidate

`that:` must be one comparison, or an `all:`, with at least one top-level conjunct
`b.<f> == <g>` (either side), where `<g>` is a bare field of the candidate: its identity or a
declared field, not `state`. This is the **tie**.

- Without a tie the test answers the same for every candidate. That is a whole-selector condition,
  and `when_related: {entity, where, exists}` already says it. One idea, one spelling.
- An untied walk reads rows other users of the target made, which a selector refuses for the same
  reason it refuses a `where` that selects every row (`row_set.rs:188-197`).
- Synthesis arranges the related rows by the tie (below).

An equality inside `any:` or `not:` is no tie. The tie may be one of several conjuncts, and other
conjuncts may compare the related row with the candidate, the input or a literal.

### The binder names the related row only

The binder must not name a field of the candidate (identity, declared field), `state`, `input`,
`subject`, `payload`, `now`, or a binder in scope. Otherwise a bare word in the body would silently
read the related row instead of the candidate. The existing shadow rule refuses only a bare word on
the right of a comparison (`crates/specify/ess-domain/src/expression.rs:206-214`). It does not
cover a bare field on the left.

### `exists` only

`forall: {in: {entity: E}, …}` is refused. A universal over every row of an entity needs the tie as
an antecedent, the predicate grammar has no implication (`predicate.rs:80-82`), and the same claim
is `not: {exists: {in: {entity: E}, as: b, that: {all: [<tie>, {not: <P>}]}}}`. The refusal's hint
writes that rewrite. One spelling, and the tie rule applies to it unchanged.

### One level

A rows quantifier inside another rows quantifier's `that:` is refused in this cut. Arranging related
rows of related rows multiplies the witness (below), and no need has been stated. A quantifier over
a `List` or `Map` field of the related row (`b.tags`) is an ordinary quantifier and stays admitted.

### Same entity

`E` may be the candidate's own entity ("a slot of the group with a higher score"). The candidate
is then one of the rows walked: there is no same-identity exclusion, as for every selector
(`filtered-related-reads.md`, "One selection, two consumers"). An author who means "another row"
writes `b.<identity> != <identity>`.

### Where it stands

| site | admitted | why |
| --- | --- | --- |
| `where:` of `{related: {entity, where, field}}` and `{related: {entity, where, order_by, first}}`, at every value site the selector stands | yes | the need |
| `where:` and `forall:` of a row-set `when_related: {entity, where, exists \| count \| forall}` | yes | the need: refuse by name when no candidate can match. Same environment, same snapshot |
| `affects[].where`, `instances:` | no, `unsupported_construct` | a set effect's filter is `check_row_predicate` without row-set semantics (`crates/specify/ess-domain/src/command/set_effects.rs:1250-1296`); its witness is the pinned arrangement of #547, which arranges no related rows. A follow-up composes with #547 |
| `when:`, `when_subject:`, identity-addressed `when_related: {via, predicate}` | no, `unsupported_construct` | `when_subject:` over the subject's related rows is `when_related: {entity, where: <tie to subject.>, exists}`, which exists. The hint says so |
| view `filter:`, aggregate measure `where:`, entity invariants, binding conditions, every other predicate site | no, `unsupported_construct` | views and invariants are evaluated by generated targets that enumerate no rows by a selector (`ordered-related-head.md`, "Targets"). Named, not silent |

The refusal is the default. Every predicate position typed by `crate::expression::check_predicate`
refuses a rows collection unless its environment admits one. Only the selector environment does
(Validation, below).

## Meaning

- **Snapshot.** The related rows are the store immediately before the branch is selected, the same
  snapshot as the candidate rows and every guard and value of the command
  (`filtered-related-reads.md`). A row the outcome creates or changes is never walked. In the
  issue's command, the membership a pair consumes is not read again by the same outcome.
- **Three values.** For one candidate, the body is evaluated once per row of `E`. `exists` is the
  Kleene disjunction: `True` if some row's body is `True`; `False` if every row's body is `False`,
  including when `E` has no rows; `Unknown` otherwise. A row whose body is `Unknown` stays a
  possible witness and is never dropped (`row_set.rs:22-24`).
- **The candidate.** The outer predicate then decides the candidate's membership as today. An
  `Unknown` candidate stays a possible member. A head, a one-row read or a row-set test that it
  leaves undecided answers `Undecidable` as it does today (`interpret/execute/row_set.rs:135-200`).
- **Precedence.** No change. In a guard the test is part of the row-set phase. In a value it is
  read after the branch is selected.
- **No row.** A candidate failing the test is not selected. If no candidate passes, a head or a
  read supplies no value, and the outcome cannot publish (`ordered-related-head.md`, "No row"). The
  model refuses by name with the guard above, before the reading branch.

## Validation

Each refusal is reported at the selector's site (`….when_related`, `….payload.<Event>.<field>`,
`….sets.<field>`, `….affects[<i>].sets.<field>`), as the selector's own refusals are.

| condition | code | message names |
| --- | --- | --- |
| `in:` a mapping other than exactly `{entity: <text>}` | `unparsable_predicate` / `type_mismatch` (as today, by site) | that `in:` takes a fact path or `{entity: <Entity>}` |
| a rows collection below `ess/27` | `unsupported_format_version` | the construct; hint `declare \`format: ess/27\`` (today: the parse refusal above) |
| a rows collection at a site that does not admit it | `unsupported_construct` | the site, and the two sites that admit it; at `when_subject:` the `when_related` idiom |
| `E` is no declared entity | `undeclared_reference` | the declared entities (`row_set::undeclared`, `row_set.rs:518-533`) |
| `forall:` over rows | `unsupported_construct` | the `not: {exists: …}` rewrite |
| no tie | `missing_declaration` | "the rows of `E` this test walks are not tied to the candidate"; hint `write \`<b>.<field> == <candidate field>\` as a conjunct of \`that:\``, or the `when_related` guard for a test that does not depend on the candidate |
| the binder names a candidate field, `state`, a root or a binder in scope | `conflicting_declaration` | what it shadows |
| a rows quantifier inside a rows quantifier | `unsupported_construct` | that one level is admitted in this cut |
| `b` read as a value (`b == …`), a field `E` does not hold, a type that does not compare | unchanged | the predicate checker's existing messages, with `b` typed as a row of `E` |

Every code exists already (`crates/specify/ess-primitives/src/error.rs:242`). The new messages
move the string-located diagnostic counts that `docs/design/review-typed-diagnostics.md` pins, so
the domain unit runs `task test-xtask` and `cargo test -p ess-compiler --test typed_diagnostics`
(`AGENTS.md`, "A new `ValidationCode` or diagnostic moves pins").

**How the default refusal is built.** `TypeEnvironment` gains one method in the existing `admits_*`
pattern (`expression.rs:160-255`): `related_rows(&self, entity) -> RelatedRows<Self::Type>`, where
the default answers "not at this site". The checker's `quantified` (`expression.rs:2446-2484`)
asks it for a rows collection instead of reading `over` as a path. Where it gets a row type, it
binds the binder to it, so `b.<field>` resolves through `member()`. `WithSubject`
(`set_effects.rs:1314-1323`) answers for the two admitted sites only. `check_row_predicate` takes
whether rows are admitted: `true` from `row_set::check` and the value selectors, `false` from the
set-effect filters. The format gate is answered there too, as `current_time_later` gates `now`
(`expression.rs:188-197`). The tie, binder, `forall` and nesting rules live in a new
`ess-domain/src/command/related_rows.rs`, called from `row_set::check`.

That every predicate position is typed through `check_predicate` is **inferred, not verified**. A
census test (U2) writes one rows quantifier into each predicate position a document has, and
asserts the refusal or the admission the table states.

## Format and compatibility

- **Rides `ess/27`** while it is unreleased. If `ess/27` ships first, this needs the next format.
  That is the coordinator's call.
- **No document changes meaning.** The shape was a parse refusal at every format (measured above).
  Below `ess/27` it stays refused, with `unsupported_format_version` instead of the parse message:
  a change to a refusal's message, as #543's was.
- **Old-reader controls:**
  - the issue's model at `ess/27` rewritten `format: ess/26` is refused with
    `unsupported_format_version` naming `ess/27`;
  - the same rows quantifier in an `ess/21` `when_related:` selector stays refused (any code);
  - a selector with an ordinary quantifier over a `List` field of the candidate compiles to the same
    IR bytes at `ess/26` and at `ess/27`.
- **`ess-primitives` API.** `Quantified::over` changes from `FactPath` to a two-variant
  `Collection { Fact(FactPath), Rows { entity: String } }`. Every crate that reads `.over` has to
  decide what a rows collection means to it. That is about 85 sites in 17 files of this workspace
  (`grep -rn "\.over\b" crates --include='*.rs'`, tests excluded), and none in the sibling
  repositories that depend on `ess-primitives` (searched under `~/beyond10x`: 0 reads of
  `Quantified` or `.over` outside `ess`). A field beside `over` would let every existing arm read a
  rows quantifier as a path without anyone deciding. The type change forces each arm to decide, for
  the reason #543 chose a new IR variant (`ordered-related-head.md`, "IR").
- **IR.** No shape change. A predicate travels parsed (`crates/specify/ess-compiler/src/ir.rs:48-66`)
  and serialises its canonical node (`predicate.rs:3895-3899`), so `ResolvedRowSelection::filter`
  carries `{exists: {in: {entity: demo.slots.Membership}, as: member, that: …}}`. The entity stays a
  name, as a predicate's paths do. `ess-domain` guarantees it is declared, and consumers look it up
  with an `Option`, as the IR note says of paths. No explicit list like #549's `reads` is needed:
  `payload.` needed one because the path could also name a candidate field, while `{entity: E}` is
  unambiguous. A model without the construct keeps its bytes.
- **Schema.** `Predicate`'s schema is `string | boolean | array | object`, with a description
  (`predicate.rs:4083-4100`). Only the description gains one sentence. `cargo xtask schema` then
  `task projection-check`.
- **Diff.** `ess verify diff` prints a selector's predicate (`ir.rs:1638`, `describe`), so a
  changed test is a visible behaviour change of the reading field or the guard. No new class, no
  `ess-diff` version.
- **Suite.** No step kind, so `ess-conformance/*` keeps its version.
- **Migration.** None.

## Interpreter and history checker

- **Evaluation in `ess-primitives`.** `FactSource` gains
  `fn rows(&self, entity: &str) -> Option<Vec<Box<dyn FactSource + '_>>>`, which by default answers
  `None`. `Quantified::evaluate` (`predicate.rs:1430-1460`) over `Collection::Rows` folds the body
  over those rows, each read through an overlay that sends `b.<rest>` to the row and every other
  path to the enclosing source. `None` gives `Unknown`. So an evaluator that knows no store can
  never answer `False` for a rows test, which is sound by construction for every external user of
  the crate.
- **Forwarding.** The delegating sources forward `rows` to what they wrap: `Element`
  (`predicate.rs`), `set_effects::Facts` (`interpret/execute/set_effects.rs:454-502`),
  `caller::Facts` (`interpret/execute/caller.rs:94`) and the history facts
  (`interpret/execute/history/facts.rs:10`). A missed forward degrades to `Unknown`, which the
  interpreter test sees as `Undecidable` instead of the value.
- **The rows.** `row_set::Reader::members` (`interpret/execute/row_set.rs:69-134`) collects, before
  its candidate loop, `set_effects::row_facts` of every stored row of each entity a rows quantifier
  in the selector or `satisfies` names. It enumerates them as it enumerates candidates
  (`self.store.instances()`) and hands them to `Facts`. One snapshot, read once per decision.
- **History.** The Reader is built from the same `State` in history mode (`execute.rs:673`,
  `:2313`, `:2324`). An abstract value in a related row leaves its body `Unknown`, so `exists` is
  `Unknown` unless another row decides it `True`, and the candidate stays a possible member. Sound
  and deliberately weak, as `ordered-related-head.md` is. A stronger proof may decide more only by
  showing the same answer for every completion.

## Synthesis

Today a row-set branch arranges its selectors' rows through the declared creating commands,
first-appearance order, with decoys per conjunct and the branch decided again over the walked rows
(`synthesize/row_set.rs:166-191`, `:773`, `:974`). A selector counts as arranged only if it is
**scoped**, by a `String`/`Uuid` equality with `input.` or `subject.` (`:206-243`, `decidable`
`:1770-1790`). A rows quantifier adds **related rows per candidate**, and a witness must show that
the test, not the ranking, decided the pick.

Terms: the candidate entity `C`, the related entity `E`, the tie `b.t == c`, and the body's other
conjuncts `B1 … Bk`.

1. **Decidable.** Each rows quantifier's tie is scopable (`scopable`, `:194-204`: `String`, `Uuid`,
   newtypes of them; members of a struct identity from `ess/23`, #463). Some creating command of
   `E` writes `t` from its input, which is the `steerable` search (`synthesize/ordered_head.rs:137`).
   Otherwise a named gap (below).
2. **Selected candidates** are arranged as today (`COUNTS`, `HEAD_COUNTS` for a head), each
   captured (`capture_instance`). For each selected candidate and each `exists` that must hold, one
   row of `E` satisfies the body. Its tie field is written from the captured candidate, as #549's
   `closed` writes an outer value (`published-value-operand.md`, "Synthesis" 3), and every `Bi` is
   steered as `arrange_row` steers a conjunct (`row_set.rs:1414`, `compared`). Rows another
   selector of the command already arranged count, because the branch is decided again over all
   walked rows (#543 U5). The issue's inner heads put their Left and Right rows under the head slot,
   and those rows serve.
3. **Under a negated test** (`not: {exists: …}`) each selected candidate gets one row per `Bi` that
   meets every conjunct but `Bi`. The test holds, and a target that drops `Bi` sees a witness and
   rejects the candidate.
4. **A failing candidate per test: the witness this issue asks for.** For each rows quantifier `Q`
   of the selector, one candidate `F_Q` meets every other conjunct of the outer `where`, including
   every other rows quantifier, and fails `Q`:
   - for an `exists`, `F_Q` holds one related row per `Bi` that meets every conjunct but `Bi`,
     the tie included, and no row that meets them all;
   - for a `not exists`, `F_Q` holds one related row meeting every conjunct.

   For a **head**, `F_Q` ranks strictly **above** the head on the first key: its score is above
   the middle head's, as `ordered-related-head.md` steers S2. Its `first` value differs from every
   other row's. For a **one-row read**, `F_Q` is a second row the selector would select if `Q` were
   ignored. For a **row-set guard**, `F_Q` changes the count a target ignoring `Q` sees. The
   `no-match` refusal is witnessed with only failing candidates open in the group (`F_left`,
   `F_right`): the healthy target refuses, and a target ignoring the test pairs.
5. **Checked, never assumed.** The branch is decided again over the walked rows (`consistent`,
   `:773`) with the interpreter's evaluation, rows quantifiers included. The expected value is the
   head the interpreter's comparator computes over those rows.

What one scenario of the issue's `paired` branch kills (each is an acceptance case):

| faulty target | answers |
| --- | --- |
| ignores the test (best score in the group) | the higher-ranked of `F_left`, `F_right` |
| tests the Left side only | `F_right` |
| drops `member.state == Present` | `F_left` or `F_right`: each holds a `Removed` member on the missing side |
| drops `member.side == Left` | `F_left`: its Right member satisfies the weakened Left test |
| drops the tie (some Left member exists anywhere) | `F_left`: the selected slots' Left members satisfy it |

Named gaps (`RefusalCause::NoWitness`). In each, the scenario stands without the missing part
where it can, as `published-value-operand.md` keeps a scenario without a cross decoy:

| situation | gap |
| --- | --- |
| the tie is not scopable, or no creating command of `E` writes `t` from its input | "the rows of `<E>` related to a candidate are not arranged: no creating command writes `<t>` from its input" (the outcome's scenario is refused) |
| the first key cannot be steered above the head | "no candidate failing `<Q>` ranks above the head of `<C>`: a target ignoring the test is not caught" |
| a `Bi` no declared command can make false on a related row | "`<Bi>` of the test over `<E>` is never violated: a target dropping it is not caught" |
| the selection admits one candidate only | the head's own `UNOBSERVABLE` gap; a failing candidate needs a second |

Cost on the issue's command (inferred, not measured): 3 selected and 2 failing slots; 6 members
for the selected slots, mostly shared with the inner heads' rows; 3 or 4 members per failing slot,
2 of them moved to `Removed`. Together with #549's inner arrangement, that is about 35 to 45 steps.

## Targets

| target | disposition |
| --- | --- |
| native interpreter | executes the test (above) |
| history checker | sound, or `Undecidable` (above) |
| synthesis | arranges related rows and failing candidates, or a named gap (above) |
| authored scenarios | run through the interpreter, with no change of their own |
| Go and TypeScript suite runtimes | none: they compare published values and outcomes |
| generated Rust and Go | no new obligation. The selector forms that admit the test are already owed (`crates/generate/ess-synth/src/determined.rs:202-209`, `:507-525`). `story:generated-behaviour-reads-several-related-rows` gains one scope line: "a rows quantifier in a selector". The invariant emitters' quantifier arms (`ess-synth/src/{rust,go}/invariant.rs`) refuse a rows collection by name. Validation keeps it out of invariants, so the arm is unreachable, but it is never a silent path |
| web player | none, as for any selector read (`ess-conformance/src/web.rs:262-263`) |
| Entity Runtime | the existing `ROW_SET` limitation (`ess-entity-runtime/src/subset.rs:100`, `:120`). Its quantifier arm (`lib.rs`) refuses by name |
| `ess-ui-check` | its enum-literal walk (`crates/ui/ess-ui-check/src/enums.rs`) skips the collection and walks the body with the binder unresolved (inferred; U1 decides) |
| OpenAPI, AsyncAPI, service contract | no change: wire shapes and `type_ref` unchanged |
| linearizer | a selector already marks the command as reading other rows (`ess-conformance/src/linearize.rs:384-424`) |
| generated docs, diff | the predicate's canonical text (above) |
| mutation audit | no operator added. The failing candidates kill the dropped-test, one-side and dropped-conjunct targets. An operator "drop a rows quantifier" is an optional later unit |

## Not in this design

- **`count` of related rows** ("a slot with at least two waiting"). The predicate grammar has no
  count of a filtered collection. A maintained aggregate or a counted quantifier would be needed,
  and no need has been stated.
- **Ranking by a related count** ("the slot with the most waiting"): the same.
- **Set-effect filters** (`affects[].where`, `instances:`): refused by name, follow-up with #547's
  witness.
- **Views and invariants:** refused by name, until generated targets enumerate rows by a selector.
- **`forall`, nesting:** refused, with the rewrite or the reason.
- **A declared relation as the collection** (`in: memberships`). See alternative (c). It can be
  added later as a spelling of a tie the relation declares.

## Fit review

1. **Need.** A command picks a row among candidates **each of which must have related rows meeting
   a condition**: the best slot of a group among those whose two sides each hold a `Present`
   membership. A model also refuses by name when no candidate qualifies. ESS cannot state the
   per-candidate test, so it cannot publish the right pair, and the suite cannot check that the
   implementation skipped an unmatchable higher-scored slot. Minimal reproduction: "The gap"
   (binary at `3fdd9479e`, three refusals measured). The requester's syntax is quoted there,
   labelled as theirs.
2. **Class: gap.** A selector's operands are the candidate, `input.`, `subject.` and `now`
   (`row_set.rs:12-16`). A quantifier's collection is a fact path (`predicate.rs:2961-2966`). No
   construct ranges over rows inside a predicate. `published-value-operand.md` names it as a
   separate need ("Not in this design"). It is not a defect: no page promises it.
3. **Already expressible? No.** Measured: the correlated `exists:` is a parse refusal, the
   relation quantifier is `unobservable_fact`, and the counter kept by `affects:` on the creating
   command is `unsupported_construct` (`ESS-COMMAND-009`). A counter kept by moving and updating
   commands only would need `Join` to address the slot rather than create the membership, which
   changes the adopter's API. It would also store a second statement of a fact the rows already
   hold, with no check that the two agree. A row-set guard per input-named slot
   (`slot_id == input.slot_id`) makes the caller choose the slot, which is
   https://github.com/beyond10x/ess/issues/544's problem.
4. **Fit.**
   - *Vocabulary.* The quantifier `exists: {in, as, that}` exists. The collection reuses
     `entity:`, the key every selector names its rows by. Bare names keep the candidate, as in every
     selector. No new root (the requester's `candidate.`) and no new keyword.
   - *Composition.* Row-set guards and value selectors (`field:` and `first:`) admit it in the same
     environment and snapshot. It composes with #549's `payload.` operand, which is admitted inside
     `that:` of a value selector. Set-effect filters, `when:`, `when_subject:`, views and invariants
     refuse it by name, each with its reason or idiom. Precedence is unchanged.
   - *Siblings.* `forall` is refused with its `not exists` rewrite, which is one spelling, not a
     missing capability. `count` is named under "Not in this design". The set-effect filter is the
     one sibling deferred, named with #547.
   - *Targets.* Every target executes, owes or refuses it by name (the targets table). The
     `Collection` type change makes every quantifier arm decide.
5. **Second, unrelated adopter.** A support desk assigns a ticket to the highest-priority team of
   a desk **that has an available agent**: `{related: {entity: Team, where: {all: [desk_id ==
   input.desk_id, {exists: {in: {entity: Agent}, as: agent, that: {all: [agent.team_id == team_id,
   agent.state == Available]}}}]}, order_by: [priority desc, team_id], first: team_id}}`. A
   warehouse picks the nearest bin **holding a lot of the product with stock**
   (`lot.bin_id == bin_id`, `lot.product_id == input.product_id`, `lot.quantity > 0`). An
   enrolment service refuses a section with a non-empty waiting list
   (`not: {exists: …}`).
6. **Cost.**
   - Format: none of its own; it rides `ess/27`.
   - Keywords: none. `in:` gains a mapping value, `{entity: <Entity>}`.
   - Diagnostics: no new code, about 9 new messages.
   - IR: no shape change, and old bytes are unchanged.
   - Schema: one description sentence.
   - Suite, `ess-diff`: unchanged. Migration: none.
   - Public Rust API: `ess_primitives::predicate::Quantified::over` changes type, and
     `FactSource` gains a defaulted method. No sibling repository reads `over` (searched).
   - Implementation: about 85 mechanical arms in 17 files, then the domain, interpreter and
     synthesis units (below).
7. **Alternatives.**
   - (a) **Change nothing.** The model publishes nothing when the best slot is unmatchable, and
     cannot refuse by name. The adopter keeps the field `{generated: true}` and a known failure.
   - (b) **The requester's correlated `exists: {entity, where}` with a `candidate.` root.** Changed:
     a reserved root collides with field names (#549's `payload`/`subject` record), and bare names
     would flip meaning inside one predicate. The need, the per-candidate `exists`, is kept.
   - (c) **A declared relation as the collection** (`exists: {in: memberships, as: m, that: …}`,
     with `Slot` owning `memberships` via `slot_id`). Attractive: no tie rule, and the domain
     declares the tie once. Rejected for now. It needs a declared relation, and the issue's model
     declares none (the validator only warns, `ESS-ENTITY-019`, measured). It ties only by an
     identity carried in `via`. A `references` relation may share its carrying field's name, which
     its own suggested fix does (`{name: slot_id, kind: references, … via: slot_id}`, measured), so
     `in: slot_id` would be ambiguous. It can be added later as a spelling of a tie.
   - (d) **A maintained aggregate**, a derived field such as `left_waiting: {count: {entity:
     Membership, where: …}}` that a selector reads and that could even rank. Rejected: a new kind of
     field that every target stores and keeps current on every write to `E`, in generated storage,
     views, the interpreter and the entity runtime. It also needs a new diff class and a storage
     migration. The need is a test, not a stored value. It remains the route if ranking by a
     related count is ever needed.
   - (e) **An author-kept counter** (lift the `affects:`-beside-`creates` refusal). Rejected: two
     statements of one fact with nothing checking that they agree, which the relations design
     rejects for the same reason (`ess-entity-relations-design-v0.1.md`, §5, "Declared on both
     ends").
   - (f) **The quantifier over `{entity: E}`, tied to the candidate.** Chosen.

## Decisions (proposed, for the story)

- **Accept, redesigned.** The requester's per-candidate `exists` is kept. Their spelling is
  changed to the existing quantifier with a rows collection, and the maintained aggregate is
  declined.
- Spelling: `exists: {in: {entity: <E>}, as: <b>, that: <P>}`. Bare names stay the candidate, and
  `<b>.` is the related row.
- A tie, `<b>.<f> == <candidate field>`, is required as a top-level conjunct.
- Admitted in value-source selectors and row-set guards only. Every other predicate site refuses by
  default, through the type environment.
- `exists` only: `forall` is refused with its rewrite. One level, no nesting.
- `Quantified::over` becomes a `Collection` enum, so every arm decides. `FactSource::rows`
  defaults to `None`, which evaluates to `Unknown`.
- No IR shape change, no new code, no suite or diff version. It rides `ess/27`.
- Generated Rust and Go stay owed under the existing selector obligation.
- Reversible: the shape was refused at every format before.

## Decisive acceptance

Each test is written first and seen red.

- `crates/specify/ess-primitives/tests/related_rows_quantifier.rs`:
  `in_entity_parses_and_prints_its_canonical_form`,
  `in_takes_only_a_path_or_one_entity_key`,
  `a_rows_test_without_a_rows_source_is_unknown`,
  `exists_over_rows_is_the_kleene_disjunction` (a `True` row, all `False`, none, one `Unknown`),
  `the_binder_reads_the_row_and_bare_paths_the_enclosing_source`.
- `crates/specify/ess-domain/tests/related_rows_quantifier.rs`:
  `a_head_over_matchable_slots_validates_at_ess_27`,
  `a_row_set_guard_refusing_no_matchable_slot_validates`,
  `the_rows_test_below_ess_27_is_refused_naming_ess_27`,
  `an_untied_rows_test_is_refused`,
  `a_tie_inside_any_is_no_tie`,
  `a_binder_naming_a_candidate_field_is_refused`,
  `forall_over_rows_is_refused_with_the_not_exists_rewrite`,
  `a_nested_rows_test_is_refused`,
  `an_undeclared_entity_is_refused`,
  `a_rows_test_in_an_affects_filter_is_refused`,
  `a_rows_test_in_when_subject_names_the_when_related_idiom`,
  `every_other_predicate_site_refuses_a_rows_test` (the census),
  `a_published_operand_in_the_body_validates` (with #549).
- `crates/specify/ess-compiler/tests/related_rows_ir.rs`:
  `the_selection_carries_the_rows_test_in_its_predicate`,
  `a_model_without_a_rows_test_keeps_its_ir_bytes`.
- `crates/verify/ess-conformance/tests/related_rows_quantifier.rs` (fixture
  `tests/fixtures/related-rows-quantifier.yaml`, the issue's command):
  - `the_interpreter_skips_a_higher_scored_slot_with_an_empty_side`;
  - `no_matchable_slot_is_refused_by_name`;
  - `a_removed_member_does_not_make_a_side_matchable`;
  - `an_abstract_related_value_leaves_the_head_undecidable`;
  - `synthesis_arranges_a_higher_scored_candidate_that_fails_the_test`;
  - one case per row of the faulty-target table, each failing a scenario:
    `a_target_ignoring_the_test_fails_a_scenario`,
    `a_target_testing_one_side_fails_a_scenario`,
    `a_target_dropping_the_state_conjunct_fails_a_scenario`,
    `a_target_dropping_the_side_conjunct_fails_a_scenario`,
    `a_target_dropping_the_tie_fails_a_scenario`;
  - `an_untied_creation_is_a_named_gap`;
  - `an_unsteerable_first_key_is_a_named_gap_and_the_scenario_stands`.

## Implementation units

Each unit is one agent and one branch, based on the integration head after #549 U2 (the domain
and IR units of the published operand). The units of this page are named R1 to R5, to keep them
apart from #543's and #549's U-numbers. Footprints are estimates, not measurements, taken from
the sibling pages: about 10 GiB per `ess-conformance` unit (`ordered-related-head.md`), 6 to 8
GiB for `ess-domain` with `ess-compiler` and `ess-cli` (`set-effect-identity-filters.md`), and
about 2 GiB for an `ess-u544`-sized partial build.

| unit | what | files | depends on | build footprint (estimate) |
| --- | --- | --- | --- | --- |
| R1 | the collection in the predicate language, and every quantifier arm deciding | `ess-primitives/src/predicate.rs` (`Collection`, `quantifier_collection`, `to_node`, `Quantified::evaluate`, the walkers over `over`, the schema description); `ess-primitives/src/facts.rs` (`FactSource::rows`, defaulted); the arms in `ess-domain/src/expression.rs`, `ess-domain/src/expression/lexical.rs`, `ess-domain/src/command/set_effects.rs`, `ess-domain/src/view/measure.rs`, `ess-synth/src/rust/invariant.rs`, `ess-synth/src/go/invariant.rs`, `ess-entity-runtime/src/lib.rs`, `ess-ui-check/src/enums.rs`, `ess-conformance/src/{input.rs,now_offset.rs,witness.rs,synthesize.rs,synthesize/aggregate.rs,synthesize/aggregate/contrast.rs,synthesize/identity.rs,synthesize/subject_fact.rs}` (each: a path keeps today's behaviour, a rows collection refuses or answers `Unknown` by name); `schemas/generated/ess.schema.json` (`cargo xtask schema`); new `ess-primitives/tests/related_rows_quantifier.rs` | — | rebuilds the workspace's dependents of `ess-primitives`: `cargo check -p` on each touched crate, `cargo test -p ess-primitives`, `ess-domain`; about 8 to 10 GiB |
| R2 | the domain: sites, format, tie, binder, `forall`, nesting; IR pin | `ess-domain/src/expression.rs` (`TypeEnvironment::related_rows`, the checker's `quantified`, a row type for the binder); `ess-domain/src/command/set_effects.rs` (`WithSubject` answers rows for selectors, `check_row_predicate` flag); `ess-domain/src/command/row_set.rs` (`check` passes the flag, calls the rules); `ess-domain/src/command/value_expression.rs` (the selector and head checks pass the flag); new `ess-domain/src/command/related_rows.rs`; new tests `ess-domain/tests/related_rows_quantifier.rs`, `ess-compiler/tests/related_rows_ir.rs` | R1, #549 U1 | `ess-domain`, `ess-compiler` (`typed_diagnostics`), `ess-cli`, `task test-xtask`; about 6 to 8 GiB |
| R3 | interpreter | `ess-conformance/src/interpret/execute/row_set.rs` (`Reader::members` collects the related rows); `interpret/execute/set_effects.rs` (`Facts` forwards and supplies `rows`); `interpret/execute/caller.rs`, `interpret/execute/history/facts.rs` (forward `rows`); new fixture `tests/fixtures/related-rows-quantifier.yaml`; interpreter cases of new `tests/related_rows_quantifier.rs` | R2, #549 U3 | one `ess-conformance --test` plus `ess-cli`; about 10 GiB |
| R4 | synthesis | `ess-conformance/src/synthesize/row_set.rs` (`decidable` for ties, related rows per selected candidate, failing candidates, `consistent` with rows); new `ess-conformance/src/synthesize/related_rows.rs` (tie detection, per-conjunct related decoys, the gaps); `synthesize/ordered_head.rs` (ranking a failing candidate above the head, visibility only); the synthesis and faulty-target cases of `tests/related_rows_quantifier.rs` | R3, #549 U4 | as R3 |
| R5 | documentation | `website/docs/guides/specify/guards-and-predicates.md` (the rows collection); `website/docs/guides/specify/values-and-views.md` (the matchable-slot example); `website/docs/reference/spec-versions.md` (`ess/27` row); `CHANGELOG.md` `[Unreleased]`; `docs/design/published-value-operand.md` ("Not in this design" points here); the scope line in `story:generated-behaviour-reads-several-related-rows` (coordinator, store); this page's status | R4 | `task test-xtask`; `task site-build` |

Order: R1 → R2 → R3 → R4 → R5, strictly. Each needs the previous one's types, and R3 and R4 share
the fixture. R1 may start beside #549 U1/U2 only if it rebases before R2. R1's arms touch
`expression.rs` and `set_effects.rs` in `ess-domain`, which #549 U1 edits.

Overlap with the units of this wave:

- **#549** (`published-value-operand.md`): U1 edits `set_effects.rs` (`WithSubject`) and
  `value_expression.rs`, the same places R2 edits (**high**). U3 and U4 edit
  `interpret/execute/row_set.rs`, `set_effects.rs` `Facts` and `synthesize/row_set.rs`, which R3
  and R4 edit (**high**). Land #549's unit first, then rebase.
- **#543** U7 (struct identities in an order, declared-unordered ties): `synthesize/ordered_head.rs`
  and `value_expression.rs` (medium).
- **#547**: U1 edits `check_filter`/`check_row_predicate` in `set_effects.rs` (medium for R1 and
  R2). U2 is in `synthesize/set_effects.rs` (none).
- R5 shares the `ess/27` row and the CHANGELOG section with every `ess/27` unit (medium).

Gates per unit (no full gate; `AGENTS.md`, the package-gate subset):

| unit | gates |
| --- | --- |
| R1 | `cargo test -p ess-primitives --test related_rows_quantifier`, `cargo test -p ess-primitives`, `cargo test -p ess-domain`; `cargo check -p` for `ess-synth`, `ess-entity-runtime`, `ess-ui-check`, `ess-conformance`; `clippy -p` on each touched crate; `cargo xtask schema` then `task projection-check`; `task ci-lint` (public API); `task fmt-check` |
| R2 | `cargo test -p ess-domain --test related_rows_quantifier`; `cargo test -p ess-compiler --test related_rows_ir`; `cargo test -p ess-compiler --test typed_diagnostics`; `ess-cli` targeted tests; `task test-xtask`; clippy; `task fmt-check` |
| R3, R4 | `cargo test -p ess-conformance --test related_rows_quantifier`, then `--test nested_ordered_heads`, `--test ordered_related_source`, `--test row_set_several_selectors` one at a time; `ess-cli` targeted tests; `clippy -p ess-conformance` |
| R5 | `task test-xtask`; `task site-build` |
