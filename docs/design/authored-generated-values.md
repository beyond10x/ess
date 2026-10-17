# An authored exact value on a generated field

Status: implemented for beyond10x/ess#544 (`story:authored-exact-value-on-generated-field-is-refused`).

## The defect

A payload field or a stored field an outcome marks `{generated: true}` is minted by the
implementation. Synthesis asserts its presence and type and never a value. An authored act could
still write an exact value for it, and `ess verify conform author` accepted the act:

```yaml
# model: SelectAgentForEntry/selected
payload:
  relay.desk.EntryAgentSelected:
    agent_id: input.agent_id
    connect_seq: {generated: true}
```

```yaml
# authored act
outcome: selected
events:
  - event: relay.desk.EntryAgentSelected
    payload: {agent_id: a1, connect_seq: 7}
```

The suite then passes only when the implementation happens to mint `7`, and fails every conforming
implementation that mints another value. The specification already says the value is not the
scenario's to choose, so this is a missing check rather than a new construct.

## The rule

`author` refuses with `ESS-AUTHOR-042` an exact value an act writes for a value the model leaves to
the implementation. An exact value is a literal, or a `{$fixture: …}`, which the suite resolves
before the run, so the implementation cannot have chosen it. A reference (`{$instance: …}`,
`{$observed: …}`) states a relation and is not refused.

| where the value is written | which outcomes are read | the field is minted when |
|---|---|---|
| an event under `events:` | the branch under `outcome:`, or with none every branch of the act's command that emits the event; and every branch of a command a binding invokes from what those publish, transitively | every one of those that emits the event marks the field minted |
| an error under `error:` | the branch under `outcome:` when it reports that error, or with none every branch of the act's command that reports it | every one of those marks the field minted |
| a view row under `contains:`, `excludes:` or `at:` | every branch any act of the scenario reaches as above, for a non-aggregate view, over the view's source entity | every one of those that writes the stored field marks it minted, and no `setup:` states the field |

A field is marked minted by:

- `{generated: true}`, and from `ess/27` `{generated: true, present: true}` (beyond10x/ess#545),
  whose value is present but still the implementation's;
- from `ess/27`, `{payload: <Event>.<field>}` in `sets:`, where the same branch fills that payload
  field in a way this list marks minted: the row stores the value the event published, so it is
  the same mint;
- `{input: f, else: {generated: true}}`, on a branch of the act's own command, when the act sends
  no value along `f` (absent or `null`); a binding's command reads its input from a mapping, so
  nothing is claimed about this form there;
- a struct filled member by member, at the leaf a member marks minted. The act's literal is walked
  through the members it writes, and the refusal names the leaf by its dotted path (`ref.label`).

"Every one" is deliberate. The refusal fires only where whichever branch publishes the event, or
writes the row, chose the value. Where one emitting branch fills the field from the input and
another mints it, the claim may hold for the branch the input selects, and the act is accepted. A
row a `setup:` states, or one a branch writes from the input, `subject`, a literal or an `each:`
element, holds a value the file can know, so a view claim of that field is accepted.

The refusal names the surface, the field path, the value written and the outcomes that mint it.
The repair says to drop the field from the expectation (presence and type stay checked by the
synthesized suite), or to state the relation through a reference: `{$observed: {event, field}}` in
a view row, or `capture:` on the event.

## Decisions

- **A refusal, not a warning.** Every such act is wrong for some conforming implementation, and
  `author` has no warning channel (the story's fit review).
- **Errors are included.** An error's `payload:` takes `{generated: true}` under the same rules as an
  event's (`ess-domain` `value_expression::error_payload`), so a pinned error field fails a
  conforming implementation in the same way.
- **What stays unchecked.** A payload field no outcome determines is also the implementation's to
  choose, but the model does not *mark* it. Refusing it would refuse every partial payload written
  today against a model that has not yet stated its sources, and it is not this change. The direct
  `response:` claim is not read either: an outcome declares no per-field response source that could
  be `{generated: true}`.

## Compatibility

No format moves. A document accepted before is accepted now unless it pins a minted value.
Removing the refusal later accepts every document accepted today plus these. The coverage readers
(Rust, Go, TypeScript, browser) admit `ESS-AUTHOR-042` as a file refusal.
