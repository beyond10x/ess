---
format: aep.planning-md/1
id: review-result:consumer-coverage-binding-pass1
kind: review-result
status: active
title: Consumer coverage candidate binding review
relations:
- reviews: story:review-consumer-coverage
revision: 1
---
unit: consumer-coverage candidate binding, SHA256 9a2a0daa07efa9d065044d8ea8d42df3cd76ee1bdbe0841ce15b5aba1bc1cda6, ESS coordinator source 239996d / opening d2057ff
verdict: NEEDS-CHANGE (candidate design only)
cases: executed 0→0, red 0; no test or suite command was run
origin: introduced 0 / pre-existing 0 / undecided 4
wrote-outside-worktree: none
needs-coordinator: resolve the two accounting/mutation authority gaps and two wording/bootstrap clarifications before accepting or selecting this candidate

## 1. Scope adapter and preservation

The assigned document-only adapter forbids Git, source mutation, tests, builds, gates, integrations and new producer reports. Accordingly, git diff --stat, git log and base-object commands were not executed. This is the explicit assignment exception to the installed adversary charter's implementation-test sequence, not a tests-only source review or either wave11 source attack. The zero case counts above mean no execution, not a passing zero-case suite.

Only this report and its hash manifest are written, beneath target/review-boundaries-11/next-scope/consumer-binding-review-1. The candidate remains unselected and unaccepted. No approval, runtime defect, verifier independence or executed implementation evidence is claimed.

Read completely: the exact installed adversary 0.8.0 charter; candidate draft; complete scoper report; current story:review-consumer-coverage revision6; repository AGENTS. Relevant current declarations and callers were read as cited below. The scoper's 53 inputs were hash-rechecked, together with the draft, scoper report, adversary charter and two additional public-module source files: 58 inputs total. Their complete before/after identities are in input-manifest.json. The original 53 all matched the scoper's recorded hashes. No Git-object or remote claim is made by this review.

The candidate source is the coordinator's published production 239996d846460aee342ce42514378c25b2be5152, with opening d2057ffb944455d0ef3a90ab7c5043ae70027289 supplied by the assignment. The ongoing coverage implementation was not inspected. Origin is undecided for every row because there is no implementation/base execution in this document-only assignment; the rows identify gaps in these exact proposed bytes rather than attributing a production regression.

## 2. Cases and suite execution

No test cases were written. No Rust, Go, browser, TypeScript, compiler, generator, helper, test binary, formatter, Clippy or task gate ran. Source examples below are reproducible review arguments and proposed future checks, not measured executable failures. The previously sealed producer preparation is outside this task and unchanged.

## 3. Findings

“Blocker” applies to accepting/implementing this candidate as a closed binding, not to the active writer or its integration. Warning rows are small corrections and do not establish implementation impossibility.

| ID | Primary location | Severity | Verdict / origin | Finding |
| --- | --- | --- | --- | --- |
| C1 | target/review-boundaries-11/next-scope/consumer-binding-draft.md:35 | blocker | NEEDS-CHANGE / undecided | The separate runtime wire inventory has no bound obligation-key, correspondence or fingerprint rule for schema-only properties and alternatives, so its required participation in exact cell accounting is underspecified. |
| C2 | target/review-boundaries-11/next-scope/consumer-binding-draft.md:168 | blocker | NEEDS-CHANGE / undecided | The source-copy red-to-green requirement does not bind the scanned declarations, compiled runtime schema provider and behavioral case binaries to the same mutated source, permitting a mutation exercise to leave wire or behavior authority unchanged. |
| C3 | target/review-boundaries-11/next-scope/consumer-binding-draft.md:179 | warning | NEEDS-CHANGE / undecided | Preselection baseline freezing lacks a defined extraction-only bootstrap and fingerprint checkpoint even though the extractor is absent and its normalized representation remains an implementation choice. |
| C4 | target/review-boundaries-11/next-scope/consumer-binding-draft.md:30 | warning | CONFIRMED / undecided | EssIrParts is described as a public assembly input although the current type, fields and EssIr::from_parts constructor are crate-private. |

### C1 — Give the wire inventory an exact accounting identity

Document observation: lines29–36 require a separate runtime-derived authored wire inventory and say both inventories must participate. Lines38–43 define IDs only as qualified Rust declaration/member/variant identities and discuss separate wire paths/names/tags. Lines96–118 then key exact eligibility and matrix cells by a discovered model ID/shape and consumer/profile. No rule says whether a schema-only node is its own obligation, changes an owning Rust obligation's fingerprint, or is discharged by an explicit correspondence record.

This is concrete on current source. ess-domain/src/types.rs:675–719 manually implements RawNamedType JsonSchema: schema_name is NamedType, distinct from its Rust name; name/naming are inserted on the object and every oneOf branch at :702–716. RawTypeBody is flattened into RawNamedType at :668. TypeRef's hand-written schema at :281–298 is String while its actual semantic variants remain at :124–137. Ranking at view.rs:225–230 substitutes String for its field/direction structure. A one-to-one mapping from derived wire paths to the Rust field graph therefore does not already exist.

