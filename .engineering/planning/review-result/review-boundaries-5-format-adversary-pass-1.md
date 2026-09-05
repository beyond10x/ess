---
format: aep.planning-md/1
id: review-result:review-boundaries-5-format-adversary-pass-1
kind: review-result
status: active
title: Format catalog source and fixture adversary pass 1
owner: impl_diagnostic
relations:
- reviews: story:review-format-catalog
revision: 1
---
unit: story:review-format-catalog — 12d3dc2019af31683f626d570b0f66a0adb71a5f
verdict: NEEDS-CHANGE
cases: n/a — documentation-only; no tests executed
origin: introduced 2 / pre-existing 0 / undecided 0
wrote-outside-worktree: none
needs-coordinator: route catalog corrections; final frozen Rust API/error-format cross-check and public delivery gates remain pending

`git --no-pager diff --stat`:

```text
```

Exit 0; the working diff is empty. `git status --short` also produced no output. `git rev-parse HEAD` returned `12d3dc2019af31683f626d570b0f66a0adb71a5f`. Raw proof: `adversary-final-diff-stat.log`, `adversary-final-status.log`, and `adversary-final-head.log` beside this report. All writes from this pass are in the assigned ignored scratch directory.

1. Subject and scope

The reviewed subject is the exact commit above, against base `4b66aac7b608b1deee9de88942390d4a6c5ec745`. I read the complete four-file documentation change, the story including its acceptance statement, the implementor report, the assigned brief, the installed adversary charter, ESS AGENTS, and the source owners/callers for the catalog claims. The subject changes:

```text
214  0    docs/design/review-format-catalog.md
73   134  website/docs/guides/track-change.md
193  0    website/docs/reference/formats.md
1    1    website/sidebars.ts
```

Those are the subject's changes, not edits made by this pass. No production, documentation, test, planning, Git index or lifecycle mutation was made. Production contracts were read at this subject; an evolving Rust feasibility tree was not substituted for it.

The acceptance statement requires every currently persisted ESS format to have a cited catalog entry separating discriminator, semantic/release version, validation level, and canonical digest profile. Finding F1 concerns an existing persisted digest identity omitted by that inventory and expressly denied by its intent row.

2. Cases and source counterexamples

No cases were added or run. The brief assigns a documentation-only source/actual-fixture review with cases n/a and prohibits invented prose tests and a full gate. The two findings below are source-backed document counterexamples, not failing runtime tests. No mutant or compile failure is claimed.

Read-only commands and their complete raw output are retained in the sibling `adversary-*.command` and `adversary-*.log` files. The final focused source/origin capture is `adversary-final-finding-sources.command/.log/.exit` (exit 0). Earlier captures include `adversary-infra-intent-digest`, `adversary-intent-digest-reach`, `adversary-digest-inventory-audit`, `adversary-source-base-document-absence`, and `adversary-impact-type`.

The whole-crates Rust search for a declaration named `ImpactReport` returned no matches (rg exit 1). That is a bounded absence result, not a failing test or proof about unavailable external repositories. The positive return type and public export establish F2 without relying on the absence search.

3. Actual fixture check; no suite run

After reading the existing corrected helper, I ran it unchanged:

```sh
node target/review-boundaries-5/fixture-hashes-2.mjs
```

Exit status: 0. Raw output:

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

The command and output are also retained as `adversary-fixture-bytes.command/.log/.exit`.

This independently reruns a read-only inspection of actual committed fixture bytes. It recomputes the InfraIR fixture's compact sorted model hash and distinguishes that hash from its enclosing file and an added LF. It also hashes the complete suite, plan and schema files. It does not execute their producers or readers, prove compatibility, or independently rederive the EssIr source/whole-model identities. The infrastructure intent digest itself was not recomputed by this helper.

