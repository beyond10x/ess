---
format: aep.planning-md/1
id: epic:qualify-initial-consumer-baseline
kind: epic
status: draft
title: Qualify the finite initial consumer baseline
owner: ESS model and consumer package maintainers
relations:
- informed_by: story:review-consumer-coverage
- informed_by: verification-report:fuzz-seed-baseline-go-panic
- serves: vision:O2
revision: 2
---
# Qualify the finite initial consumer baseline

This is separately owned follow-up work for the initial unknowns discovered by
story:review-consumer-coverage. It does not expand that gate implementation into a promise
that every current consumer behavior is already supported. It is outside the original
31 architecture-review remediation stories and does not reopen their completed fixes.

## Acceptance

Every exact eligible pair in the initial frozen manifest has an attributed, executed
Supported or named Refused result under its bounded profile, or an explicit reviewed
reconciliation for an obligation that no longer exists, with no remaining BaselineUnknown
from that finite set and no new eligibility created by checking current source.

## Evidence and boundaries

Stage 1 output-final2 and output-final3, independently read by root, agree on 1,806 model
obligations and 87 behavioral consumer profiles. Their 157,122 pairs include 54 mandatory
F01 obligations, which are excluded from this follow-up baseline. The remaining 157,068
pairs have no qualified exact assertion. This is an evidence gap, not a finding that every
pair is broken or needs a distinct test. One meaningful case may qualify several precisely
attributed pairs; source enumeration and an aggregate green package cannot qualify one.

The exact manifest is intended for
crates/edge/ess-xtask/src/consumer_coverage/initial-baseline.json. Root accepted the finite manifest at the Stage 1 checkpoint: source commit
7a6d76855e28d280138d2d439eb6f1e7c15a58d9, manifest SHA256
e005a2e74e067ad51315e594151643b702381dc174bb9af5c355c3eeeb17ad51. This epic remains draft and its qualification work is unimplemented. Each group pins its consumer/profile, exact model IDs and
shape dictionary, concrete package owner and unproven behavior statement. No wildcard,
live prefix or future source shape is included. The readback authority is coordinator
preparation/stage1-source-checkpoint-2/source-checkpoint.json, SHA256
4cdb2cb634c51752abcda8fda80ced28c64fd70304cd4c549c805ee05c3983ea.

The known Go system-level-type panic is separately evidenced by
verification-report:fuzz-seed-baseline-go-panic and its concrete repair remains with
story:fuzz-the-specification-surface. A panic remains broken evidence, never Refused.
Native Go, browser and optional TypeScript execution requires its own measured runtime;
source emission, Rust wrapper success and skipped native work establish no such claim.

## Finite ownership scope

The following package owners and current profile boundaries come from the complete
Stage 1 consumer-profiles.json and pending-owner-groups.json, read by root. The count
is the exact current unknown pair list for that profile, not a future prefix expansion.
Implementation reservations must be scoped for a selected bounded reduction before dispatch.

