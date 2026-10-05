---
format: aep.planning-md/3
id: story:feature-request-433
kind: story
status: active
title: Regenerating ESS output fails on macOS with a provenance error
tags:
- ess-0.54.0
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#433
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
scope:
- confidence: cited
  path: .github/workflows/ci.yml
- confidence: cited
  path: crates/edge/ess-cli/src/output_ownership/filesystem.rs
- confidence: cited
  path: crates/edge/ess-cli/src/output_ownership/mod.rs
- confidence: inferred
  path: crates/edge/ess-cli/tests/output_ownership.rs
- confidence: cited
  path: website/docs/guides/generate-artifacts.md
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-10-05T13:26:29Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "proposed", to: "active", at: "2026-10-05T13:26:29Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"review_outcome":2}}}
---
## Outcome
Resolve beyond10x/ess#433: Regenerating ESS output fails on macOS with a provenance error.

## Origin
beyond10x/ess#433, filed 2026-10-05; an adopter's report (2026-09-29) of a macOS regeneration refusal, recorded without the `ess` version, the command or the error text.

## Fit review
1. Need: regenerating committed output on macOS must succeed wherever it succeeds on Linux, and a refusal must name the file it refuses and the `ess` that refused. No authored syntax is involved. The reporter's data is missing, so the mechanism below is a hypothesis, not a finding. Candidate paths that print "provenance":
   - Output ownership refuses any extended attribute on an existing output that the platform did not impose (`crates/edge/ess-cli/src/output_ownership/filesystem.rs:441-446`). Only the exact Linux labels `security.selinux` and `security.SMACK64` are admitted, and on macOS nothing is (`filesystem.rs:458-460`, `cfg!(target_os = "linux")`). I believe macOS attaches `com.apple.provenance` to files written by some processes; on such a file the refusal would read `output has extended metadata outside the ordinary snapshot contract: com.apple.provenance`. Not verified on macOS.
   - `ess verify impact` owes `ProvenanceUnreadable` / `ContractMismatch` (`crates/verify/ess-diff/src/impact.rs:1110-1124`, rendered at `render.rs:218-233`). Neither depends on the platform.
   - `--target interpreted` spec_digest refusal (`crates/edge/ess-cli/src/main.rs:4003-4011`). Also platform-independent.
   Probe on the installed ess 0.52.0, Linux (`<fit-review scratch>/probe-433/`): `ess generate --path spec --out out`, then `setfattr -n user.probe` on one output, then the same command again → `error: output has extended metadata outside the ordinary snapshot contract: user.probe`, exit 1. The refusal names the attribute but not the file, and not the `ess` version. Every `filesystem::image` caller propagates with `?` and no path context (`output_ownership/mod.rs:186,222,364,388,413,490,496,589,734,960`).
   Requester's proposal (theirs): "a provenance refusal names the version and the file it disagrees with".