The implementor's earlier JavaScript reserialization setup failure on Rust `1.0` versus JavaScript `1` is explicitly retained and explained in the output. This pass uses the corrected raw-byte helper; that earlier failure is not a new finding and is not counted as a semantic red. No Cargo tests, workspace gates, site builds, live services, or external consumer executions were run.

4. Findings

| ID | File:line | Category | Severity | Verdict | Origin | Finding |
|---|---|---|---|---|---|---|
| F1 | `docs/design/review-format-catalog.md:115` | contract-drift | blocker | NEEDS-CHANGE | introduced | The intent row denies an existing InfraSpec::digest API and the catalogs omit the separate compact typed-intent digest persisted as projection provenance.specification_digest. |
| F2 | `docs/design/review-format-catalog.md:82` | contract-drift | warning | CONFIRMED | introduced | The impact row names ImpactReport, but the public producer returns and exports EssImpact. |

F1 — existing typed infrastructure intent identity omitted and denied

What was measured: the new internal intent row says `Authored bytes: —; no intent digest API.` The exact subject has this public API at `crates/infra/infra-spec/src/spec.rs:558`:

```rust
pub fn digest(&self) -> String {
    let value =
        serde_json::to_value(self).expect("a specification has no unserializable state");
    let canonical = serde_json::to_vec(&value).expect("a value serializes");
    infra_compiler::digest_of_canonical(&canonical)
}
```

The serialized typed `InfraSpec` has `format`, `name`, and the declared-order `expectations` vector (`spec.rs:519–527`). The method's source documentation at :547–556 explicitly names the full SHA-256 and key-sorted Value recipe. The input is compact serialized typed intent, with no appended LF; the result is the infrastructure family's bare 64-character lowercase SHA-256. Object maps are sorted; array order remains significant. This is a different digest domain from raw authored YAML bytes, the InfraIR model, and the full projection document.

What reaches it: `infra_project::project` directly writes this identity into `ProjectionProvenance` at `crates/infra/infra-project/src/project.rs:581–587`:

```rust
Ok(Projection {
    format: PROJECTION_FORMAT,
    specification: spec.name.clone(),
    provenance: ProjectionProvenance {
        context: ir.provenance.context.clone(),
        snapshot_digest: ir.digest(),
        specification_digest: spec.digest(),
    },
```

`Projection::document` retains that provenance (:374–388), and `to_json` serializes that persisted document (:391–397). The actual CLI `project_kubernetes` reads intent and InfraIR, calls `project`, writes projection artifacts when `--out` is provided, then renders JSON through `projection.to_json()` or YAML from `Projection` (`crates/edge/ess-cli/src/main.rs:3226–3245`). The projection Markdown renderers also display specification and snapshot digests (`crates/infra/infra-project/src/render.rs:50,179–184`).

There is an actual committed artifact from that workflow at `examples/k3d-dev-cluster/projection/SUMMARY.md:3,9–10`:

```text
Generated by `ess project kubernetes`. **Nothing here has been applied.** Every file in this tree is a change somebody has to read and decide to make.

| specification `k3d dev cluster, as it ought to be` | `6f52fb34191df4f822a31f9242e103c7e61976b69bed41b772aba8a72f747689` |
| snapshot of `k3d-dev-cluster` | `9ed0e8608fd69c43c3b0405a7a5fd599fad61a6246b42e2cdd4cffd1e29c8e75` |
```

Those are separate persisted input identities. Reading this fixture and the actual writer/caller establishes reach; no newly executed projection is claimed.

The public page's `website/docs/reference/formats.md:126` correctly limits its statement to “No canonical authored-file digest,” which is not itself contradicted by the typed API. However, neither its digest-domain table (:19–26) nor its projection rows (:130–131) explain the typed intent digest recipe and `specification_digest` identity. Both catalogs need that existing domain represented.

Origin: introduced document defect. `git ls-tree` at the assigned base shows both catalog documents absent; the subject adds the false assertion. `git show <base>:crates/infra/infra-spec/src/spec.rs` and the base's projection SUMMARY show that the API and persisted artifact already existed. I did not move the worktree to the base and do not classify this as a pre-existing producer defect.

