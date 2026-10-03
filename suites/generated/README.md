# Generated conformance suites

**Do not edit these files.** They are generated from the specifications under
[`examples/`](../../examples) by `ess verify conform synthesize`. Regenerate them with the
commands below when the specification or the suite format changes.

A suite is the other half of a specification: every check an implementation has to pass for
the word *conformant* to mean anything about it. One JSON document per specification, keyed by
scenario id, holding no handle into any particular compilation — so a runner in another
language can read it, and a fault matrix can name a scenario by an id that does not move when
a sibling is added.

```console
ess verify conform run --suite suites/generated/billing/suite.json --target billing
```

The billing suite is regenerated with the authored scenarios named separately from the
specification directory:

```console
ess verify conform synthesize --path examples/billing \
  --scenarios examples/billing-scenarios \
  --target ir --out suites/generated/billing/suite.json
ess verify conform synthesize --path examples/gatepass \
  --target ir --out suites/generated/gatepass/suite.json
ess verify conform synthesize --path examples/oracle-fixture \
  --target ir --out suites/generated/oracle-fixture/suite.json
```

| suite | checks | scenarios | authored | no scenario | generated from |
| --- | --- | --- | --- | --- | --- |
| [`billing/suite.json`](billing/suite.json) | billing v3 (model digest 096efa38ec46e97a32f81b72193e43114df1156464648a885134ffafc9ac9648, contract digest c9ecfdf5bed1bcb88068dad060895f16ca73de361204ae488d6f0d39477f6f79) | 33 | 1 | 0 | [`examples/billing`](../../examples/billing) |
| [`gatepass/suite.json`](gatepass/suite.json) | gatepass v1 (model digest 7d021b6ebe1c4715096f165d6564389be0f46311f67d791ed748f627314d611c, contract digest 2668f3034afb388a33d7add462e15a830b6010fbfe83101f1dd2526fa18d52ed) | 17 | 0 | 5 | [`examples/gatepass`](../../examples/gatepass) |
| [`oracle-fixture/suite.json`](oracle-fixture/suite.json) | oracle v1 (model digest f8b3f1c9fa884017968b683a76b8636b8d8933cd31818603c1035603883e4656, contract digest 4108a14f183b15988a40f260f1cc260bf1e516d8b6ab768d31f600bd73ab60eb) | 34 | 0 | 6 | [`examples/oracle-fixture`](../../examples/oracle-fixture) |

## Authored, and counted apart

A suite holds two populations, and the column above separates them because they are not the same
claim. A **generated** scenario is an obligation the specification derived: the model declares a
branch, so an implementation owes a check about it. An **authored** scenario is what a person wrote
down about something the model declares a contract for and no algorithm for — a matching order, a
tie-break, which of two rows is first — and it is only as good as the person. They are told apart by
their ids: an authored one is written `<domain>/authored/<name>`, so a report, a fault matrix and a
`go test -run` filter can each tell them apart without being told.

The one here is
[`examples/billing-scenarios/outstanding-invoices-rank-latest-first.yaml`](../../examples/billing-scenarios/outstanding-invoices-rank-latest-first.yaml).
Synthesis checks that `OutstandingInvoices` is in its declared order. The authored scenario also
names the expected row order for its arrangement. Fresh suites declare
`scenario_initial_state: empty`: each scenario begins with an empty logical namespace before its
arrangement, even when the implementation shares a physical target.

## What no scenario covers

A construct the specification does not say enough about to test is refused rather than quietly
omitted (design §36). A refusal is a fact about the specification, not a gap in this file — and
it is listed here rather than left in a command's output because a suite holding fewer checks
than the specification requires is the one failure a passing run cannot show. Here it is a line
in a diff instead.

### `billing`

Every construct produced a scenario, and nothing is refused.

### `gatepass`

| code | element | the scenario that is missing |
| --- | --- | --- |
| `ESS-SYNTH-011` | `entity gatepass.visit.Visit` | `gatepass.visit.Visit/invariant/after/gatepass.visit.AdmitVisitor/admitted` |
| `ESS-SYNTH-011` | `entity gatepass.visit.Visit` | `gatepass.visit.Visit/invariant/after/gatepass.visit.AdmitVisitor/admitted` |
| `ESS-SYNTH-011` | `entity gatepass.visit.Visit` | `gatepass.visit.Visit/invariant/after/gatepass.visit.RegisterVisit/registered` |
| `ESS-SYNTH-011` | `entity gatepass.visit.Visit` | `gatepass.visit.Visit/invariant/after/gatepass.visit.SignOutVisitor/signed-out` |
| `ESS-SYNTH-011` | `entity gatepass.visit.Visit` | `gatepass.visit.Visit/invariant/after/gatepass.visit.SignOutVisitor/signed-out` |

What would close them:

* declare a view that holds an instance in this state, or the invariant cannot be read after this branch
* publish the fields the invariant reads in a view of this entity, or state the invariant over what one already publishes

### `oracle-fixture`

| code | element | the scenario that is missing |
| --- | --- | --- |
| `ESS-SYNTH-011` | `entity oracle.order.Order` | `oracle.order.Order/invariant/after/oracle.order.AmendOrder/amended` |
| `ESS-SYNTH-011` | `entity oracle.order.Order` | `oracle.order.Order/invariant/after/oracle.order.CancelOrder/cancelled` |
| `ESS-SYNTH-011` | `entity oracle.order.Order` | `oracle.order.Order/invariant/after/oracle.order.HoldOrder/held` |
| `ESS-SYNTH-011` | `entity oracle.order.Order` | `oracle.order.Order/invariant/after/oracle.order.PlaceOrder/accepted` |
| `ESS-SYNTH-011` | `entity oracle.order.Order` | `oracle.order.Order/invariant/after/oracle.order.ShipOrder/shipped` |
| `ESS-SYNTH-010` | `binding handoff-on-shipped` | `handoff-on-shipped/binding/on-failure` |

What would close them:

* `drop` is unobservable by design; write `escalate:` with an event if the failure has to be provable
* publish the fields the invariant reads in a view of this entity, or state the invariant over what one already publishes

One question the specification does not answer, so no scenario is owed for it (a note, not a
refusal): `oracle.order.AmendOrder` declares no `wrong_state` outcome, so what it answers when its
`instance:` names no record is undeclared
([`docs/design/typed-literals-and-unknown-instances.md`](../../docs/design/typed-literals-and-unknown-instances.md)).
