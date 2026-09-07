---
format: aep.planning-md/1
id: review-result:authored-discovery-binding-pass1
kind: review-result
status: active
title: Authored discovery candidate binding pass 1
relations:
- reviews: story:review-authored-discovery
revision: 1
---
unit: unselected authored-discovery candidate v2; source subject ecb7efc22ad9b19b85ef4debd8143491d6a66ef3
verdict: nothing found
cases: not run (read-only candidate document attack; source and retained-receipt inspection)
origin: introduced 0 / pre-existing 0 / undecided 0
wrote-outside-worktree: none
needs-coordinator: none

`git --no-pager diff --stat` was not run: this assignment prohibited Git operations. No files were written anywhere.

The exact candidate is [binding-candidate-v2.md](/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/review-boundaries-13/preparation/authored-discovery-candidate/binding-candidate-v2.md), SHA256 `56422a3eeaad06cb92a5de587a24bfdef03e2b26be0cfc7543a276ac4535715b`, 26,155 bytes. Its recorded preparation subject is `dbe78c5b15df478ec2cd4883c67d0012cdf90e17`; the coordinator supplied `ecb7efc22ad9b19b85ef4debd8143491d6a66ef3` as the current published source. The hashes below identify the source bytes actually inspected; no independent Git verification is claimed.

The complete existing story and prerequisite were read. The candidate preserves the story’s acceptance: a documented mixed layout must select the intended authored inputs deterministically without ingesting generated YAML. It also preserves the implementation boundary, migration requirement, explicit-empty refusal and validation obligations. No story was rewritten, narrowed or supplemented.

This is one bounded candidate critique. It neither selects the manifest approach over uniform typed discovery nor accepts `ess-inputs/1`, the proposed model locations, or a next implementation wave.

**Source-backed judgment**

No concrete unsupported invariant, incompatible guarantee or acceptance gap was found in this pass.

- The proposed shared owner covers the three actual acquisition implementations: [model loading](/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/crates/edge/ess-cli/src/load.rs:25), [legacy authored loading](/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/crates/edge/ess-cli/src/main.rs:2833), and [coverage authored loading](/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/crates/edge/ess-cli/src/coverage.rs:16). Explicit-file extension independence, shallow legacy authored selection, omitted scenarios and the different legacy symlink policies remain distinct.
- The candidate’s manifest selection does not depend on successful document parsing. This preserves headerless fragments and visible selected-document refusals. [RawSpecFile](/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/crates/specify/ess-domain/src/spec.rs:45) has optional header fields; its [parser](/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/crates/specify/ess-domain/src/spec.rs:151) rejects unknown fields and duplicate YAML mapping keys. [Header absorption](/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/crates/specify/ess-domain/src/spec.rs:625) and [header resolution](/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/crates/specify/ess-domain/src/system.rs:715) remain the semantic owners.
- Checked source identity is appropriately separated from a physical path and diagnostic origin. The proposed lexical grammar matches [SourceIdentity::new](/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/crates/verify/ess-conformance/src/coverage.rs:73). [CoverageSource](/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/crates/verify/ess-conformance/src/coverage_build.rs:84) retains the supplied identity and original text. Canonical-path rejection and deliberate preservation of distinct hardlink/copy identities are compatible choices.
- The candidate preserves source-acquisition bypass for committed execution. [Run dispatch](/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/crates/edge/ess-cli/src/main.rs:2495) selects `--suite-input` or `--suite` before fresh model/scenario acquisition; [argument declarations](/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/crates/edge/ess-cli/src/main.rs:492) distinguish their conflicts.
- Acquisition failures are correctly distinguished from document-level semantic refusals. The candidate does not promise that every old semantic failure preserves outputs: [legacy author](/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/crates/edge/ess-cli/src/main.rs:2767), [legacy web](/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/crates/edge/ess-cli/src/main.rs:2719), and [coverage generation](/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/crates/edge/ess-cli/src/coverage.rs:94) can retain incomplete evidence after acquisition succeeds.
- Output protection is preserved as an independent boundary. [Model types](/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/crates/edge/ess-cli/src/model_types.rs:48) and [normalization](/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/crates/edge/ess-cli/src/normalize.rs:178) protect their input directories. A manifest does not grant permission to place those outputs inside the model input root.