Bounded correction: remove the false API denial and document the typed intent hash input, representation and projection provenance consumer in the internal/public catalogs. Keep the distinction from raw authored-file bytes. No producer, reader, format version or hash change is needed. The blocker classification is limited to the unit's stated complete-current-contract acceptance; it is not a production incident claim.

F2 — impact producer type name differs from the new row

What was measured: `docs/design/review-format-catalog.md:82` names `ImpactReport`. At this subject, `crates/verify/ess-diff/src/impact.rs:700` declares `pub struct EssImpact`; :741 owns its canonical writer; and :766–771 returns:

```rust
pub fn impact(
    before: &EssIr,
    after: &EssIr,
    suite: Option<&ConformanceSuite>,
    tree: Option<&GeneratedTree>,
) -> Result<EssImpact, ImpactRefusal> {
```

`crates/verify/ess-diff/src/lib.rs:137–139` exports `EssImpact` with `impact`. The public result type is therefore positively identified; the whole-crates declaration search found no `ImpactReport` alias/type/enum/struct.

What reaches it: the documented `ess_diff::impact` API and the CLI route at `crates/edge/ess-cli/src/main.rs:2025` use that result. The incorrect catalog name misdirects a reader looking for the actual result/writer type; its linked source file itself is valid.

Origin: introduced. The internal catalog is a new file in this subject and its added row contains the wrong name; the producer type was already present at the base. No runtime failure or compatibility break is claimed.

Bounded correction: name `EssImpact`. This is a warning because the source link still reaches the right module and no persisted bytes or runtime admission behavior are affected.

5. Other attacks and limits

