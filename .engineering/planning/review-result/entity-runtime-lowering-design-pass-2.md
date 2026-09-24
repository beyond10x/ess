---
format: aep.planning-md/2
id: review-result:entity-runtime-lowering-design-pass-2
kind: review-result
status: active
title: Entity Runtime lowering final whole design review pass 2
relations:
- reviews: initiative:ess-evolution
revision: 1
---
needs-revision

Covered proposal: `docs/design/ess-evolution/entity-runtime-lowering.md` at
`sha256:40f646c8147ceb95230c36bd56502c2327d0e50b3ee664bff208504387d840f0`,
`docs/design/models/entity-runtime-lowering/domains/contract.yaml` at
`sha256:cc051062b940184c4d7544551bdc8ee2a2ceeda79fe5d905676abdc6ab8a216c`, and
`docs/design/models/entity-runtime-lowering/system.yaml` at
`sha256:a88342f8bcfb4a331949b0656ae4865f5c3524b9fd01c39c35f5f91406bd9564`,
against ESS/extraction `be604d874ee9e567ae104e565e7cbddf953885a9` and accepted Entity Runtime
`da5d368756f5a63e4b2efd5589f7bc3441cd7aff` (tree
`c84addcccd7141131fe155ef818feeb5124e2572`). This is the second and final whole-design
examination of two. It does not reopen either accepted source/extraction review budget.

## Findings

### 1. Omitted operation fields are still silently specialized to unchanged, contradicting both required fixtures

**Verdict:** NEEDS-CHANGE. **Origin:** introduced. **Severity:** blocker.

**Requirement.** The lowerer must preserve updates as well as creation and must keep host-owned facts
explicit. The original brief requires all ten dimensions, including updates, and forbids claiming
billing/gatepass complete by refusing or omitting required behavior. Review 1 required one rule for
every undetermined optional entity/event/response output: preserve the per-invocation host choice or
return a precise `OptionalBoundOutputUnsupported` and keep the affected fixture acceptance blocked.

**What was measured.** The corrected design still says that on an update or move every entity field
omitted from `sets` remains unchanged
(`docs/design/ess-evolution/entity-runtime-lowering.md:400`). That is not an ESS source fact. The
resolver states that a field with no source is deliberately not checked and that a branch naming two
fields of six says what it determines rather than closing the other four
(`crates/specify/ess-compiler/src/resolve.rs:1727`). The conformance contract likewise says that
without `sets` nothing is asserted about the field value
(`crates/verify/ess-conformance/src/synthesize.rs:132`).

Both required real fixtures reach the difference. `billing.invoice.IssueInvoice.issued` declares a
move and event but no `sets` (`examples/billing/domains/invoice.yaml:270`), while the accepted billing
realization changes the optional `issued_at` from absent to the host clock value during that move
(`examples/billing-realization/src/invoice.rs:217`). `gatepass.visit.AdmitVisitor.admitted` likewise
declares a move and event but no `sets` (`examples/gatepass/domains/visit.yaml:203`), while the real
realization writes the supplied badge into the optional entity field
(`examples/gatepass-realization/src/visit.rs:158`). The proposed definitions leave both fields
unchanged, so the admitted billing ordering behavior loses its issue time and gatepass never records
the badge it emits.

**Target evidence.** The proposal already acknowledges that the accepted target cannot conditionally
write an operation field (`entity-runtime-lowering.md:467`). The actual target enforces this:
`validate_conditional_outcome` emits `ConditionalSetOnOperation` for any operation
`set_if_present` (`crates/entity-core/src/validation.rs:582` at `da5d3687`). The accepted capability
therefore fixes top-level conditional creation, event and response members, but it does not implement
these two operation updates. The design nevertheless claims both invoice-service and pass-service
lower completely (`entity-runtime-lowering.md:470`) and says no factual target gap blocks them in the
target-adoption result.

