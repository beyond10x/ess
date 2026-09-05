---
format: aep.planning-md/1
id: review-result:review-boundaries-5-format-adversary-pass-2
kind: review-result
status: active
title: Format catalog source and fixture adversary pass 2
owner: scope_conformance_design
relations:
- reviews: story:review-format-catalog
revision: 1
---
unit: story:review-format-catalog — f2c81b8ed07a6b522bb97b1e6709a9f2d0a3be57
verdict: nothing found
cases: n/a — documentation-only; no tests added or executed
origin: introduced 0 / pre-existing 0 / undecided 0
wrote-outside-worktree: none
needs-coordinator: immutable recording/comparison, integrated source/site gates and publication remain coordinator-owned

git --no-pager diff --stat
```text
```
Exit 0; the working diff and final `git status --short` are empty. No tracked file changed.

Nothing found in this second and final bounded documentation pass.

## Subject and scope

Base `4b66aac7b608b1deee9de88942390d4a6c5ec745`; exact subject above remained unchanged. Read the complete four-file change, unit acceptance and AGENTS, original brief, prior review, correction report and frozen-Rust addendum. The four files are `docs/design/review-format-catalog.md`, `website/docs/reference/formats.md`, `website/docs/guides/track-change.md` and `website/sidebars.ts`.

The requested Codex-cache 0.7.0 charter path was absent. The same installed 0.7.0 charter was found and read at `/home/timo/.claude/plugins/cache/beyond10x/aep-drive/0.7.0/agents/adversary.md`; the coordinator was informed. This is the file-backed documentation-only adaptation: there are no added test cases, suite counts or executable producer/readers to report. Existing read-only local helper programs were inspected before execution. No Cargo, AEP, network, Git mutation or source correction ran.

New Rust source claims were read only from Git object `f9a7cf7fcca79448a34b2754adb12f1a411573bd`, not the independently reviewed Rust unit's evolving working files. The catalog does not assert universal compilation of every model. Separate model counterexamples reported by that unit's reviewer are not counted as catalog contradictions where the catalog explicitly bounds the claim.

## Prior findings and current source checks

The previous report is unchanged: SHA-256 `0251b7d9e850126bbe137202c2517a698ed36b70b25cc80c1693aa4179cdc14e`. Its original finding signatures remain preserved there.

| Original finding | Observation in this pass |
|---|---|
| `docs/design/review-format-catalog.md:115`, NEEDS-CHANGE / introduced: intent digest omission and false API denial | Resolved in the read text. Current intent row :155, projection rows :159–160 and digest row :196 distinguish typed-intent identity, InfraIR model identity and whole-file bytes. Public page carries the same distinction. `InfraSpec::digest` at `crates/infra/infra-spec/src/spec.rs:547–562` serializes the typed value through sorted JSON objects, hashes compact bytes without LF, and preserves array order. `infra-project/src/project.rs:581–587` actually places `spec.digest()` in `provenance.specification_digest`. Retained projection summary :9–10 visibly names two distinct input digests. No invented raw-source digest claim remains. |
| `docs/design/review-format-catalog.md:82`, CONFIRMED / introduced: `ImpactReport` instead of `EssImpact` | Resolved in the read text. Current internal :83 and public impact row name `EssImpact`. Actual public type is `crates/verify/ess-diff/src/impact.rs:700`; `impact` returns `Result<EssImpact, ImpactRefusal>` at :766–771; `lib.rs:137–140` exports it. |

Descriptive comparison from the documents read: carried 0, new 0, resolved 2. The coordinator computes the formal comparison after immutable recording; this paragraph does not modify the earlier signatures or ledger.

Additional claims attacked without establishing a contradiction:

- Frozen `failure.rs:8–126` has exactly the eight documented kebab-case enum codes, private fields/construction, nonempty sorted/deduplicated causes and source identities, nonempty details, the four-field `ess-target-failure/1` envelope, read-only accessors, `Display`/`Error`, Serialize only and typed pretty JSON plus LF. The unchanged cloned plan can be empty; no capability is fabricated. No failure-file digest or persisted reader is claimed.
- Frozen synthesis facade `lib.rs:254–337`, direct Rust `rust/mod.rs:89–157` and Web `web/mod.rs:287–375` agree with the documented Result signatures and whole-outcome withholding. Successful `Synthesis` and existing partial `TargetReport` remain distinct. Direct caller plan/IR assumptions and surviving internal assertions are explicitly disclosed in the catalog.
- Frozen CLI `main.rs:2385–2427` returns exit 1 from the target error branch before `write_artifacts`; JSON/YAML use the generic renderer at :1118–1124, text uses Display. This claim is limited to a returned target failure; later I/O rollback is not asserted. No destination was written or CLI run in this documentation pass.
- Frozen Web `web/mod.rs:378–399` retains the separate semantic `browser_catalog` entry point without running the workspace fatal gate. The public/internal catalogs qualify equality with a successfully emitted workspace and make no new host/compiler guarantee.
- Current source discriminator, semantic version and hash-profile distinctions remain separate: authored/compiled composition array versus map; realization `type`; typed source versus whole/slice contracts; delivery prefixed payload digests; current suite/4/report/1 versus explicitly planned successors. This pass found no source-backed contrary claim in the complete reviewed catalog.
- Guide flags/defaults match `VerifyCommand` at current CLI :225–251 and the impact call at :2025, which passes `None` for `GeneratedTree`. Actual `IMPACT_FORMAT` is `/3`; the current delta writer supports /1 and /2. The guide distinguishes owed regeneration from attesting an earlier result, and does not advertise a nonexistent `--generated` option.

A read-only exploratory search used absent `examples/infra` and returned exit 2; it was corrected using `rg --files examples`, which found the existing `examples/k3d-dev-cluster/projection/SUMMARY.md`. This search miss is not a finding or a fixture execution.

## Executed local checks

The exact command arrays, raw combined outputs and exit codes are retained in the correspondingly named `adversary2-*.command.json`, `.log` and `.exit` files under this report's directory. All recorded commands below exited 0. The `git show` commands are source reads, not builds or tests.

| Lane | Exact command argv | Exit |
|---|---|---|
| `diff` | `{"cwd": "/home/timo/.local/state/worktree/trees/b10x/ess/review-format-catalog", "argv": ["git", "--no-pager", "diff", "4b66aac7b608b1deee9de88942390d4a6c5ec745..f2c81b8ed07a6b522bb97b1e6709a9f2d0a3be57"]}` | 0 |
| `frozen-failure` | `{"cwd": "/home/timo/.local/state/worktree/trees/b10x/ess/review-format-catalog", "argv": ["git", "show", "f9a7cf7fcca79448a34b2754adb12f1a411573bd:crates/generate/ess-synth/src/failure.rs"]}` | 0 |
| `frozen-facade` | `{"cwd": "/home/timo/.local/state/worktree/trees/b10x/ess/review-format-catalog", "argv": ["git", "show", "f9a7cf7fcca79448a34b2754adb12f1a411573bd:crates/generate/ess-synth/src/lib.rs"]}` | 0 |
| `frozen-rust` | `{"cwd": "/home/timo/.local/state/worktree/trees/b10x/ess/review-format-catalog", "argv": ["git", "show", "f9a7cf7fcca79448a34b2754adb12f1a411573bd:crates/generate/ess-synth/src/rust/mod.rs"]}` | 0 |
| `frozen-web` | `{"cwd": "/home/timo/.local/state/worktree/trees/b10x/ess/review-format-catalog", "argv": ["git", "show", "f9a7cf7fcca79448a34b2754adb12f1a411573bd:crates/generate/ess-synth/src/web/mod.rs"]}` | 0 |
| `frozen-cli` | `{"cwd": "/home/timo/.local/state/worktree/trees/b10x/ess/review-format-catalog", "argv": ["git", "show", "f9a7cf7fcca79448a34b2754adb12f1a411573bd:crates/edge/ess-cli/src/main.rs"]}` | 0 |
| `frozen-design` | `{"cwd": "/home/timo/.local/state/worktree/trees/b10x/ess/review-format-catalog", "argv": ["git", "show", "f9a7cf7fcca79448a34b2754adb12f1a411573bd:docs/design/review-rust-target-feasibility.md"]}` | 0 |
| `links` | `{"cwd": "/home/timo/.local/state/worktree/trees/b10x/ess/review-format-catalog", "argv": ["node", "target/review-boundaries-5/frozen-rust-documentation-check.mjs"]}` | 0 |
| `fixtures` | `{"cwd": "/home/timo/.local/state/worktree/trees/b10x/ess/review-format-catalog", "argv": ["node", "target/review-boundaries-5/fixture-hashes-2.mjs"]}` | 0 |
| `final-head` | `["git", "rev-parse", "HEAD"]` | 0 |
| `final-status` | `["git", "status", "--short"]` | 0 |
| `final-working-diff` | `["git", "--no-pager", "diff", "--stat"]` | 0 |

