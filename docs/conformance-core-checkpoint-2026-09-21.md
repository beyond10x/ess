# Conformance core release candidate — 2026-09-21

Version 0.28.0 is prepared on `feat/conformance-core-semantics`, based on
`809d7e31c3b22840643d56f47768fc22d605048a`. It has not been released or integrated into main.

The candidate adds ess/6 input-eligible external outcomes and observed enum subject history,
plus explicit silent subject preservation. Suite/10 and its coverage counterpart /11 carry
before/after snapshots and no-error assertions. The bounded designs and mutation witnesses are in
`docs/design/guarded-external-outcomes.md`, `docs/design/observed-subject-history.md`, and the
`guarded_external` / `subject_history` integration tests, which execute emitted Go and TypeScript.
Legacy format refusal and canonical projection checks remain binding.

The approved missing planning epic was recovered exactly from commit `16fb2cfe`:
SHA256 `622634c42b46e80dd6d811dc30cd3a6b43a69921b01320f1de5c894d491df9dc`.
The two stories and `task:release-0-28-0` remain active pending integration and publication.
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

The repository's existing CI/release workflow explicitly excludes consumer accounting; local
`task check` retains it. An operator decision on applying the CI profile to this release is pending
as `decision-blocker:local-release-gate-profile`. Until that decision or complete qualified
accounting, do not report the full gate green, merge this candidate, or push its release tag.
Branch publication for review does not qualify a release. No consumer download pin has changed.

Run the native consumer lane without `CARGO_TARGET_DIR`, under its reviewed toolchain and flags.
A different target directory is refused before accounting. The standalone semantic, runtime and
metadata tests do not substitute for that matrix's qualified cases.
