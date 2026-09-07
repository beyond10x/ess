# Public source and release support claims

Status: accepted for review wave 14. Evidence source: ESS `ecb7efc22ad9b19b85ef4debd8143491d6a66ef3`.

Derived from the complete read-only scoper report SHA256 `b865e515ccd320abe0a7f8fc6f771c8687c7a977eeb751de978ece10603d5579`; its 58 complete input hashes were verified independently by root. No implementation, source check or publication of these changes is claimed.

## Scope

Derived 2026-09-07 by `aep-drive:story-scoper` 0.8.0 against ESS ecb7efc22ad9b19b85ef4debd8143491d6a66ef3 — cited.

- Primary surface: public capability documentation and the existing Rust repository maintenance command — cited.
- File: `website/docs/status/where-this-stands.md` — cited; current-release wording, projection output description and maintained source/support matrix.
- File: `website/docs/concepts/ess.md` — cited; obsolete site-output row and prose, plus the incomplete synthesis-target enumeration.
- File: `website/docs/reference/cli.md` — cited; obsolete site-output command row, omitted docs-ir spelling and source/release labels on schema-command rows.
- File: `website/docs/guides/synthesize.md` — cited; three-emitter table omits the existing Clap target and needs its distinct obligations/dependency boundary.
- File: `website/docs/guides/verify-conformance.md` — cited; distinguish current-source suite/5 instructions from the separately observed 0.20.0 release.
- File: `crates/edge/ess-xtask/src/main.rs` — cited; existing Command/run dispatch, workspace_version, projections and generated-artifact inventory provide the maintenance owner.
- File: `crates/edge/ess-xtask/src/support.rs` — inferred; proposed private Rust support-table renderer/checker and its unit tests.
- File: `Taskfile.yml` — cited; existing offline check and separate site-build/release-status composition.
- File: `docs/design/review-public-support-claims.md` — inferred; proposed binding for the finite maintained claims, their evidence boundaries and drift-check behavior.
- Symbols: `Command`, `run`, `workspace_version`, `projections`, `Generated`, `Projection`, and `ess_gen::generators` — cited.
- Proposed symbol: `support::check` — inferred; no command or implementation currently exists.
- Confidence: high — cited; the story names the public discrepancy, and the actual CLI dispatch, emitters, release receipt and maintenance path resolve its owners.
- Would collide with: edits to any exact file reserved above, especially the shared concept page, CLI reference, xtask dispatch and Taskfile — inferred.
- Read-only evidence owners outside these reservations require no implementation edits for this candidate — inferred.

## Binding

1. Keep one maintained source-capability matrix in the status page. Distinguish **workspace source**, **observed release metadata**, and **limits/evidence**. Correct duplicate statements in the reserved pages, using links to the maintained matrix where duplication adds no value.
2. Add a private Rust `cargo xtask support --check` command and a `support-check` task included in `task check`. Reuse the existing `projections` route through the public CLI. It already consumes the actual JSON artifact map and obtains generator metadata from `ess_gen::generators`; there is no need for another generator dispatcher. `crates/edge/ess-xtask/src/main.rs`:466–535.
3. Derive mechanically available values: workspace version, default generator inventory, actual emitted output kinds, opt-in `docs-ir`, CLI adapter directions, target availability and actual report markers. Maintain narrowly qualified explanatory text alongside explicit owner/test references. Do not introduce a generic support registry, persisted metadata format or new domain model.
4. Compare the complete maintained blocks, including row presence, order and relevant cells. Missing, duplicated, additional or changed rows must fail. Publication prose around those blocks remains ordinary documentation, reviewed against this binding.
5. Exercise source behavior through existing CLI/library routes and retain the existing owned support/refusal tests. The check must not establish support by searching Rust source for tokens. Runtime artifact/header checks establish the stated output kind only; they do not replace schema, synthesis, adapter or conformance tests.
6. Keep release verification outside the offline command. The checker must neither contact GitHub nor infer publication by equating the workspace version with a release. Record a dated release observation in public prose; retain the full receipt as coordinator evidence outside the public allowlist.
7. Preserve the current format catalog and its reader/digest distinctions. This candidate does not regenerate the entire format reference or expand support to make a table pass. The relevant existing catalog already distinguishes unversioned outputs, old readers, current writers and conformance versions. `website/docs/reference/formats.md`:8–15,39–79,163–176.
8. Keep implementation within the nine exact reservations above. No dependency change is required for the proposed approach: xtask already has `ess-domain`, `ess-gen`, `serde_json`, Clap and its CLI-probe mechanism. `crates/edge/ess-xtask/Cargo.toml`:10–16; `crates/edge/ess-xtask/src/main.rs`:472–527.

