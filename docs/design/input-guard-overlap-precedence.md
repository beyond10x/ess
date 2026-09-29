# Input-guarded refusals take precedence over the accepting branches they overlap

beyond10x/ess#178, and beyond10x/ess#217 for two accepting branches. No source or suite format
change.

## The gap

```yaml
- name: demo.tickets.SetTicketOpen
  input: [{name: ticket_id, type: demo.tickets.TicketId}, {name: open, type: Boolean}]
  outcomes:
    - {name: closed, when: open == false, moves: demo.tickets.Ticket.close, instance: ticket_id}
    - {name: reopened, moves: demo.tickets.Ticket.reopen, instance: ticket_id}
    - {name: id-required, when: ticket_id == "", error: demo.tickets.TicketIdRequired}
```

`{ticket_id: "", open: false}` satisfies both `closed` and `id-required`. `validate` accepts the
command, and it cannot be repaired from the accepting side: `closed` may not read `ticket_id` to
step aside, because an identity is opaque (ESS-COMMAND-003). Nothing said which branch that input
takes, and synthesis never sent it.

## The rule

An **input-guarded refusal** is an outcome with a `when:` over the command's input and an `error:`.
It is taken before any accepting branch whose guard it overlaps. A refusal guard names what the
command cannot act on at all; an accepting guard names which of the things it can do it does.
Checking the first before the second is what every implementation that refuses an empty id before
reading `open` already does.

