# Input-guarded refusals take precedence over the accepting branches they overlap

beyond10x/ess#178. No source or suite format change.

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

The rule orders refusals ahead of accepting branches and nothing else:

| pair | order |
|---|---|
| input-guarded refusal, accepting branch with an input guard: a plain `when:`, the `when:` beside a `when_subject:`, or an external branch's `when:` | the refusal, before any stored row is read or any provider asked |
| input-guarded refusal, default | not an overlap: the default is what no other guard selects |
| two input-guarded refusals | unordered: a witness of one refutes the other with or without a default, so it selects exactly one outcome; where no input does, synthesis refuses the scenario naming both guards (beyond10x/ess#209) |
| two accepting guarded branches | unchanged |
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
3. **A branch every input of which a refusal claims is refused naming it.** Beside a default,
   `zero: count == 0` next to `too-few: count <= 0` has no input that reaches it. Its refusal
   (`ESS-SYNTH-003`) says `count == 0 outside too-few (count <= 0), the input-guarded refusal taken
   first` rather than that a satisfiable guard has no witness. The same record (`Shadow`) is kept
   by the input search (`reach`), the external search (`reach_external`) and the stored-row search
   (`subject_fact::prepare`, over every row it arranged), and each renders it only where every
   input its own guards admitted was claimed by a refusal.

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

## Entity Runtime

Entity Runtime selects the first branch whose guard holds. The lowering in
`crates/generate/ess-entity-runtime/src/lib.rs` ordered guarded branches by source position, so
`closed` written before `id-required` won the overlap. It now orders every input-guarded refusal
first, then the other guarded branches, the default, and the wrong-state branch.

The ordering is exercised by `crates/generate/ess-entity-runtime/tests/input_guard_overlap.rs`
with a refusal over an ordinary input. Whether an empty identity, #178's own case, reaches branch
selection at all depends on how a host addresses the subject (`decide_before_load` refuses a blank
storage id before selecting), and is not measured here.

## What is not changed

`validate`, ESS-COMMAND-003, the finite coverage proof, the source format and the suite format.
Generated Rust and Go targets leave the command decision to an owed method, so they carry no branch
order to change.
