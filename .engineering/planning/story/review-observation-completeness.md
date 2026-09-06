---
format: aep.planning-md/1
id: story:review-observation-completeness
kind: story
status: draft
title: Preserve observation scope and selector uncertainty
tags:
- P1
- review-2026-09-05
relations:
- decomposes: epic:review-boundary-remediation
- serves: vision:O2
- depends_on: story:review-secret-sanitization
- depends_on: story:review-infra-ir-invariants
scope:
- confidence: cited
  path: crates/edge/ess-cli
- confidence: cited
  path: crates/infra/ess-kubernetes
- confidence: cited
  path: crates/infra/infra-analyze
- confidence: cited
  path: crates/infra/infra-compiler
- confidence: cited
  path: crates/infra/infra-domain
- confidence: cited
  path: crates/infra/infra-project
- confidence: cited
  path: crates/infra/infra-spec
- confidence: cited
  path: docs/design/review-format-catalog.md
- confidence: inferred
  path: docs/design/review-observation-completeness.md
- confidence: inferred
  path: examples/k3d-dev-cluster
- confidence: cited
  path: website/docs/guides/check-infrastructure.md
- confidence: cited
  path: website/docs/reference/formats.md
revision: 18
---
## Finding and source

F06 (P1) from `docs/reviews/2026-09-05-architecture-review.md:286`, reviewed at `fd06a4d61bfb7b4990617810655dc181d6a3ab00`. Evidence locations: `crates/infra/infra-domain/src/raw.rs:159`, `crates/infra/ess-kubernetes/src/lib.rs:70`. Findings are attributed to the review; proposed implementation choices below are coordinator inferences.

## Acceptance

Unsupported selectors or a reduced collection scope produce qualified unknown/refused infrastructure conclusions instead of complete match-all or absence claims.

## Implementation boundary

First design typed scope/per-kind completeness and unsupported-term accounting against the existing observation/IR formats, with strict old-reader fixtures. Preserve selector semantics or refuse their use; never reduce an unrecognized expression-only selector to a broad empty conjunction. Retry only when scope is preserved or visibly acknowledged; ensure scope qualifications reach analysis, drift and projection.

## Validation

Fake kubectl fixtures distinguish permission failure, namespace fallback and genuinely scope-preserving retries. Selector fixtures cover matchLabels, matchExpressions, mixed and unknown terms. Round-trip incomplete observations and assert consumers retain uncertainty.

Run package-scoped checks while implementing; the integration coordinator runs every step of `task check` and `task site-build` when required by `AGENTS.md`, retaining individual exit statuses and executed-case counts.

## Compatibility and exclusions

No live cluster needed; keep the separate infra bounded context and obtain an explicit format decision before changing persisted fields.

## Scope