Where this rule sits among existence, the held state and related rows is
[the precedence order](cross-record-and-stored-field-guards.md#the-precedence-order). The rule
orders refusals ahead of accepting branches, refusals among themselves and accepting guarded
branches by the order they are declared in:

| pair | order |
|---|---|
| input-guarded refusal, accepting branch with an input guard: a plain `when:`, the `when:` beside a `when_subject:`, or an external branch's `when:` | the refusal, before any stored row is read or any provider asked |
| input-guarded refusal, default | not an overlap: the default is what no other guard selects |
| two input-guarded refusals | the first declared whose guard holds; a witness of one refutes the refusals declared before it, and where no input does, synthesis refuses the scenario naming them (beyond10x/ess#227 correction 1, superseding the unordered rule of #209) |
| two accepting guarded branches | the first declared whose guard holds (beyond10x/ess#217, below) |
| an accepting guarded branch and an external branch | the same declaration order: the first declared whose guard holds, an external one where its provider takes it (beyond10x/ess#217, below) |
| a refusal decided by the stored row, the held state or a provider; the wrong-state branch | unchanged |

Without a default, validation's finite coverage proof already refuses every overlap it can decide,
and a guard it cannot decide (a text or a number) requires a default. The rule therefore changes
nothing for a command without a default.

## Why stated precedence and not a refusal

The alternative was for `validate` to refuse the overlap and name both outcomes, as it does for a
closed enum (`closed-enum-outcome-coverage.md`). That needs an overlap analysis over text and number
guards that does not exist, a format version, and leaves the author of #178 with no repair: the
only spelling that removes the overlap is the one ESS-COMMAND-003 refuses.

## What synthesis does

`crates/verify/ess-conformance/src/synthesize.rs`:

1. **An accepting branch's witness refutes every sibling input-guarded refusal**, with or without a
   default (`admits_plain`). Before #178 it refuted its siblings only where the command had no
   default, so beside a default `closed` could be witnessed at a `count` that `too-few: count < 5`
   claims, and a target honouring the precedence failed the `closed` scenario. The witness search
   runs over the branch's own guards first, which is the search there always was, so a witness that
   already refuted every refusal is unchanged; only where none did does it run again over the
   refusals' guards as well (`searched_guards`), which puts their literals on the ladders. The
   boundary witnesses of an accepting branch are decided by the same rule.
2. **Each input-guarded refusal is sent again at every overlap point** (`overlap_inputs`, through
   `boundary_inputs`): for each accepting guarded sibling, the first candidate that satisfies both
   guards and refutes every other input-guarded refusal. The invocation is appended to the
   refusal's own `…/outcome/<refusal>` scenario and requires the refusal and its error. For #178
   that is `{ticket_id: "", open: false}` requiring `id-required`. A refusal that overlaps no
   accepting branch gains nothing, and an overlap point equal to the refusal's own witness is not
   sent twice.
3. **A branch every candidate of which a refusal claims is refused naming it.** Beside a default,
   `zero: count == 0` next to `too-few: count <= 0` has no input that reaches it. Its refusal
   (`ESS-SYNTH-003`) says `count == 0 outside too-few (count <= 0), the input-guarded refusal taken
   first` rather than that a satisfiable guard has no witness. The same record (`Shadow`) is kept
   by the input search (`reach`), the external search (`reach_external`) and the stored-row search
   (`subject_fact::prepare`, over every row it arranged), and each renders it only where every
   candidate its own guards admitted was claimed by a refusal. The search is bounded, so this is a
   statement about the candidates tried, not about every input.

The same rule reaches the other accepting shapes:

| accepting branch | rule 1 | rule 2 |
|---|---|---|
| `when:` beside `when_subject:` | the stored-row search (`subject_fact::selects`) takes the refusal wherever it and the branch are both selected, so the branch's row and input refute it | a refusal sent for an arranged row gets one further row per such sibling, which the sibling's stored guard admits, sent an input both input guards admit (`subject_fact::overlaps`); the row is observed unchanged afterwards |
| external, with a `when:` | its witness refutes every input-guarded refusal (`reach_external`) | the refusal is sent the overlap with no provider configured (`overlap_inputs`) |

A refusal over the identity field of the subject a `when_subject:` command reads is not sent for an
arranged row (`subject_fact::routes`). Sending the row replaces the empty identity the guard admits
with the row's own, so the scenario would require the refusal for an input it does not claim. It is
sent as a plain invocation, with the value its guard admits, and its overlap points follow rule 2
without a row: no row is named by an empty identity.

No scenario id is added, so no suite format changes. The committed suites under
`suites/generated/` are byte-identical: none of their models declares such an overlap.

## Two accepting guarded branches (beyond10x/ess#217)

```yaml
outcomes:
  - {name: small, when: amount < 100, creates: demo.Order}
  - {name: flagged, when: amount > 50, creates: demo.Order}
  - {name: refused, error: demo.NotPlaced}
```

`amount: 75` satisfies both, and `validate` accepts the command. Among the accepting guarded
branches of one command, **the first declared whose guard holds answers**: `75` takes `small`.
Every input-guarded refusal still comes first. The rule is the one Entity Runtime already applied,
since its lowering keeps accepting guarded branches in source order. It is stated rather than
refused because a refusal would break specifications that validate today.

Synthesis holds a target to it for each pair of accepting `when:` branches, in the same two ways
as for a refusal:

1. **A later branch's witness refutes every accepting `when:` branch declared before it**, beside a
   default (`admits_plain`, `earlier_accepting`). `flagged` is witnessed at an amount `small`
   does not claim, so a target honouring the precedence is never asked to take `flagged` at `75`.
   The search runs over the branch's own guards first, then with the refusals' guards, and only
   then with the earlier branches' guards (`searched_guards`), so a witness an earlier search found
   is unchanged. Boundary witnesses are decided by the same rule.
2. **The first-declared branch is sent again in each overlap** (`overlap_inputs`, through
   `boundary_inputs`): for each later accepting `when:` sibling, the first candidate that satisfies
   both guards and refutes every input-guarded refusal and every accepting branch declared earlier.
   The invocation is appended to the branch's own `…/outcome/<branch>` scenario and requires it,
   so a target answering `flagged` there fails. An overlap point equal to a witness already sent is
   not sent twice.
3. **A branch is refused naming an earlier one when every candidate tried for it is claimed**
   (`Shadow`): `tiny: amount < 10` after `small` reads `amount < 10 outside small (amount < 100),
   the accepting branch declared first` under `ESS-SYNTH-003`. The search is bounded, so the
   refusal says what the candidates showed, not that no input reaches the branch.

Without a default, validation's coverage proof refuses every overlap it can decide and a number or
text guard requires a default, so these witnesses change nothing there.

**External branches take the same order.** An `external:` branch, with or without a `when:`, sits
among the accepting guarded branches in declaration order: an accepting `when:` branch declared
before it answers an input its guard claims, whatever the provider says, and the provider decides
only an input no earlier accepting guard claims. That is Entity Runtime's order, which keeps both in
one category by source position (below). So the external branch's witness (`reach_external`)
refutes every accepting `when:` branch declared before it, searched after the refusals, and a
binding that would force such a branch without observing the input it maps is refused
(`PrecededExternalEligibility`), as a guarded one already was. An external branch declared first is
taken first wherever its provider takes it.

### Overlaps no ladder value reaches

Both overlap searches, and every witness search (`reach`, `reach_in_state`, `reach_external`), run a
second pass where the first finds nothing: each `Decimal` leaf is also tried at the exact midpoint
of every two adjacent literals the guards compare it with (`witness::candidates_between`). The
ladder tries each literal and one either side, so `amount > 11` beside `amount < 12` held nothing it
tried; `11.5` lies in every interval two literals bound. A witness the first pass finds is the one
it always was, so no committed suite changes.

### Nothing unsent is silent

After the suite is built, `unwitnessed_overlaps` reads it back: for each overlap, the scenario of the
branch taken first must send an input in it and require that branch. Where it does not, the
synthesis carries a `Note::UnwitnessedOverlap` naming the scenario and both branches, with why:

| gap | when |
|---|---|
| `Unreached` | no candidate lies in the overlap, and the candidates do not cover every region the literals divide the input into |
| `Unsent` | a candidate lies in it, and the scenario is arranged by a search that does not send it: a stored row (`when_subject:`), a replay, a preserved subject, or a held state another branch also claims |

An overlap is not noted where the candidates are exhaustive (`witness::exhausts`: every guard is
built from `all`, `any`, `not` and comparisons of one input leaf with a literal, over numbers, text
equality, Booleans and enums, and the regions fit within the candidate bound) and none lies in both:
the overlap is then shown empty. The check reads the finished suite rather than the generators, so a
path that forgets to send an overlap is noted whichever path it is.

### A command whose branches read the held state

In a command with a `SubjectState` or `StateChange` sibling, a plain accepting branch is witnessed in
a held state by `reach_in_state`, which requires it to be the only branch selected. It is now also
sent each overlap in the state its scenario arranged (`overlap_inputs_in_state`, through
`boundaries`), at an input that refutes every state-guarded branch admitting that state. Where a
state-guarded branch with no guard beyond the state admits it, no such input exists and the overlap
is noted `Unsent`. Boundary witnesses are still not sent there.

The interpreter (`crates/verify/ess-conformance/src/interpret/execute.rs`) returned every branch
whose `when:` held, so it refused the overlap as open. It now reads the branches in the declared
order and stops at the first that answers: every input-guarded refusal first, where every one that
holds stays open since the model orders none of them; then the accepting `when:` branches and the
external branches in declaration order, the first whose guard holds answering — an external one
where the provider takes it (`Forced`), never while it is withheld, and while it is `Open` as one
possible answer beside whatever the declarations after it select; then the default. A guard read
after the answer is never evaluated, so an optional it reads may be absent; a guard read before it
that the input cannot decide is `Undecidable`.

## Entity Runtime

Entity Runtime selects the first branch whose guard holds. The lowering in
`crates/generate/ess-entity-runtime/src/lib.rs` ordered guarded branches by source position, so
`closed` written before `id-required` won the overlap. It now orders every input-guarded refusal
first, then the other guarded branches, the default, and the wrong-state branch. The sort is stable
on source position, so among accepting guarded branches the first declared answers, which
`the_first_declared_accepting_branch_answers_the_overlap` in the test below pins. An external branch
is lowered as a guard over the provider's verdict in the same category, so an accepting branch
declared before it answers first whatever the verdict says, which
`an_accepting_branch_declared_before_an_external_one_answers_first` pins.

The ordering is exercised by `crates/generate/ess-entity-runtime/tests/input_guard_overlap.rs`
with a refusal over an ordinary input. Whether an empty identity, #178's own case, reaches branch
selection at all depends on how a host addresses the subject (`decide_before_load` refuses a blank
storage id before selecting), and is not measured here.

## The model interpreter

`crates/verify/ess-conformance/src/interpret/execute.rs` returned every branch whose guard held,
so an overlap point was "open" and the interpreted target refused it. It now answers an
input-guarded refusal naming no subject before it reads anything else (`refused_by_input`): the
first declared refusal the input selects is the answer, as Entity Runtime takes it; synthesis
likewise has a refusal's witness refute only the refusals declared before it
(`synthesize::sibling_refusals`). On a command guarded by a related row, a missing row is answered
by its `exists: false` branch first (`related_absent`), as
[the precedence order](cross-record-and-stored-field-guards.md#the-precedence-order) puts it. That
is also how
it decides a refused request on a command whose other branches read a held state or a stored row,
which it does not interpret yet (beyond10x/ess#227).

## What is not changed

`validate`, the finite coverage proof, the source format and the suite format. ESS-COMMAND-003 is
not changed for this rule; beyond10x/ess#227 later admitted a refusal naming no subject beside
held-state branches (`outcome-shapes.md`, "Beside held-state branches").
Generated Rust and Go targets leave the command decision to an owed method, so they carry no branch
order to change.
