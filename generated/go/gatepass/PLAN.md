<!--
  generated from gatepass v1
  model digest f8ccea748a49e127ca2e18f725481394cc0eab1787fafd77d16c52485bf2abba
  contract digest a6fdd92f3a88ac0abbe59789406f3001df466e87f222e4aad1a8348c17f91d7c
  do not edit: regenerate with `ess synthesize`
-->
# Synthesis plan — gatepass v1

Scope: `component-skeletons`, planned by `ess-synth`. Regenerate with `ess synthesize`.

29 capabilities: **26 generated**, **1 obligations**, **2 refused**. An obligation is yours to implement against its contract; a refusal is a fact about this synthesis scope, not about the specification.

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
| command behaviour | `gatepass.visit.AdmitVisitor` |
| command contract | `gatepass.visit.RegisterVisit` |
| command contract | `gatepass.visit.SignOutVisitor` |
| command behaviour | `gatepass.visit.SignOutVisitor` |
| event type | `gatepass.visit.VisitRegistered` |
| event type | `gatepass.visit.VisitorAdmitted` |
| event type | `gatepass.visit.VisitorDeparted` |
| error type | `gatepass.visit.InvalidVisitLength` |
| error type | `gatepass.visit.VisitStateConflict` |
| view type | `gatepass.visit.ExpectedVisits` |
| view query | `gatepass.visit.ExpectedVisits` |
| view type | `gatepass.visit.VisitById` |
| view query | `gatepass.visit.VisitById` |
| component port | `pass-service` |
| component transport | `pass-service` |

## Ports — yours to provide

What the specification fully determines is generated; what it cannot determine is an obligation. A generated command behaviour or view query reads and writes through the ports below, and they are yours to provide: synthesis generates each port's contract and never an implementation of one, so where instances live stays your decision.

| port | what it answers |
| --- | --- |
| storage | one per entity a generated behaviour or query reads or writes: the instance stored under an identity; storing, replacing and removing one; and every stored instance, in the order the store keeps them |
| context | where a generated behaviour asks it: the caller's attributes, every identity and value the specification says the implementation assigns, and whether each `external:` branch is taken |

## Obligations — yours to implement

| capability | source | why not generated | contract |
| --- | --- | --- | --- |
| command behaviour | `gatepass.visit.RegisterVisit` | kept an obligation by `creates:` leaving the required field `visitor` of `gatepass.visit.Visit` undetermined, in `registered` | given `gatepass.visit.RegisterVisit` input, decide and enact exactly one outcome — `registered` when `expected_minutes > 0`, creates `gatepass.visit.Visit`, emits `gatepass.visit.VisitRegistered`; `refused` otherwise, error `gatepass.visit.InvalidVisitLength` |

## Refused — not represented by this synthesis

| capability | source | stage | why |
| --- | --- | --- | --- |
| actor grants | `gatepass.visit.Receptionist` | planning | may invoke `gatepass.visit.AdmitVisitor`, `gatepass.visit.RegisterVisit`, `gatepass.visit.SignOutVisitor`; generated as data, not enforced: the grant is available as the declared actors and the qualified commands each may invoke, and enforcement stays with the caller, because a grant is checked against a caller identity, which types do not carry |
| actor grants | `gatepass.visit.SecurityAuditor` | planning | observes only; it may invoke no command; generated as data, not enforced: the grant is available as the declared actors and the qualified commands each may invoke, and enforcement stays with the caller, because a grant is checked against a caller identity, which types do not carry |