The caller inventory was checked against dispatch and source calls: validation, compilation, inspection, graph, composition service inputs, realization specification inputs, runtime system inputs, projections, structural synthesis, model types, normalization models, diff/impact revisions and fresh conformance. Flat and area spellings converge through [command dispatch](/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/crates/edge/ess-cli/src/main.rs:979). For projection, the model-consuming route is specifically [OpenAPI `--path`](/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/crates/edge/ess-cli/src/main.rs:3105); unrelated persisted-IR projection inputs remain separate.

**Model and parser constraints**

The proposed declaration contains three named value types and no entity, lifecycle or file-ownership relation. Its singleton enum spelling is supported by [RawTypeBody::Enum](/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/crates/specify/ess-domain/src/types.rs:627), [shape validation](/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/crates/specify/ess-domain/src/types.rs:447) and [enum resolution](/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/crates/specify/ess-compiler/src/resolve.rs:1022). The unrelated restrictive grammar belongs to [entity StateName](/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/crates/specify/ess-domain/src/entity.rs:68).

The constraint accounting is appropriately limited:

| Constraint | Source-backed disposition |
|---|---|
| Required three-field struct | Non-optional fields produce required properties; the existing [object projection](/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/crates/generate/ess-gen/src/types.rs:625) refuses additional properties. |
| Exact format value | The [enum projection](/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/crates/generate/ess-gen/src/types.rs:662) is enforcing; the declaration does not install an acquisition reader. |
| Nonempty path | The newtype invariant is expressible. Its schema projection carries an annotation, so reader enforcement remains explicitly required. |
| Full path grammar and filesystem checks | These remain reader obligations; no string newtype is presented as proof of containment, file kind or symlink handling. |
| Active-role nonempty selection | The role is supplied by the caller. Imposing unconditional list cardinality would contradict the allowed inactive empty list. |
| Uniqueness and disjointness | No complete general constraint for these scalar lists was found in the inspected authored predicate syntax. Membership operators take literal values; they do not supply dynamic set membership. |
| Scalar binder comparison | [Operand parsing](/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/crates/specify/ess-primitives/src/predicate.rs:259) treats undotted right-hand text as a literal. Structured comparison uses the same parser at line 991. |
| `.value` workaround | [Typed resolution](/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/crates/specify/ess-domain/src/expression.rs:397) unwraps aliases transparently; a scalar binder does not gain a `.value` field. |
| Sorting and exact byte retention | `List` is an ordered sequence, not a canonical acquisition algorithm. Both remain concrete reader obligations. |

**All A1–A24 cases inspected**

These are assessments of the candidate requirements and their source compatibility, not executed results.