| Profile | Package owner | Exact pairs | Unproven boundary |
| --- | --- | ---: | --- |
| acquisition-authored-direct-file | ess-cli | 1806 | authored acquisition under direct-file: exact input identities/bytes, role-specific filesystem checking and discovery refusal before output; authored(None, coverage=false/true) remains empty before acquisition; specification requires an explicit path. |
| acquisition-authored-legacy-directory | ess-cli | 1806 | authored acquisition under legacy-directory: exact input identities/bytes, role-specific filesystem checking and discovery refusal before output; authored(None, coverage=false/true) remains empty before acquisition; specification requires an explicit path. |
| acquisition-authored-manifest | ess-cli | 1806 | authored acquisition under manifest: exact input identities/bytes, role-specific filesystem checking and discovery refusal before output; authored(None, coverage=false/true) remains empty before acquisition; specification requires an explicit path. |
| acquisition-authored-omitted-scenarios | ess-cli | 1806 | authored acquisition under omitted-scenarios: exact input identities/bytes, role-specific filesystem checking and discovery refusal before output; authored(None, coverage=false/true) remains empty before acquisition; specification requires an explicit path. |
| acquisition-coverage-direct-file | ess-cli | 1806 | coverage acquisition under direct-file: exact input identities/bytes, role-specific filesystem checking and discovery refusal before output; authored(None, coverage=false/true) remains empty before acquisition; specification requires an explicit path. |
| acquisition-coverage-legacy-directory | ess-cli | 1806 | coverage acquisition under legacy-directory: exact input identities/bytes, role-specific filesystem checking and discovery refusal before output; authored(None, coverage=false/true) remains empty before acquisition; specification requires an explicit path. |
| acquisition-coverage-manifest | ess-cli | 1806 | coverage acquisition under manifest: exact input identities/bytes, role-specific filesystem checking and discovery refusal before output; authored(None, coverage=false/true) remains empty before acquisition; specification requires an explicit path. |
| acquisition-coverage-omitted-scenarios | ess-cli | 1806 | coverage acquisition under omitted-scenarios: exact input identities/bytes, role-specific filesystem checking and discovery refusal before output; authored(None, coverage=false/true) remains empty before acquisition; specification requires an explicit path. |
| acquisition-specification-direct-file | ess-cli | 1806 | specification acquisition under direct-file: exact input identities/bytes, role-specific filesystem checking and discovery refusal before output; authored(None, coverage=false/true) remains empty before acquisition; specification requires an explicit path. |
| acquisition-specification-legacy-directory | ess-cli | 1806 | specification acquisition under legacy-directory: exact input identities/bytes, role-specific filesystem checking and discovery refusal before output; authored(None, coverage=false/true) remains empty before acquisition; specification requires an explicit path. |
| acquisition-specification-manifest | ess-cli | 1806 | specification acquisition under manifest: exact input identities/bytes, role-specific filesystem checking and discovery refusal before output; authored(None, coverage=false/true) remains empty before acquisition; specification requires an explicit path. |
| authored-admission | ess-domain | 1806 | Admit the authored RawSpecFile wire grammar; parsing does not qualify later consumer behavior. |
| authored-assembly | ess-domain | 1806 | Assemble original source parts and run model validation. |
| authored-validation | ess-domain | 1806 | Validate a retained specification including manually assembled model values. |
| browser-legacy-replay | ess-cli | 1806 | Legacy browser replay is a bounded display/replay contract, not full view evaluation. |
| browser-paired-replay | ess-cli | 1806 | Paired suite/report replay retains selected claims and visible unknown-state limits. |
| compiler-private-parts | ess-compiler | 1806 | Private IR assembly, exercised only through compilation or compiler-local cases. |
| compiler-resolution | ess-compiler | 1806 | Resolve typed model references through the public compiler boundary. |
| composition-compile | ess-composition | 1806 | Compile imported service identities and references; no typed runtime payload guarantee. |
| composition-rust-byte-transport | ess-composition | 1806 | Generated Rust client transports bytes with service/operation identity; it does not validate typed payloads. |
| conformance-authored | ess-conformance | 1806 | Compile authored scenario claims under the model. |
| conformance-generated | ess-conformance | 1806 | Generate model conformance requirements. |
| conformance-go-runner | ess-cli | 1806 | Generate and execute Go conformance; an outer Rust early return is not execution. |
| conformance-rust-runner | ess-conformance | 1806 | Execute the admitted finite scenario selection on an actual Rust target. |
| coverage-authored-batch | ess-conformance | 1806 | Compile exact original authored source batches. |
| coverage-execution | ess-cli | 1806 | Execute an admitted suite selection; committed runs bypass current manifest acquisition. |
| coverage-final-merge | ess-cli | 1806 | Combine generated and authored obligations with exact source identities; input refusal precedes output. |
| coverage-selection | ess-cli | 1806 | Retain original selection lineage; committed suite selection bypasses specification discovery. |
| coverage-suite5 | ess-conformance | 1806 | Create suite/5 with admitted coverage inventory and outcomes. |
| dependency-graph | ess-compiler | 1803 | Build dependencies for each retained model construct. |
| deployment-build | ess-deployment | 1806 | Compile the build projection from admitted model/realization inputs; no live application is implied. |
| deployment-component | ess-deployment | 1806 | Compile the component projection from admitted model/realization inputs; no live application is implied. |
| deployment-deployment | ess-deployment | 1806 | Compile the deployment projection from admitted model/realization inputs; no live application is implied. |
| deployment-runtime | ess-deployment | 1806 | Compile the runtime projection from admitted model/realization inputs; no live application is implied. |
| docs-ir | ess-gen | 1806 | Model-to-document IR content, distinct from Markdown or HTML rendering. |
| generator-asyncapi | ess-gen | 1806 | Published generator asyncapi emits artifacts; semantic behavior needs exact output assertions. |
| generator-docs | ess-gen | 1806 | Published generator docs emits artifacts; semantic behavior needs exact output assertions. |
| generator-openapi | ess-gen | 1799 | Published generator openapi emits artifacts; semantic behavior needs exact output assertions. |
| generator-schema | ess-gen | 1801 | Published generator schema emits artifacts; semantic behavior needs exact output assertions. |
| generator-site | ess-gen | 1806 | Published generator site emits artifacts; semantic behavior needs exact output assertions. |
| graph-rendering | ess-gen | 1806 | Model graph rendering and presentation relationships. |
| html | ess-gen | 1806 | Render document IR/site content to HTML; browser execution is separate. |
| http-embedding | ess-gen | 1804 | Embed selected model contracts into component HTTP surfaces. |
| markdown | ess-gen | 1806 | Render document IR to Markdown; transport/rendering effects remain separately attributed. |
| model-normalization-check | ess-cli | 1806 | Model-backed recipe check uses Sources::plan, ModelTypes::select and Plan::check_with_models before writing the checked recipe. |
| model-normalization-go-emission | ess-cli | 1806 | Model-backed normalization go emission; generate dispatches Plan target emitters; native behavior requires a generated-library case. |
| model-normalization-go-native-execution | ess-cli | 1806 | Model-backed normalization go native-execution; generate dispatches Plan target emitters; native behavior requires a generated-library case. |
| model-normalization-go-refusal | ess-cli | 1806 | Model-backed normalization go refusal; generate dispatches Plan target emitters; native behavior requires a generated-library case. |
| model-normalization-reference | ess-cli | 1806 | Model-backed reference execution uses Sources::plan then Plan::run_json on exact original input text; generated target execution remains separate. |
| model-normalization-rust-emission | ess-cli | 1806 | Model-backed normalization rust emission; generate dispatches Plan target emitters; native behavior requires a generated-library case. |
| model-normalization-rust-native-execution | ess-cli | 1806 | Model-backed normalization rust native-execution; generate dispatches Plan target emitters; native behavior requires a generated-library case. |
| model-normalization-rust-refusal | ess-cli | 1806 | Model-backed normalization rust refusal; generate dispatches Plan target emitters; native behavior requires a generated-library case. |
| model-normalization-typescript-emission | ess-cli | 1806 | Model-backed normalization typescript emission; generate dispatches Plan target emitters; native behavior requires a generated-library case. |
| model-normalization-typescript-native-execution | ess-cli | 1806 | Model-backed normalization typescript native-execution; generate dispatches Plan target emitters; native behavior requires a generated-library case. |
| model-normalization-typescript-refusal | ess-cli | 1806 | Model-backed normalization typescript refusal; generate dispatches Plan target emitters; native behavior requires a generated-library case. |
| model-types-go-emission | ess-cli | 1806 | ModelTypes go emission; emission/refusal executes the Rust host; actual native behavior requires an attributed generated-library case. |
| model-types-go-native-execution | ess-cli | 1806 | ModelTypes go native-execution; emission/refusal executes the Rust host; actual native behavior requires an attributed generated-library case. |
| model-types-go-refusal | ess-cli | 1806 | ModelTypes go refusal; emission/refusal executes the Rust host; actual native behavior requires an attributed generated-library case. |
| model-types-rust-emission | ess-cli | 1806 | ModelTypes rust emission; emission/refusal executes the Rust host; actual native behavior requires an attributed generated-library case. |
| model-types-rust-native-execution | ess-cli | 1806 | ModelTypes rust native-execution; emission/refusal executes the Rust host; actual native behavior requires an attributed generated-library case. |
| model-types-rust-refusal | ess-cli | 1806 | ModelTypes rust refusal; emission/refusal executes the Rust host; actual native behavior requires an attributed generated-library case. |
| model-types-selection | ess-gen | 1806 | Select structural model roots and exact provenance before choosing a target. |
| model-types-typescript-emission | ess-cli | 1806 | ModelTypes typescript emission; emission/refusal executes the Rust host; actual native behavior requires an attributed generated-library case. |
| model-types-typescript-native-execution | ess-cli | 1806 | ModelTypes typescript native-execution; emission/refusal executes the Rust host; actual native behavior requires an attributed generated-library case. |
| model-types-typescript-refusal | ess-cli | 1806 | ModelTypes typescript refusal; emission/refusal executes the Rust host; actual native behavior requires an attributed generated-library case. |
| neutral-synthesis-plan | ess-synth | 1806 | Neutral generated/obligated/refused synthesis decisions. |
| observed-binding-comparison | ess-cli | 1806 | Compare model realization identity and qualified observed bindings; scope uncertainty remains visible. |
| realization-v1 | ess-realization | 1806 | Realization v1 declared ownership and runtime identity; version alternatives have distinct admission contracts. |
| realization-v2 | ess-realization | 1806 | Realization v2 declared ownership and runtime identity; version alternatives have distinct admission contracts. |
| release-report-model-qualification | ess-cli | 1806 | Qualify admitted report/model/selection consistency; does not authenticate producer identity or establish execution. |
| semantic-diff | ess-diff | 1775 | Compare exact semantic structures and retain unclassified residual changes. |
| semantic-impact | ess-diff | 1801 | Account for changed contract slices, conformance selection and generated artifact obligations. |
| semantic-references | ess-compiler | 1806 | Stable named construct references and their exact variants; allocation handles are not external identities. |
| shared-schema-mapping | ess-gen | 1805 | Map semantic type bodies into emitted schema structure and annotations. |
| suite-admission | ess-conformance | 1806 | Admit original suite bytes, counts, input identities and retained lineage. |
| synthesis-clap-emission | ess-synth | 1806 | Clap emission; emitted bytes, named refusal and actual generated runtime behavior are separate claims. |
| synthesis-clap-generated-execution | ess-synth | 1806 | Clap generated-execution; emitted bytes, named refusal and actual generated runtime behavior are separate claims. |
| synthesis-clap-refusal | ess-synth | 1806 | Clap refusal; emitted bytes, named refusal and actual generated runtime behavior are separate claims. |
| synthesis-go-emission | ess-synth | 1806 | Go emission; emitted bytes, named refusal and actual generated runtime behavior are separate claims. |
| synthesis-go-generated-execution | ess-synth | 1806 | Go generated-execution; emitted bytes, named refusal and actual generated runtime behavior are separate claims. |
| synthesis-go-refusal | ess-synth | 1806 | Go refusal; emitted bytes, named refusal and actual generated runtime behavior are separate claims. |
| synthesis-rust-emission | ess-synth | 1806 | Rust emission; emitted bytes, named refusal and actual generated runtime behavior are separate claims. |
| synthesis-rust-generated-execution | ess-synth | 1806 | Rust generated-execution; emitted bytes, named refusal and actual generated runtime behavior are separate claims. |
| synthesis-rust-refusal | ess-synth | 1806 | Rust refusal; emitted bytes, named refusal and actual generated runtime behavior are separate claims. |
| synthesis-web-emission | ess-synth | 1806 | Web emission; emitted bytes, named refusal and actual generated runtime behavior are separate claims. |
| synthesis-web-generated-execution | ess-synth | 1806 | Web generated-execution; emitted bytes, named refusal and actual generated runtime behavior are separate claims. |
| synthesis-web-refusal | ess-synth | 1806 | Web refusal; emitted bytes, named refusal and actual generated runtime behavior are separate claims. |

No decomposition was created: this single finite ownership epic remains draft, so a cross-item critic panel has no proposed child set to compare.
