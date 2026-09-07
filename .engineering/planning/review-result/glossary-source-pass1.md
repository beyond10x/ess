---
format: aep.planning-md/1
id: review-result:glossary-source-pass1
kind: review-result
status: active
title: Glossary source document attack pass 1
relations:
- reviews: story:review-glossary-boundaries
revision: 1
---
unit: story:review-glossary-boundaries, subject c99776eb14173b467143525947cd0933b983b459
verdict: nothing found
cases: not run (document attack; source and link inspection)
origin: introduced 0 / pre-existing 0 / undecided 0
wrote-outside-worktree: none
needs-coordinator: none

`git --no-pager diff --stat` — initial own tracked diff, direct exit 0:
```text
```
The actual empty stdout and stderr are commands/initial-own-diff-stat.stdout and .stderr. The final own diff is also empty. No tracked path was edited by this adversary.

The separate subject change, exact base 38cfefc07ae27ccc5e9b51ca2159b5015fab1f50 to exact subject c99776eb14173b467143525947cd0933b983b459, is:
```text
 website/docs/concepts/ess.md                 | 90 +++++++++++++++++++++++++++-
 website/docs/guides/record-realization.md    | 22 +++++--
 website/docs/guides/write-a-specification.md | 21 +++++--
 3 files changed, 123 insertions(+), 10 deletions(-)
```
The full Git-produced change is retained verbatim in commands/subject-diff.stdout. This is the implementor's three-page change committed by the coordinator before dispatch, not an adversary edit. Initial and final HEAD readbacks equal the exact subject. Initial and final `git status --porcelain=v1 --untracked-files=all` return empty output; assigned target scratch is ignored.

## 1. Cases and suite

No case was added or run. No suite, compiler, Clippy, formatter, generated fixture, browser rendering, integration or publication was executed. The dispatch brief expressly applies the developer's exception for reversible explanatory prose and prohibits tests mirroring this edit. Therefore no phrase-search check is represented as a red case, and no implementor or earlier coordinator test count is claimed as execution by this pass. Existing test files were read as source only.

The actual checks below are Git diff/status/readback commands and local source/link inspection. The small inline bookkeeping programs retained command bytes, input copies, hashes and native metadata; they are not an executable product checker or an automated assessment of prose correctness.

## 2. Judgment findings

Nothing found in the bounded changed terminology, interface qualifications, identity claims, or selected examples at c99776eb14173b467143525947cd0933b983b459. The findings table is empty; no CONFIRMED, NEEDS-CHANGE or INFEASIBLE row is returned. This is agent judgment informed by source inspection. It neither approves the change nor claims structural or automated independence.

The complete story, accepted binding, original F12 (review lines 424–447), complete Preferred conceptual names proposal (583–604), complete three changed pages, implementor report and complete subject diff were read. The accepted binding chooses labels for existing owners and a current multiple-interface limitation; it does not select a new persisted interface entity or authorize existing API/type renames. The finite attack below follows that scope.

## 3. Finite term, boundary and example coverage

Every line below records what was inspected and the public path that reaches that claim. “Not broken” means no contradiction found by this document attack, not observed execution.