The dated release paragraph is:

> Latest published release observed on 7 September 2026: [0.20.0](https://github.com/beyond10x/ess/releases/tag/0.20.0), published on 6 September 2026. Its release record lists archives for Linux and macOS on x86-64 and ARM64, plus SHA256SUMS. The source checkout’s workspace version is 0.20.0 and includes separately documented unreleased changes.

This states what the receipt establishes. It does not claim downloaded assets were checked, installed or executed.

**Concrete source-and-release matrix**

| Required material statement | Exact source/receipt evidence | Boundary the public wording must retain |
|---|---|---|
| Workspace package version is `0.20.0` | `Cargo.toml`:39–46 | Cargo metadata is independent of remote publication |
| Release record `383642123`, tag/name `0.20.0`, non-draft/non-prerelease; published `2026-09-06T16:07:45Z`, observed `2026-09-07T02:25:44.927685Z` | `target/review-boundaries-13/preparation/public-support-scope/latest-release-readback.json`:2–21; `target/review-boundaries-13/preparation/public-support-scope/latest-release.json`:1 | Dated API observation, not an indefinitely current assertion |
| Record lists four architecture archives and `SHA256SUMS` | `target/review-boundaries-13/preparation/public-support-scope/latest-release-readback.json`:12–18; `target/review-boundaries-13/preparation/public-support-scope/latest-release.json`:1 | Asset-list evidence only; no installation, checksum-file verification or execution claim |
| Five default projections: `docs`, `site`, `schema`, `openapi`, `asyncapi` | `crates/generate/ess-gen/src/lib.rs`:46–59; `crates/edge/ess-cli/src/main.rs`:2355–2379 | `docs-ir` is an additional explicit choice, not a sixth default generator |
| `docs` emits Markdown/Mermaid from the document model | `crates/generate/ess-gen/src/docs.rs`:69–105 | Documentation projection, not implementation behavior |
| Explicit `site` emits HTML at its output root with local assets; default combined generation uses the generator’s `site` directory | `crates/edge/ess-cli/src/main.rs`:2370–2379; `crates/edge/ess-cli/src/site.rs`:67–71,152–155; `crates/generate/ess-gen/src/html.rs`:51–58,625–642 | Explicit authored pages/downloads are supported; ESS does not host the resulting site |
| `docs-ir` emits `docs-ir/document.json`, carrying `ess-docs/1` | `crates/edge/ess-cli/src/main.rs`:2358–2368; `crates/generate/ess-gen/src/document.rs`:54–55 | Document IR, not HTML or a general persisted `EssIr` reader |
| Schema output is draft 2020-12 for named types, entities, command inputs, events and errors | `crates/generate/ess-gen/src/schema.rs`:66,103–144 | Structural projection does not establish implementation behavior |
| Native OpenAPI projection emits OpenAPI 3.1; AsyncAPI projection emits 3.0 | `crates/generate/ess-gen/src/openapi.rs`:210,239–254; `crates/generate/ess-gen/src/asyncapi.rs`:154,187–190 | These are projection directions; no AsyncAPI importer is declared by the inspected CLI adapter surface |
| OpenAPI import accepts the supported 3.1 service/interface subset and produces `ess-openapi-import/1` with retained source/accounting | `crates/edge/ess-cli/src/main.rs`:2976–3058; `crates/generate/ess-openapi/src/accounting.rs`:9–14,140–150; `crates/generate/ess-openapi/src/lib.rs`:495–508 | External references are refused; semantic gaps/unresolved references remain visible |
| Checked OpenAPI import projection refuses semantic gaps, unresolved references and legacy interface-only input | `crates/generate/ess-openapi/src/accounting.rs`:140–150,224–253; `crates/generate/ess-openapi/tests/accounting.rs`:15–60,77–88 | Annotation normalization alone may be allowed; this is not universal OpenAPI round-trip support |
| Kubernetes import accepts an existing sanitized bundle or explicit live context and produces infrastructure IR | `crates/edge/ess-cli/src/main.rs`:584–601,2915–2968; `crates/infra/ess-kubernetes/src/lib.rs`:54–112 | The live scanner is the credential edge. The CLI’s fixed supported-category list and empty `coverage_gaps` field do not prove complete observation |
| Kubernetes projection consumes intent plus observed IR and produces patches/new objects/obligations | `crates/edge/ess-cli/src/main.rs`:640–649,3094–3099; `crates/infra/infra-project/src/lib.rs`:8–35; `crates/infra/infra-project/src/project.rs`:552–574 | No apply operation; missing decisions remain obligations and unsupported conditions can refuse |
| BuildKit projection consumes checked build IR and emits Dockerfile/Bake inputs | `crates/edge/ess-cli/src/main.rs`:615–624,3065–3075; `crates/generate/ess-deployment/tests/deployment.rs`:313–334 | Projection does not execute BuildKit |
| Helm projection consumes runtime IR and emits a configuration-neutral chart | `crates/edge/ess-cli/src/main.rs`:625–638,3078–3091; `crates/generate/ess-deployment/tests/deployment.rs`:447–458 | Projection does not apply the chart or establish live resource availability |
| Full structural synthesis exposes Rust, Go, Web and Clap | `crates/generate/ess-synth/src/lib.rs`:80–115; `crates/edge/ess-cli/src/main.rs`:379–384 | Structural output carries obligations/refusals, not complete business behavior |
| Clap emits grammar, completion support and handler seams; handlers receive `ArgMatches` | `crates/generate/ess-synth/src/clap/mod.rs`:23–32,62–65,101–105; `crates/generate/ess-synth/tests/clap.rs`:249–279 | No additional type layer or implemented command behavior; generated dependencies include Clap and clap_complete 4, `crates/generate/ess-synth/src/clap/tree.rs`:453 |
| Full synthesis refuses unsupported modeled Binary64 across all four targets | `crates/generate/ess-synth/tests/feasibility.rs`:50–84 | Do not conflate full synthesis with separately supported structural data libraries |
| CLI conformance run selects built-in `billing` or `oracle-fixture` targets | `crates/edge/ess-cli/src/main.rs`:576–580,2518–2522 | A production adapter must establish its own execution boundary; the built-ins do not prove independent deployment |
| Suite/4 and report/1 remain defaults; report/2/run/2 are explicit count surfaces | `crates/edge/ess-cli/src/main.rs`:449–450,501–520; `crates/verify/ess-conformance/src/counts.rs`:12–15; `crates/edge/ess-cli/tests/count_reports.rs`:62–117 | Legacy default success is not a complete-conformance claim |
| Legacy all-pass execution can remain inconclusive conformance | `crates/verify/ess-conformance/src/counts.rs`:319–345; `crates/edge/ess-cli/tests/count_reports.rs`:5–34 | Keep execution status separate from coverage qualification |
| Current source suite/5 can qualify only for a nonempty passing selection with complete inventory and no in-scope refusal | `crates/verify/ess-conformance/src/counts.rs`:333–345; `crates/verify/ess-conformance/src/coverage.rs`:358–366; `crates/edge/ess-cli/tests/coverage_cli.rs`:246–299 | Suite/5 requires explicit report/2 before target execution, `crates/edge/ess-cli/src/coverage.rs`:203–205 |
| Suite/5/carrier/replay features are listed under `[Unreleased]` | `CHANGELOG.md`:3–14 | The observed 0.20.0 release notes establish report/2, but do not establish these later suite/5 features |
| Browser conformance presentation is replay, without an execution report or publisher authentication | `crates/edge/ess-cli/src/main.rs`:453–461; `crates/verify/ess-conformance/src/web_replay.rs`:1,120–129; `website/docs/guides/verify-conformance.md`:145–150 | A green replay is not independent execution evidence |
| Runtime compilation checks supplied identities, component coverage, replica bounds and stateful storage | `crates/generate/ess-deployment/src/runtime.rs`:482–531,622–685 | It does not establish live provisioning or all resource requirements |
| Explicit executor verbs invoke external clients; reconciliation compares supplied current/desired documents and applies the affected set | `crates/edge/ess-cli/src/main.rs`:1275–1279,1481–1486,1634–1745,3199–3207 | No continuous control plane or automatic recovery proof; caller/current-state and credential boundaries remain material |
| Schema import/type/normalization commands are present in current source | `crates/edge/ess-cli/src/schema.rs`:15–36,73–83 | Remove blanket “Unreleased” labels. The current changelog records older additions under 0.19.0 and normalization additions under 0.20.0, but a changelog alone is not a remote release receipt: `CHANGELOG.md`:148–196; `target/review-boundaries-13/preparation/public-support-scope/latest-release.json`:1 |

The status page’s remaining broad capability bullets need not be expanded or renamed individually to claim new guarantees. Their surrounding label must clearly say whether they describe source capabilities or release evidence. This avoids turning the unchanged Cargo version into proof that all later source changes shipped.

## Acceptance and validation matrix

| Case | Expected result / evidence |
|---|---|
| Default projection inventory | Exactly the five registered generator names; actual artifact map agrees with their owned directories |
| Explicit site | Actual CLI result contains HTML pages and local stylesheet/Mermaid assets; no Markdown/sidebar classification |
| Authored site boundary | Retain selected-source links, downloads and HTML/Mermaid assertions in `crates/edge/ess-cli/tests/authored_site.rs`:85–111; do not imply automatic hosting |
| Explicit docs | Actual Markdown output remains distinct from HTML |
| Explicit docs-ir | Parse the artifact contents as JSON; require `ess-docs/1` and the explicit artifact path |
| Schema/OpenAPI/AsyncAPI rows | Match actual generated artifacts and version markers; retain existing generator tests rather than substituting prose checks |
| Maintained-block mutations | Alter an output kind/version/target/direction, remove or duplicate a row, or add an unowned row: `support --check` fails with the file and differing row |
| Workspace version mutation | Changed Cargo input changes the expected source-version row; stale public source-version text fails |
| Release/source separation | Workspace version changes do not rewrite or validate the dated release observation; check remains offline |
| OpenAPI supported input | Existing import → checked reload → projection assertions remain applicable, `crates/generate/ess-openapi/tests/accounting.rs`:15–60 |
| OpenAPI unsupported input | Gaps survive reload and block checked projection; external references refuse, `crates/generate/ess-openapi/tests/accounting.rs`:77–88 and `crates/generate/ess-openapi/src/lib.rs`:1232–1240 |
| Kubernetes direction and obligations | Preserve generated/owed/refused distinctions, including unstated remedies and false predicates: `crates/infra/infra-project/tests/projection.rs`:113–179,438–478 |
| BuildKit/Helm output | Preserve the existing output assertions in `crates/generate/ess-deployment/tests/deployment.rs`:313–334,447–458; do not execute external clients in the new documentation check |
| Synthesis inventory and limits | Four targets, Clap handler obligations, and complete Binary64 target refusal: `crates/generate/ess-synth/src/lib.rs`:86–115, `crates/generate/ess-synth/tests/clap.rs`:249–279, `crates/generate/ess-synth/tests/feasibility.rs`:50–84 |
| Legacy conformance | Passing execution with unknown coverage stays inconclusive; strict mode fails, `crates/edge/ess-cli/tests/count_reports.rs`:5–34 |
| Current suite/5 conformance | Explicit report pairing, admitted inventory and qualified result remain covered by `crates/edge/ess-cli/tests/coverage_cli.rs`:246–299; public wording identifies its source/release boundary |
| Replay wording | Remains replay-only; owned actual-browser admission cases are `crates/edge/ess-cli/tests/coverage_browser.rs`:9,211,311 |
| Public rendering | `task site-build` succeeds and rendered status contains the maintained rows, release link/date and source/release distinction |
| Integration | Coordinator runs every `task check` and `task site-build` step, retaining statuses and executed-case counts, as required by `.engineering/planning/story/review-public-support-claims.md`:34–38 |
| Release publication | Separately verify the exact cited release record; preserve repository publication → Website source lock → Atlas snapshot/delivery gates |
| Publication boundary | Public allowlist remains unchanged; internal finding, binding, planning and raw preparation records are not copied into it |

The existing offline gate already includes workspace tests, projection/schema drift and local release consistency. Remote `release status` remains a separate operation. `Taskfile.yml`:38–46,95–131,144–160; `crates/edge/ess-xtask/src/main.rs`:105–144.



## Coordinator release-source readback

After the scoper returned, root resolved remote `refs/tags/0.20.0`: annotated tag object `e40d029dd53dc69fc83f1dfec4e6111705e4fa08`, peeled commit `c90ca1b2a3a5db02d7580dab63be6cbc56679e0b`. Exact Git bytes are retained under this preparation directory's `release-source`. The release commit's changelog has an empty Unreleased section and explicitly describes report/2 with suite versions 1–4 retaining unknown coverage. The current suite/5 coverage module paths do not exist at that release commit; those exact Git absence results are retained. This strengthens the source-versus-release qualification without claiming an installed or executed release binary.

The preparation evidence remains outside the public allowlist. Public pages should use the release link/date and concise source/release qualification, not local record paths or internal review history.

## Acceptance record

Accepted under the standing remediation implementation and publication approvals for wave 14.
The independent read-only candidate attack returned no findings; its full report is
`review-result:public-support-binding-pass1` (SHA256
`f28c45c3658564230b41570695097400549a7c090cb42f8186ef3e6d52242127`). Root read the entire
report and independently verified all 70 input hashes, including 53 exact Git blobs.
The source and documentation owners are unchanged at closing commit
`9d84a425e3a0c052bb08975766c5dbd600d04ef0`; the review executed no tests. This acceptance
selects the contract, not an implementation verdict. The implementor must confirm inferred
paths and root must retain the later independent source attack, complete gate and live delivery.
