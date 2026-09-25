# Finite reconciliation of changed consumer obligations

Status: accepted design for story:consumer-accounting-baseline-never-extended, 2026-09-15.
This corrects the full local gate required by ESS evolution; it does not broaden initial unknown
eligibility or qualify every historical unknown. The existing finite-baseline epic remains separate.

## Measured defect

At source 41da2281e99402602c25d8faf219fc75b1954d04, the current extraction has 2,421 model IDs and
90 profiles. Relative to the frozen initial dictionary, 618 IDs were added, 62 changed and ten
removed; two original consumer signatures changed. The execution plan reports 55,590 unaccounted
cells and 9,746 stale cells. Its 147,931 BaselineUnknown count is current eligible unknowns, not
the number of unaccounted pairs. No behavioral cases executed in that refused run.
Evidence: local-evidence:ess-evolution/waves/0000-scope/consumer-accounting-analysis-result.md and
verification/scope-20260915/task-check-1.log, actual task exit 201. Current scope/design edits did
not change the Rust, model inputs or executable accounting authorities that produced the failure.

The source has no explicit retirement/supersession path. `consumer_coverage/enforce.rs` first
inserts every initial unknown and then current claims. A valid new claim for a changed old key
duplicates that key; a removed model cannot have a current claim. Deleting or refreshing initial
rows would erase the frozen authority instead of reconciling it.

## Authority and exact allowed transitions

Keep initial-baseline.json and its existing digest pin byte for byte. Preserve its owners,
follow-ups and original eligible pair count. New or changed model/profile obligations require
Supported or named Refused behavior, except the six already accepted metadata relationships and
the exact aggregate proof modes in [the applicability amendment](consumer-accounting-applicability.md).
Its eight acquisition profiles retain separately mandatory executed obligations and source inventory.
The narrow representation-only Number exception does not cover these wire and semantic changes.

Add a separate closed Rust authority `ess-consumer-reconciliation/1`. It pins the exact original
baseline digest and contains a finite ordered set of decisions, each naming the old model ID,
shape hash, consumer ID and profile hash, source evidence and review reference. No wildcard, live
prefix, default rule, guessed owner or generated current-source eligibility is permitted.

- Retire: that exact eligible old cell exists, and its model or consumer no longer exists in the
  current inventory. The retirement records historical accounting only and grants no replacement
  support. If the identity is current again, retirement refuses.
- Replace: that exact eligible old cell exists and at least its shape or profile changed. Name
  the exact current identity and the exact current behavioral claim/cases that replace it. That
  claim must be Supported, named Refused or a qualified exact AggregateClosure under the companion
  amendment. BaselineUnknown and metadata cannot replace it; changed profiles inherit no unknown.
  Identity renames are an explicit retirement plus an ordinary new claim, not an inferred match.

Reject missing/extra/duplicate decisions, wrong old or new hashes, wrong baseline digest, absent
review/evidence references, unchanged replacement identities, unexecuted replacement behavior and
retirement of a current identity. Each decision must be consumed exactly once by the matched old
cell; extra decisions do not disappear. Unaffected initial unknowns remain exactly unchanged.
All other current cells still need ordinary accounting. Candidate enumeration never creates or
accepts a reconciliation decision, and the gate never writes these root-reviewed authorities.

Planning validates the exact finite transitions before inserting current behavioral claims.
Superseding an old unknown is allowed only together with its exact replacement claim. Execution
and qualification still require the attributed cases under the actual current source/profile;
a review reference or a syntactically valid manifest is not executed evidence. The gate retains
its compiled-provider, source/binary, case-AST, native-runtime and metadata guard checks.

## Format consequence

This changes persisted accounting provenance, so introduce `ess-consumer-accounting/2` for the
new pipeline's candidate, execution-plan and qualified outputs. Retain the meanings and closed
readers of v1 historical artifacts; never relabel an old receipt as v2 or accept unknown fields in
v1. Add an old-reader rejection test before the new envelope. Existing ESS IR, schema, service and
native-case formats are unchanged.

The v2 execution/qualification authority binds the initial baseline digest and reconciliation
manifest digest and records exact retired/replaced identities. Count current cells separately
from retired historical cells. Current totals still conserve Supported + Refused + BaselineUnknown
+ SchemaDocumentMetadata + AggregateClosure across the current model-consumer matrix. Acquisition
obligations are separately required and counted; stage outputs cannot claim
qualification before required case/guard execution. Old unknown count is never reused as a current
count after retirement/replacement. No AEP library or planning-store read enters ESS: review/source
references are evidence metadata and root records their backing through the owning store.

## Behavioral evidence and implementation order

First review the exact finite source differences and group them by actual consumer boundary and
assertion. A meaningful case may qualify several explicitly listed pairs, but aggregate package
success, one package-wide alias or a live prefix qualifies none. Existing tests may be reused only
when their changed/control assertions, causal sensitivity and finite attribution actually cover
each claimed obligation. A no-effect claim must vary the relevant input and assert exact preserved
output/runtime facts at that specific consumer. Panic or arbitrary failure is never Refused.

Preserve existing exact claims and the six metadata rows. In particular, the two changed CLI
profiles require evidence for retained noncompact behavior, compact IR bytes and the named
compact/target refusal at the attributed entrypoint. Do not silently downgrade native execution to
emission or drop tool-specific profiles to make the matrix complete.

Implement and review the finite authority plus attributed current claims/tests. Decisive red tests
use a synthetic baseline with an unchanged unknown, changed shape/profile and removed identity:
replacement must fail under the old duplicate/stale behavior. Removing authorization/cases,
changing either hash, reviving a retired identity or changing a reviewed current signature must
fail. Bypassing reconciliation validation must be caught by those negative controls.

Once the finite current accounting is complete, run the actual task consumer-check, requiring
case/guard execution and a completed qualification receipt with zero stale/unaccounted cells.
Then run default task check and task site-build. Keep consumer coverage enabled. Partial framework
or case-group progress is not acceptance of this story or the ESS scope changes.
