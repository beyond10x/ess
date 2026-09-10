---
format: aep.planning-md/1
id: story:ess-er-semantic-crosswalk
kind: story
status: active
title: Crosswalk ESS service semantics to the ER kernel before lowering
relations:
- informed_by: initiative:ess-evolution
- informed_by: story:service-contract-extraction
- serves: vision:O2
scope:
- confidence: cited
  path: docs/design/ess-evolution/README.md
- confidence: cited
  path: docs/design/ess-evolution/semantic-crosswalk.md
revision: 7
---
## Outcome
Establish the complete source-pinned mapping required by migration step 3 before implementing ess-entity-runtime. Classify preserved rules, ER kernel gaps, binding obligations and unresolved source semantics without weakening ESS or moving semantic decisions into the SDK.

## Acceptance
Every create/update/transition, predicate/invariant/branch, identity/relation/revision, exact-value, event-cardinality and external-outcome requirement has a cited mapping and an independent counterexample or existing evidence for each claimed mismatch, with the remaining kernel and binding work explicit.

## Authority
This is the operator-authorized interactive ESS evolution implementation. docs/design/ess-evolution/migration.md section 3 and dependency-policy.md own the boundary. Existing ResolvedEntity, ResolvedCommand, ResolvedOutcome and Predicate declarations are the typed source; no new domain entity is introduced.

## Source vector
ESS f300965f409f0370f843b45793541cf66aaee17a; Entity Runtime 074818fd119d0bc74210b3fb27b58088f3d4f80c; Service SDK 03026ec181b6d343672a3863dc1c9874398c6105. Remote main refs inspected separately; these published features are not claimed merged.

## Scope

Cited paths: docs/design/ess-evolution/semantic-crosswalk.md and docs/design/ess-evolution/README.md. This records the semantic comparison; new ER definitions, ess-entity-runtime and SDK execution migration remain implementation work outside this design-only diff.

## Work
Read actual resolved structures and kernel evaluation/replay paths. Record type admission, Unknown truth, selection ambiguity, optional values, exact numbers, creation event limits and observation revisions. Use bounded pure-kernel probes where source inspection needs executable confirmation. Do not run a full gate or change existing runtime artifacts.

## Observed result

The crosswalk is written in docs/design/ess-evolution/semantic-crosswalk.md and linked from the evolution index. It accounts for commands/state/results, predicates and every value family, identity/relations/revisions, queries and external bindings, and opt-in reader/replay migration. Kernel gaps include named outcome selection, multi-event creation, observation semantics, nullable typed values, quantifiers, typed maps/unions and validation that plain JSON/string shapes do not supply. Branch priority and overlapping conditional outcomes are unresolved source semantics; no first-match rule was invented.

Eight pure source-pinned characterization probes passed: creation multi-event rejection; absent named-outcome structure; absent versus null string field; zero-event revision/replay; ordered multi-event operations and literal dollar escaping; explicit signed integer bounds; differing ill-typed primitive ordering; and empty/unobserved quantification. The ordering probe compares primitive evaluators, not admission of an invalid compiled ESS service. The harness pins ESS f300965f and ER 074818fd with serde_json arbitrary_precision required by ER. Manifest, Cargo.lock and Rust source are retained at local-evidence:ess-evolution-20260910/semantic-crosswalk-probe/; final output is semantic-crosswalk-probe-final.log. The earlier offline fetch failure and a probe construction compile error remain separate logs; neither is a passing run. Final cargo run --locked exits zero.

No runtime implementation, persisted format, SDK fixture or dependency pin changed in this design publication. No full gate ran. No decomposition panel was dispatched because this is one crosswalk artifact, not a decomposition into multiple children. Next: specify the opt-in ER definition/decision envelope and semantic dispatch, implement its missing kernel rules, then build the ESS target and delegate SDK decisions/replay. SQL facades, the single AEP migration story and actual application adoption remain unfinished.

A final port audit found that the existing entity-store RecordedObservation requires entity/id and revision at least 1. The design explicitly records that refused creation and subjectless commands need an observation envelope able to represent no existing entity. Existing observation storage is not falsely claimed to cover those cases.