What reaches it: normal RawSpecFile schema generation through xtask main.rs:425 reaches these hand-written schemas. A source edit inside the existing RawNamedType json_schema body can add a wire property, alter required membership or change oneOf shape without adding a Rust member or representation attribute. This review did not perform that edit or run it. The question is which exact required cell/fingerprint must then become unaccounted. “Both participate” does not choose that answer.

Minimum correction choices, within the same four reservations:
1. Define a tagged wire-obligation namespace, root/definition/reference identity, property/alternative/required/constraint granularity, recursive-reference handling and canonical shape profile; account those IDs as well as Rust IDs.
2. Or retain Rust-keyed cells but define an explicit total wire-to-owner correspondence with separately keyed unassociated nodes and exact derived-schema fragments included in the owner's shape. A new unmatched wire property/alternative must fail rather than be folded into an already accepted owner.

Either choice must state treatment of description/examples versus structural constraints, requiredness, flattened duplicate paths, custom schema names and alternatives. Reject unrecognized schema keywords/forms explicitly. Bind it before freezing baseline hashes. A regenerated schema comparing equal to its own current generator is insufficient. This requests a finite accounting convention, not automatic semantic equivalence or a second compiler.

### C2 — Bind mutation authorities to the same copied source

Document observation: lines164–171 require source-copy mutations through the production extraction path and meaningful behavioral red/green; lines34–35 require runtime schemars; lines124–130 execute Cargo-built linked cases and retain run identities. The draft does not state how a source copy changes the compiled schema provider or which source root those linked binaries must have compiled.

Current mechanism makes the distinction observable without executing a probe: xtask main.rs:423 accepts a root path, but :424 uses it only to locate the output/check file; :425 computes schema_for!(ess_domain::spec::RawSpecFile) from the type linked into xtask. Passing a mutated source-copy root to this current pattern would not change that compiled RawSpecFile. Likewise an existing owner test executable remains bound to its compiled source, not to an AST fixture directory.

What reaches it: the mandatory “new optional field absent from inhabited Billing” and schema/custom-representation mutation exercises. A future test can scan the changed declaration copy, make accounting red, then repair cell metadata and execute unchanged owner binaries. It would show extractor/accounting mechanics, while the schema and runtime side still describe the original model. This is a design hole in the required proof recipe, not a claim that an unimplemented gate currently accepts it.

Minimum correction: distinguish a discovery-only mutation lane from a semantic mutation lane. Discovery-only uses the real extractor but claims only missing-ID/shape/accounting failure. For any required schema/behavior red-to-green proof, compile the schema provider and linked owner cases against the same isolated source copy (including fixtures/dependencies), or define an equivalently explicit compiled fixture root with source identity checked across all three observations. Record source-root, extraction-profile, schema-provider build inputs, and case-binary inputs; reject mismatched roots/checkpoints before calling that a semantic proof. A fresh same-source schema helper is one bounded implementation route.

Specify the causal F01 mutation separately: a reviewed concrete behavior change and its meaningful control must drive the attributed owner case, rather than merely swapping case labels or injecting a fake command result. This does not demand a whole-callgraph proof or rejection of every conceivable false attribution. Exact mutation identity and attribution remain reviewed finite claims.

These changes can stay in xtask/its scratch command orchestration and the design. If selected owner cases need new assertions or receipt hooks, reserve those exact test owners before dispatch, as lines7 and143 already require. Do not silently widen the implementation scope.

### C3 — Name the baseline bootstrap checkpoint

Document observation: lines179–182 require an exact eligibility manifest before selection; lines107–109 permit tool-assisted enumeration once; lines183–185 leave private normalized AST representation to implementation and require returning unsupported shapes. The existing xtask Command at main.rs:54–73 has Generate, Schema and Release only; its Cargo.toml has no AST parser. The scoper explicitly reports that no working extractor has enumerated the closure.

This is not a finding that one-time preparation is forbidden or that the policy is intrinsically circular: the explicit enumeration allowance can solve it. The underspecified part is which stage implements/runs the enumerator, fixes the fingerprint profile, and produces the preselection manifest, without treating that work as accepted coverage enforcement or silently making implementation-dependent fingerprints eligible later.

Minimum correction choices:
1. Authorize a distinct extraction-only bootstrap after the writer integrates. Freeze model source, profile and identity/fingerprint rules first; build/review the enumerator; publish an unaccepted finite manifest for root inspection; then freeze exact baseline eligibility and select enforcement work.
2. Or explicitly stage the future selected unit: accepted policy and baseline source first, reviewed extractor and baseline output second, with the gate unable to accept BaselineUnknown or qualify any cells until root freezes that output.

In either route baseline eligibility is never recomputed on ordinary checks, the bootstrap cannot mint Supported/Refused claims, and later representation changes must preserve the bound fingerprint profile or trigger a reviewed migration. This is a workflow clarification, not a demand to prepare a full gate before selecting it.

### C4 — Preserve the current private assembly boundary

Document observation: line30 calls EssIrParts “the public assembly input,” and line73 lists its assembly beside public consumer families. Actual current ir.rs:1291 declares pub(crate) struct EssIrParts, its members at :1292–1307 are pub(crate), and EssIr::from_parts at :1312 is pub(crate). An external xtask/consumer crate cannot call that assembly API.

