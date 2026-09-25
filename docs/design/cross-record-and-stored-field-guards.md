# Guards over stored fields and constraints across records

Status: proposed (beyond10x/ess#75). No source, IR or format change is made by this page.

## Behavior and authority

A trial specified two real services from their source. Two rules had no ESS form, and both were
written down as `UNMAPPED:`:

1. "Express parcels over 20 kg are refused at dispatch."
2. "A room is never double-booked."

In rule 1 the weight is recorded when the parcel is created. The dispatch command's input carries
the parcel identity and nothing about its weight, and `when:` is a predicate over the command's
input only. The refusal depends on a field of the row the command addresses.

In rule 2 no single command input decides the outcome. Whether a booking is admitted depends on
every other booking of the same room whose time range overlaps the new one. The rule is over a set
of records, not over one.

## What exists today

| Construct | What it can say | Why it does not state the rule |
|---|---|---|
| `when:` | A predicate over the command's own input fields. | The weight is not input at dispatch. Copying it into the dispatch input makes the caller the authority on a stored fact, and the guard then checks what the caller says instead of what was recorded. |
| `when_subject_state:` (`subject-state-outcome-guards.md`) | Selects an outcome by the addressed entity's held **lifecycle state**, read immediately before selection. | It reads the state name only. That page explicitly declares no "cross-entity read, view query, field comparison". |
| `when_subject: {field, equals}` (ess/6, `observed-subject-history.md`) | Selects an outcome by equality of one declared **enum** field of the existing subject against one variant, conjunctive with an optional input `when:`. | Equality on an enum only. `weight_kg > 20` is an ordering over a number, and "Express and over 20 kg" is a conjunction of two stored fields. |
| `wrong_state:` | Answers in the complement of a move's `from` set. | A lifecycle state, not a field value. |
| Per-slot lifecycle workaround | Rule 2 can be approximated by modelling each bookable unit (room × fixed slot) as an entity with `Free → Booked`. A second booking of the same slot becomes a `wrong_state` refusal. | Only exact for fixed, pre-enumerated slots. Free time ranges overlap without being equal, so the entity identity cannot be the slot. It also changes the model the service actually has, which is one booking per request. |
| `UNMAPPED:` | Records honestly that the rule is outside the model. | Nothing is checked. The rule is not in the specification, generated artifacts or conformance. |

## Proposal for rule 1: a predicate over the subject's stored fields

This extends the existing `when_subject` from one enum equality to a predicate. It keeps the same
subject, the same read point and the same restrictions as `when_subject` and `when_subject_state`,
and widens what may be asked of the subject's declared fields.

```yaml
outcomes:
  - name: refused-overweight
    moves: shipping.parcel.Parcel.refuse
    instance: parcel_id
    when_subject:
      predicate:
        all:
          - service == Express
          - weight_kg > 20
    error: shipping.parcel.ExpressOverweight

  - name: dispatched
    moves: shipping.parcel.Parcel.dispatch
    instance: parcel_id
```

`when_subject` stays a closed mapping: either today's `{field, equals}` or the new `{predicate}`,
never both, so every ess/6 document keeps its meaning and bytes.

- **Subject.** The same as `when_subject`: the existing entity and identity already named by
  the outcome's `moves:` or `updates:` and `instance:`. Refused on `creates:`, on an outcome without
  a subject, and beside `external:` or `wrong_state:`. Every input-selectable outcome of a guarded
  command names the same entity and identity field.
- **Namespace.** The predicate reads only the subject's declared fields, never input. Input stays in
  `when:`, and the two are conjunctive, as they already are for `when_subject`. A separate key keeps
  an input field named like an entity field from acquiring a second meaning.
- **Read point.** The row immediately before command selection. An absent subject satisfies no
  outcome, as today.
- **Validation.** The existing expression checker, over the entity's fields instead of the input's,
  and the existing finite guard checker over closed domains (enums, Booleans, lifecycle states)
  with the 64-assignment and 128-node caps. An open domain such as `weight_kg > 20` needs a genuine
  default, which is how open input guards already work.
- **Conformance.** The `when_subject` and subject-state witness strategies already establish a held
  subject through reachable commands or typed entity setup before running the command. The same
  arrangement establishes a stored field value: create the parcel with `weight_kg: 21` and
  `service: Express`, dispatch, observe the refusal; create one at 20 kg, dispatch, observe
  `dispatched`. A mutant that ignores the stored weight fails the first case. Arrangement search
  must then treat the guarded fields as part of its visited key, as it already does for the bounded
  enum facts. Observation needs an immediate unfiltered view exposing the identity and the guarded
  fields.
- **Formats.** A new condition shape in the domain and resolved IR, and a new source major when
  it occurs. Unchanged models keep their bytes.

## Rule 2: out of scope for now

A view-level uniqueness or non-overlap constraint was considered:

```yaml
views:
  - name: booking.room.BookingsByRoom
    source: booking.room.Booking
    constraints:
      - never_overlapping: [starts_at, ends_at]
        per: room_id
```

It is **not proposed**, for these reasons:

- **Not a guard.** It is an invariant over every pair of rows, not a predicate over one command. A
  specification would also have to say *which* command's outcome answers a violation, which the
  constraint above does not.
- **Conformance cannot witness it with the existing strategy.** Synthesize would need to arrange a
  *set* of rows (an existing booking), choose an input that overlaps it on a boundary (touching
  ranges, containment, identical ranges, a different room), and prove the refusal. A refused
  booking only shows that one overlap was caught. Proving "never" needs an adversarial search over
  pairs, and concurrency: two overlapping requests racing, which the single-client runner does not
  model.
- **Ordering over Timestamps.** Overlap is `a.starts_at < b.ends_at && b.starts_at < a.ends_at`.
  Synthesize orders two Timestamps within one input since beyond10x/ess#74; an overlap witness
  needs that ordering between an arranged row and a new input, which the candidate search does
  not do.
- **Consistency.** A constraint over an `eventual` view cannot be enforced at command time. The
  model would need to say the constraint is read from a `read_your_writes` or stronger source, a
  distinction views do not yet carry for enforcement.

Until then `UNMAPPED:` remains the honest form for rule 2, and the per-slot lifecycle is the
approximation to use when slots are genuinely fixed.

## Open questions

1. Should the predicate form of `when_subject` also subsume `when_subject_state` (with
   `state == Bridged` as one of its predicates)? Subsuming is one construct; keeping both leaves
   `ess/3` models' bytes unchanged.
2. May `when_subject` read `Optional` fields, and what does `None` compare as?
3. Does a stored-field guard make a field part of the command's documented contract in the
   generated HTTP and OpenAPI surfaces, as the state condition is?
4. For rule 2: is the first useful step a declared **uniqueness** constraint (one active booking per
   `(room_id, slot)`) rather than range overlap, because equality needs no ordering and its
   witness is a single duplicate?
