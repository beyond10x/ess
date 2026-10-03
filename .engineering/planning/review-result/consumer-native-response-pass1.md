---
format: aep.planning-md/3
id: review-result:consumer-native-response-pass1
kind: review-result
status: active
title: 'Native response authority review: creation identity remains unwitnessed'
relations:
- reviews: story:interpreted-response-values
revision: 1
---
needs-revision

Read-only source review of candidate patch `831fc5b434664ccfe839d3822bf8c885335448a4ef7f419c1c582f2f83c5ecb9` over base `617cfa2248bda9214c7fe58ac1de62519d640dbc`. Nine candidate files, no reviewer source edits. Reviewer test executions:0; no builds. This is an agent source-review result, not signed verifier evidence or a release approval.

One acceptance blocker: a creation whose emitted identity reads a response field still has no response authority when that field is evaluated.

At `crates/verify/ess-conformance/src/interpret/execute.rs:1098`, `take` calls `create` while `Work.response` remains None from line1089. `create` obtains the declared identity source and calls `existence::identity`; that function calls `value` at `interpret/execute/existence.rs:43`. The new ResponseField arm calls `values::response`, which rejects None at `interpret/execute/values.rs:182`. Response preparation occurs only later at `interpret/execute.rs:1162`, after creation and set effects. Thus this source-selected creation cannot reach preparation, even though the new native authority is responsible for its response-derived emitted field.

Source admission is supported by the actual validators: `crates/specify/ess-domain/src/entity.rs:1274` validates a created identity by finding its name/type in an emitted event and returns success when that type matches the entity identity; it does not prohibit response ownership. `crates/specify/ess-domain/src/command.rs:3084` resolves a ResponseField from the declared response and permits the same source/target type. Source4 admits the response vocabulary at line3225. `crates/specify/ess-compiler/src/resolve.rs:2585` lowers that response source as such. No unknown-instance, existing-instance or guarded-source combination is needed.

Minimal source for the coordinator's actual compiled regression:

```yaml
format: ess/4
system: demo
version: v1
domain: demo.response
entities:
  - name: demo.response.Row
    identity: {name: id, type: Uuid}
    lifecycle: {initial: Held, states: [Held], terminal: [Held]}
events:
  - name: demo.response.Created
    fields: [{name: id, type: Uuid}]
commands:
  - name: demo.response.Create
    response: [{name: id, type: Uuid}]
    outcomes:
      - name: created
        creates: demo.response.Row
        instance: id
        emits: [demo.response.Created]
        payload: {demo.response.Created: {id: {response: id}}}
```

Expected regression: compile this source, invoke the actual Interpreted target, require success and equality of returned response.id, emitted Created.id and the stored row identity; repeat to require independent identities. Current source trace reaches the missing-response capability refusal above. The reviewer has not executed this fixture; the coordinator is running the actual red under its own resource allowance. The unsupported behavior also existed in the base; the blocker is an unclosed case within the accepted response-authority scope, rather than a newly introduced behavior regression.

Smallest correction: prepare one staged response before evaluating a response-owned creation identity, retain that prepared object for all event copies and final completion, and preserve current missing/wrong-state selection before preparing unrelated accepting update outcomes. Keep the response outside Store/Step/Debug and commit its issuance only after full candidate success and uniqueness. Add a failed response-owned creation/invariant control to prove issuance, row and event rollback against a fresh target.

Cleared by source inspection: private Prepared/Authority have no Debug or serialization; Issued remains redacted; each candidate clones original issuance rather than previous candidate issuance; completed candidates remain local until unique selection; ordinary response generation reads declarations without suite expectations/input examples; public execute wrappers retain no response authority; source4 and nested emitted mappings use the same prepared value; explicit conversions fail closed; missing-input outcomes are statically errors and do not acquire a response. Post-commit downstream dispatch failures remain outside this unit's rollback boundary, as the accepted scope states.

Implementor evidence inspected, not rerun: `response-values-recovered-tests.log` reports131passed/0failed (SHA256 `3d4bfbfcb7da5bda31474ca512cd510abc48b5406d8ab1afcb2ffab8271a7bb7`); scoped strict Clippy exits0 (SHA256 `37435befcd17e275d675802f2ddb19b83093ce8dba93c936bbfbe08d1871daa7`). Integrated one-time tests and updated suite metadata remain coordinator validation. Those successful tests omit the creation-identity source case above.

```findings
[
  {
    "file": "crates/verify/ess-conformance/src/interpret/execute.rs",
    "line": 1098,
    "category": "acceptance",
    "severity": "blocker",
    "verdict": "NEEDS-CHANGE",
    "origin": "pre-existing",
    "message": "A source-admitted creation whose emitted identity reads a response field evaluates that identity before the invocation response is prepared, so the accepted native response authority remains unavailable on this path."
  }
]
```
