# Guards over stored fields and constraints across records

Status: rule 1 implemented in source format `ess/9` (beyond10x/ess#75). Rule 2 stays out of scope, as
below.

## Behavior and authority

A trial specified two real services from their source. Two rules had no ESS form, and both were
left as `UNMAPPED:` markers — the `ess-specify` skill's convention for a rule the model cannot
state (`plugins/ess-specify/skills/specify/SKILL.md` in `beyond10x/agentplugins`); nothing in this
repository reads the marker:

1. "Express parcels over 20 kg are refused at dispatch."
2. "A room is never double-booked."

In rule 1 the weight and the service are recorded when the parcel is created. The dispatch
command's input carries the parcel identity and nothing else, and `when:` is a predicate over the
command's input only. The refusal depends on two fields of the row the command addresses, one of
them compared by ordering.

In rule 2 no single command input decides the outcome. Whether a booking is admitted depends on
every other booking of the same room whose time range overlaps the new one. The rule is over a set
of records, not over one.

## What exists today

| Construct | What it can say | Why it does not state rule 1 |
|---|---|---|
| `when:` | A predicate over the command's own input fields. | The weight is not input at dispatch. Copying it into the input makes the caller the authority on a stored fact; the guard then checks what the caller says instead of what was recorded. |
| `when_subject_state:` (`subject-state-outcome-guards.md`), `when_state_changes:` (`state-change-outcome-guards.md`) | Select an outcome by the addressed entity's held **lifecycle state**, read immediately before selection. | They read the state name only, and the first page declares no "cross-entity read, view query, field comparison". |
| `when_subject: {field, equals}` (ess/6, `observed-subject-history.md`) | Selects an outcome by equality of one declared **enum** field of the existing subject against one variant, conjunctive with an optional input `when:`. | Enum equality only. `weight_kg > 20` is an ordering over a number, and "Express and over 20 kg" is a conjunction of two fields. It also cannot sit on a refusal: see the table below. |
| `wrong_state:` | Answers in the complement of a move's `from` set, with no subject of its own. | A lifecycle state, not a field value. The *shape* — a refusal decided by the subject the siblings name — is the one rule 1 needs. |
| entity `invariants:` | "What must hold of every instance, at rest" (`crates/specify/ess-domain/src/entity.rs`), checked over the fields plus the `state` pseudo-field. Rule 1's safety property is writable today: `not (state == Dispatched and service == Express and weight_kg > 20)`. Conformance asserts every invariant after each state-changing branch through a view that publishes the fields (`ViewExpectation::Satisfies`, `crates/verify/ess-conformance/src/synthesize.rs`); the Entity Runtime projection lowers it to a runtime rule (`crates/generate/ess-entity-runtime/src/lib.rs`). | It names no command, no outcome and no error: it says the bad row never exists, not what `Dispatch` answers. Its witness is the ordinary scenario of each branch, which never arranges an overweight Express parcel on purpose, so it is only as strong as the rows those scenarios happen to produce. |
| Per-slot lifecycle | Rule 2 can be approximated by modelling each bookable unit (room × fixed slot) as an entity with `Free → Booked`; a second booking becomes a `wrong_state` refusal. | Only exact for fixed, pre-enumerated slots. Free time ranges overlap without being equal, so the entity identity cannot be the slot. It also changes the model the service actually has, which is one booking per request. |

### What today's `ess` accepts for rule 1

Measured with `ess 0.32.0` (`ess specify validate`, `ess verify conform synthesize --target ir`)
on a scratch model: `Parcel` with `service: Service` (enum `Standard | Express`) and
`weight_kg: Integer`, `Create` setting both, `Dispatch` moving `Created → Dispatched`.

| Shape written | Result |
|---|---|
| `when_subject: {field: service, equals: Express}` + `moves:` + `error:` on the refusal | refused, `refusal_mutated_state`: "a refused command changes nothing, so a refusal has no subject" (`command.rs`) |
| the same without `moves:` | refused, `conflicting_declaration`: "a subject-state guard requires an existing moves or updates subject and input identity" — sited at `outcomes.<name>.when_state_changes`, a key the author never wrote (`held_state_key` in `command.rs`) |
| guarded success `dispatched` + effect-free `error:` default; `Create` sets `service: input.service` | validates; `dispatched` gets `ESS-SYNTH-001` "no bounded arrangement establishes the subject fact and eligible input" when the guard names the variant the create witness did not pick |
| the same with a driver writing the literal `sets: {service: Express}` | validates; `dispatched` is witnessed through that driver. The `error:` default is witnessed by sending an identity no row carries (`witness.rs` `uuid`, which "identifies nothing") and requiring `ExpressOverweight` |
| two `when_subject` branches covering both variants, no default | refused, `non_exhaustive_branches` and `unreachable_branch`: the stored fact enters no partition |
| `when_subject` on one branch, `when_subject_state:` on a sibling | refused, `conflicting_declaration`: "subject fact and lifecycle guards cannot be combined in one command" (`command/subject_fact.rs`) |
| `when_subject: {predicate: …}` | reader error `unknown field predicate, expected field or equals` (`RawSubjectField` is `deny_unknown_fields`) |

So the only writable form of rule 1 today is inverted — the guard on the success, an unconditional
`error:` for everything else — and it is reachable only through a driver that writes the guarded
value as a literal. Its refusal witness arranges no parcel: the suite requires `ExpressOverweight`
for an identity no row carries and never dispatches a parcel the guard excludes, so an
implementation that ignores the stored field passes it. That is the gap this page closes.

## Proposal for rule 1: a predicate over the subject's stored fields

`when_subject` takes a second shape, `{predicate}`, beside today's `{field, equals}`; never both in
one branch. It reads the same subject at the same point as `when_subject` and `when_subject_state`
and widens what may be asked of the subject's declared fields. Rule 1, as the service states it:

```yaml
entities:
  - name: shipping.parcel.Parcel
    identity: {name: parcel_id, type: Uuid}
    fields:
      - {name: service, type: shipping.parcel.Service}   # enum: Standard | Express
      - {name: weight_kg, type: Integer}
    lifecycle:
      initial: Created
      states: [Created, Dispatched]
      terminal: [Dispatched]
      transitions:
        - {name: dispatch, from: [Created], to: Dispatched}

commands:
  - name: shipping.parcel.Create
    input:
      - {name: service, type: shipping.parcel.Service}
      - {name: weight_kg, type: Integer}
    outcomes:
      - name: created
        creates: shipping.parcel.Parcel
        instance: parcel_id
        sets: {service: input.service, weight_kg: input.weight_kg}
        emits: [shipping.parcel.Created]
        payload:
          shipping.parcel.Created:
            parcel_id: {generated: true}

  - name: shipping.parcel.Dispatch
    input: [{name: parcel_id, type: Uuid}]
    outcomes:
      - name: refused-overweight
        when_subject:
          predicate:
            all:
              - service == Express
              - weight_kg > 20
        error: shipping.parcel.ExpressOverweight
      - name: dispatched
        moves: shipping.parcel.Parcel.dispatch
        instance: parcel_id
        emits: [shipping.parcel.Dispatched]

views:
  - name: shipping.parcel.Parcels
    source: shipping.parcel.Parcel
    consistency: read_your_writes
    fields:
      - {name: parcel_id, type: Uuid}
      - {name: state, type: shipping.parcel.Parcel.State}
      - {name: service, type: shipping.parcel.Service}
      - {name: weight_kg, type: Integer}
```

The view is part of the rule: it is the immediate, unfiltered view through which the guarded
fields are observed, and without one the witness is refused.

`weight_kg` is an `Integer` and not a `Decimal` on purpose: `sets:` admits no literal spelling for
`Decimal` or `Binary64` (`crates/verify/ess-conformance/src/input.rs`, `primitive_literal`), so a
`Decimal` field cannot be arranged to a value and cannot be guarded until it can.

### The refusal carries the guard

Two designs were open: (a) admit `error:` on a subject-guarded branch that names no subject of its
own, the subject being the one its siblings name; (b) keep the guard on the success and extend the
ess/7 effect-free default (`has_state_refusal` in `command.rs`, `is_state_refusal` in
`synthesize.rs`) from lifecycle guards to subject facts, so the default is arranged against a real
row. **(a) is chosen**, and (b)'s arrangement follows from it anyway:

- It reads as the rule. Under (b) the author writes the complement, `any: [service != Express,
  weight_kg <= 20]`, and the rule the service states appears nowhere — a second copy of one fact,
  negated, which is what `state-change-outcome-guards.md` refuses for the same reason.
- Every refusal keeps its own error. A command refusing for two stored-field reasons names two
  errors under (a); under (b) both collapse into one default. `command.rs` already argues this for
  `wrong_state:`: "do not generate vague *operation fails* tests if the domain declares a specific
  error".
- It is the shape the model already has. `wrong_state:` and the ess/7 effect-free default are both
  refusals without a subject, decided by the subject the command addresses. `refusal_mutated_state`
  stays: a refusal still carries no `moves:` or `updates:`.
- Both designs need the same new arrangement (below): a row satisfying a predicate for the refusal
  under (a), a row refuting one for the default under (b). Once the default is arranged by refuting
  every guarded sibling over the row, the inverted shape is witnessed correctly as a consequence,
  without being the recommended one.

### Declaration

- **Subject.** The command's common subject: the existing entity and identity field that every
  input-selectable branch with a subject names (`command/subject_fact.rs`: "subject fact
  selection requires one existing subject identity"). A guarded `error:` branch names none and
  takes that one; a guarded `moves:` or `updates:` branch names it itself. A command whose only
  subject-bearing branch is `creates:` has no subject to read, and is refused.
- **Namespace.** The predicate reads the entity's declared `fields` and nothing else: not the
  input, and not `state`. Input stays in `when:`, conjunctive with the subject predicate as it is
  today for `{field, equals}`. The lifecycle stays with `when_subject_state:` and
  `when_state_changes:`, and combining those with `when_subject` on sibling branches stays refused
  (below, "one strategy or two").
- **Literals.** The bare-word rule of `crates/specify/ess-primitives/src/predicate.rs` applies
  unchanged: a right-hand side without a dot is a literal, so `service == Express` compares with
  the text `Express`, and the checker refuses a bare word that names a declared field
  (`text_literal` in `crates/specify/ess-domain/src/expression.rs`).
- **Read point.** The row immediately before command selection, as for `when_subject_state`.
- **Optional fields.** May be read. `evaluate_compare` answers `Unknown` when an operand does not
  resolve (`predicate.rs`), and an unknown fact selects no branch and never the default
  (`observed-subject-history.md`). An author who wants `None` to select something writes
  `not defined(field)`. No new refusal; the finite prover already declines `Optional` paths.
- **Absent subject.** Satisfies no predicate and reaches no declared outcome, as today; it is not
  created and not read as an initial-state row. This is a witness requirement below, because it is
  exactly what the unarranged refusal witness gets wrong today.

### What it refuses

| written | refusal |
|---|---|
| `{field, equals}` and `{predicate}` in one `when_subject` | the reader's own refusal: one shape per branch |
| beside `when_subject_state:`, `when_state_changes:`, `external:` or `wrong_state:` on the same branch | `conflicting_declaration` — "a subject fact has one selection authority" (`command.rs`), as today |
| on a `creates:` branch, or in a command with no subject-bearing sibling | `conflicting_declaration`, sited at `outcomes.<name>.when_subject` — not at `when_state_changes` |
| a field the entity does not declare, `state`, or an input path | `unobservable_fact` at `outcomes.<name>.when_subject`, from the expression checker with the entity's fields as its environment |
| a literal of the wrong type, a bare word naming a field, an ordering against a non-RFC 3339 text for a `Timestamp` | `type_mismatch` / `undeclared_reference` — the checker's existing diagnostics, unchanged |
| sibling `when_subject` predicates over different entities or identity fields | `conflicting_declaration` at `outcomes` — "subject fact branches must share one entity, identity" (`command/subject_fact.rs`), with the "and enum field" clause dropped |
| `when_subject` beside a lifecycle guard in one command | `conflicting_declaration` at `outcomes` — "cannot be combined in one command", as today |

### Validation: a joint entity-field × input partition

Today the stored fact enters no partition (`finite_coverage` in `command.rs` collects each branch's
*input* predicate only, and `command/subject_state.rs` partitions lifecycle guards only), which is
why two `when_subject` branches over both variants of an enum are refused as non-exhaustive. The
proposal adds a partition over the entity fields the predicates name, crossed with the input fields
the `when:` guards name, in `command/subject_fact.rs`:

- **Evaluator and environment reused.** The expression checker runs over
  `DomainEnvironment::new(types, &entity.fields)` for the subject side — the environment
  `entity.rs` already builds for invariants, minus the `state` pseudo-field — and over the input
  for the `when:` side. `finite::analyze_with_states` generalises: its held side becomes an
  assignment over the guarded entity fields instead of one lifecycle state.
- **Closed domains are what `finite.rs` admits today:** `==`/`!=` and membership
  (`any_of`/`none_of`) against text literals over non-optional, non-collection enum paths.
  Booleans are not among them — a `true` literal parses as `FactValue::Bool`
  (`crates/specify/ess-primitives/src/facts.rs`) and the analyzer declines it — and this page does
  not claim them; admitting a two-variant domain is a separate, small change to `finite.rs`.
- **Open domains** — `weight_kg > 20` — make the prover decline, and the command then needs a
  genuine default, as an open input guard does today. Rule 1's default is `dispatched`.
- **Caps** stay at 64 joint assignments and 128 guard nodes.
- **Diagnostics** are the partition's existing ones, at the existing sites:
  `non_exhaustive_branches` at `<command>.outcomes` ("declare a genuine default"),
  `conflicting_declaration` at `<command>.outcomes` naming the assignment and the branches it
  selects, and the checker's `type_mismatch`, `undeclared_reference` and `unobservable_fact` at
  `<command>.outcomes.<name>.when_subject`.

### Conformance: arranging a row to a goal

The strategy in `crates/verify/ess-conformance/src/synthesize/subject_fact.rs` arranges the
subject at its initial state, advances it through `moves:`/`updates:` drivers with those drivers'
own witness inputs, reads one enum fact as the text literal the arrangement settled (`fact`),
compares it as a string (`reach_fact`, `held == equals`) and keys its search on
`(lifecycle state, that text)`. None of that reaches `weight_kg: 21`. The measured failure above is
this: with `sets: {service: input.service}` the value the row holds is whatever input the create
witness chose, and the search has no way to choose differently. The strategy is rewritten in four
parts, and the `{field, equals}` form runs on it as a one-leaf predicate:

1. **Goal-directed input choice through `sets:` mappings.** The search carries a goal — an
   assignment over the guarded fields — and, for each driver whose `sets:` maps a guarded field
   from an input field (`ResolvedPayloadValue::InputField`, the case `settled` in `synthesize.rs`
   already records), chooses that driver's input so the mapped field carries the goal value,
   subject to the driver's own guards still selecting its outcome. A literal `sets:` is a fixed
   point the search takes as it is. A field written through a declared conversion, or mapped from
   nothing the search can set, is not arrangeable: the branch is refused with `ESS-SYNTH-001`
   naming the field and the reason, as `subject_fact.rs` refuses today.
2. **Goal values from the guard's own literals**, by the rule `alternatives` in `witness.rs`
   already uses for input search: a number compared with `n` is tried at `n`, `n + 1`, `n - 1`,
   `0` and `-1` (an `Integer` only at integral values); an enum at each variant; a `Timestamp` at
   the instant written and a second either side. The first assignment that satisfies the branch's
   predicate — or, for the default, refutes every guarded sibling's — is the goal.
3. **A typed fact source over the arranged row.** The `settled` map (`Determined { value,
   type_ref }`) implements the evaluator's `FactSource`: an `Integer` as a number, an enum as text,
   a `Boolean` as a Boolean, a `Timestamp` as text compared by instant, which `evaluate_compare`
   does for declared timestamps already. This replaces the text-only `fact` and the string
   comparison in `reach_fact`, and it is the same evaluator validation and the runner use.
4. **Visited key = branch-selection vector.** The search node is the lifecycle state crossed with
   the truth (`True`, `False`, `Unknown`) of every guarded branch's predicate over the row — finite
   where the field values are not, and exactly the distinction that matters: two arrangements that
   decide every branch alike are one node. The cap of 64 nodes stays. As implemented the node is
   refined by how each *leaf* of those predicates decides and whether the row sits on the leaf's
   own literal, which is still finite and keeps a boundary row from being merged into a far one.
   Where several rows at one depth reach the branch, the one satisfying the most leaves, then
   sitting on the most literals, is taken — which is what makes the default's witness `Express`
   at `20` rather than the plain `Standard` at `1`.

**Observation.** `observe` in `subject_fact.rs` already requires an immediate, unfiltered,
parameterless view exposing the identity, `state` and the one guarded field, and asserts them
before the command runs. That requirement widens to every guarded field; a specification without
such a view gets the existing typed refusal.

**The witness for `refused-overweight`**, in the suite's own step vocabulary:

1. `execute_command` `Create` with `service: Express, weight_kg: 21` — chosen through the
   `sets:` mappings; `21` is the guard's `20` plus one.
2. `expect_outcome` `Create/created`; `capture_instance` `parcel` from `Created.parcel_id`.
3. `query_view` + `expect_view` `Parcels` contains `{parcel_id: parcel, state: Created,
   service: Express, weight_kg: 21}` — the arranged facts observed, not assumed.
4. `execute_command` `Dispatch` with `parcel_id: parcel`.
5. `expect_outcome` `Dispatch/refused-overweight`; `expect_error` `ExpressOverweight`.
6. `expect_no_event` for `Dispatched` and every other event.
7. `query_view` + `expect_view` `Parcels` contains the same row as step 3 — a refusal changes
   nothing.

**The witness for `dispatched`** is the same with a row that refutes the sibling — such as
`Express` at `20`, the boundary — then `expect_outcome dispatched`, `expect_no_error`, `expect_event
Dispatched`, and the view holding `state: Dispatched`. An implementation that ignores the stored
weight fails step 5 of the first witness; one that refuses every Express parcel fails the second.

**The witness for an absent subject:** `execute_command` `Dispatch` with an identity no row
carries, then `expect_no_event` for every event and `expect_view` `Parcels` *excludes* that
identity (`ViewExpectation::Excludes` exists in `scenario.rs`). It asserts nothing about the error,
because the specification declares no absent-subject outcome, and a suite that required
`ExpressOverweight` there — today's — would be inventing one. It opens the scenario of every branch
that names no subject of its own — `refused-overweight` here — before that scenario's arrangement,
so it needs no scenario id and no new vocabulary.

**Transition and invariant scenarios** that route through a guarded branch reuse the same
arrangement; a `dispatch` transition witness arranges a row the guard does not refuse.

### One strategy or two

Today synthesis has two disjoint strategies — `subject_fact::prepare` for a fact,
`prepare_state_input` for a lifecycle state (`run` in `synthesize.rs` picks one) — and validation
refuses a command that would need both. **Two remain.** The fact strategy is the one rewritten
above and serves both `when_subject` shapes; the lifecycle strategy and its complete finite
partition (`finite::StateGuard`, every state × every closed input) are untouched; the refusal on
mixing them in one command stands. Folding the two would either give the lifecycle guards the open
partition of the field predicate, losing the completeness proof they have, or bound field
predicates to closed domains, losing rule 1. A command that needs both today has `wrong_state:`
beside `when_subject` for the lifecycle complement, which the ess/6 fixture
(`crates/verify/ess-conformance/tests/fixtures/subject-history.yaml`) already does. Admitting
`state` as a predicate leaf is a later milestone, if a model needs it.

### Projections

The projections render the `{field, equals}` form as prose in docs
(`crates/generate/ess-gen/src/docs.rs`) and with `Debug` formatting of the input predicate in
`OpenAPI`, the native plan and the semantic diff (`openapi.rs` beside it,
`crates/generate/ess-synth/src/plan.rs`, `crates/verify/ess-diff/src/diff.rs`). The predicate
form renders as `SubjectState` does: the
predicate through `Display`, in all four, so a reader of the published contract sees `service ==
Express and weight_kg > 20` and not a Rust structure. The guarded fields become part of the
command's documented contract that way, as the state condition is. The HTTP status is unchanged:
`http.rs` already answers `409` for a refusal decided by the subject (`SubjectField` sits with
`WrongState` and `SubjectState` under `CONFLICT`). Entity Runtime lowers `SubjectField` to
`$fields.<field> == <variant>` and entity predicates through `PathRewrite::Entity` already; the
predicate form lowers through the same rewrite.

### Persisted formats and compatibility

Two precedents: `when_subject: {field, equals}` took ess/6
(`crates/specify/ess-domain/src/primitive_admission.rs`: "subject facts and preservation require
specification format ess/6"); `when_state_changes:` took
no bump, because it was a new key on `RawOutcome`, which denies unknown fields, so an older build
refused it by name (`state-change-outcome-guards.md`). This construct is neither: it is a new shape
of an existing key's value, and an older reader fails it with `unknown field predicate` — a parse
error with no version hint. `AGENTS.md` requires a new format version when meaning changes, and the
meaning of `when_subject` does. **The predicate form requires `ess/9`**, gated beside the ess/6 and
ess/7 gates in `primitive_admission.rs` with `unsupported_format_version`: "subject predicates
require specification format ess/9". `{field, equals}` documents keep ess/6 and their bytes.

- The domain gains `OutcomeCondition::SubjectPredicate` and the IR
  `ResolvedCondition::SubjectPredicate { predicate, input }` beside `SubjectField`, which is
  unchanged so ess/6 models keep their IR bytes. Every exhaustive match over the enum gains an arm
  — there is no wildcard arm by design, and the compile errors are the drift alarm. Its
  `test_strategy` stays `observe_subject_fact`: what the scenario does — arrange, observe through a
  view, run — is the same.
- `RawSpecFile` changes, so `schemas/generated/ess.schema.json` is stale until `cargo xtask schema`
  runs, and `projection-check` is the only thing that sees it.
- The suite needs no new vocabulary: every step above exists. Old-reader refusal of `ess/9`, byte
  preservation of ess/6 and ess/7 models, and the arranged witness against a guard-ignoring mutation
  are the deciding checks. The test in `crates/specify/ess-domain/src/spec.rs` that uses `ess/8` as
  a header this build cannot read moves to `ess/10`, past both `ess/9` and the `ess/8` that
  #95 takes (`docs/design/string-predicate-operators.md`).

### What was rejected

**The inverted default** (design (b) alone): stated above.

**`state` in the predicate namespace**, subsuming `when_subject_state:`. It would give two spellings
of one condition, need `validate_move`'s "cannot take move `X`, which does not start there" applied
to predicate leaves, and cannot express `when_state_changes:`, whose whole content is a derivation
from the move. Rule 1 does not need it.

**Widening `SubjectField` in place.** It changes the IR of every ess/6 model that uses it, against
the byte-preservation rule, for no gain over a sibling variant.

## Rule 2: out of scope, and the one step that is in

A view-level non-overlap constraint was considered:

```yaml
views:
  - name: booking.room.BookingsByRoom
    source: booking.room.Booking
    constraints:
      - never_overlapping: [starts_at, ends_at]
        per: room_id
```

It is **not proposed**:

- **Not a guard.** It is an invariant over every pair of rows, not a predicate over one command. A
  specification would also have to say *which* command's outcome answers a violation, which the
  constraint above does not.
- **Conformance cannot witness it.** Synthesis would need to arrange a *set* of rows, choose an
  input that overlaps one on a boundary (touching, containment, identical, another room) and prove
  the refusal. A refused booking shows one overlap was caught; "never" needs an adversarial search
  over pairs, and concurrency — two overlapping requests racing — which the single-client runner
  does not model.
- **Ordering across records.** Overlap is `a.starts_at < b.ends_at and b.starts_at < a.ends_at`.
  Validation orders a `Timestamp` against an RFC 3339 literal, and refuses a bare word naming a
  field on the right-hand side — beyond10x/ess#74, `text_literal` in `expression.rs`, which points
  two fields of one struct at `window.starts_at < window.ends_at`. An overlap witness needs an
  ordering between an arranged row's field and a new input's, which no candidate search does.
- **Consistency.** Views already declare `consistency: read_your_writes | eventual`
  (`crates/specify/ess-domain/src/view.rs`), but that decides how a generated scenario *asserts*
  the view (`assertion_style`), not what a command reads at selection time. Nothing today says
  that a command's selection reads a view, and a constraint enforced at command time over an
  `eventual` view would be a promise nobody can keep.

**The cheapest in-scope step is a duplicate-identity outcome on `creates:`.** A create may already
publish an identity the caller supplied — `payload: <Event>: booking_id: input.booking_id`
validates and is witnessed on 0.32.0 — and what an implementation answers to a second create with
the same identity is unspecified and unwitnessed: the witness derives a fresh identity that
"identifies nothing", and no outcome exists to name the collision. Proposed, in the shape of
`wrong_state:`:

```yaml
- name: already-booked
  already_exists: true
  error: booking.room.SlotTaken
```

Admitted only beside a `creates:` sibling whose published identity is taken from input; refused
beside `{generated: true}`, where nothing can collide. Its witness is one duplicate: create, capture
the identity, create again with it, `expect_error`, `expect_no_event`, and `expect_view` still
containing the first row unchanged. It needs no ordering, no set arrangement and no composite
identity — `EntitySpec::identity` is one `Field` — and it reuses the subject-less refusal shape
this page and `wrong_state:` already have. For rule 2 it means: with the booking identity being the
slot key the caller composes, a double booking of a fixed slot is a declared, witnessed refusal,
without pre-creating every slot as a `Free` row. Range overlap, composite keys and the concurrency
argument stay out of scope, and `UNMAPPED:` remains the honest marker for them.