2. Class: defect. The ownership contract covers "one local mounted filesystem on Linux or macOS" (`website/docs/guides/generate-artifacts.md:102-103`), so refusing an attribute the OS imposes on every file contradicts the documented support. The hypothesis matches the Linux precedent: SELinux labels were admitted for this reason in 391b2651c0 (2026-09-10). The unnamed file is a defect whatever the mechanism turns out to be.
3. Existing idiom: for "which `ess` produced this output", `requires: ess X.Y.Z` in `ess-inputs/2` plus `ess specify toolchain install --pin` (`website/docs/reference/cli.md:66-68,82-98`, `crates/edge/ess-cli/src/requires.rs:1-4`). Stamps record system, specification version and two digests, and no tool version (`crates/generate/ess-gen/src/provenance.rs:23-46`). That is deliberate, because a tool version in the stamp would rewrite every committed artifact on every release. The idiom answers the version half of the report.
4. Fit: (a) Wrap every `filesystem::image` / `ordinary_metadata` refusal with the output-relative path and `ess <CARGO_PKG_VERSION>`. Same sentence, more context, and `user.*` and other foreign attributes are still refused (`filesystem.rs:737-799` stays green). (b) Add exact Darwin system labels to `platform_xattr` the way the Linux ones were added, by exact name and never by `com.apple.*` namespace (`filesystem.rs:455-457` rationale). This lands only when (c) shows the refusal on macOS. (c) Add a native case to the existing macOS lanes, `output-ownership-macos` on `macos-15-intel` and `macos-15` (`.github/workflows/ci.yml:430-470`): regenerate over an output carrying `com.apple.provenance`. Today those lanes test ownership on fresh runner checkouts. Packaging only runs `--help`/`--version` (`.github/workflows/package.yml:122-145`). Whether a test process may set `com.apple.provenance` itself: I don't know. If it is refused, the test reports a skip by name rather than passing silently.
5. Second adopter: a contributor clones a repository whose committed `generated/` came from Linux CI, opens it on macOS with an editor that writes provenance-tracked files, and runs the generator's `--check`, or regenerates, before review. Same refusal, no local policy.
6. Cost: no format bump, no authored keyword, no new diagnostic code. The refusal text changes on stderr only. The macOS lanes get one more test binary run. If (b) lands, the snapshot contract admits one or more exact Darwin names. Any `com.apple.*` name that changes execution (quarantine) stays refused unless the native test shows it is OS-imposed on ordinary outputs.
7. Alternatives: change nothing until the reporter answers. That loses a defect (the unnamed file) that is real now. Stamp the `ess` version into every artifact: refused, it churns committed bytes per release and `requires:` already answers this. Admit all `com.apple.*`: refused, it is the namespace admission 391b2651c0 rejected. Chosen: (a) now, (b) gated on (c).

## Decisions
accept, redesigned. Build (a), a refusal naming the output-relative path and the `ess` version, and (c), the native macOS case on the existing ownership lanes. Write (c) first. Land (b), exact admission of an OS-imposed Darwin label, only if (c) is red on a macOS runner without it. The story closes when (c) is green on both macOS lanes, with or without (b). Reply to the reporter: ask for `ess --version`, the command and the error text; name `requires: ess X.Y.Z` as the way to record the producing release; say that the next release names the file in the refusal. Stamp bytes do not change. No `ess/23` or `ess-conformance/N` bump.

## Acceptance
- foreign_attribute_refusal_names_output_path_and_ess_version: an output carrying `user.ess_test` is refused with its output-relative path and `ess <version>` in the message, and nothing is written.
- platform_label_admission_stays_exact: `security.selinux` / `security.SMACK64` admitted on Linux only; `user.*`, `security.capability`, ACLs and `com.apple.quarantine` still refused (extends `only_exact_platform_access_labels_are_ordinary`).
- macos_regenerates_over_os_provenance_label: on `macos-15` and `macos-15-intel`, regenerating over an output carrying `com.apple.provenance` exits 0 and leaves the output unchanged. This one case decides the story: it must pass on both lanes. A refused `setxattr` of the test's own label fails the case; it is never a skip.
- requires_pin_answers_producing_release: the section `## Repeated generation and recovery` of `website/docs/guides/generate-artifacts.md` contains "`requires: ess X.Y.Z`" and "producing release" and links `specify/layout-and-validation.md`. Neither phrase is on the page today. A case of that name in `crates/edge/ess-cli/tests/output_ownership.rs` reads the section and fails naming the missing phrase or link.

## Scope
- crates/edge/ess-cli/src/output_ownership/filesystem.rs  cited — `ordinary_metadata`, `platform_xattr`, xattr tests
- crates/edge/ess-cli/src/output_ownership/mod.rs  cited — `image` callers that drop the path
- crates/edge/ess-cli/tests/output_ownership.rs  inferred — native regeneration case run by the macOS lane
- .github/workflows/ci.yml  cited — `output-ownership-macos` lane
- website/docs/guides/generate-artifacts.md  cited — filesystem contract and producing-release note
