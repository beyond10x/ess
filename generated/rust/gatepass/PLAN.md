<!--
  generated from gatepass v1
  model digest f8ccea748a49e127ca2e18f725481394cc0eab1787fafd77d16c52485bf2abba
  contract digest a6fdd92f3a88ac0abbe59789406f3001df466e87f222e4aad1a8348c17f91d7c
  do not edit: regenerate with `ess synthesize`
-->
# Synthesis plan — gatepass v1

Scope: `component-skeletons`, planned by `ess-synth`. Regenerate with `ess synthesize`.

29 capabilities: **22 generated**, **5 obligations**, **2 refused**. An obligation is yours to implement against its contract; a refusal is a fact about this synthesis scope, not about the specification.

## Generated

| capability | source |
| --- | --- |
| domain type | `gatepass.visit.Badge` |
| domain type | `gatepass.visit.Building` |
| domain type | `gatepass.visit.Deposit` |
| domain type | `gatepass.visit.EmployeeId` |
| domain type | `gatepass.visit.Host` |
| domain type | `gatepass.visit.VendorRef` |
| domain type | `gatepass.visit.Visit.State` |
| domain type | `gatepass.visit.VisitId` |
| domain type | `gatepass.visit.VisitorName` |
| entity lifecycle | `gatepass.visit.Visit` |
| command contract | `gatepass.visit.AdmitVisitor` |
| command contract | `gatepass.visit.RegisterVisit` |
| command contract | `gatepass.visit.SignOutVisitor` |
| event type | `gatepass.visit.VisitRegistered` |
| event type | `gatepass.visit.VisitorAdmitted` |
| event type | `gatepass.visit.VisitorDeparted` |
| error type | `gatepass.visit.InvalidVisitLength` |
| error type | `gatepass.visit.VisitStateConflict` |
| view type | `gatepass.visit.ExpectedVisits` |
| view type | `gatepass.visit.VisitById` |
| component port | `pass-service` |
| component transport | `pass-service` |

## Obligations — yours to implement

| capability | source | why not generated | contract |
| --- | --- | --- | --- |
| command behaviour | `gatepass.visit.AdmitVisitor` | the contract is declared; the algorithm is not | given `gatepass.visit.AdmitVisitor` input, decide and enact exactly one outcome — `admitted` otherwise, takes `arrive` of `gatepass.visit.Visit`, emits `gatepass.visit.VisitorAdmitted`; `wrong-state` from a state no declared move starts in, error `gatepass.visit.VisitStateConflict`, and for an instance no record carries, without the error's fields |
| command behaviour | `gatepass.visit.RegisterVisit` | the contract is declared; the algorithm is not | given `gatepass.visit.RegisterVisit` input, decide and enact exactly one outcome — `registered` when `expected_minutes > 0`, creates `gatepass.visit.Visit`, emits `gatepass.visit.VisitRegistered`; `refused` otherwise, error `gatepass.visit.InvalidVisitLength` |
| command behaviour | `gatepass.visit.SignOutVisitor` | the contract is declared; the algorithm is not | given `gatepass.visit.SignOutVisitor` input, decide and enact exactly one outcome — `signed-out` otherwise, takes `depart` of `gatepass.visit.Visit`, emits `gatepass.visit.VisitorDeparted`; `wrong-state` from a state no declared move starts in, error `gatepass.visit.VisitStateConflict`, and for an instance no record carries, without the error's fields |
| view query | `gatepass.visit.ExpectedVisits` | how the projection is kept current is a storage decision | a query answering `gatepass.visit.ExpectedVisits` with rows projected from `gatepass.visit.Visit` at `read_your_writes` consistency, containing instances where `state == Expected` |
| view query | `gatepass.visit.VisitById` | how the projection is kept current is a storage decision | a query answering `gatepass.visit.VisitById` with rows projected from `gatepass.visit.Visit` at `eventual` consistency |

## Refused — not represented by this synthesis

| capability | source | stage | why |
| --- | --- | --- | --- |
| actor grants | `gatepass.visit.Receptionist` | planning | may invoke `gatepass.visit.AdmitVisitor`, `gatepass.visit.RegisterVisit`, `gatepass.visit.SignOutVisitor`; a grant is checked against a caller identity, which types do not carry, and enforcement belongs to the layer that knows who is calling |
| actor grants | `gatepass.visit.SecurityAuditor` | planning | observes only; it may invoke no command; a grant is checked against a caller identity, which types do not carry, and enforcement belongs to the layer that knows who is calling |
