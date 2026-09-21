# Conformance core release verification — 2026-09-21

Version [0.28.0](https://github.com/beyond10x/ess/releases/tag/0.28.0) is published.
[PR #56](https://github.com/beyond10x/ess/pull/56) integrated
`68581d70bb47a048cd399e55c68f225b80303977` into main before the organization bot created the
annotated `0.28.0` tag at that same commit. The base was
`809d7e31c3b22840643d56f47768fc22d605048a`.

The candidate adds ess/6 input-eligible external outcomes and observed enum subject history,
plus explicit silent subject preservation. Suite/10 and its coverage counterpart /11 carry
before/after snapshots and no-error assertions. The bounded designs and mutation witnesses are in
`docs/design/guarded-external-outcomes.md`, `docs/design/observed-subject-history.md`, and the
`guarded_external` / `subject_history` integration tests, which execute emitted Go and TypeScript.
Legacy format refusal and canonical projection checks remain binding.

The approved missing planning epic was recovered exactly from commit `16fb2cfe`:
SHA256 `622634c42b46e80dd6d811dc30cd3a6b43a69921b01320f1de5c894d491df9dc`.
The two stories and `task:release-0-28-0` are implemented, with exact-tag release evidence recorded.
An internal adopter retains its own full reports and named regression comparisons; these are not
public release evidence and carry no public source paths here.

## Verification

Formatting, strict Clippy, workspace tests, native repository tooling tests, documentation,
examples, projections, source support, fuzz replay, release metadata, action checks and the site
build pass. Release preparation initially exposed a documentation test that still treated ess/6
as unreleased. Its future-version fixture now names ess/7; all 147 tooling unit tests and their
integration targets pass after that correction. No behavioral assertion was removed.

The full local gate is not green: required consumer accounting was executed and refuses:

```
unclassified concrete consumer entry ess_cli::bin(ess)::enum::SuiteTarget/variant/Typescript; finite review required
```

Its accounting diagnostic additionally names:

```
unknown claimed model wire:RawSpecFile#/definitions/NamedType/oneOf/2/properties/variants/items/type
```

The Typescript declaration is present and its classification absent at the base commit above.
The NamedType schema and reviewed-candidates manifest are unchanged from that base. Read-only
inventory comparison found 140 unclassified concrete entries, including inherited additions.
New semantics also require qualification; neither all debt nor all diagnostics are claimed to
predate this candidate. No classification exemption, unknown eligibility expansion or behavioral
claim was added merely to make this gate pass.

The new RawOutcome/RawSubjectField schema shape invalidated three root-container metadata rows.
Those exact rows were re-reviewed with unchanged guard/profile identities; the fresh provider's
metadata guard passes. No descendant was exempted.

## Publication boundary

The repository's existing CI/release workflow explicitly excludes consumer accounting; default
local `task check` retains it. After the distinction and refusal were explained, the operator
explicitly instructed merging and tagging PR #56 on 2026-09-21. The bounded 0.28.0 release is
authorized under that existing CI profile; `decision-blocker:local-release-gate-profile` records
the approval and is cleared. This does not claim that the full default gate passes and does not
change the consumer-accounting baseline or the default gate configuration.

The exact tag commit passed `task check SKIP_CONSUMER_CHECKS=true`, `task site-lab`, and the shared
security/privacy check before tagging. [Release workflow 35639713420](https://github.com/beyond10x/ess/actions/runs/35639713420)
passed its required gate, browser checks and all four native package jobs. The published release's
four archives were downloaded and verified against `SHA256SUMS` and their API digests. The Linux
x86_64 binary reports `ess 0.28.0`. `task release-status` confirms 38 published version tags, each
on main and backed by a release and dated change record. Documentation publication is asynchronous
and was not used as a source-release completion condition.

Run the native consumer lane without `CARGO_TARGET_DIR`, under its reviewed toolchain and flags.
A different target directory is refused before accounting. The standalone semantic, runtime and
metadata tests do not substitute for that matrix's qualified cases.