Minimum correction: replace “public assembly input” with “compiler-private assembly parts.” Keeping it as an explicitly selected AST root is feasible because the draft already includes private members. Classify assembly as compiler-internal behavior exercised through the public compiler entry, or through already scoped compiler-local cases; do not invent an external callable profile or widen visibility to satisfy the inaccurate description. No production fix is requested.

## 4. Boundaries checked without another finding

- The explicit owned-family/public-entry boundary at lines22–24 and64–69 is a defensible finite scope. This review does not demand arbitrary private-reader or whole-Rust callgraph completeness. Exact post-wave profiles/API identities are acknowledged future freeze inputs; their mere absence now is not another finding.
- The runtime generator list at ess-gen/src/lib.rs:52 and real ess-synth::Target enum at src/lib.rs:86 establish concrete discovery anchors. A new target/API must be explicitly classified; package inheritance is already prohibited.
- Closed BaselineUnknown eligibility does not authorize future-field wildcarding. Keeping a known Go panic as broken unknown rather than Refused follows the observed distinction. Explicit no-effect cases need changed/control inputs.
- Go CLI test go_conformance.rs:164 can return at :167 without Go; the draft explicitly refuses to qualify that outer pass. With Go, current source asserts 30 scenario passes at :173–177 but captures nested logs at :139–145 and deletes its directory at :190. A future native profile needs a concrete observer or a scoped hook; the draft's baseline-unknown/owner-test fallback already states this. I do not count an unexecuted preflight or historical report as current runtime evidence.
- Optional normalization Typescript support actually requires its feature, absolute compiler file, TypeScript6.0.3, Node22.23.1 and V8 identity in tests/support/normalization_typescript.rs:10–25. Ordinary workspace results cannot qualify it, as the candidate correctly says.
- Composition's current case at tests/composition.rs:318 compiles an emitted client and executes its generated test binary at :360–362, retaining subprocess output through :366–377. Its boundary remains opaque byte forwarding. Neither that case nor browser pairing proves typed payload validation or fixes F15.
- Exact libtest selection and zero/ignored/failure refusal are feasible in principle. This task did not execute libtest, infer a concrete output grammar from memory, or establish dependency setup/runtime cost. A tested parser and selected profile still need implementation evidence.
- Source/fixture/tool/lock stability and separate Taskfile execution preserve ordinary gates in the proposed design. This review performed no task check or site-build.

## 5. Writes, limitations and final identity

Only two files are written:
- target/review-boundaries-11/next-scope/consumer-binding-review-1/binding-review-report.md
- target/review-boundaries-11/next-scope/consumer-binding-review-1/input-manifest.json

Outside-worktree writes: none. No temporary source copy, test, generated artifact, lockfile, store record, Git state or external integration was written. No cleanup was performed. The report and manifest are immutable on handoff, and all assigned writes are relinquished.

The complete 58-entry input manifest records source identity and stable recheck, not exhaustive inspection of every item in the model closure. The declared bounded review read exact relevant declarations/callers and complete specified documents; it did not extract a whole declaration graph, discover all public entry points, inspect the active writer or claim that the remaining candidate checks are implemented.

Draft SHA256: 9a2a0daa07efa9d065044d8ea8d42df3cd76ee1bdbe0841ce15b5aba1bc1cda6.
Complete scoper-report SHA256: 2194fea88d194d9c54b22e38f5990d6e2b00b64ed633ac039ebf833ca29abd61.
The manifest carries this report's hash; root receives the manifest hash separately.

```findings
- file: "target/review-boundaries-11/next-scope/consumer-binding-draft.md"
  line: 35
  category: "acceptance"
  severity: "blocker"
  verdict: "NEEDS-CHANGE"
  origin: "undecided"
  message: "The separate runtime wire inventory has no bound obligation-key, correspondence or fingerprint rule for schema-only properties and alternatives, so its required participation in exact cell accounting is underspecified."
- file: "target/review-boundaries-11/next-scope/consumer-binding-draft.md"
  line: 168
  category: "acceptance"
  severity: "blocker"
  verdict: "NEEDS-CHANGE"
  origin: "undecided"
  message: "The source-copy red-to-green requirement does not bind the scanned declarations, compiled runtime schema provider and behavioral case binaries to the same mutated source, permitting a mutation exercise to leave wire or behavior authority unchanged."
- file: "target/review-boundaries-11/next-scope/consumer-binding-draft.md"
  line: 179
  category: "judgement"
  severity: "warning"
  verdict: "NEEDS-CHANGE"
  origin: "undecided"
  message: "Preselection baseline freezing lacks a defined extraction-only bootstrap and fingerprint checkpoint even though the extractor is absent and its normalized representation remains an implementation choice."
- file: "target/review-boundaries-11/next-scope/consumer-binding-draft.md"
  line: 30
  category: "contract-drift"
  severity: "warning"
  verdict: "CONFIRMED"
  origin: "undecided"
  message: "EssIrParts is described as a public assembly input although the current type, fields and EssIr::from_parts constructor are crate-private."
```