**Bounded correction.** Remove the invented unchanged rule and account explicitly for every
undetermined operation field. At this accepted target, return path-bearing
`OptionalBoundOutputUnsupported` for at least
`billing.invoice.IssueInvoice.issued.issued_at` and
`gatepass.visit.AdmitVisitor.admitted.badge`, and report invoice/pass-service acceptance blocked.
Complete fixture admission requires a separately admitted mechanism that records a post-selection
host value in the ER decision; this review does not invent that mechanism or reopen the accepted ER
review. Do not preserve the complete-fixture claim by choosing unchanged on the host's behalf.

### 2. The SDK handoff names a target type that does not exist

**Verdict:** NEEDS-CHANGE. **Origin:** introduced. **Severity:** warning.

**Requirement.** The final implementation interface must cite actual accepted Rust symbols rather
than matching prose.

**What was measured.** The target-capability and SDK-flow sections call
`RegistryRuntime::decide_before_load`
(`docs/design/ess-evolution/entity-runtime-lowering.md:486` and `:540`). At the accepted target the
public registry-backed type is `entity_core::Runtime`
(`crates/entity-core/src/runtime.rs:406`), and its method is
`Runtime::decide_before_load` (`crates/entity-core/src/runtime.rs:488`). No `RegistryRuntime` symbol
exists. The free `entity_core::decide_before_load` cited beside it does exist.

**Bounded correction.** Replace both `RegistryRuntime` references with `Runtime` (or the fully
qualified `entity_core::Runtime`) and retain the existing method arguments and continuation flow.

## First-review disposition

1. **Pre-load PayInvoice ordering:** resolved. The accepted `PreloadDecision::{Refused, Load}`,
   opaque `PreparedOperation`, exact `PreparedSubject`, and consuming `continue_with` preserve the
   nonpositive/unknown-identity refusal before lookup and the positive exact-subject continuation.
   The proposed matrix and no-lookup/no-append controls cover the original counterexample. Finding 2
   above corrects only the concrete runtime type name.
2. **Optional host-owned output presence:** partially resolved and still blocking. Creation fields
   `CreateInvoice.note`, `CreateInvoice.issued_at`, and `RegisterVisit.badge`, plus top-level optional
   event and response members, now use optional no-default slots and the accepted conditional maps.
   Shared slots preserve one presence/value and independent slots remain independent. The proposal
   did not apply the same source-ownership rule to operation entity fields, and the two real fixture
   counterexamples in finding 1 remain.
3. **Lossy companion model presented as the Rust contract:** resolved. The companion types are
   renamed `*Inventory`/`*Kind`, both files state that the model is a non-normative diagnostic
   inventory, and the Rust interface is explicitly normative. The installed CLI validated the
   two-file model and compiled it to nonempty JSON, both with exit status 0.

## Examination boundary

I examined the complete ten-dimension crosswalk, public lowering inputs and outputs, definition
identity and selected relation closure, `ValidatedDefinition`/registry admission, logical identity
versus storage address, exact values, ordered outcomes and duplicate events, deterministic slots and
digests, accepted conditional-presence and pre-load source, `/3` record/request framing, and the SDK
`/4` handoff. I compared the real billing and gatepass specifications with their Rust realizations
and conformance/effect bindings. External billing email remains correctly assigned to the existing
SDK/provider publication binding; no stateless ER expansion is required by this review. Apart from
the findings above, no additional finite defect survived examination.

No Cargo, browser, native build, source/test edit, planning write, publication, or external
integration invocation was performed. Connector use was limited to the required local readiness
diagnostic. The three proposal files remained at their frozen hashes. The sole write outside the
assigned worktree is this immutable report. This closes the lowerer's maximum two whole design
reviews; there is no third pass or target re-review.

```findings
- file: docs/design/ess-evolution/entity-runtime-lowering.md
  line: 400
  category: acceptance
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: The design silently fixes every omitted operation field unchanged even though ESS leaves it implementation-owned and the required billing IssueInvoice and gatepass AdmitVisitor realizations write issued_at and badge, which accepted ER cannot express with operation set_if_present, so those exact paths must be refused and fixture acceptance remain blocked until an admitted mechanism exists.
- file: docs/design/ess-evolution/entity-runtime-lowering.md
  line: 486
  category: contract-drift
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: The target and SDK sections call nonexistent RegistryRuntime::decide_before_load even though the accepted public type is entity_core::Runtime, so both citations must name the actual symbol.
```
