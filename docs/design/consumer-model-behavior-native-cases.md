# Native binary-unit model behavior cases

Status: accepted implementation contract under consumer-accounting-baseline-never-extended;
review-result:native-model-behavior-design-pass-2 approved with zero findings.
This extends the next behavior slice, not the completed two examinations of accounting mechanics.
It qualifies no cell and adopts no reviewed authority. Implement against this reviewed contract.

## Separate authority and closed readers

Keep the unversioned legacy reviewed-candidates.json bytes and its integration-only CaseIdentity
reader unchanged. Do not add `bin` to its TestTarget enum. Introduce a separate root-reviewed
`reviewed-model-behavior.json` with closed `ess-consumer-model-behavior/1` authority. Its Rust type
has exactly `format`, `cases` and `claims`; reject unknown fields, versions, duplicate map keys and
duplicate claim identities. This first version admits binary-unit targets only. It is not an
acquisition authority and cannot satisfy any of acquisition/2's eight obligations.

`cases` is an ordered map keyed by the current extractor's exact case ID. Each value contains:

- `identity`: package, target_kind (the sole literal `bin`), target_name, full_name, features,
  target_profile, tool_requirements and nested_runtime, with the existing execution-policy values.
- `target_source` and `target_source_file_sha256`: Cargo's actual binary target root and raw hash.
- `source`, `source_file_sha256` and `case_ast_sha256`: the actual defining test module and case AST.
- `reviewed_assertion`, `attribution_limit`, `source_evidence` and `review_reference`: nonempty
  exact root-reviewed evidence text/references, never executable input.

`claims` is an ordered list with the following exact fields per claim:

```text
identity: { model, shape, consumer, profile }
cases: [case ID, ...]
entrypoint: production entrypoint ID
entrypoint_sha256: current declaration hash
entrypoint_source: production module path
entrypoint_ast_sha256: full production function AST hash
entrypoint_source_file_sha256: raw production module hash
behavior: { kind: Supported, reason } | { kind: Refused, reason, refusal }
source_evidence: nonempty reference
review_reference: nonempty reference
```

Identity reuses the existing reconciliation::Identity wire fields. Every claim identity is unique;
case lists are nonempty and distinct, and their union equals the nonempty case-map keys. Reject
unreferenced cases and unresolved references. Case keys equal
`{package}:bin:{target_name}:{full_name}`. Digests have exactly64lowercase hexadecimal characters.
All structures and behavior variants reject unknown fields; duplicate keys anywhere in raw JSON
refuse before constructing a Value or typed map. Preserve case/claim array order in authority bytes.
The initial version requires a profile with exactly the named production entrypoint; multi-entrypoint
profiles need a later explicitly designed extension. Bind current profile/declaration, full AST and
raw module bytes. A declaration hash alone cannot bind the function body. No unknown, retirement,
metadata or aggregate disposition is admitted by this authority. Execution policy is exactly:
features `[]`; target_profile `x86_64-unknown-linux-gnu/default`; tool_requirements
`["frozen Rust 1.98.1; locked offline owner-target build"]`; nested_runtime
`None claimed; direct Rust assertions only`. Other policies refuse in this first version.

Root adopts only claims backed by causal observations through the actual consumer. The initial
direct-file preparation proposes seven cases and90current cells, but that is analysis, not an
adopted row set. `load::specification` returns IR or diagnostics; independently reparsing a fixture
does not prove the loader handled a raw/assembled field. Rows without a causal observation remain
unclaimed. Aggregate parents remain the separate five-identity closure mechanism.

## Source and native target binding

Compare package, target kind/name/root with fresh Cargo metadata and the compiled source inventory.
Compare the exact case ID/full name, defining module, AST/file bytes and ignored status separately.
Never compare a submodule's path to Cargo's binary root as if they were the same source coordinate.
Reject ignored, missing, renamed, wrong-owner, wrong-kind and ambiguous cases before execution.

Normalize legacy integration cases and validated new binary cases into a private execution enum
only after their own closed readers and source guards succeed. No deserializable private proof,
new lifecycle engine, test-copy loader or acquisition-classification shortcut is introduced.
The native cache identity includes package, target kind and target name plus the admitted build
profile; identically named integration and binary targets cannot share an executable accidentally.

For a binary case build the exact Cargo target with `test --bin <name> --no-run` through the normal
observed command path. Validate Cargo artifact package/manifest, test profile, kind/name/root and
unique executable; retain the actual native image. Use the existing exact libtest listing/execution
runner, source/tool/native freshness checks and one-executed/one-passed/zero-ignored assertions.
Never substitute an unobserved nested CLI process, or count a plain cargo test invocation as the
consumer-coverage executor's receipt. Legacy integration build and receipt bytes stay unchanged.

## Digest propagation and diagnostic execution

The authority format is `ess-consumer-model-behavior/1`. Execution uses a separate closed format
`ess-consumer-model-behavior-execution/1` and stage `candidate`, `execution-plan` or `executed`.
Every stage contains exactly format, stage, authority_sha256, source_sha256,
provider_profile_sha256, selected_cases, cases, claims and qualified_cells. The last field is
always zero. `cases` and `claims` retain the typed authority maps/list above; selected_cases is the
sorted distinct exact case-map key set. A candidate is source-validated, never executed evidence.

