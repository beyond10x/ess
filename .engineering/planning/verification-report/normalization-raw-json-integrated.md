---
format: aep.planning-md/1
id: verification-report:normalization-raw-json-integrated
kind: verification-report
status: draft
title: Raw JSON normalization integrated verification
relations:
- verifies: story:raw-json-normalization-provenance
revision: 3
---
## Outcome

Format 4 raw JSON normalization preserves explicitly selected token bytes as
canonical base64, with checked field/items/root selectors and explicit retained
document helpers. It retains grammar, Unicode and original-root depth rules while
preserving duplicate members and huge number spellings inside captures. Decoded
value APIs refuse capture branches without original lexical provenance. Supported
reference, Rust and Go entrypoints implement the binding in
docs/design/raw-json-normalization.md. TypeScript remains a separate target story.

## Source and integration

Source unit 3e2eb52e84f5ed7b94381076270d4cdc6f2970fd has both the organization bot
author and committer. It incorporates the green implementor handoff and the
test-only adversary addition. The reviewed complete working-tree file manifest was
a8415a4ff0d688840ec384000a5393b1d6ccb6d980e1ad4ae66bf83d23a4a6e9.
The coordinator merged it with current main1667d022ed2041342c8928250ab8bddfe9c988b9,
preserving the independent expression/OpenAPI work. One changelog conflict retained
both entries. The coordinator also corrected the CLI help's supported recipe list
to include format4; no additional runtime correction was needed.

Planning changes were privately snapshotted before incoming main was fast-forwarded,
then replayed through AEP. The canonical incoming journal was retained; no textual
journal union or direct planning-file edit occurred. The Binary64 design is a bound
next-unit document, not an implementation included in this result.

## Review

review-result:normalization-raw-json-adversary-pass1-public records the exact
returned public report, with its declared filesystem path normalization. Original
private logs remain in assigned scratch. The public body SHA-256 is
f72d85491d317d06871e18b178422f543a82c380753c6f18a351edf5c8bef385.
Its findings list is empty; review_outcome is no-op. The adversary added four
top-level tests and 42 document-derived lexical cases. Reference, generated Rust
default/arbitrary_precision and generated Go race execution passed. No human
approval or verifier independence is claimed for an agent report.

## Integrated gates

The actual combined working source passed:

| Command | Exit | Seconds | Finished UTC |
| --- | --- | --- | --- |
| task check | 0 | 96.098 | 2026-09-06T09:06:33Z |
| focused raw/native tests with go-typecheck | 0 | 7.985 | 2026-09-06T09:08:07Z |
| task site-build | 0 | 16.164 | 2026-09-06T09:08:23Z |

The workspace gate reports 1,894 passing Rust cases in 150 summary groups, zero
failed or ignored. Its full formatting, Clippy, test, documentation, example,
projection, release and action checks exited zero through the literal task.
The native lane executes six top-level tests: 149 Go corpus cases plus UTF-8 and
composition controls; 156 Rust cases in each configuration; the additional 42-case
adversarial matrix in reference, Rust and Go. Seven reference/Rust decoded-value
cases do not apply to Go, which exposes no decoded-value API. The separate complete
legacy maps and existing native fixtures retain old-format behavior and the
2,490-case base64 qualification.

An initial additional native invocation exited101 because the coordinator omitted
ESS_GO_COMPILER. Its Go test stopped before execution, while the other three
adversary tests passed. Repeating the focused lane with ESS_GO_COMPILER=/usr/bin/go
passed; no source or assertion changed. Both results are retained. The site task
ran only after the corrected native lane passed. No failed process was reported
as a successful pipeline through a shell tail.

Exact logs and machine-readable exits are under the assigned recovery scratch's
ess-raw-json/gate directory. The implementor and adversary scratch roots retain
their initial red cases, original logs and per-file fingerprints. A suspected
legacy Go depth-cap difference remains source evidence without a separate legacy
native reproduction; it is not a confirmed introduced regression.

## Resources and workflow limits

The unit used managed tree wt-c12a5474a249, branch impl/normalization-raw-json,
base60ffcb2. Integration uses managed tree wt-bf45625a6a50, branch
impl/normalization-base64-resume. Assigned raw scratch is outside the worktrees.
The coordinator's existing target was reused serially during implementation and
review, a recorded deviation from the wave rule requiring a separate build
directory per tree. No simultaneous unit builds used it. Future units receive
their own target directory. At the integrated-gate checkpoint the coordinator
target occupied24,382,360KiB and available filesystem space was22,184,230,912bytes.
Builds used four jobs; the next unit must preserve the8GiB reserve.

The raw unit's initial state was recorded in its story and private brief, without
a separate pre-dispatch wave page. This integration record preserves the missing
recovery details now; it does not invent a prior proposal or approval event.
The original full-gap continuation authorizes the work. The next serial unit
will have its scoped page before dispatch. Harness-specific typed agent roles
were unavailable; generic agents explicitly loaded the implementor/adversary
charters. Token/cost totals are unavailable and are not reported as zero.

## Publication boundary

This record first captures the local integrated gates. Raw unit main publication,
CI and Atlas source-set delivery are recorded separately when observed. No release
tag or version bump occurred at this checkpoint. Model Binary64, TypeScript,
positional decoding and adopter consumer decisions remain open.

## Published source

Published main6c78676c35193423fe326b9dde21b8fc21681b8a contains source unit3e2eb52
and the combined verification/planning record. The clean primary checkout was
fast-forwarded to that commit. Both direct commits have the exact organization bot
author and committer and were pushed through the intact Atlas bot wrapper.

Exact-head CI34023996607, documentation validation34023996589 and passive source
bundle34023996599 completed successfully:
https://github.com/beyond10x/ess/actions/runs/34023996607
https://github.com/beyond10x/ess/actions/runs/34023996589
https://github.com/beyond10x/ess/actions/runs/34023996599

A normal Atlas source-set reconciliation was requested after observing the green
bundle. Its final publication/provenance will be recorded after completion; a
successful dispatch alone is not delivery evidence. No version bump or tag yet.