| Public claim / reachability | Source inspected and bounded result |
|---|---|
| Logical component, concepts/ess.md:45 | ess-domain/src/component.rs:77,361,949–968: domain/command/event ownership; duplicate logical domain ownership refuses. A process or release name is not the logical owner. |
| Deliverable descriptor, concepts/ess.md:46 | ess-deployment/src/component.rs:20,30,44,159 and complete compile_component: referenced repository input paths and independent runtime/chart release units; descriptor compilation does not open the referenced files. |
| Imported component alias, concepts/ess.md:47 | ess-composition/src/lib.rs:190,479,1184–1290: a local key binds exact system, version, compiled-model digest and component. This is not catalog release selection. |
| Stack member, concepts/ess.md:48,112 | ess-deployment/src/stack.rs:20,195,401,633: resolve_stack receives StackSpec and ReleaseCatalog, carries the supplied composition_digest and has no compiled-composition input. No alias correspondence or complete operation-payload compatibility proof is implied. |
| Semantic runtime requirement, concepts/ess.md:31,49 | ess-domain/src/topology.rs:114,223: one component's replicas, statefulness and resource requirements. This owner does not group components into processes or report observed placement. |
| Runtime workload mapping, concepts/ess.md:50 | ess-deployment/src/runtime.rs:177,197,233,482–700: desired component/process/container/storage mapping; exact input digests, coverage, replicas and stateful storage are checked. The paragraph expressly does not infer resource provisioning or satisfaction of every semantic resource requirement. |
| Implementation manifest, concepts/ess.md:51 | ess-realization/src/lib.rs:399,648,790–878: selected components/actors, artifacts and entrypoints; implementation assignments cover each selected component exactly once. |
| Entrypoint prerequisite, concepts/ess.md:52 | ess-realization/src/lib.rs:301,312,1008–1045: the closed OS/architecture/environment/filesystem/network/cgroup/credential vocabulary is descriptive; prerequisite resolution does not check environmental availability. |
| Reach and CLI contract, concepts/ess.md:53,64–73; write-a-specification.md components section | ess-domain/src/component.rs:95,101,148,178,234,581–745: exactly InProcess, Network, CommandLine; default InProcess; cli required for CommandLine and refused for either other reach. One reach field and exactly-once accepted-command placement cannot declare simultaneous independent CLI and HTTP surfaces. |
| CLI command and flag naming, concepts/ess.md:53,99 | Complete ess-synth/src/clap/tree.rs and clap/mod.rs:1–80: binary/group layout comes from the CLI declaration, command words and flags from model names and input fields; emitter supplies parsing/completion and handler seams, not command behavior. |
| Physical invocation and attachment, concepts/ess.md:54,75–79; record-realization.md | ess-realization/src/lib.rs:332,357,878–1007,1090–1250: one argv or HTTP(S) invocation per entrypoint, unique IDs, exactly one primary, repeated implementation references allowed. Resolution checks references and invocation syntax; no semantic-reach comparison, actor authorization or execution proof follows. |
| Behavioral and actor ownership, concepts/ess.md:66–68 | ess-compiler/src/ir.rs:782,944 retains command contracts and actor may grants. Realization's selected actor-reference validation does not establish authorization for a physical entrypoint. |
| Derived HTTP contract, concepts/ess.md:55,72–73 | ess-gen/src/http.rs:162,188,224 emits command POST routes and network-owned view GET routes. ess-synth/src/rust/http.rs:108 and go/http.rs:55 select Network for HTTP server generation. Projecting a contract is not proof of a listener, served URL or public availability. |
| Model identity, concepts/ess.md:81–86 | ess-compiler/src/ir.rs:1104–1195,1575 hashes the serialized IR; nondefault reach and present CLI serialize, explicit/default InProcess is omitted alike. Realization lib.rs:1047 hashes specification identity, synthesis identity and implementations, excluding entrypoints. The claimed entrypoint-only digest stability follows directly from those bytes. No identity is described as authorization or execution proof. |
| Future multiple-interface design, concepts/ess.md:88–91 | The accepted binding and current component/realization owner definitions support criteria for an additive future design. The paragraph introduces no current interface collection, combined reach, migration or multi-surface synthesis capability. |
| CLI-only desk-service, concepts/ess.md:99 | Complete ess-synth/tests/clap.rs: desk binary, visits group, command_line and generated grammar byte assertions. Read together with the emitter source; fixture tests were not executed here and do not prove implemented command behavior. |
| HTTP-only Gatepass, concepts/ess.md:100 | Complete examples/gatepass/components.yaml and gatepass-realization/src/bin/gatepass-server.rs: pass-service is Network; server links handwritten realization with generated server; PORT is an environment input and the native main has no argument parser. The page labels source linkage rather than a running deployment. |
| Simultaneous CLI/HTTP, concepts/ess.md:101 | One Reach and the cli/reach guard in component.rs, together with realization's entrypoint resolver, support the stated current limitation. Repeated physical records cannot supply the absent semantic interface collection. Duplicate domain ownership is also refused. |
| Billing physical invocation, concepts/ess.md:102 | Complete examples/realizations/billing-local.yaml assigns invoice-service to billing-binary and records local-tui argv plus MODEL environment prerequisite. Its example.invalid artifact locator and fixture digest do not establish an available executable. Complete billing/components.yaml has no authored command_line/cli contract. The public qualification preserves this distinction. |
| Todo/Usage composition, concepts/ess.md:103,106–110 | Complete workbench composition and two-components component/system fixtures select two different components from the same workbench model under local aliases. ess-composition lib.rs:699,744,1295–1455 and ess-cli main.rs:93,1896 support Rust byte-buffer transport and selected operations, without complete payload definitions/codecs or view-parameter named-type closure; no all-target client promise remains. |
| Oracle delivery/runtime/stack, concepts/ess.md:104,112–116 | Complete examples/oracle-fixture/components.yaml plus ess-deployment/tests/deployment.rs:127–440 distinguish one deliverable's release units from order-service/dispatch-service. The fixture realization implements both and runtime maps both into one workload; stack constraints resolve runtime/chart versions through fixture catalog data. No live deployment or compiled-composition compatibility is demonstrated. |
| Topology synthesis boundary, concepts/ess.md:115 | ess-synth/src/plan.rs:1105,1113 records TopologyDeferred; desired workload mapping is separately compiled by ess-deployment runtime. The public paragraph preserves that separation. |

