# The first report of a state, and every later one

## Behaviour and authority

A `when:` is a predicate over the command's **input**, so a push that re-reports a state its
subject already holds is indistinguishable from the push that first put it there. An adopter
reducer rebuilds every row from every push without comparing what the row already holds: a
conference member resting in `Added` is re-reported as `added` by the next push about **any** member
of that conference, so one guard — `when: state == Added` — answers the join and, a second later, a
hold or an unhold that changed fields only.

Three branches of that one model carry the defect, not one:

| guard | branch | the move it draws |
|---|---|---|
| `state == Added` | `joined` | `join`, and then every repeat |
| `state == Removing` | `removing` | `removing`, and then every repeat while the leg tears down |
| `state == Finishing` | `finishing` | `finish`, on the conference rather than the member |

It is measurable rather than argued: an adopter's authored scenario for a member held and released
sends `state: Added, on_hold: false` twice, expects `joined` and then `refreshed`, and fails on the
second — `took joined … says refreshed`. Five sites across two of that model's domains carry the
same marker.

This page adds the one word that separates the two pushes. It adds no expression language, no
cross-entity read and no field comparison.

## Declaration

An outcome may add `when_state_changes: true` or `false` beside its ordinary input `when:`. The two
conditions are conjunctive; omitting `when:` means any admitted input under the answer given.

```yaml
- name: joined
  when: state == Added
  when_state_changes: true          # the first report: the subject was not already Added
  moves: conf.room.Member.join
  instance: member_id
- name: refreshed                   # every later one falls to the default, fields only
  updates: conf.room.Member
  instance: member_id
```

**It names no state, and that is the whole construct.** `moves:` already declares the state the
branch arrives at and the states it may start from, so the held states this condition admits are
**that transition's own `from` set partitioned by whether each state is the one it arrives at**:
`true` takes the side that differs from the arrival state, `false` takes the arrival state itself.
Computed, never authored — the rule
[`WrongState`](../../crates/specify/ess-domain/src/command.rs) already follows, and for the reason
it gives: "a rule restating an absence is a second copy of one fact, and nothing keeps the copy
honest."

`Option<bool>` and not `bool`: `false` is a condition, not an absence. `wrong_state: false` is the
same document as leaving that key out, and `when_state_changes: false` is not — it selects the
restatement. This is the distinction `refuses:` draws on the same struct.

## What it refuses

| written | refusal |
|---|---|
| beside `when_subject_state:` | `conflicting_declaration` — a branch reads the held state one way |
| beside `external:` or `wrong_state:` | `conflicting_declaration` — one condition authority per branch |
| on `updates:`, `creates:`, or no subject | `conflicting_declaration` — nothing else arrives at a state to compare against |
| `true` where `from` ⊆ `{to}` | `unreachable_branch` — the move starts only where it arrives, so it never changes a state |
| `false` where `to` ∉ `from` | `unreachable_branch` — the move does not start where it arrives, so it never restates one |
| a state name, a list, a quoted Boolean, a misspelt key | the reader's own refusal; the key carries an answer, not a state |

The last two `unreachable_branch` rows are the ones worth having. A pinned `join` declared
`from: [Pending]` only, and `when_state_changes: false` on it admits nothing — the refusal's hint
says where the restatement actually goes: "a push that restates the state already held is answered
by the default branch beside this one rather than by this move." Authors reach for the move first,
and the empty side is what tells them not to.

## Shared finite proof

Nothing new. The condition joins the existing held-state family, so it routes through the same
`subject_state` shape check and the same joint partition: every declared held state crossed with
every closed input assignment must select exactly one branch, under one cap of 64 joint assignments
and 128 guard nodes. Its contribution is a **set** of admitted states rather than one state, which
is the only change the prover needed — `finite::StateGuard` now carries `states: Option<BTreeSet<…>>`
where it carried `state: Option<&StateName>`, and a literal guard contributes a set of one. That
generalisation removes a special case rather than adding one: the prover asks *does this branch
admit this state* and both conditions answer it the same way.