- **Derivation:** cited — refreshed 2026-09-06 under the installed aep-drive 0.8.0 story-scoper charter from draft revision 12 and F06, against coordinator identity 3811e902cd42bffbafbf4fa345613e6905b6a7a4 supplied by root. The 68 infrastructure files and CLI main/load bytes match the retained published-source baseline 239996d846460aee342ce42514378c25b2be5152. The earlier wave-8 scope is provenance.
- **Acquisition reservation:** cited — `crates/infra/ess-kubernetes`; the ordered 17-kind collector, unconditional retry that removes `-A`, observation writer, credential-boundary contract and actual fake-command/redaction tests belong here. Preserve value-free diagnostics, redaction-before-write and refusal destination guards while replacing only scope-invalid retry assertions.
- **Observation reservation:** cited — `crates/infra/infra-domain`; raw bundle/List extraction, RawSelector, workload/PDB validation and Observation own information currently discarded before typed reasoning. Required absent kinds already refuse; optional absent and present-empty kinds already differ.
- **IR reservation:** cited — `crates/infra/infra-compiler`; compile, InfraModel, provenance, persisted document, canonical model digest and closed read_document admission must carry the chosen qualification meaning. Existing model()/try_transform ownership and source-on-failure guarantees remain requirements; checked handles do not establish collection completeness.
- **Analysis reservation:** cited — `crates/infra/infra-analyze`; pdb_covers, diagnosis, observed-property counts, invariant candidates and graph/ownership derivation are actual consumers of selectors and collection membership.
- **Intent-consumer reservation:** cited — `crates/infra/infra-spec`; typed unknown reasons, fact withholding, WorkloadAbsent, PDB/config-reference conclusions and context-only drift membership comparison need the chosen qualification policy.
- **Projection reservation:** cited — `crates/infra/infra-project`; fixed-point simulation, admitted detached changes, PDB construction, final gap dispositions and report/artifact rendering are actual consumers. The current summaries retain undecidable counts, while entries represent gaps; reason retention therefore needs an explicit design, not merely an additional simulation reason.
- **CLI reservation:** cited — `crates/edge/ess-cli`; current loader, Kubernetes import help/scan/reporting, diagnose/graph/diff and projection commands expose the old formats and conclusions. Keep projection refusal before artifact writes and cover both area and flat spellings with package-owned tests.
- **Internal binding reservation:** inferred — `docs/design/review-observation-completeness.md` is absent. It must bind selectors, requested/effective scope and per-kind completeness, trusted versus declared claims, legacy admission, transformation rules, consumer refusal/unknown behavior, format/digest decisions and relying-party order before implementation.
- **Example reservation:** inferred — `examples/k3d-dev-cluster`; existing public fixtures and generated outputs are test inputs, and the README explicitly describes a trimmed/synthesized scan. Changes depend on the chosen migration. Preserve independently frozen old-writer bytes before any regeneration; do not relabel the trimmed historical example as a complete real scan.
- **Public guide reservation:** cited — `website/docs/guides/check-infrastructure.md`; lines 8–21 and 37–71 teach observation/1 to IR/1 import, comparison, projection and live scanning. Update the actual qualification, refusal and migration behavior here.
- **Public format reservation:** cited — `website/docs/reference/formats.md`; infrastructure rows at lines 230–237 document observation, IR, drift, simulation, graph and the distinct JSON/YAML projection surfaces. Change only facts justified by the accepted migration; preserve unrelated coverage/Binary64/normalization rows.
- **Internal catalog reservation:** cited — `docs/design/review-format-catalog.md`; lines 214–221 and 237 own the corresponding format/digest/admission and AdapterReport inventory. Record actual reader/writer consequences, without assuming every downstream format needs a major bump.
- **Verification boundary:** inferred — package-owned scanner, admission, compile/read/clone/transform, consumer and CLI regressions plus independently frozen old-reader/old-writer fixtures are required after binding. Use offline fake subprocesses and synthetic populations; existing tests do not prove F06. Future integration includes declared package/root checks and site delivery, with actual results retained separately.
- **Exclusions:** inferred — no EssIr/InfraIr merger, semantic-impact migration, generated ESS schema/xtask/root-manifest rewrite, Kubernetes client-library replacement, deployment recovery implementation, generic execution-evidence schema or live cluster. Planning and publication records remain root-owned.
- **Confidence:** inferred — medium: owner packages and current loss sites are established and unchanged, but selector support, completeness authority, consumer qualification and format/rollout choices still determine the implementation.
- **Would collide with:** inferred — writers of any of the seven reserved packages, the example tree or the four exact document files above. No broad Website reservation follows from read-only citations. If store collision reporting uses literal tokens, root must separately check package/file containment; a distinct token does not make overlapping files safe.
- **Final refresh prerequisite:** inferred — recheck integrated CLI help/dispatch/report bytes, public formats, internal catalog and declared gates after the active coverage writer closes. This scope does not inspect that unit or authorize concurrent edits, a new binding, a format default or a next-wave selection.

## Candidate preparation

The 2026-09-06 scope refresh is recorded from the complete read-only report. Root verified all102 input hashes and the unchanged68 infrastructure files plus CLI main/load against retained239996d. The seven package reservations remain, with binding/example and three exact documentation reservations. Binding is still absent; selector support, collection/completeness authority, semantic identity, consumer reason retention, per-format compatibility and relying-party/default order remain decisions before implementation. The trimmed historical example cannot become proof of a complete real scan. Execution-recovery observation/freshness remains UNMAPPED and outside this scope. This is candidate preparation only; no wave, new format/default, runtime verification or live collection is selected. Refresh integrated CLI and format documentation after the active coverage writer closes.

The later local relying-party inventory searched 37 checkouts and found no established current
foreign infrastructure-format reader. Root verified all 132 pinned inputs, including the same
70 infrastructure/CLI files. Atlas ROADMAP:49 and org-brain's design assign Kubernetes observation
consumption to intended work; archived infra-scout is an old observation/1 producer. AEP's checked
conformance adapters and Website's documentation caches are distinct contracts. These local
observations add no foreign reservation or AEP prerequisite. The complete report is retained at
wave11 next-scope/observation-relying-parties/report.md, SHA256
ec1a268036bb7dcdc634632a08b1014b71ee0cd58629f24903195c54e36eebef, with manifest
81e2e6e05d3d95294a29203310ca9201c600420d0664d0dfc72d2135780a9354. The twelve scope
entries remain unchanged. The inventory does not establish deployed-adopter absence, accept an
ADR or resolve format/default decisions; post-writer CLI/docs refresh remains required. A separate
retained clarification resolves root's display-escaping misread of a correct query receipt; it
changes no inventory finding, and its already completed 37-query replay found identical matches.