Source crate paths in the table are under crates/specify/ (ess-domain, ess-compiler, ess-realization, ess-composition), crates/generate/ (ess-deployment, ess-gen, ess-synth), or crates/edge/ (ess-cli). Exact full paths, whole-byte hashes and actual content-inspection ranges are in the input inventory below.

### All 13 original naming proposals

| Review proposal at architecture-review.md:583–604 | Accounting at the exact subject |
|---|---|
| ESS → system specification | Existing concepts/ess.md:9 definition and model/pipeline distinguish authored specification from toolchain; no format/tool rename selected. |
| ess-domain component → logical component | New row names the existing logical owner and command/event/domain surface. |
| ess-deployment component → deliverable | Accepted label is the more precise deliverable descriptor; existing ComponentSpec and ess-component/1 stay named. |
| Composition service → imported component alias | New alias row, Todo/Usage example and linked composition-client limitations. |
| Stack service → stack member | New stack row and catalog/composition-digest qualification. |
| Semantic workload → runtime requirement | Corrected model table and new owner row preserve correctness requirements. |
| Runtime workload → workload placement | Accepted label runtime workload mapping deliberately denotes desired mapping rather than observed placement. |
| Realization → implementation manifest | New owner row and realization guide preserve the existing RealizationSpec and ess-realization/1 names. |
| Runtime → runtime specification / resolved runtime model | RuntimeSpec/RuntimeIr remain the typed desired-runtime owners; complete linked component-delivery.md distinguishes these documents from a running program. |
| Deployment IR → deployment plan | Linked component-delivery.md distinguishes desired release/binding/order plan from explicit execution; environment.rs:137 defines the desired DeploymentIr. No persisted rename or executor change is selected here. |
| Conformance → suite, run, report, coverage | Existing concept conformance section and inspected verify-conformance.md:91–162 distinguish these artifacts and their limits. No conformance contract redesign belongs to this prose change. |
| Docs IR → document model | ess-gen/src/document.rs:1–68 identifies Document/ess-docs/1; inspected generate-artifacts.md:133–173 explains that owner and rendering split. Existing site-output wording is already owned by the separate F16 support story, as noted below. |
| Source digest → compiled-model digest | New identity paragraph names the hash of compiled IR, including reach/CLI serialization consequences. Broader format/catalog and successor-identifier proposals are not selected by this binding. |

The two guide edits were read in full with the concept page. The authoring guide points to the same one-reach/CLI owner and does not imply adding a second domain owner. The realization guide qualifies physical argv/HTTP invocation, preserves model-owned behavior/actors/reach and does not equate several entrypoints with simultaneous semantic interfaces.

