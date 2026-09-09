---
format: aep.planning-md/1
id: review-result:cli-schema-metadata-decision-r1-20260909
kind: review-result
status: active
title: Exact CLI schema metadata accounting design review
relations:
- reviews: story:cli-presentation-binding
revision: 1
---
approve

Independent design review, 2026-09-09. No concrete blocking findings.

Reviewed candidate: docs/design/cli-schema-metadata-accounting.md,
SHA-256 d6b6d4d8f1d15d3416008ad5ddb61c48ab4f18920b4ae7f0cd1b731eec4c27e1.
This is a policy/accounting migration review, not a third attack on the CLI
implementation. I read the owning instructions, both prior coverage decisions,
and the current account, enforce, proposal, mod and wire implementations. I
also inspected the relevant native proof/authority boundary and actual three
CLI profile definitions. No Cargo command, test, AEP mutation, implementation
change or worktree lifecycle operation was performed. Only this report was
written.

The candidate makes the necessary policy change explicit. The existing policy
requires every inventoried wire node, including the root dialect and definitions
container, to participate in the matrix (docs/design/review-consumer-coverage.md:
49-66), but its current closed algebra contains only Supported, Refused and
BaselineUnknown (:126-149). The authored-pipeline decision explicitly left that
algebra unchanged (docs/design/cli-consumer-pipeline-coverage.md:3-6). The new
candidate acknowledges both facts (:3-6), adds a separately named nonbehavioral
relationship (:10-16), and preserves the complete matrix. It therefore does not
misrepresent an unknown authored-field refusal as CLI enforcement of a schema
document or relabel enumeration as behavioral evidence.

The admitted scope is finite and reviewable: two literal node IDs times the
three literal current CLI consumers, exactly six cells (candidate:24-41).
The current profile entries do describe authored input pipelines
(crates/edge/ess-xtask/src/consumer_coverage/profiles.json:3-42), consistent with
the earlier decision (:14-30). The manifest alone cannot admit more pairs: the
guard must independently require that exact six-row set (candidate:45-50).
Descendants and fourth profiles remain ordinary obligations. This preserves
the prohibition on automatic NotApplicable, live prefixes and package-wide
aliases in the existing policy (:163-169).

The proposed evidence boundary is sufficient for the stated bookkeeping claim.
Fresh schema output already comes from the compiled RawSpecFile provider after
source/build checks (consumer_coverage/mod.rs:45-98). The current wire extractor
checks the exact dialect and definitions-map representation (wire.rs:29-52),
recursively hashes complete structural subtrees (:161-175), and resolves local
references against the same schema (:177-237). Reusing that extractor and
comparing its complete obligations/reference inventory with fresh provider
output (candidate:52-66) preserves the existing authority instead of trusting a
saved schema file. The candidate also requires an opaque executed proof,
current-run authority, exact planned/proved identities and fingerprints, and
pre/post verification (:64-74). That is consistent with the existing private
VerifiedCases construction and repeated source/provider/build checks
(native.rs:25-65). One executed guard for six metadata cells is accurately
reported separately from behavioral cases; it does not invent six tests.

Declaration fingerprints are not whole-call-graph evidence. The candidate
explicitly preserves that limitation and assigns the semantic relationship to
review (:76-79), matching the existing policy (:24-27, :324-337). The guard
establishes current schema representation and reviewed boundary identity. It
does not claim to prove every possible future private implementation ignores
JSON Schema.

The format consequence is appropriately explicit. AGENTS.md:45-49 requires a
format migration when persisted meaning or the envelope changes. The current
candidate cells and planned dispositions are closed Rust types
(account.rs:7-42; enforce.rs:47-73), while qualification currently reads an
unversioned plan (enforce.rs:255-284). Introducing the first named accounting
envelope, closed stages and explicit legacy/unknown/stage refusals
(candidate:95-108) addresses the actual compatibility boundary. Keeping
Behavior and ClaimedBehavior unchanged (:83-85) prevents bookkeeping metadata
from entering ordinary behavioral claims. Old closed readers must reject the
new disposition, and new typed qualification must reject legacy receipts and
wrong stages; the candidate explicitly requires those tests (:117-124).

Baseline preservation is concrete. The candidate pins the accepted bytes and
all 87 existing profiles (:35-39, :105-108); the current enforcement path already
checks the same baseline SHA-256 (mod.rs:401-406). I independently read the file
hash as 3dd8dff59335c8a77c93c2734118566fd1b2d5165c0590d0aa9be397374a47de.
This review does not claim that the final regenerated profile fingerprints or
full matrix have already been verified. Those remain implementation evidence.

Approval is for the bounded design decision and its additive policy amendment.
It does not qualify the six cells in advance. The prescribed missing/stale/
extra/contradictory row, authority, reference, descendant, count-conservation
and format tests remain required, followed by fresh same-source qualification
and the full task check and task site-build gates (candidate:117-130). No
broader applicability framework, baseline expansion or ESS product-format
change is needed to implement this decision.

```findings
[]
```