| Case | Bounded disposition |
|---|---|
| A1 | Existing directory/single-file/headerless inputs are retained; the manifest introduces a separate selection branch. |
| A2 | Legacy shallow extension filtering and explicit-file extension independence are retained. |
| A3 | Omitted scenarios return no sources; a model manifest does not activate authored selection. |
| A4 | Explicit empty, nested-only and nonmatching legacy selections retain actionable non-success before output. |
| A5 | Exact lists cover nested mixed layouts; unlisted generated, malformed and ESS-shaped copies stay outside acquisition. |
| A6 | Sorting relative identities supports enumeration-order, list-order and relocation controls. Byte equality remains a future executable obligation. |
| A7 | Unlisted-file changes cannot affect manifest acquisition under the specified no-walk rule. |
| A8 | Header filename/location can vary in manifest mode while existing assembly rules still apply. |
| A9 | Missing/duplicate headers, declarations and malformed selected fragments remain visible; no parse-success filtering is proposed. |
| A10 | Closed configuration, duplicate keys, one document and exact version are explicit reader requirements. |
| A11 | Whole-manifest structural checks coexist with active-role-only filesystem access and active nonempty selection. |
| A12 | Selected foreign/generated documents reach the selected role’s reader; no silent role switching or omission is proposed. |
| A13 | Lexical duplicates/overlap are checked before active filesystem resolution; repeated active canonical targets refuse. |
| A14 | Distinct requested hardlink/copy identities remain present for deterministic downstream duplicate refusal. |
| A15 | The grammar matches SourceIdentity and preserves accepted spelling. |
| A16 | Manifest/root/selected-path file-kind and symlink checks are explicit; unlisted entries remain uninspected. |
| A17 | Legacy link behavior remains separate; sorted traversal addresses first-visited canonical-directory aliases. |
| A18 | Exact text retention preserves LF/CRLF source-digest distinctions. |
| A19 | The no-output/no-runner requirement is scoped to discovery/acquisition failures. |
| A20 | Committed-suite bypass and argument conflicts match current dispatch. |
| A21 | All three acquisition adapters and their actual model/authored caller routes are accounted for. |
| A22 | Reserved-name migration is explicit. Old readers must reject the unsupported configuration; the candidate correctly allows existing incomplete semantic evidence behavior. |
| A23 | Model-types and normalization containment protections remain explicit. |
| A24 | Named-model validation is retained separately from future reader/fixture agreement checks and expression gaps. |

Existing regression source was inspected for empty selection, shallow/direct-file behavior, ordering, legacy links, coverage relocation and exact newline bytes. None of those tests ran during this pass.

**Retained validation receipts**

The pinned binary’s complete bytes hash to `fb1121b632781343fd06955778d28ae095cff5f305e9f3c5005f2ca93c8a8d1f`, matching `model-validator-pin.json`. Each retained stdout/stderr hash matches its command receipt; all recorded direct exits are zero and all stderr streams are empty.

| Recorded operation | Recorded result |
|---|---|
| `tools/ess --version` | `ess 0.20.0` |
| `tools/ess specify validate --path model-v1` | `discovery v1 — 2 file(s), valid` |
| `tools/ess specify compile --path model-v1 --format json` | JSON containing exactly the proposed InputManifest, ManifestFormat and RelativeInputPath types. |

The receipts record execution at `2026-09-07T02:38:54Z`. I inspected those existing results; I did not execute the binary. They validate the named declarations, not a manifest instance, discovery behavior, filesystem policy or A1–A24 implementation.

**Exact input inventory**

Path prefixes below expand literally:

- `R/` = `/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/`
- `C/` = `R/target/review-boundaries-13/preparation/authored-discovery-candidate/`
- `F` = complete content read; `S` = selected sections/search hits; `H` = complete-byte hash only.

Whole-file hashes do not imply whole-file semantic review.

