<!--
  generated from billing v3
  model digest 1e7906786567af32118eb2d0a8c3fcafa16c32c9649a80b60487fd2eeebc4c9c
  contract digest a21fd36f0055057629f4c235962163cdd34a3d178aa068925bcb53be623af301
  do not edit: regenerate with `ess synthesize`
-->
# Synthesis plan — billing v3

Scope: `component-skeletons`, planned by `ess-synth`. Regenerate with `ess synthesize`.

48 capabilities: **40 generated**, **4 obligations**, **4 refused**. An obligation is yours to implement against its contract; a refusal is a fact about this synthesis scope, not about the specification.

## Generated

| capability | source |
| --- | --- |
| domain type | `billing.email.EmailAddress` |
| domain type | `billing.email.MessageId` |
| domain type | `billing.email.TemplateId` |
| domain type | `billing.invoice.Account.State` |
| domain type | `billing.invoice.AccountId` |
| domain type | `billing.invoice.Channel` |
| domain type | `billing.invoice.CompanyRef` |
| domain type | `billing.invoice.Email` |
| domain type | `billing.invoice.Invoice.State` |
| domain type | `billing.invoice.InvoiceId` |
| domain type | `billing.invoice.LineItem` |
| domain type | `billing.invoice.Money` |
| domain type | `billing.invoice.Payee` |
| entity lifecycle | `billing.invoice.Account` |
| entity lifecycle | `billing.invoice.Invoice` |
| command contract | `billing.email.SendEmail` |
| command behaviour | `billing.email.SendEmail` |
| command contract | `billing.invoice.CancelInvoice` |
| command behaviour | `billing.invoice.CancelInvoice` |
| command contract | `billing.invoice.CreateInvoice` |
| command contract | `billing.invoice.IssueInvoice` |
| command behaviour | `billing.invoice.IssueInvoice` |
| command contract | `billing.invoice.PayInvoice` |
| event type | `billing.email.DeliveryEscalated` |
| event type | `billing.email.EmailSent` |
| event type | `billing.invoice.InvoiceCancelled` |
| event type | `billing.invoice.InvoiceCreated` |
| event type | `billing.invoice.InvoiceIssued` |
| event type | `billing.invoice.InvoicePaid` |
| error type | `billing.email.Undeliverable` |
| error type | `billing.invoice.InvalidAmount` |
| error type | `billing.invoice.InvoiceStateConflict` |
| view type | `billing.invoice.InvoiceById` |
| view query | `billing.invoice.InvoiceById` |
| view type | `billing.invoice.OutstandingInvoices` |
| conversion | `billing.invoice.Email -> billing.email.EmailAddress` |
| binding transformation | `notify-on-invoice-created` |
| binding delivery | `notify-on-invoice-created` |
| component port | `email-service` |
| component port | `invoice-service` |

## Ports — yours to provide

What the specification fully determines is generated; what it cannot determine is an obligation. A generated command behaviour or view query reads and writes through the ports below, and they are yours to provide: synthesis generates each port's contract and never an implementation of one, so where instances live stays your decision.

| port | what it answers |
| --- | --- |
| storage | one per entity a generated behaviour or query reads or writes: the instance stored under an identity; storing, replacing and removing one; and every stored instance, in the order the store keeps them |
| context | where a generated behaviour asks it: the caller's attributes, every identity and value the specification says the implementation assigns, and whether each `external:` branch is taken |

## Obligations — yours to implement

| capability | source | why not generated | contract |
| --- | --- | --- | --- |
| command behaviour | `billing.invoice.CreateInvoice` | kept an obligation by `creates:` leaving the required field `payee` of `billing.invoice.Invoice` undetermined, in `accepted` | given `billing.invoice.CreateInvoice` input, decide and enact exactly one outcome. Declared outcomes (declaration order, not selection precedence): `accepted` when `amount.amount > 0`, creates `billing.invoice.Invoice`, emits `billing.invoice.InvoiceCreated`; `rejected` otherwise, error `billing.invoice.InvalidAmount` |
| command behaviour | `billing.invoice.PayInvoice` | kept an obligation by the fields of error `billing.invoice.InvalidAmount`, which the specification gives no source, in `rejected` | given `billing.invoice.PayInvoice` input, decide and enact exactly one outcome. Declared outcomes (declaration order, not selection precedence): `settled` when `amount.amount > 0`, takes `settle` of `billing.invoice.Invoice`, emits `billing.invoice.InvoicePaid`; `rejected` otherwise, error `billing.invoice.InvalidAmount`; `wrong-state` from a state no declared move starts in, error `billing.invoice.InvoiceStateConflict`, and for an instance no record carries, without the error's fields |
| view query | `billing.invoice.OutstandingInvoices` | kept an obligation by an order over the optional field `issued_at`, which has no order against a present value | a query answering `billing.invoice.OutstandingInvoices` with rows projected from `billing.invoice.Invoice` at `read_your_writes` consistency, containing instances where `state == Issued` |
| binding escalation | `notify-on-invoice-created` | the contract is declared; the algorithm is not | the declared `billing.email.DeliveryEscalated`, recording that delivering `billing.email.SendEmail` for `notify-on-invoice-created` was given up on — the event is declared; how its fields are filled from the failed invocation is not |

## Refused — not represented by this synthesis

| capability | source | stage | why |
| --- | --- | --- | --- |
| actor grants | `billing.invoice.Auditor` | planning | observes only; it may invoke no command; generated as data, not enforced: the grant is available as the declared actors and the qualified commands each may invoke, and enforcement stays with the caller, because a grant is checked against a caller identity, which types do not carry |
| actor grants | `billing.invoice.Customer` | planning | may invoke `billing.invoice.CreateInvoice`; generated as data, not enforced: the grant is available as the declared actors and the qualified commands each may invoke, and enforcement stays with the caller, because a grant is checked against a caller identity, which types do not carry |
| workload | `email-service` | planning | requires at least 2 replica(s); topology synthesis is deferred with its design |
| workload | `invoice-service` | planning | requires at least 2 replica(s); topology synthesis is deferred with its design |
