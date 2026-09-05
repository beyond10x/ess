---
format: aep.planning-md/1
id: verification-report:review-boundaries-4-integrated
kind: verification-report
status: draft
title: Boundary wave 4 integrated verification
relations:
- verifies: story:review-semantic-diff-coverage
- verifies: story:review-infra-ir-invariants
revision: 2
---
## Subject and source

Clean combined source `acb7859e3202ffdc1ca840dde67f7ca4da33c746` merges infrastructure
unit `12d99b1414eea710e56a9776d32e898ff028f33a` and semantic unit
`21f274476c644be35bd9c7905d72211ae5f8682c`. The source remained frozen for the independent
SDK candidate experiment. Repository gate completion was observed at 2026-09-05T17:48:39.524976+00:00.
The CLI accepts whole-second evidence instants; the recorded instant is the next whole
second, 2026-09-05T17:48:40Z, rather than an invented rerun time.

## Unit verification

Infrastructure ownership is private, with shared queries and checked detached transformations
that remint handles through the existing document admission boundary. The projector admits
a candidate before recording emitted changes, and its callers propagate failure before writes.
Valid infra-ir/1 shape, canonical bytes and digest remain frozen. The five-package checks
rose from 244 to 265 cases during implementation and to 270 under independent review, all
passing. Public mutation probes and deliberate admission/transaction mutations first failed;
complete reports retain actual execution and setup failures separately. One full adversarial
pass found no defect. Full domain admission, unresolved-accounting completeness and cross-owner
handle identity remain outside this privacy repair.

Semantic comparison now accounts for eight omitted change families and surviving unexplained
content independently of classified changes. Default delta /2 and impact /3 use closed
vocabularies; exact legacy delta /1 admission, bytes and identifiers remain supported. The
graph has 26 relations, including actual served views and complete reusable row definitions.
Constructs identity carries slice-sha256/2 and strict emitted-envelope admission. Source and
whole-model identity, complete suite/4 bytes and neutral synthesis plans remain frozen.

The four semantic packages rose from 459 to 483 cases during implementation. Full adversarial
pass 1 added ten cases and found two defects through four failing witnesses; correction 1
passed all 496 cases. Full pass 2 added nine cases and found two new graph omissions through
three failing witnesses. Both earlier findings were resolved. The bounded final correction
retained all assertions and added two graph boundary cases; all 507 package cases, formatting
and strict Clippy passed. The coordinator inspected the complete correction and independently
executed all 17 named pass-1, pass-2 and final graph regression cases on its exact clean commit.
No third full adversarial pass ran. Both immutable review outcomes are fixed.

Final semantic regeneration measured 97 committed counterparts. Only two reserved Gatepass
OpenAPI contract-digest leaves changed in the last correction; the other 95 files, including
neutral plans, are unchanged. Original implementation regeneration and existing generated
drift are distinguished in the full reports. Actual Cargo and clap emitted provenance cases
pass; no generated clap compilation or individual-manifest impact caller is claimed. The
current CLI impact path does not supply a generated tree; the review's byte-verification
witnesses exercised the public library caller.

## Integrated gate

Each underlying task check step ran directly with its own observed exit status, followed
by the required site build and planning validation. Every lane returned zero:

```json
{
  "subject": "acb7859e3202ffdc1ca840dde67f7ca4da33c746",
  "lanes": [
    {
      "name": "fmt-check",
      "argv": [
        "task",
        "fmt-check"
      ],
      "exit_code": 0,
      "seconds": 1.0743982599815354
    },
    {
      "name": "clippy",
      "argv": [
        "task",
        "clippy"
      ],
      "exit_code": 0,
      "seconds": 6.770077908004168
    },
    {
      "name": "test",
      "argv": [
        "task",
        "test"
      ],
      "exit_code": 0,
      "seconds": 22.83891939697787
    },
    {
      "name": "doc-check",
      "argv": [
        "task",
        "doc-check"
      ],
      "exit_code": 0,
      "seconds": 7.076289146963973
    },
    {
      "name": "example-check",
      "argv": [
        "task",
        "example-check"
      ],
      "exit_code": 0,
      "seconds": 3.2968171010143124
    },
    {
      "name": "projection-check",
      "argv": [
        "task",
        "projection-check"
      ],
      "exit_code": 0,
      "seconds": 5.509232426004019
    },
    {
      "name": "release-check",
      "argv": [
        "task",
        "release-check"
      ],
      "exit_code": 0,
      "seconds": 0.1048332640202716
    },
    {
      "name": "action-check",
      "argv": [
        "task",
        "action-check"
      ],
      "exit_code": 0,
      "seconds": 0.04471106798155233
    },
    {
      "name": "site-build",
      "argv": [
        "task",
        "site-build"
      ],
      "exit_code": 0,
      "seconds": 15.511752648977563
    },
    {
      "name": "planning",
      "argv": [
        "aep",
        "artifact",
        "validate"
      ],
      "exit_code": 0,
      "seconds": 0.48592538002412766
    }
  ]
}
```