| Input | Read | Bytes | SHA256 |
|---|---:|---:|---|
| `R/AGENTS.md` | F | 7816 | `9f7275e26a79f61a1f5560cfaa64584c4e6df9461496c8005bb34a583b3419ec` |
| `R/.engineering/planning/story/review-authored-discovery.md` | F | 4443 | `752f7b97448a85e52e2868c1c8c47103eb3f8eb0291875776d8dc0e52536b56f` |
| `R/.engineering/planning/story/scenarios-directory-compiles-nothing.md` | F | 7050 | `d031e07c8bed3cfa7a50d977ce17c7413c08a0303786e8b96743acd1572cff61` |
| `R/crates/edge/ess-cli/src/load.rs` | F | 5879 | `47efbb430eb57640138f2731d3b409219bdde1047674ab08c69c7c86d5dcbfec` |
| `R/crates/edge/ess-cli/src/coverage.rs` | F | 8137 | `fa6bfe23553296e53cf7e8b60ff2021ce6ac886bff32cdd741607e4907a82982` |
| `R/crates/edge/ess-cli/src/model_types.rs` | F | 2377 | `713755f149115cd05e4f454a1e21bcf783d13da4ce76a724d124e706fb078e86` |
| `R/crates/edge/ess-cli/src/main.rs` | S | 139710 | `1332526c7fe67a3e92a23724e862ed0cc86ccd14c3f7ea5bfa0797c13937a029` |
| `R/crates/edge/ess-cli/src/normalize.rs` | S | 8531 | `a38a4b32053703832f9012187a57d3a0902f5cf9e78d1a05d083ef99039e1a64` |
| `R/crates/edge/ess-cli/tests/authored_scenarios.rs` | S | 15428 | `361b7e983a8f90b148022fb09f5e536f1f039d7f74c2e9977275c6e8591be51e` |
| `R/crates/edge/ess-cli/tests/authored_scenarios_adversary.rs` | S | 11899 | `4ce6981231e3ed795c6d2063f936488eded76de8d6e054b742a46b8fd30508cb` |
| `R/crates/edge/ess-cli/tests/coverage_cli.rs` | S | 16006 | `6602c0ec47f090bbac6a0566fd9ded620f9d35494b3d814be1712982a5f7f424` |
| `R/crates/specify/ess-domain/src/spec.rs` | S | 52421 | `f2bb53ee20752155e9442fdf8ee6149edc3157acb55a00caf3c9d51b4b7c4b0c` |
| `R/crates/specify/ess-domain/src/system.rs` | S | 66199 | `a5ae4ad79e5d3d8d9a5ab40cb9babc3e888416ca5e8460bcf0f678ca45396c22` |
| `R/crates/specify/ess-domain/src/types.rs` | S | 53278 | `c375e55b60328e5ceace44005e7aec0e1e0819cb23d511c72b56626425e39b7d` |
| `R/crates/specify/ess-domain/src/entity.rs` | S | 119664 | `be3458aa4f57bfdfd9ef477c5657041d22b33c49cb69e35f705c802da83a8551` |
| `R/crates/specify/ess-domain/src/expression.rs` | S | 26985 | `1a399d89799c411a32cd157cc2966a4d59bee8664dca6b219df61f11b5dfcb74` |
| `R/crates/specify/ess-primitives/src/predicate.rs` | S | 72149 | `8320986799db4ebd195e1894fc6d125b13b9ff003935054e7f5ddd299a4afd43` |
| `R/crates/specify/ess-compiler/src/resolve.rs` | S | 130225 | `62862f79fb8a454d68ae19c2af0b7713586d40914091c1d91004736303839147` |
| `R/crates/generate/ess-gen/src/types.rs` | S | 51910 | `6ef7728086204f69c782b9cd2134b0b7d176aa0b105e4ab74a058872c1fa76c1` |
| `R/crates/verify/ess-conformance/src/coverage.rs` | S | 33620 | `4f244ec4872b5c7cdd0756d5d149eafd5b3ba988b6c30a6eff87b223fded3128` |
| `R/crates/verify/ess-conformance/src/coverage_build.rs` | S | 20753 | `5ede58ce8f9e952813a49fac8bd67a083187332c78b8577f398ffceac7afcd4c` |
| `R/crates/verify/ess-conformance/src/authored.rs` | S | 95379 | `7ac2a8248d74815fd2fac0cf207cea3e0e17de6afcbd9d7ee9b995e1102e11ec` |
| `R/website/docs/guides/write-a-specification.md` | S | 16988 | `b8a72fa03865a251cd0d60709c1c3c5ef8a32abd42409f6daf4a89ce3c8a7ed7` |
| `R/website/docs/guides/verify-conformance.md` | S | 9101 | `13bd30869cb53e12b193d032a5d53f33953663b47a52e11d18f9be766dfc7898` |
| `R/docs/reviews/2026-09-05-architecture-review.md` | S | 55490 | `fecce053b6633546f10613c580eba1b5c48be3315d70c6b4369f26143020f828` |
| `C/binding-candidate-v2.md` | F | 26155 | `56422a3eeaad06cb92a5de587a24bfdef03e2b26be0cfc7543a276ac4535715b` |
| `C/model-v1/system.yaml` | F | 74 | `7eae2216cb1f8cd08fee0c8c46a3fb0eb3958959e3ffcee9bbfe7aeaae25f773` |
| `C/model-v1/domains/discovery.yaml` | F | 538 | `29f9968f8740b444d8708dfa2631a5ac37fe7a6302f2df64eb08132b74d3777e` |
| `C/model-proposal-report.md` | F | 11561 | `97c985458f12b4bd49c0d36e3e93355b209d9db030d32d0f0e0363bfc76d5cc0` |
| `C/model-validation-v1/commands.json` | F | 1895 | `66692d42462d4fe9226e9bbbb63f28d65333745ff8297b90d1f7b578a6dc574d` |
| `C/model-validator-pin.json` | F | 636 | `e9382fbd3cabcdb134696b5d8005683c860d1940889372365217534eb83fb6ba` |
| `C/model-validation-v1/version.stdout` | F | 11 | `940a3a88601d2ad87706925034f99d67dbfc6e1e7665ec26bd6ae903d953d01e` |
| `C/model-validation-v1/version.stderr` | F | 0 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `C/model-validation-v1/validate.stdout` | F | 34 | `782d802e4e9f397b6123b345f5b0b29f3ce7b568382ca6a678ccd64f3de60cc2` |
| `C/model-validation-v1/validate.stderr` | F | 0 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `C/model-validation-v1/compile.stdout` | F | 2192 | `92c21069e9da3bdb5a5b46337e3e0a051414f7bbbe7946e8a958e432fc2bfec7` |
| `C/model-validation-v1/compile.stderr` | F | 0 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `C/tools/ess` | H | 99458352 | `fb1121b632781343fd06955778d28ae095cff5f305e9f3c5005f2ca93c8a8d1f` |
| `/home/timo/.codex/plugins/cache/beyond10x/aep-drive/0.8.0/agents/adversary.md` | F¹ | 19212 | `680ac11700dc215bf39201d7019b36277f8b260b673f99c6f09e8a4eb051f795` |

