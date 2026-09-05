---
format: aep.planning-md/1
id: verification-report:review-boundaries-3-integrated
kind: verification-report
status: draft
title: Boundary wave 3 integrated verification
relations:
- verifies: story:review-persisted-delivery-validation
- verifies: story:review-typescript-root-collision
- verifies: story:review-conformance-format-design
revision: 1
---
## Subject and source

Tested clean integration source 1ef817ea9d51b2e927febce18eb2d540256e8e76 combines delivery4289e3eb636a97b73f8261dd6d68f3027afe9f65, TypeScript96a6c6626fd10df3f49dfe85768d561963da9ce1 and design6acd811a8f53a3e5ab56ec7a68a13fd0a727ba2b. Completion was observed2026-09-05T14:26:23.918501Z. Closing changes record planning/evidence/lifecycle; production, tests and binding design remain unchanged after this gate.

## Unit evidence

Delivery package cases rose53 to73 across implementation and independent review (deployment21, CLI52). The implementation first executed11 meaningful cases with9 failures; setup/compiler errors are separately recorded. Final checked readers cover BuildIr, RuntimeIr, ComponentIr, ReleaseManifest, ReleaseBundle, ReleaseCatalog, StackLock and DeploymentIr, including raw duplicate map keys and recoverable graph/order/digest relations. Public mutable values are revalidated at consuming boundaries; CLI desired/current admission precedes analysis and fake-executor calls. Six independent review additions passed on first isolated execution, followed by both package test/fmt/strict Clippy gates. No review finding remained. Compiler-produced valid controls and unchanged valid canonical bytes remain covered.

TypeScript default package cases rose9 to19; the explicitly selected real compiler lane adds6 cases. It rejects normalized root/definition collisions, illegal export bindings and actual Array helper shadowing while preserving wire property spelling and established normalization. The independent review added6 cases across the default and compiler targets and found nothing. TypeScript6.0.3 was selected through ESS_TYPESCRIPT_COMPILER=/usr/lib/node_modules/typescript/lib/tsc.js; the selected missing-compiler probe failed all3 implementation-stage cases rather than skipping them. This feature is not wired into default CI and no such claim is made.

The design is document-only. Two immutable reviews found3 then2 introduced issues, with carried0/new2/resolved3 in the exact CLI comparison. The first correction preserves scenario/refusal coexistence, source identities and complete suite/report pairings and fixes the impact format citation. The bounded final correction preserves every refusal occurrence and defines exact u64 timestamps for new report/run contracts with checked adapters and frozen legacy behavior. Coordinator personally read the bounded diff and source-backed cases and independently confirmed all68 earlier matrix/pairing rows unchanged. Final design has75 matrix rows and10 pairing rows. No third full attack or runtime compatibility execution was claimed. Both review outcomes are fixed; the design does not discharge its dependent implementation or downstream rollout obligations.

Original implementation/review/correction reports and patches remain in docs/reviews and immutable review-result artifacts. Reports were preserved before routing. All directly introduced commits through the tested source were checked for exact bot author and committer.

## Integrated gate

Every underlying task check step ran with its own observed exit status on the clean combined source:

```text
fmt-check 0
clippy 0
test 0
doc-check 0
example-check 0
projection-check 0
release-check 0
action-check 0
site-build 0
typescript-compiler 0
```

The workspace emitted110 runner summaries totaling1505 passed,0 failed,0 ignored. The feature-selected schema-contract run emitted4 summaries totaling25 passed,0 failed,0 ignored;19 cases overlap the default workspace run and6 are additional compiler cases. All selected cases executed. The site task completed its WASM/browser checks, pinned npm installation and Docusaurus build with '[SUCCESS] Generated static files in "build".' Existing dependency audit/install-script advisories remain outside this wave's acceptance statements.

Raw logs, individual exit files, exact commands, durations and timestamps are under target/review-boundaries-3/integration. Each task was run directly without a pipe hiding its exit status. CARGO_NET_OFFLINE=true governed Cargo; npm retained the required network access for its pinned dependency install. TMPDIR stayed in the coordinator target; RUSTC_WRAPPER=/usr/bin/sccache, incremental0, dev/test debug0 and CARGO_CACHE_RUSTC_INFO=0 were set. No CARGO_TARGET_DIR was set and no build target was shared.

## Resource handling and cleanup evidence

Free space before the integrated gate was140240187392 bytes, above the8589934592-byte reserve; coordinator target was1890908 KiB and website/node_modules661020 KiB. The dedicated foreground cache's original session ended normally at its idle timeout. Its replacement used the same task-owned socket with SCCACHE_IDLE_TIMEOUT=0 and answered --show-stats before review resumed. Failed startup attempts executed zero cases. No unrelated default cache server was stopped or cache purged. The coordinator owns exact socket shutdown after all uses.

Wanted unit scratch was archived and byte-compared: TypeScript50 files/130782 bytes, delivery66 files/307256 bytes, design11 files/141659 bytes. Archives remain under coordinator target/review-boundaries-3/archive; portable engineering evidence is committed. Publication must precede cargo clean, lease end, worktree finish, fresh full-profile dry-run and exact-ID cleanup. The original dirty architecture review/outlook tree and PDF are separate user work and remain preserved.

## Compatibility and practical limits

Valid delivery writer bytes are unchanged; admission establishes only invariants recoverable from the supplied envelopes. It does not reconstruct omitted ESS/realization inputs, verify OCI origin/evidence authenticity, prove conformance truth or implement execution recovery. Those have distinct stories. No live cluster, real ORAS/Helm service or credentials were used by delivery tests. TypeScript uses a separately selected local compiler feature. Conformance suite5/report2/run2 remain designed future contracts, not current writers or deployed consumer compatibility.

This wave changes no public documentation allowlist source and needs no Website lock refresh. The clean Atlas authority7b00adf3b1004e0cdd8dd12aa4fa8cc8435a0432 equals current remote main; its last organization fence remains red on separately documented sibling issues. Required fences will run again when ESS main moves. No organization-wide convergence, release tag or version bump is claimed.