The workspace emitted 114 runner summaries: 1,579 passed, zero failed, zero ignored.
The site completed the WASM build, 21 browser-boundary claims, the deterministic 28-step
64-row lab run, pinned npm installation and successful Docusaurus build. Existing npm
advisories are retained in the raw output. No extra TypeScript feature lane was run because
this wave made no new target compiler change; that explicit lane passed in wave 3.

Cargo ran offline with each checkout's own target, task-owned TMPDIR, the coordinator-owned
sccache socket, incremental/debug disabled and CARGO_CACHE_RUSTC_INFO=0. No CARGO_TARGET_DIR
was set. Raw logs, exit files, exact argv, durations and timestamp are under
target/review-boundaries-4/integration. All introduced direct commits use verified bot
author and committer identities.

## Compatibility and publication boundary

The SDK compatibility experiment used identical SDK source
`48833c6d14ec37cb3b614fca05cf7dd78f63b743` on both sides. The old graph retains seven ESS
0.13.1 packages at `d1a66772a91b5411d942d7a45bbf08dfc5de4651`; the candidate's seven ESS
0.18.0 packages resolve to the exact frozen combined source. The full SDK graph has 260
packages, and the separate generated-service graph has 256. The targeted resolution
changes only those seven ESS package identities; no other package identity or record changes.

Both sides passed the three named SDK cases. Candidate selected package tests passed 26
cases, with formatting and strict Clippy exit zero. Both old and candidate generated services
passed actual offline, locked, all-targets Cargo checks in separate scratch workspace fixtures.
All 63 emitted files and the four generated Rust files were preserved. These are compilation
checks on a synthetic fixture, not generated-service execution or deployment.

Old-check-old, new-check-new and new-check-regenerated succeeded. Both cross-checkers
reported whole-file drift. The candidate changed 14 of 63 files, preserved 49, and added or
removed none. Regenerating a distinct copy of the old tree matched all 63 fresh candidate
files exactly. Canonical ESS IR, neutral synthesis plans, SDK runtime and realization data,
client/connectors/conformance controls, SDK HTTP OpenAPI and whole-model topology output
remained byte-identical. The upgrade spans ESS 0.13.1 to the current 0.18.0 source; the
deployment chart schema difference is not attributed solely to F01.

The first candidate metadata invocation retained the old locked graph and unused patches;
an explicitly authorized targeted offline update resolved the exact candidate. The first
strict Clippy invocation refused its lock before checking. Forwarding the same scratch
configuration after clippy passed with --locked, --offline and -D warnings and an unchanged
lock. That configuration adjustment worked; no child-process argument trace was captured
to establish the initial subcommand's internal forwarding. Both setup
attempts and successful invocations are retained separately. No source or dependency pin
was published in SDK. The compact compatibility report and exact raw evidence are retained
as `docs/reviews/2026-09-05-review-boundaries-4-sdk-compatibility.md` and the verified scratch
archives described by the wave page.


Atlas ADR 0036's initial decision is published at 974b2a2bc4896bd76293a734f36ac254895221c4,
and its final 26-relation clarification at 7b67e8e2437ec9956135930435875a8a76139c3f.
The consumer catalog-intent reconciliation obligation remains open independently of this
local experiment. No SDK pin upgrade, external deployment, release tag or version bump is
claimed. The broad ESS rollout obligation remains open for the separate conformance,
observation, number and other planned migrations.

This wave changes no public documentation allowlist source, so it introduces no Website
lock refresh. The full Atlas organization fence remains separately recorded with its known
sibling failures; an ESS green gate is not organization-wide convergence.

## Evidence preservation and cleanup

The completed infrastructure unit's 57 scratch files were archived and byte-compared
before its own target was cleaned. Archive SHA-256:
bb2bffce84680ed4c409c4f364e8bdc8c4269b510c3648d3c77a1734e8ee0115.
The semantic unit's 196 scratch files were archived and byte-compared before its own target
was cleaned. Archive SHA-256:
02582e161d1588cf9643ad17967d06d5e9243ac3333531f88f403f4a486f5eed.
Both clean source trees remain managed and active until wanted commits have advertised
recovery proof. Publication precedes lease end, finish, full-profile GC review and exact-id
removal. SDK and coordinator evidence are preserved before their respective cleanup.
The original dirty review/outlook tree, original PDF and unrelated worktrees are preserved.
