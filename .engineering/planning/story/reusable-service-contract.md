---
format: aep.planning-md/2
id: story:reusable-service-contract
kind: story
status: implemented
title: Extract reusable selected service contracts without losing outcome semantics
relations:
- decomposes: initiative:ess-evolution
- serves: vision:O2
scope:
- confidence: cited
  path: CHANGELOG.md
- confidence: cited
  path: Cargo.lock
- confidence: cited
  path: Cargo.toml
- confidence: cited
  path: crates/specify/ess-service-contract
- confidence: cited
  path: docs/design/ess-evolution/service-contract.md
- confidence: cited
  path: docs/design/models/service-contract
revision: 10
---
## Outcome

Extract reusable selected service semantics into ess-service-contract for approved ESS evolution
revision 1 step 6 (root completion row M5). A selected component exposes exact operations, complete
ordered outcomes/results/errors/events, owned-domain views and ownership, backed by the existing
compiler definition graph and original synthesis obligations/refusals. This is direct approved
implementation, not a new prerequisite or a replacement for full ER/SDK service acceptance.

## Existing evidence and bounded deliverable

Local source-routing result ess-evolution/waves/0009-service-convergence/source-routing-result.md,
SHA-256 9c128180d46b9b591f34829fc3a4557abdb77e87b32944485a6ffb76cbdc1200,
identifies duplicated SDK lowering, flattened outcome emissions and no reusable ESS crate. Existing
compiler types already preserve these semantics. No current test demonstrates selected ServiceIr
extraction, because that API does not exist.

Owner-local design: docs/design/ess-evolution/service-contract.md. Typed selection/index vocabulary:
docs/design/models/service-contract/system.yaml and domains/selection.yaml. Actual installed
`ess specify validate --path docs/design/models/service-contract` exited 0 and printed
`service_contract v1 — 2 file(s), valid`. Complete definitions remain the compiler's existing
Resolved* types; the model is not a replacement IR or a new persisted envelope.

Implement the design's pure borrowed ServiceIr with private construction, exact component selection,
full SynthesisPlan admission and ordered capability/obligation access. Preserve the source IR and
existing handle lookups. No Eventlog, ER, SDK, runtime format, application IO or generic registry.

## Acceptance

Billing/gatepass and focused multi-component fixtures demonstrate exact surfaces, complete branch
and event order/multiplicity, exact values, contextual types/relations, explicit obligations and
refusals, and unchanged source canonical bytes. Unknown selection and altered/foreign/incomplete
plans refuse deterministically. Dropped branch/event and wrong-field/operation faults must fail.
Pure crate tests, strict Clippy, formatting and actual Rust 1.85 closure precede author completion.
Root then owns submitted-source examinations, full ESS task check with consumer coverage, required
site-build and local integration. No skipped or missing gate supports completion.

## Scope

- cited: crates/specify/ess-service-contract/ — new pure crate named by approved dependency policy.
- cited: Cargo.toml and Cargo.lock — new workspace member/dependencies, no other package changes.
- cited: docs/design/ess-evolution/service-contract.md — final typed interface before implementation.
- cited: docs/design/models/service-contract/ — typed selection/index vocabulary above.
- cited: CHANGELOG.md — coordinator-owned resulting behavior record at submission.

Source work occurs in a separate managed tree based on ESS f1af8280338b97d862a6c474ec50f78d5157d71c.
The established ESS orchestration tree remains the sole planning-store writer/lineage. Source workers
never mutate their copied planning store, and no journal is merged or concatenated. The accounting
worker's Cargo/lock/source edits are not shared; root serializes eventual integration and reruns
complete preservation gates against the combined source. Frozen baseline remains unchanged.

## Stopping condition and downstream boundary

The implementor closes after the exact crate, focused checks, two causal faults/restoration and
fingerprint handoff. Extra work needs a separately justified assignment. Design examination is
bounded to this first concrete interface, at most two passes on this unit; source examination uses
the recorded two-pass policy, never a renamed reset. Full integration closes this story, not M5.
M5 still includes ess-entity-runtime and missing ER semantics; M6 still requires SDK delegation and
actual durable billing/gatepass HTTP/restart acceptance. None is waived by pure extraction.

## Accepted concrete extraction interface

Final design SHA14ce1677ae0f201d0751e335e69eee91edbe4f8145080d9e4e03987991932159 is accepted
under the operator's approved ESS evolution step6 scope. review-result:service-contract-design-pass-2
holds original report53c7dfd744306af38e119d847c052ec6863939936368808ebd28dab2d8ee654d,
approve/findings[]: both first-pass blockers resolved, no new findings. Both design assignments
closed and their own leases were released; no third design round. This accepts a design, not code.

The pure source assignment owns only crates/specify/ess-service-contract/, workspace member and
dependency entries in Cargo.toml, and necessary Cargo.lock resolution. Root retains design/model,
CHANGELOG and sole canonical planning-store writes. A separate managed source tree avoids concurrent
Cargo/lock writes with the existing preservation tree; final integration is serialized and full
combined-source consumer coverage remains mandatory. No copied planning journal is merged.