¹ The complete charter read was retained from the preceding assignment; its hash was rechecked here.

Selected inspection covered:

| Source | Content extent |
|---|---|
| `main.rs` | Argument conflicts; flat/area dispatch; realization/runtime inputs; resolved model callers; output sequencing; complete authored helper and conformance acquisition branches; model-versus-IR OpenAPI projection. |
| `normalize.rs` | Lines 52–252, particularly acquisition at 81–125 and output protection at 170–194 and 233–251. |
| Authored scenario tests | Empty-output matrix at 149–188; shallow/direct/order controls at 279–321; additional search hits. |
| Authored adversary tests | Root-link control at 150–176; file-link/direct-extension control at 262–287; additional search hits. |
| Coverage CLI tests | Lines 31–145, including relocation, newline identity and invalid selected paths. |
| `spec.rs` | Lines 1–300, duplicate insertion at 521–551, header absorption at 625–692, and selected member-assembly sections. |
| `system.rs` | Format admission at 50–126 and header/format checks at 715–805. |
| Domain `types.rs` | Constructors at 120–260, named-type checks at 409–483 and 495–655. |
| `entity.rs` | StateName at 52–109. |
| `expression.rs` | Typed root/path resolution at 330–505. |
| `predicate.rs` | Operators, operands, quantifier/value definitions at 1–440; quantifier parsing at 876–925; structured comparison/membership at 950–1045; parser search hits. |
| `resolve.rs` | Named-body resolution at 998–1040. |
| Generator `types.rs` | Required object fields, closed objects, enum/newtype projection and invariant statements at 615–740. |
| Semantic coverage files | SourceIdentity at 40–115; CoverageSource/compilation/inventory at 1–225; authored compilation at 1400–1555. |
| Public guides | Specification layout at 1–88; conformance guide through line 168, with bounded reread of previously truncated coverage text. |
| Architecture review | Lines 356–399, including complete F10. |

No writes, builds, tests, binary execution, Git operations, planning commands, lifecycle actions, integrations or cleanup occurred. No report file was created by this pass; this final message is the complete handoff. The sealed glossary scratch remains untouched. The bounded pass is complete and quiescent.

```findings
[]
```