Known separate context, not a newly returned finding: the unchanged concepts/ess.md:146,158–159 Markdown-only site-output wording differs from the current document/HTML guide. Original review F16 at 522–542 already records that discrepancy, and the complete draft story review-public-support-claims explicitly owns its site-output-kind/source-dispatch matrix. The complete subject diff shows that paragraph is unchanged. This bounded glossary pass does not claim the entire older reference is globally correct, reopen that already-owned issue, or create another story.

## 4. Actual commands and link readbacks

All retained command receipts record exact argv, cwd, direct exit, UTC start/end, duration, stdout hash and stderr hash. Raw stdout and stderr are separate files. An empty stream is an actual empty file. No shell pipeline status substitutes for a command's direct status.

The following commands all exited 0:

| Receipt stem under commands/ | Actual argv | Result |
|---|---|---|
| initial-head | `git rev-parse HEAD` | exit 0; 41 stdout bytes; 0 stderr bytes retained |
| initial-status | `git status --porcelain=v1 --untracked-files=all` | exit 0; empty stdout/stderr |
| initial-own-diff-stat | `git --no-pager diff --stat` | exit 0; empty stdout/stderr |
| subject-diff-stat | `git --no-pager diff --stat 38cfefc07ae27ccc5e9b51ca2159b5015fab1f50 c99776eb14173b467143525947cd0933b983b459` | exit 0; 251 stdout bytes; 0 stderr bytes retained |
| subject-diff-check | `git diff --check 38cfefc07ae27ccc5e9b51ca2159b5015fab1f50 c99776eb14173b467143525947cd0933b983b459` | exit 0; empty stdout/stderr |
| working-tree-diff-check | `git diff --check` | exit 0; empty stdout/stderr |
| subject-diff | `git --no-pager diff --no-ext-diff 38cfefc07ae27ccc5e9b51ca2159b5015fab1f50 c99776eb14173b467143525947cd0933b983b459` | exit 0; 15443 stdout bytes; 0 stderr bytes retained |
| subject-log | `git log -1 --format=fuller` | exit 0; 589 stdout bytes; 0 stderr bytes retained |
| final-head | `git rev-parse HEAD` | exit 0; 41 stdout bytes; 0 stderr bytes retained |
| final-status | `git status --porcelain=v1 --untracked-files=all` | exit 0; empty stdout/stderr |
| final-own-diff-stat | `git --no-pager diff --stat` | exit 0; empty stdout/stderr |
| final-working-tree-diff-check | `git diff --check` | exit 0; empty stdout/stderr |
| final-subject-diff-check | `git diff --check 38cfefc07ae27ccc5e9b51ca2159b5015fab1f50 c99776eb14173b467143525947cd0933b983b459` | exit 0; empty stdout/stderr |
| subject-diff-names | `git --no-pager diff --name-status 38cfefc07ae27ccc5e9b51ca2159b5015fab1f50 c99776eb14173b467143525947cd0933b983b459` | exit 0; 122 stdout bytes; 0 stderr bytes retained |

The 20 commands/link-NN.json receipts retain exact `git cat-file -e c99776eb14173b467143525947cd0933b983b459:<resolved_path>` argv; all 20 exited 0 with empty stdout/stderr. link-readback.json records every occurrence, whether introduced, its resolved path, SHA256 and fragment heading. There are 11 introduced occurrences, six GitHub source URLs, and three fragment occurrences. Both occurrences of `logical-interface-and-delivery-owners` and the one `composition-clients-selected-operations-and-byte-transport` fragment match exact headings inspected in the source. GitHub source URLs were mapped to tracked subject blobs; no remote HTTP request was made. This is source/link readback, not rendered-site or remote-publication verification.