- Checked current marker/default/support literals against their source owners: authored ESS, compiled and authored composition, client plan, realization, delivery, diff/impact, conformance, generation, and infrastructure families. Apart from F1's digest domain and F2's type name, this pass found no additional concrete document counterexample. Literal searches include comments/tests and are not an assumption that every mentioned successor is a current producer.
- Checked `ess/1` authored source versus Serialize-only compiled EssIr, authored service arrays versus compiled composition service maps, and `type` versus `format` discriminators (`crates/specify/ess-domain`, `ess-compiler/src/ir.rs`, `ess-composition`, `ess-realization`). The documentation separates these shapes and does not invent an EssIr envelope version.
- Checked hash recipes in `ess-compiler/src/ir.rs`, `ess-gen/src/slice.rs`, delivery digest/canonical owners, realization, InfraIR and the actual fixture files. Compact typed/source bytes, compact sorted model bytes, pretty+LF documents, whole-model and `slice-sha256/2:` identities are distinguished. This is source/fixture coverage, not a fresh whole-model producer execution.
- Checked all eight delivery admission macro users against `crates/verify/ess-delivery/src/validation.rs` and document owners. The text distinguishes closed checked generic decoding, explicit validation after public mutation, and supplied-authority limits; it does not claim JSON parsing establishes external truth.
- Checked current diff /2 and legacy /1 support/write guards, impact /3 with no persisted reader, the 26-relation graph, and the current guide route (`ess-diff/src/delta.rs`, `raw.rs`, `impact.rs`; `ess-compiler/src/graph.rs`). No additional mismatch found in these bounded source reads.
- Checked the actual impact argument definition at `crates/edge/ess-cli/src/main.rs:241–250`, its dispatch at :1010–1015, and execution at :2007–2037. This exact subject has no `--generated` argument there and calls `ess_diff::impact(..., None)` at :2025. The guide no longer promises that CLI input or surviving-test reuse. This is source-backed CLI behavior, not a newly executed help/argument-rejection probe; the stale GeneratedTree source comment alone was not treated as actual caller behavior.
- Checked the guide's `ess verify conform` route against `VerifyCommand::Conform` at `main.rs:224–230`; its nested spelling is real.
- Checked conformance suite /4, standalone report /1 and current unversioned detailed run against `scenario.rs`, `evidence.rs`, `report.rs`, `runner.rs` and `go/runtime.go`. Syntax parsing/declared support is not represented as uniform execution admission; Rust `Timestamp(u64)` and Go `int64`, producer-specific outcome vocabularies, and detailed versus standalone outputs remain distinct.
- Checked planned suite /5, standalone report /2 and run /2 against the final conformance binding document. The catalog explicitly labels these as planned, with current-reader/execution gaps; no execution compatibility was established by this pass.
- Checked documentation IR/per-page stamps, browser catalog, imported interfaces and generated schema/OpenAPI/AsyncAPI against their owners under `crates/generate`, plus the source schema generator under `crates/edge/ess-xtask`. External dialect and service/specification versions are not silently equated with ESS format versions. These source reads do not establish external validator readiness.
- Checked Rust and Go startup writers (`ess-synth/src/rust/http.rs`, `go/http.rs`): `log: ess/1` and shared startup facts are actual source contracts, while runtime rendering and dynamic facts do not imply cross-language complete-byte equivalence.
- Checked observation/scanner output, InfraIR document reader and checked ownership transform, and projection JSON/YAML (`ess-kubernetes`, `infra-domain`, `infra-compiler/src/ir.rs`, `read.rs`, `infra-project/src/project.rs`, CLI). The scanner's no-appended-LF output and full-file scan hash are separate from the model hash. InfraIR admission is expressly limited: it does not rederive every observation fact/domain value or establish collection completeness. Projection JSON artifacts versus YAML patches/objects are correctly separate despite a shared marker.
- Checked actual generated provenance envelopes and readers, including comment/structured agreement, legacy versus current digest profiles, four-line Cargo comment frame and per-page docs refusal (`ess-gen/src/stamp.rs` and emitters). Header admission is not claimed to parse or prove the complete Cargo artifact semantics.
- Checked `website/sidebars.ts` and `b10x.docs.yaml:83–90`: the new public formats page is navigable within the existing website docs source allowlist; the internal catalog is not added to that allowlist. Public references inspected do not expose private Atlas or planning paths. This is not a new site/link/delivery gate run; the implementor's reported 183 link and 17 sidebar checks remain implementor output.
- Checked the qualifications on cited prior downstream inventory: historical source/pin facts and digest-only references are not described as runtime parsers or evidence of currently deployed readiness. No external checkout/pin/build/deployment was refreshed by this pass.

Pending coordinator work: the separate Rust feasibility unit is still evolving. Its newly bound `ess-target-failure/1` and checked synthesis API are not implemented/frozen in this catalog subject. Their absence here is not counted as a current-producer finding. After that source is frozen/merged, the coordinator must reconcile the actual plan/target/failure rows and public API description. Source publication, Website source-lock/snapshot refresh, Website/Atlas delivery gates and any final integrated gates also remain coordinator-owned; this report does not claim they ran.

6. Writes and handoff

Writes outside the worktree: none.

All pass output lives under `/home/timo/.local/state/worktree/trees/b10x/ess/review-format-catalog/target/review-boundaries-5/`: this report, read-only command/output captures named `adversary-*`, final clean-state captures and report checksum. The existing fixture helper was read and executed unchanged. No tracked or untracked source, document or test was written. No AEP command, Git mutation, build, live service request, publication or cleanup operation was performed. No active process is retained by this pass. Writes are relinquished with this report; fixes belong to the coordinator's routed implementor.

```findings
- file: docs/design/review-format-catalog.md
  line: 115
  category: contract-drift
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: The intent row denies an existing InfraSpec::digest API and the catalogs omit the separate compact typed-intent digest persisted as projection provenance.specification_digest.
- file: docs/design/review-format-catalog.md
  line: 82
  category: contract-drift
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: The impact row names ImpactReport, but the public producer returns and exports EssImpact.
```