Existing link helper output, verbatim:

```text
Frozen producer target: crates/generate/ess-synth/src/failure.rs @ f9a7cf7fcca79448a34b2754adb12f1a411573bd
Frozen producer target: crates/generate/ess-synth/src/failure.rs @ f9a7cf7fcca79448a34b2754adb12f1a411573bd
Frozen producer target: docs/design/review-rust-target-feasibility.md @ f9a7cf7fcca79448a34b2754adb12f1a411573bd
Frozen producer target: docs/design/review-rust-target-feasibility.md @ f9a7cf7fcca79448a34b2754adb12f1a411573bd
docs/design/review-format-catalog.md: 124 Markdown links inspected
Frozen producer target: crates/generate/ess-synth/src/failure.rs @ f9a7cf7fcca79448a34b2754adb12f1a411573bd
Frozen producer target: crates/generate/ess-synth/src/failure.rs @ f9a7cf7fcca79448a34b2754adb12f1a411573bd
website/docs/reference/formats.md: 79 Markdown links inspected
website/docs/guides/track-change.md: 6 Markdown links inspected
Sidebar: 17 existing document IDs; reference/formats appears once
Links: 209 total; 126 relative targets; 83 public ESS source targets resolved against this checkout or the exact frozen producer
Frozen producer fallbacks: 6
Scope: local file/reference/navigation sanity only; no HTTP availability or rendered-site validation
Documentation link/navigation check passed

```

Existing fixture helper output, verbatim:

```text
examples/k3d-dev-cluster/cluster.ir.json
  claimed digest: 9ed0e8608fd69c43c3b0405a7a5fd599fad61a6246b42e2cdd4cffd1e29c8e75
  compact sorted model SHA-256: 9ed0e8608fd69c43c3b0405a7a5fd599fad61a6246b42e2cdd4cffd1e29c8e75
  same model plus LF SHA-256: 73de2f7d076b5ddeaf2d69b2dc378bab9f06df18dc6c4630d6a3d2e7436bf881
  committed envelope SHA-256: 3186a06dd21e8f90cd2af371604cf2119a2fe534c383921a31da490465496c09
suites/generated/billing/suite.json
  bytes: 154846; exactly one terminal LF: true
  complete bytes SHA-256: 508b6a3d75d6dabd6fa686b67dcb6c7c881374375aaed9dd8921445a5edc894e
  same bytes without terminal LF SHA-256: 3e1c5611aa64bfa7fd2dcaeffbda1f6056e0ef686c5138fc7689bd4f71e7121c
  provenance: {"suite_version":"ess-conformance/4","system":"billing","specification_version":"v3","spec_digest":"56090788443a14b4a51ad151eb5cb3ebded2b98f6defe9ac50826296ac5d0942","contract_digest":"cb634bd5e6f1afa6ebc8e9dca752e9901a9a68a2e51fc5009d099f155680606c"}
generated/rust/billing/plan.json
  bytes: 11506; exactly one terminal LF: true
  complete bytes SHA-256: 7bf0c7d9ccf6f950770f87958d24308207a1de863b2398c17397f1a51bed354e
  same bytes without terminal LF SHA-256: 74290ad4a7c68eadf96cb673b1cd902e1e2cd4c6dc131090b3a9f7e73d06ab13
  provenance: {"system":"billing","specification_version":"v3","source_digest":"56090788443a14b4a51ad151eb5cb3ebded2b98f6defe9ac50826296ac5d0942","contract_digest":"cb634bd5e6f1afa6ebc8e9dca752e9901a9a68a2e51fc5009d099f155680606c"}
generated/schema/events/billing.invoice.InvoiceCreated.schema.json
  bytes: 2293; exactly one terminal LF: true
  complete bytes SHA-256: a26b339eab01004553762b299bbb85c8e73c83fe8734f2c83b111c43ceff3ecc
  same bytes without terminal LF SHA-256: 766f332b7c0bb9c73b25788be75b248a96e3b497078acabadba9cad089b10dca
  provenance: {"system":"billing","specification_version":"v3","source_digest":"56090788443a14b4a51ad151eb5cb3ebded2b98f6defe9ac50826296ac5d0942","contract_digest":"slice-sha256/2:2a2646ad19cd728f290c0e8f88922be4f8acaedfd4a1de7855f9ed73aab6270b","regenerate":"ess generate"}
Existing suite and neutral plan carry equal source/whole identities despite different complete-file hashes
Initial probe limitation: JavaScript roundtrip differs at byte 6566; existing Rust fixture "amount\": 1.0,\n      "; JavaScript "amount\": 1,\n        "
Typed serde_json float 1.0 is not JavaScript number rendering 1; complete fixture bytes, not a JavaScript suite reserialization, were hashed above
Scope: existing fixture bytes only; no generators, Rust readers, tests or external consumers executed; EssIr source/whole digests were not independently rederived

```

The fixture observations establish these concrete byte distinctions only. The existing InfraIR model digest was independently recomputed; adding LF or hashing its full envelope produces a different value. The actual suite/plan/schema files have exactly one terminal LF and distinct full-file hashes, while suite and plan reference the same source/whole identities. Source/whole EssIr identities and the typed-intent digest were not independently rederived. JavaScript cannot be treated as the typed Rust suite canonical serializer: the helper explicitly records the actual `1.0` versus `1` roundtrip difference.

The link check measures 209 local/reference targets and 17 sidebar IDs. Six occurrences use exact frozen Git-object fallbacks. It does not establish that unpublished GitHub main URLs exist, inspect fragment anchors or render Docusaurus. No Website delivery, Atlas snapshot, SDK migration, AgentIDE pin, release or deployment was measured here.

## Findings

| File:line | Verdict | Origin | What was measured | What reaches it |
|---|---|---|---|---|

No current findings. No failing producer case was constructed or claimed.

## Retained observations and write boundary

Raw source excerpts for the two corrections and guide callers are in `adversary2-source-excerpts.log`; complete frozen source files and base-to-subject diff are retained in the named logs. All raw log SHA-256 values are recorded in `adversary2-evidence-sha256.txt` (including the exact helper outputs above). Earlier reports/helpers are untouched; only newly named ignored files in the assigned scratch directory were written. Paths written outside the worktree: none. No tracked implementation, test, documentation or planning file changed. Writes are relinquished on handoff. This was the final full catalog pass; it gives no approval and makes no independence claim.

```findings
[]
```