| Link receipt | Public source | URL | Introduced |
|---|---|---|---|
| link-01 | website/docs/concepts/ess.md | `../examples/specification-to-contracts.md` | false |
| link-02 | website/docs/concepts/ess.md | `../guides/check-infrastructure.md` | false |
| link-03 | website/docs/concepts/ess.md | `../guides/record-realization.md` | true |
| link-04 | website/docs/concepts/ess.md | `../guides/synthesize.md` | false |
| link-05 | website/docs/concepts/ess.md | `../guides/track-change.md` | false |
| link-06 | website/docs/concepts/ess.md | `../guides/verify-conformance.md` | false |
| link-07 | website/docs/concepts/ess.md | `../reference/cli.md#composition-clients-selected-operations-and-byte-transport` | true |
| link-08 | website/docs/concepts/ess.md | `component-delivery.md` | true |
| link-09 | website/docs/concepts/ess.md | `https://github.com/beyond10x/ess/blob/main/crates/generate/ess-deployment/tests/deployment.rs` | true |
| link-10 | website/docs/concepts/ess.md | `https://github.com/beyond10x/ess/blob/main/crates/generate/ess-synth/tests/clap.rs` | true |
| link-11 | website/docs/concepts/ess.md | `https://github.com/beyond10x/ess/blob/main/crates/specify/ess-composition/tests/fixtures/compositions/workbench.yaml` | true |
| link-12 | website/docs/concepts/ess.md | `https://github.com/beyond10x/ess/blob/main/examples/gatepass-realization/src/bin/gatepass-server.rs` | true |
| link-13 | website/docs/concepts/ess.md | `https://github.com/beyond10x/ess/blob/main/examples/gatepass/components.yaml` | true |
| link-14 | website/docs/concepts/ess.md | `https://github.com/beyond10x/ess/blob/main/examples/realizations/billing-local.yaml` | true |
| link-15 | website/docs/guides/write-a-specification.md | `../concepts/ess.md` | false |
| link-16 | website/docs/guides/write-a-specification.md | `../concepts/ess.md#logical-interface-and-delivery-owners` | true |
| link-17 | website/docs/guides/write-a-specification.md | `../examples/specification-to-contracts.md` | false |
| link-18 | website/docs/guides/write-a-specification.md | `./verify-conformance.md` | false |
| link-19 | website/docs/guides/write-a-specification.md | `generate-artifacts.md` | false |
| link-20 | website/docs/guides/record-realization.md | `../concepts/ess.md#logical-interface-and-delivery-owners` | true |

The 43 subject and three base input snapshots were obtained by direct `git show <exact-commit>:<path>` commands, all exit 0 with empty stderr. The exact per-input argv/status and raw stdout bytes are retained in inputs/NN.json, inputs/NN.stderr and inputs/NN.source. Four local read-only inputs were copied verbatim without a Git lookup; their source kind is explicit. Every subject input was also compared byte-for-byte with its worktree path and matched. Complete snapshots bind input bytes; they do not increase the claimed content-review extent.

## 5. Complete input inventory and changed-file identity

input-hashes.json SHA256: `84959b113fbc1396d8bb10fbd4bc56cd454214ba93ad25f74a6040ceb1d95a2a`. This inventory has 50 rows: 43 exact subject blobs, four local inputs, and three exact base page blobs. Each row below includes its complete SHA256; input-hashes.json additionally preserves bytes, Git blob identity where applicable, snapshot and receipt paths.