Implementation contract: local-evidence:ess-evolution/waves/0009-service-convergence/service-contract-implementation-brief.md.
Test-first pure extraction, exact design fixture assertions, crate tests/Clippy/fmt/Rust1.85 and
two extraction mutation failures/restoration precede fixed author handoff and closure. Root owns
source examinations, full task check/site-build and integration. No ER/SDK/Eventlog work or later
service acceptance is delegated to this assignment. Additional work needs a separately justified
assignment after closure, never an extension to keep a worker busy.

## Pure extraction implementation submitted for source examination

The fixed implementation assignment is complete and its worker lease released. The new
ess-service-contract crate provides the accepted borrowed ServiceIr/extract API, full-plan admission,
selected surfaces and contextual capability closure. No persisted format, runtime execution or ER
admission is added by this pure unit.

The implementation changed only the new eight-file crate, two workspace manifest entries and the
new package's nine-line lock stanza. Root supplies the accepted design/model and changelog entry.
Library source SHA-256 28f3c1f42e94265c00dbb4296fea32f8dc6823dad662b2e9cfdf0f67b173accb;
integration source458978c6aa40c3f65f35c78e5ae4a32f955f55e9fd835ed6fd816427d83d62bc.

Actual Rust1.98.1 and1.85 crate suites each execute5integration cases; unit/doc lanes each have0cases.
Both compile their actual dependency closure. Strict all-target Clippy and scoped Cargo formatting
pass. The initial compiled red was an intentional unimplemented-extract panic. Two later compile-
valid mutations omit a selected event and substitute a foreign operation; each fails its concrete
assertion with101, the exact source is restored, and the complete five-case suite then passes.

Evidence: local-evidence:ess-evolution/waves/0009-service-convergence/service-contract-implementation/handoff.md
and its raw logs, original diffs and final hashes. Source examination, complete ESS accounting,
task check with consumers enabled, site-build and local integration remain required. M5 also retains
ess-entity-runtime, missing ER semantics and opt-in runtime/realization4; M6 retains real generated
service acceptance. This implementation handoff closes none of those broader outcomes.

## First source examination and bounded conversion correction

Independent source pass1 examined local submission
68f68ed172b65a1c70bb03ca1175e402edb0f37b. Its original report is recorded unchanged as
review-result:service-contract-code-pass-1, SHA
501b407e3ce4e25ac76ccdff7403fa9a7cc67d9db7aefdd74a00ddea2fab6c0d.
The single introduced blocker is measured: an included binding's compiled selection-input
preparation conversion is present in the synthesis plan, but absent from ServiceIr capabilities
and therefore its obligations. Final exact targeted case exits101; final full suite preserves
all five submitted cases green and adds one red. Reviewer fixture contamination was corrected
using only the new fixture paths and is explicitly excluded from product findings.

The implementation and first reviewer assignments are closed. A separately bounded correction
now owns only the pure crate's extraction closure and new exact regression/control files. It must
preserve original source inputs, plan order, dispositions and exact conversion identity, then pass
the agreed crate/MSRV/Clippy/format and correction-revert checks. The remaining second source
examination follows that corrected submission; no third design or source review is introduced.
This is not full M5 acceptance: ER semantics/lowering, SDK runtime4 and durable service acceptance
remain required. Exact local evidence: local-evidence:ess-evolution/waves/0009-service-convergence/.

## Corrected extraction submitted for final source examination

The bounded correction assignment is closed. Exact local bot submission
be604d874ee9e567ae104e565e7cbddf953885a9 includes the original review regression unchanged,
an unused-conversion control and the typed conversion-closure repair. Root verified all thirteen
recorded source/test/manifest hashes; the submitted source tree is clean. The correction's actual
seven-case crate suites pass on Rust1.98.1 and1.85, as do strict Clippy and formatting. One
compile-valid revert fails the original reviewer assertion with101; exact restoration passes.

The remaining final source examination uses a fresh reviewer and a separate managed checkout at
this exact commit. Its bounded contract covers the accepted pure extraction interface, targeted
new acceptance cases, full crate suite, scoped test formatting and one immutable findings report.
No third source/design review or provider work is authorized by this assignment. Full ESS gates
with complete consumer accounting, site-build and integration remain required before this story
closes. Local evidence: ess-evolution/waves/0009-service-convergence/service-contract-correction-1/
and service-contract-source-review-2-brief.md.

## Final source examination closed

The second and final source pass at be604d874ee9e567ae104e565e7cbddf953885a9 returned
nothing found, findings [], with the exact prior regression and exclusion control passing and
all seven crate integration cases passing. Scoped formatting passed; submitted files remained
unchanged and the review checkout remained at the exact commit. The original immutable report
is review-result:service-contract-code-pass-2. Its worker released its lease and closed.
Both source passes and the bounded correction are complete; no further source/design review
is assigned. This accepts the source examination only. The story remains active until required
complete ESS accounting, task check with consumer coverage, site-build and local integration.
The broader M5 and M6 requirements remain unchanged.
