---
format: aep.planning-md/3
id: review-result:consumer-312-typescript-migration-publication
kind: review-result
status: active
title: TypeScript suite migration review publication copy
relations:
- reviews: story:feature-request-312
revision: 1
---
approve

```findings
[]
```

Frozen patch: managed tree ess-backlog-next-20261002, repository-relative target/backlog-input/312-ts-migration-review.patch
SHA-256: 80341f5e5269c087cd8c4dd817ce2b16f3c9a409a14f5c5a77030a380195ca8e
Base identified by implementor: 617cfa2248. Scope: runtime_parity_typescript_28_35.rs only.

Reviewer test/build executions: 0. Read-only inspection of the frozen patch, surrounding source and retained implementor logs. No source edits or builds.

The /34 and /35 expectations reflect fresh initial-state authority. Both historical /26 documents remove only that new metadata. The ordinary status fixture still explicitly removes direct-response steps and must admit; the malformed-direct fixture retains those steps and must refuse. No unsupported construct is silently removed from a negative fixture.

The depth helper now constructs the document before serializing it, allowing the negative historical fixture to change actual provenance instead of replacing a /28 string absent from the fresh /34 suite. This also avoids reparsing the deliberately deep value through serde_json's unrelated default recursion limit. Serialization remains the same compact Value::to_string path, with the response before the step tag. The depth-128 positive boundary, depth-129 refusal, unrelated and forged nested-path refusals, exact parent bytes, no callback and no report assertions remain unchanged.

Retained evidence: first correction 27 passed / 1 failed, terminal exit 101, specifically "native admitted old-deep-direct". Final correction 28 passed / 0 failed / 0 ignored, terminal exit 0. The earlier missing Node types was an environment prerequisite; this patch does not bypass or skip that compilation. These are implementor executions; this approval is source/evidence review only, not a release or full-gate result.

Publication copy omits the local absolute path; scope, verdict and evidence are unchanged. The original report remains preserved privately.