| Input / revision | Inspection extent | SHA256 |
|---|---|---|
| `.engineering/planning/story/review-glossary-boundaries.md` (subject) | full content read | `5470a6230011ee3a588d01ec62028ecc8acf19767b7460446a2af90c8d69b55b` |
| `.engineering/planning/story/review-public-support-claims.md` (subject) | full content read | `207796572a233c91cf937580536ab3ac28d3a058885be575f21f1f6e55c5d313` |
| `AGENTS.md` (subject) | full content read | `9f7275e26a79f61a1f5560cfaa64584c4e6df9461496c8005bb34a583b3419ec` |
| `crates/edge/ess-cli/src/main.rs` (subject) | selected source sections: 80–110, 1875–1920 | `1332526c7fe67a3e92a23724e862ed0cc86ccd14c3f7ea5bfa0797c13937a029` |
| `crates/generate/ess-deployment/src/component.rs` (subject) | full content read | `d6acc168081e1978b8fbfde6dcfd54bf7f086b07dff6956eff4879997d145b82` |
| `crates/generate/ess-deployment/src/environment.rs` (subject) | selected source sections: 125–165 | `fe92b2b0ef2b561f0cd862cd5c669dfcb1d12c0e9e695e7e44b928431d2fab7e` |
| `crates/generate/ess-deployment/src/runtime.rs` (subject) | selected source sections: 160–265, 475–700 | `b119a25325ba7e01bb533febbd6a59f4be26e122d72bfcdfc9fd1f8f4bc12138` |
| `crates/generate/ess-deployment/src/stack.rs` (subject) | selected source sections: 20–95, 188–225, 390–455, 625–665 | `8668ac4bf52bc78c63e065fe242aaa078adf1cab451c3f4f6d681b935d6c51f1` |
| `crates/generate/ess-deployment/tests/deployment.rs` (subject) | selected source sections: 1–440 | `189c6584925154335c7c2c7b1777f79222c5e2391202ab115ec2d8ef8f4cbbc3` |
| `crates/generate/ess-gen/src/document.rs` (subject) | selected source sections: 1–68 | `bf3c6a1ec618a2e7269a4e1eef51e03d341608d427981b16dac38929a92a0d6c` |
| `crates/generate/ess-gen/src/http.rs` (subject) | selected source sections: 135–245 | `d170e0243bbfaf963ec419f7d67231064ed9731ec2a655c5fa9cf86c60d916a1` |
| `crates/generate/ess-synth/src/clap/mod.rs` (subject) | selected source sections: 1–80 | `b29f950bf6e7acc80d6912469a179da316fffd6e3ff488b24b887f711e8440d5` |
| `crates/generate/ess-synth/src/clap/tree.rs` (subject) | full content read | `028b76d8d982388d1360e5cc886e010994623e4c369adf1303878e178918296c` |
| `crates/generate/ess-synth/src/go/http.rs` (subject) | selected source sections: 40–80 | `f878e2fb986cabaf95eacb5f9d9e156341c2b252641a588e2ed85965d8c92f61` |
| `crates/generate/ess-synth/src/plan.rs` (subject) | selected source sections: 1090–1130 | `8c9ae703b95224a2146a28269d2f06ed6d45c9a4de9a9702fc6e5cf031a0bd5d` |
| `crates/generate/ess-synth/src/rust/http.rs` (subject) | selected source sections: 90–135 | `957baf4ec74c64b4141aad29934f9df45469dff17f9f789d84b7a701fdc85eb5` |
| `crates/generate/ess-synth/tests/clap.rs` (subject) | full content read | `c28481cf6d6ea2782b87145facac138c9d3b9502c315c574b27f6a567aca3c7f` |
| `crates/specify/ess-compiler/src/ir.rs` (subject) | selected source sections: 775–825, 938–970, 1100–1195, 1560–1595 | `8df6236b197e791a9ebdb8890b956235004920c3eb3acb90f7ad6118eb4ab773` |
| `crates/specify/ess-composition/src/lib.rs` (subject) | selected source sections: 175–245, 470–525, 680–790, 1175–1455 | `db71d32fd00681c3047b1efd41759898ee4e3923ba3373b0580430a81c309afb` |
| `crates/specify/ess-composition/tests/fixtures/compositions/workbench.yaml` (subject) | full content read | `cc50466ef53efc0fe5a2d830e77a5f55dcfb8e876b1d1d71bcccf99acea24a0c` |
| `crates/specify/ess-composition/tests/fixtures/two-components/components.yaml` (subject) | full content read | `74c6db02c074ee0113543dad098ffcec6575afa116dab99cf5ad8ea0f7e34869` |
| `crates/specify/ess-composition/tests/fixtures/two-components/system.yaml` (subject) | full content read | `5dc7bf027680f8012d714bdf869856cafe2e9f4d2140517ec0234193bc4d7524` |
| `crates/specify/ess-domain/src/component.rs` (subject) | selected source sections: 1–260, 345–405, 560–745, 920–980 | `52ba006da3f5929f30f55b2bb80e4f29aef4ec39a80a26181b7cccdfaeb0d309` |
| `crates/specify/ess-domain/src/topology.rs` (subject) | selected source sections: 105–240 | `a01940c6f513951ea59ca5c6d950305be507bd04544329b77c4e8024afcb1dfb` |
| `crates/specify/ess-realization/src/lib.rs` (subject) | selected source sections: 1–50, 285–455, 640–720, 725–1250 | `feeeedbfb1b0c4bb5d6cac0d7b3ca422ebf1ac0edcbcfa2f206eba2ae0745caa` |
| `docs/design/review-concept-boundaries.md` (subject) | full content read | `271910e53562be0e018db7b39fa9d52f2aee7333201d6bf437f7793cd7a563cd` |
| `docs/reviews/2026-09-05-architecture-review.md` (subject) | selected source sections: 410–455, 520–545, 575–612 | `fecce053b6633546f10613c580eba1b5c48be3315d70c6b4369f26143020f828` |
| `examples/billing/components.yaml` (subject) | full content read | `ee4986381f8dbb44f76ab05c44742f4654795e369ef46c0ed3aab8e3b85f2583` |
| `examples/gatepass-realization/src/bin/gatepass-server.rs` (subject) | full content read | `ae87e69267ee66c09b059497bdcdb20257d169985eecfa11cb76c086c49b6c10` |
| `examples/gatepass/components.yaml` (subject) | full content read | `23a3c0027844d261f07448abb3c4b8de505eec28e3c78f33e21d21f42a59e06d` |
| `examples/oracle-fixture/components.yaml` (subject) | full content read | `2c82e4c250f99f00c32e7e9656427184a68519458c59aa8c64d95abd09e724ba` |
| `examples/realizations/billing-local.yaml` (subject) | full content read | `287b2e8335e298185ba9b9149a2a0a13d36740f067f37f23a1b1273220189136` |
| `website/docs/concepts/component-delivery.md` (subject) | full content read | `990fd0b9140513b94ca7ac5f9f92a994bf0d49f33898e1db7aadfc8021912cfb` |
| `website/docs/concepts/ess.md` (subject) | full content read | `c75543b6f6ad17d733a9212c218ebe8af17cb33ea3f02a3d0aa575bb4221c825` |
| `website/docs/examples/specification-to-contracts.md` (subject) | link target existence and whole-byte hash only | `71fe883868ff340abfeb95a59e79666de443ed82c963293b50b59bfdb41b5927` |
| `website/docs/guides/check-infrastructure.md` (subject) | link target existence and whole-byte hash only | `5577fbe4b18dc4232f4a5073a4ab17716e3a023ecdd843f5e927a248db6cc2eb` |
| `website/docs/guides/generate-artifacts.md` (subject) | selected source sections: 133–173 | `2dcafb34ff80e19622aef7d3565f9353d6e32420d2f7503e53a3f1047fee9f53` |
| `website/docs/guides/record-realization.md` (subject) | full content read | `f5bf75505f058556c35cbf33aee957ef75380599a4884ce897d8ad40ceb15659` |
| `website/docs/guides/synthesize.md` (subject) | link target existence and whole-byte hash only | `448ada65617b7ed650b10fe432be0de7b8890fb7d30e865ac021b8ed4a4178ea` |
| `website/docs/guides/track-change.md` (subject) | link target existence and whole-byte hash only | `a5627d671a46326301ea52557f5c3bec8b95c0ebcc44cf86bd8fe844809e1461` |
| `website/docs/guides/verify-conformance.md` (subject) | selected source sections: 91–162 | `13bd30869cb53e12b193d032a5d53f33953663b47a52e11d18f9be766dfc7898` |
| `website/docs/guides/write-a-specification.md` (subject) | full content read | `b8a72fa03865a251cd0d60709c1c3c5ef8a32abd42409f6daf4a89ce3c8a7ed7` |
| `website/docs/reference/cli.md` (subject) | selected source sections: 50–132 | `9e24a993e5647c99ecbb78563c6b8a9074c8870e3f1101e6858ed4d8ff413e60` |
| `/home/timo/.codex/plugins/cache/beyond10x/aep-drive/0.8.0/agents/adversary.md` (local input) | full content read | `680ac11700dc215bf39201d7019b36277f8b260b673f99c6f09e8a4eb051f795` |
| `/home/timo/.local/state/worktree/trees/b10x/ess/ess-glossary-boundaries/target/review-boundaries-13/adversary-pass-1/brief.md` (local input) | full content read | `6f225159617f657f2f2bc40151c937e1c05cd0e5fee6734f7441203fa57d1e63` |
| `/home/timo/.local/state/worktree/trees/b10x/ess/ess-glossary-boundaries/target/review-boundaries-13/glossary/brief.md` (local input) | full content read | `a239146fcf3bd838f509e1f0b8bb2951ded239b2001a1c03689864ddc6bcc69d` |
| `/home/timo/.local/state/worktree/trees/b10x/ess/ess-glossary-boundaries/target/review-boundaries-13/glossary/implementation/report.md` (local input) | full content read | `7a3d758cf5e47267ec0545e5ce3c5020c7f690e75945f1ea6df52ac958dd9dbb` |
| `website/docs/concepts/ess.md` (base) | whole-byte base snapshot and complete subject-diff comparison; not a separate full-page read | `18bdfdcb0e955e9f5f50f44551d15f6a7228852a8314f9719cb99bb674dc45ca` |
| `website/docs/guides/write-a-specification.md` (base) | whole-byte base snapshot and complete subject-diff comparison; not a separate full-page read | `b35bf28f23d989d161c63e7c2493793d1523a35597bbe17fe96a1feeadbcba98` |
| `website/docs/guides/record-realization.md` (base) | whole-byte base snapshot and complete subject-diff comparison; not a separate full-page read | `543630c139f5ea4908057991f7a14218cc71faa015c8eef64fde600e6daeacff` |

