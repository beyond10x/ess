# Unread optional inputs are sent absent

Issue: <https://github.com/beyond10x/ess/issues/546>. Story:
`story:synthesis-leaves-unread-optional-inputs-absent`. Code: `crates/verify/ess-conformance/src/witness.rs`
(rules 1 and 2 of its module documentation). Test: `crates/verify/ess-conformance/tests/unread_optional_inputs.rs`.

## What was wrong

Rule 1 of the witness strategy filled every field of a command's input, optionals included, "so a
guard reading one is decided rather than `Unknown`". The reason covers an optional a guard reads. It
does not cover one nothing reads, and that one was filled too, on every synthesized step, the
arranging steps included.

The value was also wrong. A `Timestamp`, number, `Duration` or `Bytes` witness is taken from the
instance alone, so two such fields of one input carried one literal. An optional deadline beside a
required queue instant was sent equal to it: every synthesized item claimed to expire at the moment
it was queued. An implementation that acts on the deadline (it expires the item) then fails
scenarios that never mention expiry.

The minimal reproduction is the model in the test file: `Submit` takes `queued_at: Timestamp` and
`expires_at: Optional<Timestamp>` and stores both, `Withdraw` moves the item, and nothing reads the
deadline. Synthesized with 0.59.0, all four `Submit` steps carry
`expires_at == queued_at == 2020-01-01T00:00:00Z`.

## The rule

**An `Optional` member of a command input is left out of the input unless the command can read its
value.** "Member" is a top-level input or a member of a struct inside one, at any depth. It is left
out, never sent as `null`, except where a presence policy says otherwise (below). It is sent where any
of these reads it, the optional itself or anything inside it:

| reader | where it is found |
|---|---|
| a guard the witness search is asked to decide | the search's own guards, with quantifier bodies rebound (rule 3) |
| any input guard of the command | `when:`, and the input part of every subject, related and row-set condition |
| an invariant over the input | the struct invariants of the input's types; every invariant of an entity that reads a stored field a branch copies the input into, also one that reads other fields beside it (`invariant_reads`): it is checked after every later step |
| an effect that is not a plain copy | `{input: x, else: …}` and its input fallback; a `Supplied` instance; an input `via`; the filter of `instances:`, `affects:` and a row selection; `each:` over an input list; a copy through a declared conversion |
| an input `example:` | the author's own value for that input |
| a later step reading the stored copy | translated onto this command's input by the arrangement that needs it (`subject_fact`, `related_guard`, view filters), so it arrives as a guard of that search |

A plain copy (`input.x` into a stored field, an event payload field or an error payload field) is not
a read for a step that only arranges. An absent input copied into an optional field is an absent
field, which is what the suite then expects; a target that stores or publishes a value nobody sent
fails.

The one exception is **the branch under test**. The scenario of a branch — its `/outcome/` id, and
the `/transition/` id of a move it makes — observes that branch's copies, so there the copied
optional is sent present, at a value apart from every other leaf, and a target that drops or mangles
the copy fails. Every arranging step and every other scenario leaves it absent. Synthesis marks
the branch with `witness::under_test(ir, command, outcome)` around `exercise_as`,
`outcome_scenario_noting` and each lifecycle driver; within that scope a search for that command
reads the paths the branch copies, and a search for any other command is unaffected. The scope is
part of the witness memo's key: only the copied paths whose input field reaches an `Optional` are
recorded, so a branch copying no optional shares its answers with the arranging steps.

A path a filter reads is matched both as written and with a leading `input` segment removed. That
over-approximates: an entity field spelled like an input keeps the input filled, which is the old
behaviour, never a new absence.

### How it is built

The existing presence-policy pairing (beyond10x/ess#139, `presence_inputs`) already sends a chosen
set of optional members absent first: every candidate is built once with them left out and once as
written, the absent one first. The unread optional members join that set. So:

- every scenario whose branch the absent candidate reaches sends the input absent;
- the filled candidate stays behind it, so a caller that needs a value from it (one that reads the
  field off a further instance) still finds one, and a branch the absent candidate cannot reach is
  still reached;
- a presence-policy member keeps the spelling that policy gives it (`null` under `null_when_absent`).

### Present, and apart

Where an optional member is sent, a primitive whose witness comes from the instance alone takes a
value no required leaf of the same input takes, and no other optional leaf of it either. Each such
leaf, numbered `k` from zero in the order the base witness first reaches it:

| type | required leaf at instance `n` | optional leaf `k` at instance `n` |
|---|---|---|
| `Timestamp` | day `1 + n mod 28`, `00:00:00Z` | the same day, `(1 + k) × 61` minutes past midnight, modulo a day (`01:01`, `02:02`, …) |
| `Integer`, `Decimal` | `1 + n` | `1 + n + 100 × (1 + k)` |
| `Duration` | `PT{1 + n}S` | `PT{1 + n + 61 × (1 + k)}S` |
| `Bytes` | byte `n` | byte `n + 1 + k`, wrapping |

61 is coprime with the 1440 minutes of a day, so the first 1439 optional `Timestamp` leaves of one
input take distinct times, none of them midnight. A `Boolean` and an enum have too few values to keep
apart and are unchanged. `String`, `Uuid` and `Json` were already built from the path (rule 2).

A guard or invariant that relates two leaves still decides the values: a ladder tries each side at the
other's base (rule 6, `equality_copies`, `offset_copies`), so `expires_at == queued_at` is still
witnessed, at an equal pair.

## What changes for an existing suite

Only inputs that carry an `Optional` member. A suite whose commands declare none keeps its bytes.
Among the committed ones (`suites/generated/`), only `gatepass` has one: `AdmitVisitor.badge` is a
required struct whose `printed_at: Optional<Timestamp>` is copied into the stored badge and the
payload and read by nothing. It is now absent wherever a visit is only arranged: 18 occurrences,
inputs and the stored and published badge they lead to. The 7 that remain are in the 3 scenarios
observing `AdmitVisitor/admitted` (its outcome, the `arrive` transition, and the denied grant run
of it), moved from the base instant `2020-01-01T00:00:00Z` to
`2020-01-01T01:01:00Z`, apart from every other timestamp of the input.

## What this does not do

- **Event payloads and delivery contexts.** `witness::fields` builds a delivered event or a delivery
  context the same way and still fills every optional. Leaving one out there needs the same reading
  of binding conditions and mappings; it is not part of this change.
- **Other families' own branch.** Only the outcome, transition and disclosure scenarios (through
  `exercise_as`) mark a branch under test. State refusals, unknown instances, absent input,
  existence, set effects and invariant scenarios do not, so an error payload copying an unread
  optional in one of those is sent it absent. An invariant reading the copy still fills it
  (`invariant_reads`).
- **An input-level relation or synthesis hint** (`synthesize: absent`, `offset_from:`), which the
  issue offers as alternatives. No authored surface is added.
