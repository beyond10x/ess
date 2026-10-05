---
title: Author scenarios
sidebar_position: 2
description: "Add scenarios a person wrote to a generated suite: selection, compilation, subsets, expected branches and refusals, backend state, instances and captured event identities."
---

# Author scenarios

## Select authored scenarios explicitly

Scenarios a person wrote (`ess-scenario/*` documents) join the generated ones only when
`--scenarios` names them. It takes one file or one directory:

| `--scenarios` names | What is read |
|---|---|
| a file | that file, whatever its extension |
| a directory with an `ess-inputs.yaml` | exactly the files its `scenarios:` list names, nested or not, of any extension |
| a directory without one | its immediate lowercase `.yaml` and `.yml` files; subdirectories are not searched |
| nothing | no authored scenarios, even when the model's `ess-inputs.yaml` lists some |

An empty selection is refused before anything is written or run. The
[mixed-layout example](../specify/layout-and-validation.md#keep-sources-and-generated-output-together) keeps
the model and the scenarios in one directory, so the same directory serves both options:

```shell-session
$ ess verify conform author --path . --scenarios . --suite-format 5 --out output/authored.json
```

`ess-inputs.yaml` refuses duplicate paths, a path listed as both a specification and a scenario,
a path that leaves the directory, and a symlink. A coverage suite (`--suite-format 5`) records each
scenario's relative path and source text: moving the directory or reordering the list leaves the
suite unchanged, and changing line endings changes it. Its coverage describes the selected
scenarios, not every scenario file below the directory.

`ess verify conform run --suite FILE` runs the committed suite as it is and selects nothing.

## Compile authored scenarios on their own

`ess verify conform synthesize --scenarios` compiles authored scenarios beside the generated ones.
`ess verify conform author` compiles only the authored ones, and nothing the specification obliges:

```shell-session
$ ess verify conform author --path examples/billing --scenarios examples/billing-scenarios \
    --out target/authored.json
1 authored scenario(s) from 1 file(s), 0 refusal(s), suite ess-conformance/4, written to target/authored.json
$ ess verify conform author --path examples/billing --scenarios examples/billing-scenarios \
    --suite-format 5 --out target/authored-coverage.json
1 selected scenario(s), 1 authored source(s), 0 refusal occurrence(s)
```

Every command, actor, outcome, event, error, view, entity, field, enum variant and lifecycle state
a scenario names is resolved against the model here, so a name the model does not declare is
refused now rather than at the first run that reaches it. The act checks on this page
(`ESS-AUTHOR-037` and the others) apply the same way. Use it in a pre-commit or CI step that only
checks scenario files, and run the result like any other suite.

## Run a chosen subset of a coverage suite

`ess verify conform select` narrows a coverage suite (`--suite-format 5`, or the `/7` and `/9`
versions some constructs select) to scenario IDs you list. `--ids` names a JSON file holding a
sorted array of distinct IDs; `[]` selects none, explicitly:

```shell-session
$ ess verify conform synthesize --path examples/billing --suite-format 5 --out target/coverage-suite.json
32 selected scenario(s), 0 authored source(s), 0 refusal occurrence(s)
$ cat create-only.json
["billing.invoice.CreateInvoice/outcome/accepted","billing.invoice.CreateInvoice/outcome/rejected"]
$ ess verify conform select --suite target/coverage-suite.json --ids create-only.json \
    --out target/create-only.json
$ ess verify conform run --target billing --suite-input target/create-only.json --report-format 2
billing v3 against billing-reference 0.51.0 — passed
  passed billing.invoice.CreateInvoice/outcome/accepted
  passed billing.invoice.CreateInvoice/outcome/rejected
  2 scenarios: 2 passed, 0 failed, 0 error, 0 unsupported
```

The output is an `ess-conformance-input/1` carrier that keeps the full parent suite, so a report on
the subset still shows what was left out. Narrow it again with `--suite-input` in place of
`--suite`. `select` refuses, writing nothing:

| Input | Refusal |
|---|---|
| IDs out of order, or repeated | `explicit IDs must be sorted and distinct` |
| an ID the parent does not hold | `explicit ID absent from parent` |
| a string that is not a scenario ID | `invalid scenario id identifier`, listing the ID shapes |
| an ordinary suite (`/4`) | `input/1 requires coverage suite/5, suite/7 or suite/9` |

A subset passing is not the whole suite passing: only a nonempty, complete, all-pass selection
qualifies as conformance ([Opt into declared coverage](runners.md#opt-into-declared-coverage)).

## Expect the branch the input selects

`validate` reads each act's literal input against the command's `when:` guards, in the order a
conforming target answers them: input-guarded refusals first, the first declared of them; then
accepting `when:` and external branches in declaration order; the default only where no `when:`
holds. An act is refused with `ESS-AUTHOR-041` when that order decidedly does not take the branch
it expects under `outcome:`. Where no `outcome:` is written, the check applies to the branches that
report its `error:`, and the act is refused only when none of them is taken.

```yaml
# `id-required: ticket_id == ""` is an input-guarded refusal, so it answers before `closed`.
timeline:
  - at: 2026-01-05T09:00:00Z
    command: demo.tickets.SetTicketOpen
    input: {ticket_id: "", open: false}
    outcome: closed
```

The refusal names the branch that answers first and its guard, or the expected branch's own guard
that the input refutes. Only what the input decides is read. What a branch reads beyond the input
(a held state, a stored or related row, an external answer, the target's clock) is not decided, and
neither is a guard over a field sent as `{$instance: …}` or another reference. A guard over `now`
is decided only where it reads the same at every run, such as a start already in the past when the
operand was introduced. Where any of these leaves the answer open, the act is accepted.

## Expect an external branch in an authored scenario

No input decides a branch declared `external:`, so an authored act that expects one names it under
`outcome:`:

```yaml
timeline:
  - at: 2026-01-05T09:00:00Z
    command: billing.email.SendEmail
    input: {recipient: nobody@example.test, template: welcome}
    outcome: failed
    error: {name: billing.email.Undeliverable}
```

The act compiles into a `configure_external_outcome` for `failed` immediately before its
`execute_command`, as a generated scenario does, so the target is told which answer to give for
that call.

That is the only external answer an act can state. An act is refused with `ESS-AUTHOR-037`
when one of its claims holds only on an external answer it does not state, whether or not
`outcome:` is written. The check covers the act's error, its direct response, each event it
claims published and each event it claims absent. The answers it considers are those of the act's
own command and of every command a binding invokes from what the act publishes, however many
bindings along. A binding's escalation event needs its invoked command to fail. The refusal names
every such branch as `command/branch`.

For example, an act on `billing.invoice.CreateInvoice` that claims
`billing.email.DeliveryEscalated` is refused naming `billing.email.SendEmail/failed`. The binding's
`SendEmail` call fails only on that external branch, and an authored act has no key for the
answer a binding's call gives. Generated scenarios cover the escalation. An event that some
command's input-decided branch publishes exempts a claim only when the act reaches that command.

## Expect the refusal an ungranted actor gets

An act sent as an actor the specification does not grant its command is refused with
`ESS-AUTHOR-009`, because it checks a system the model does not describe. To claim that such an
actor is refused, write `refused: not_granted` on the act, in `type: ess-scenario/4`:

```yaml
timeline:
  - at: 2026-01-05T09:00:00Z
    command: billing.invoice.CreateInvoice
    actor: billing.invoice.Auditor
    input:
      account_id: 00000000-0000-4000-8000-000000000001
      customer_email: buyer@example.test
      amount: {amount: 10, currency: EUR}
    refused: not_granted
    no_events: [billing.invoice.InvoiceCreated]
```

The act compiles into its `execute_command`, sent as that actor, and an `expect_not_granted` step
naming it. A target passes the step when it refuses the command before running it with the
standard refusal naming that actor: on a served surface, `403`
`{"refused": "not granted", "actor": "billing.invoice.Auditor"}`. The step is suite/26 vocabulary,
so the suite is written at `ess-conformance/26` or later.

A refused command takes no branch and publishes nothing, so `no_events:` is the only claim it can
carry. It is checked against the target's whole event log: the log may hold no more of each listed event
after the send than just before it, counting repeats, and a refusal that hands back events fails.
So a target that runs the command and only then refuses it fails. The generated runners perform
these observations through `ObserveEvents` / `observeEvents`. A custom runner must do the same.
On reaching the act's `execute_command`, it looks ahead to the `expect_not_granted` that follows.
Before sending, it observes each `no_events:` event in the scenario's correlation and counts the
occurrences. After the refusal, it observes each event again in the same correlation. The step
fails if any count grew. A target that cannot observe its log leaves the scenario `unsupported`,
never passed. The command's answer and view comparisons cannot replace these observations. See
[the target interface](./runners.md#hold-your-own-implementation-to-the-suite) for the full
contract. No extra authored step is needed. The act is refused:

| When | Refusal |
|---|---|
| its actor holds the grant | `ESS-AUTHOR-038` |
| it names no `actor:`, or carries `outcome:`, `error:`, `response:`, `events:` or `capture:` | `ESS-AUTHOR-039` |
| the document is `ess-scenario/1` to `/3` | `ESS-AUTHOR-001`, naming `refused: not_granted` |

Where the specification serves a component (`reached_by: network`), synthesis already witnesses the
refusal once per command, as `<command>/grant/denied`, for every
command some declared actor lacks the grant for. An authored act adds a particular input or a
particular actor.

A served command no declared actor is granted is refused to every caller, so an act that sends it
with no `actor:` and expects it to run is refused with `ESS-AUTHOR-040`. Grant the command to an
actor and send the act as that actor, or expect the refusal as above.

## Establish backend state in an authored scenario

`ess-scenario/2`, introduced in 0.23.0, supports typed setup for entities whose rows arrive
from an upstream system. A scenario can establish those rows and query their
view without inventing a creator command:

```yaml
type: ess-scenario/2
domain: calls.history
scenario: recent-call
summary: A stored call appears in history.
arrange:
  - instance: recent
    entity: calls.history.CallRecord
    setup:
      identity: 00000000-0000-4000-8000-000000000001
      fields: {started_at: '2026-01-05T09:00:00Z', duration_seconds: 12}
      state: Completed
assert:
  - view: calls.history.CallHistory
    contains: {call_id: {$instance: recent}, duration_seconds: 12}
```

The model must declare that entity, its field types, lifecycle state and view.
Setup validates identity, required fields, nested values and invariants. Duplicate
qualified identities, null identities and inconclusive invariants refuse.

The adapter must establish actual isolated backend state and make it visible
before acknowledging setup. Setup emits no command or event and claims no
lifecycle path. Rust and generated Go expose an optional setup capability;
unsupported adapters produce a non-passing result. These steps use suite/6 or
coverage suite/7 and require explicit report/2. Source scenario/1 refuses setup.

## Name several instances in one input

`{$instance: name}` stands wherever the declared type at that position is the instance's
identity type: a whole input field, a list element, a map value, a struct member or the
payload of a union variant whose type is the identity, at any depth. A rollout over three release rings names them in order:

```yaml
    input:
      ring_sequence: [{$instance: a}, {$instance: b}]
      by_stage: {canary: {$instance: b}, general: {$instance: c}}
      pair: {primary: {$instance: c}, note: first}
```

Each reference resolves to the identity the run bound for that instance, and the command
receives the list or mapping with those identities in place. A reference at a position of
any other type is refused as `ESS-AUTHOR-022`, naming the position (`labels[1]`,
`pair.note`, `tags[owner]`, `target.value`); one at a member the model does not declare, or
inside a value of the wrong shape, is refused naming the member or the position. One nested
inside an event payload or an error is refused as `ESS-AUTHOR-021`. A suite carrying such a
value is suite/32 (/33 with coverage); Rust, Go and TypeScript resolve it with report/2.

## Compare an event's identity with a captured instance

An expected event may name a captured instance for a payload field typed as that instance's
identity, so an implementation that drops the identity, or publishes another one, fails:

```yaml
    events:
      - event: billing.invoice.InvoicePaid
        payload: {invoice_id: {$instance: invoice}, amount: {amount: 10, currency: EUR}}
```

The reference resolves to the identity the run bound, as it does in a command input, and the
event must carry exactly that value. The act compiles into an `expect_event_values` step, so the
suite is written at `ess-conformance/18` or later; Rust, Go and TypeScript resolve it, and browser
replay shows it as a declaration. A reference at a field of any other type, including another
entity's identity, an error field, or a position nested inside a payload field, is refused as
`ESS-AUTHOR-021`.

Synthesis makes the same comparison wherever the arrangement determines an identity an event
carries: an input sent as a captured instance, the subject's own identity (`{subject: <identity>}`),
and a related row's identity read through `{related: …}`.

## Where the observation sections went

What a suite observes beyond an act's own answer is on [Observations](observations.md). Each
section that used to be on this page is listed here under its old anchor, so an older link still
finds it.

- <a id="observe-outcomes-selected-by-held-state"></a>[Observe outcomes selected by held state](observations.md#observe-outcomes-selected-by-held-state)
- <a id="observe-retries-of-the-original-result"></a>[Observe retries of the original result](observations.md#observe-retries-of-the-original-result)
- <a id="observe-selection-periodic-activity-and-clock-evidence"></a>[Observe selection, periodic activity and clock evidence](observations.md#observe-selection-periodic-activity-and-clock-evidence)
- <a id="deliver-an-event-with-its-context"></a>[Deliver an event with its context](observations.md#deliver-an-event-with-its-context)
- <a id="observe-bounded-binding-accessors"></a>[Observe bounded binding accessors](observations.md#observe-bounded-binding-accessors)