Selected source ranges are inclusive and stop at EOF where a requested endpoint exceeds file length. Numbered display/output limits are not counted as full reads: incomplete initial displays were followed by bounded rereads of the substantive sections, and only the actual covered sections are listed. The late billing-components display that overflowed the conversation was reread successfully before sealing. Full hashes and whole-byte snapshots do not imply the uninspected remainder of a selected file was semantically reviewed.

Changed public-file subject hashes:

| Path | Bytes | SHA256 |
|---|---|---|
| website/docs/concepts/ess.md | 17278 | `c75543b6f6ad17d733a9212c218ebe8af17cb33ea3f02a3d0aa575bb4221c825` |
| website/docs/guides/record-realization.md | 3308 | `f5bf75505f058556c35cbf33aee957ef75380599a4884ce897d8ad40ceb15659` |
| website/docs/guides/write-a-specification.md | 16988 | `b8a72fa03865a251cd0d60709c1c3c5ef8a32abd42409f6daf4a89ce3c8a7ed7` |

## 6. Writes, native inventory and handoff boundary

Every path written outside this managed worktree: none. External scratch: none. All adversary writes are under target/review-boundaries-13/adversary-pass-1: the existing dispatch brief, commands/ receipts and raw streams, inputs/ verbatim input snapshots and receipts, changed-file-subject-hashes.json, link-readback.json, input-hashes.json, this report.md, and the closing scratch-inventory.json, seal.json and seal-readback.json carriers. The brief was supplied by root and was not rewritten. No production/test/fixture/document, Git, planning-store or worktree-lifecycle mutation occurred; no cleanup or output deletion occurred.

scratch-inventory.json enumerates the entire assigned scratch without following symlinks, including hidden and empty directories, literal symlink targets if any, native path-name bytes, file kinds, SHA256 for regular payload files, mode/uid/gid/size/mtime/ctime/device/inode/link count/blocks, and available extended attributes. Access time is observational only and excluded from stability comparison because reading can update it. The three named carrier files are pre-created before the directory census and explicitly excluded from recursive payload hashes/native stability checks; their names and kinds remain in the complete census. This is a self-reference exception, not a missing payload. Their completed hashes and native metadata are returned after the closing readback.

seal.json binds the report, input manifest and complete payload inventory. seal-readback.json retains the subsequent byte-hash, native-metadata and complete-census comparison, including exact subject-input and HEAD/status readbacks. The final carrier hashes and metadata are supplied in the quiescent handoff. No build binaries or test outputs exist from this prose pass. Retention, later full ESS/site/delivery gates, any review-result recording and every lifecycle operation remain coordinator-owned.

The attack is pass 1 of at most two; no broader third attack was attempted. All assigned writes cease at handoff.

```findings
[]
```