Measured on the adopter's lifecycle, where every transition runs from every state — 5 held states ×
5 input variants = 25 pairs, all covered, none overlapping. Held `Pending` with input `Added`
selects `joined`; held `Added` with input `Added` selects `refreshed`, which is what the failing
scenario asked for.

## What a consumer reads

`ResolvedCondition::StateChange` carries `changes`, the `predicate`, and `states` — the partition
already performed:

```json
{"kind": "state_change", "changes": true, "states": ["Gone", "Pending"], "predicate": "state == Added"}
```

`states` is resolved rather than derived by the reader, which is this module's own rule: *a question
a projection will ask must have an answer in here.* A consumer arranging a scenario needs the held
states the branch admits, and re-deriving them means re-implementing the partition beside every
generator that asks. It is never empty — an empty side is refused above.

`test_strategy` is `construct_input_in_state`, the existing strategy, because what a scenario has to
do is unchanged: establish a held state, then choose an input that decides every competing branch
against that same state.

A new variant owns every exhaustive match over the enum, and there is no wildcard arm on
`ResolvedCondition` anywhere by design — the docs projection says so in its own module header. The
HTTP status mapping folds it in with the other two held-state conditions (a refusal from one is a
conflict with the subject's state, not a bad request); the docs, `OpenAPI` and plan renderings print
the admitted states rather than the word, so a reader does not partition a `from` set by hand; and
the semantic diff renders **both** halves, because either can move without the other — an author
flipping the answer, and a lifecycle gaining a state.

## Formats and compatibility

Source format stays `ess/4`. The construct is additive and `RawOutcome` denies unknown fields, so a
build older than this one refuses `when_state_changes:` by name without a version bump — the
precedent `{cleared: true}` set at 0.26.0. A document below `ess/4` that reaches for it is refused
with `state-change outcome guards require specification format ess/4`, beside the existing `ess/3`
gate on `when_subject_state:`; the two are gated separately because they arrived in different
formats and an author has to be told which one they reached for.

A model that does not use the construct compiles to the same bytes: the IR variant is new, so
nothing is added to any condition that already existed.

`RawSpecFile` gains a key, so `schemas/generated/ess.schema.json` is stale until
`cargo xtask schema` runs. `projection-check` is the only thing that sees it.

## What was rejected

**A literal `when_subject_state:` guard per branch.** It expresses the same selection for one
transition and is a second copy of that transition's `from` set: re-derived per branch, silently
wrong when a transition gains a state, and unwritable at all where `from` holds more than one state
that is not the arrival state — which is the adopter's own lifecycle, where every move runs from
every state. The adopter's model records `when_subject_state` as tried; this repository's own
`models/toolchain` records it as tried and reverted, for a different reason.

**Inferring it from the lifecycle.** `join` is legal `from: [Pending]` in the pinned model, so ess
*could* have read "input says `Added`, the row already holds `Added`, therefore not this branch"
without any new word. Refused: it changes what an existing specification means, silently, and
"imports never guess" is the same rule pointed at selection. The author says which of the two
readings their reducer has.

**A reading over an input field** — "the state I am moving to is the state the input reports". It
needs the field nominated and an input enum compared against a lifecycle state, which is the
cross-namespace field comparison the guard vocabulary excludes on purpose. Comparing against the
transition's own `to` needs nothing nominated and nothing crossed.

**A narrowed second `wrong_state:` branch.** The shape leaves room for one, and the adopter tried
it. It is the wrong construct: `wrong_state:` answers the complement of *every* move this command
makes, as one branch with one error, and what is wanted here is a distinction *within* one move's
own `from` set.

**Admitting the key on `updates:`.** There is no arrival state, so there is nothing to compare, and
reading it as "any state" would make the key decide nothing. The branch that answers a restatement
is the default beside the move, or a self-transition the lifecycle declares.

**Carrying it outside the condition**, as a field on the resolved outcome. It would not have broken
any exhaustive match, which is exactly why it was refused: the semantic diff would have rendered a
change to the construct as no change at all, and the docs would have published a guard weaker than
the one declared. The compile errors are the drift alarm this repository designed for.