Authority digest is existing hash_json over the successfully duplicate-checked typed authority's
canonical JSON value, including review text. Source digest is hash_json(source_profile["source"]);
provider profile digest is hash_json(source_profile). Plan additionally contains candidate_sha256,
the canonical full candidate hash. Re-read actual authority/source and require exact digest,
case/claim/selected-set and current provider equality before building a plan.

Executed stage additionally contains candidate_sha256, plan_sha256 and receipts. Plan digest is
the canonical full execution-plan hash. Receipts is an ordered map with keys exactly selected_cases.
Each closed receipt uses `ess-consumer-model-behavior-case/1` and exactly these other fields:
authority_sha256, plan_sha256, identity, target_source, target_source_file_sha256, source,
source_file_sha256, case_ast_sha256, native, source_sha256, provider_profile_sha256, executed,
passed, failed, ignored and nested_runtime. Counts are1/1/0/0; outer nested_runtime is the exact
string `none claimed`, while identity retains the reviewed policy string above. `native` reuses
the existing retained native-image receipt fields and semantics.
Persisted stage readers can diagnose data; deserialization never constructs private run evidence.
Fresh executed proof is created only by normal observed native execution and revalidated source.

Before accepting executed evidence, compare its selected_cases, cases and claims to the plan by
exact typed equality, preserving array order; compare the plan's corresponding values to the
reloaded authority and source-validated candidate. Repeated authority/source/provider/candidate/plan
digests must equal their independently computed bindings, not merely one another. For each receipt
key, require identity, target_source, target_source_file_sha256, source, source_file_sha256 and
case_ast_sha256 to equal that exact keyed case in the plan and reloaded authority. Require the
receipt's authority_sha256, plan_sha256, source_sha256 and provider_profile_sha256 to equal the
enclosing verified run's independently computed values. The retained native image must be the one
actually listed/executed for that case under the exact admitted Cargo target and current policy.
Enforce format, counts and both nested_runtime policy strings as well. No unchanged plan digest
can authorize substituted repeated contents or a receipt moved under another case's key.

Binary receipts change the admitted value/reference meaning of accounting evidence. Introduce
`ess-consumer-accounting/3`; preserve closed accounting/1 and /2 readers and their receipt bytes,
plus acquisition/2. Root selects this explicit format change under the repository versioning rule.
The following are concrete typed /3 members, not a heterogeneous evidence registry:

- Candidate retains /2's non-format fields and adds model_behavior_candidate.
- Execution plan retains /2's non-format fields and adds model_behavior_plan and
  required_legacy_cases. required_cases is the exact disjoint union of required_legacy_cases and
  model_behavior_plan.selected_cases. Every new authority claim must be consumed exactly once by
  the same current cell and Supported/named Refused disposition; overlap with a legacy claim refuses.
- Qualified retains /2's non-format fields, including legacy-only executed_cases with unchanged
  entry bytes, and adds model_behavior_execution. executed_case_count counts the disjoint combined
  executed set; the model-behavior member remains zero qualified cells because only outer accounting
  qualifies cells. Existing per-disposition accounting totals still conserve the full matrix.

The /3 native path holds separate legacy receipts and typed model-behavior evidence in private
VerifiedCases. Qualification requires legacy IDs equal required_legacy_cases, binary IDs equal
the model-behavior plan and their disjoint union equal required_cases. Reject missing/extra IDs,
wrong authority/candidate/plan/source/provider bindings, and any claim not in the exact cell plan.
All existing reconciliation, acquisition, aggregate and metadata proofs remain mandatory. Old
versions retain their original execution/qualification entry points; no fallback treats /3 as /2.

Add `cargo xtask consumer-behavior --output <fresh-directory>` as diagnostic execution of exactly
the root-reviewed new authority's cases through that same production native executor. It emits
versioned planned/executed evidence with zero qualified cells and exits1 on missing/stale authority
or any unexecuted case. It cannot produce ACCOUNTED status, a reconciliation manifest or a passing
consumer-check result. This enables causal execution before the complete matrix is available.
The full consumer-check uses accounting/3 and the same readers/native branch once complete planning succeeds;
there is no separate permissive runner. Output/source ancestry guards remain mandatory.

## Admission and decisive checks

Before adding readers, add legacy rejection fixtures for the new version and bin identity, plus
new-reader rejection of legacy/unversioned data and unknown fields. Preserve old bytes explicitly.
Accounting/1 and /2 must reject /3; /3 refuses old envelopes and mixed-family duplicate identities.
Exercise same-name test/bin cache collisions, distinct root/submodule binding, ignored/zero-selected
cases, changed AST/body/module, wrong native artifacts, wrong authority digest and receipt family.
Mutate executed selected_cases, cases and claims independently while retaining original digests;
swap two otherwise valid receipts under each other's keys; substitute receipt identity, target-root
or case-module hashes, and each per-receipt binding digest. Every mutation must refuse through the
same guard used before accounting/3 qualification, including same-size and same-count substitutions.
Run each actual loader case through the common native path and retain named causal mutations.
Refuse unsupported rows instead of padding the90-cell proposal. Root alone adopts source/profile
classification changes and authority JSON after review. Complete finite reconciliation, all8380
replacement behaviors, aggregate children, default consumer-check/task check and site-build remain
the parent acceptance boundary. This accepted design does not close or weaken any of them.
