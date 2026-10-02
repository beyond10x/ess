---
format: aep.planning-md/3
id: story:interpreted-related-values
kind: story
status: active
title: The native target reads related values from the original typed row
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/interpret/execute.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/interpret/execute/related.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/interpret/execute/values.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/interpreted_related_values.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/related_values.rs
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-10-02T23:32:12Z", actor: "human:timo", revision: 3, executor: "agent:codex-ess-backlog", correlation: "consumer-runtime-20261002"}
- {from: "proposed", to: "active", at: "2026-10-02T23:32:12Z", actor: "human:timo", revision: 4, executor: "agent:codex-ess-backlog", correlation: "consumer-runtime-20261002"}
---
## Outcome

The native interpreted target executes every admitted RelatedField value from the exact original related row, preserving typed identity, source-selected outcomes and atomicity.

## Acceptance

Named conformance controls in tests/interpreted_related_values.rs cover Input-via, Subject-via, both admitted creation carriers, typed and nested identities, related identity reads, recursive/Optional leaves, original-reference and original-referent snapshot semantics, missing-row versus absent-field distinction, selected refusal precedence and related error payloads. The existing SHIPPING synthesized scenarios in related_values.rs execute with every assertion retained. Healthy actual target passes; wrong-row, later-reference and later-value alternatives fail the controls. Focused neighbors and strict lint pass, with independent review before integration.

## Fit review

Internal implementation gap under the operator's explicit mandate that all conformance runtimes support all admitted features. Existing source design docs/design/value-expressions.md E8 and current resolved RelatedField already define the semantics; no new authored syntax or entity is introduced. Current native evaluator refuses this value while handwritten targets execute the source fixtures. The bounded implementation reuses typed Store keys, immutable pre-outcome snapshots and existing validation. Two existing adopters are the SHIPPING customer's region copied into a shipment and the related-guard/copied-value job fixture. Source authority, guards and response generation remain unchanged. This extends implemented story:interpreted-command-execution without rewriting that historical delivery claim.

## Decisions

Accept the implementation slice within the full consumer-backlog task. Parent task retains complete cross-runtime scope and subsequent ResponseField work. No standalone PR or full gate; integrate after review into the grouped carrier. Root alone writes AEP, integrates and publishes. All committed executable source is Rust; managed tree/lease and bounded cache rules apply.

# Native RelatedField: bounded next unit

Read-only scope preparation on subject-values commit 1cf63c283ac2e690f5472b2b93161ad7ee09565e, from carrier f26efc2eb. No builds, source edits, AEP writes or publication in this preparation. Root records/authorizes implementation separately.

## Contract and correction to the earlier brief

Binding source is docs/design/value-expressions.md E8 (around lines206–268). RelatedField is a one-hop read of the exact related row before the outcome. Via comes from input, or the existing subject's original field/identity. The entity is already resolved from its identity type or explicit reference/ownership relation; native execution must use the resolved handle rather than rediscover a relation or scan rows by matching values. Related identity fields are valid read targets, and leaves are admitted in nested mappings. A missing related row is not an absent Optional field and this expression does not choose a not-found outcome.

IMPORTANT CORRECTION: the earlier interpreted-values-next-scope.md said creation Subject-via reads are compiler-lowered to Input. Current resolve.rs:2935 preserves ResolvedRelatedVia::Subject. The special creation rule is source-authorized but must be interpreted explicitly. Domain command/related_value.rs:204 subject_field_from_input admits a subject field whose selected branch sets it from unchanged input. It also admits the created identity when that identity carries a one-to-one references relation and the published identity is sourced unchanged from input. Do not read partially built fields from Work.next to implement this exception.

## Small implementation design

1. Extend the private expression read context with an immutable borrow of the command's original Store, alongside the already-reviewed Work.before subject snapshot. Keep this context separate from Work.next and minting. A borrowed original store avoids another full clone.
2. Add one RelatedField evaluator in execute/values.rs. Resolve its address from Invocation for Input, or Work.before for an existing Subject. Validate the address at via.type_ref and the resolved entity identity type. Look up only Store::instance_typed(resolved_entity.name, exact Node). No coercion/stringification, no first/last row scan, no value matching.
3. For a creation Subject-via, resolve only the selected outcome's admitted unchanged InputField carrier. Ordinary field: find that exact target in outcome.sets. Identity carrier: read the selected published identity's InputField source (existence::identity_source already identifies it). Verify the admitted identity reference relation when needed. Keep this as an explicit creation-carrier mapping/context; do not manufacture a preexisting subject and do not permit generated/literal/fallback/arbitrary expression carriers.
4. Read the related identity from the actual lookup key; read ordinary fields from the original related Instance. Reuse presence/type validation from the subject-value helper: missing required fields fail, absent Optional/newtype-Optional remains absent, and a stored Node::Null stays a value where its type permits it. Validate target constraints. Missing row always returns an explicit undetermined/refused execution with no state/event publication, even when the field's type is Optional.
5. Route assignments and recursive event leaves through the ordinary value evaluator. RelatedField is also admitted by the shared error-payload validator/compiler (command.rs:3177 only excludes response sources; resolve.rs:2405 delegates to payload_field), so use the same original-store evaluator for recursive declared-error leaves. This requires passing read context to declared_error/error_value, including early input/related refusals and wrong-state paths; preserve each path's existing selection/subject authority. Do not mint arbitrary error fields.
6. Keep selection untouched: existing existence -> related-missing -> input-refusal -> selected branch/guard authority remains in its current functions. Evaluate only sources of the outcome that actually answers. A missing row needed only by an unselected branch cannot preempt the selected refusal. Do not broaden ResolvedCondition::Related guard syntax or fix the separately tracked ambiguous existence/related combinations in this unit.

Likely owned production files: interpret/execute.rs and execute/values.rs, with at most a small read-only helper extracted in execute/related.rs if it does not mix value lookup with guard decisions. No public Store/API, compiler/source-schema, suite format or response-authority change.

## Red-capable actual sources

- tests/related_values.rs SHIPPING, lines24–109: Register creates customers; Pack reads region via input.customer_id on creation; Dispatch reads the original shipment.customer_id for both stored and emitted region. Existing synthesis arranges the addressed customer between two decoys. Add an actual Interpreted runner of the existing synthesized suite without dropping any assertions. Existing tests use handwritten targets and do not prove native support.
- fixtures/related-guard-copied-value.yaml: missing/archived related-row refusals precede Start, which stores and emits the selected item's note. Execute its real generated suite and direct missing/archived controls to prove value support preserves guard-selected outcomes.
- fixtures/related-copied-view-parameter.yaml: Pack copies both related identity and depot; views later select those exact stored copies. This is an existing identity-field regression source.
- fixtures/subject-guard-copied-field.yaml: CreateRun copies optional policy flags, Report branches from those stored values. This joins related reads to actual later subject-guard behavior.

## Required direct controls

- Two or three different typed identities (Integer adjacent above2^53 and Json structured identity through an unambiguous relation), distinct referent fields; exact addressed row must win over decoys. Include equal ordinary field values with different identities and read the identity itself to detect matching-by-value.
- Input-via and existing Subject-via; nested mapping leaves; related identity-as-field; optional present/absent/null and newtype-Optional. Required Json null must remain distinguishable from an absent member.
- Change the subject's reference field during the outcome while assigning and emitting from the original referenced row: both reads stay on the old referent, while the stored reference changes.
- Mutate the referent itself in the same outcome (the simplest admitted case is an input-via read of the updated row itself) and emit its old field: reading Work.next would fail this control.
- Both admitted creation Subject-via forms: ordinary field sourced directly from input, and created identity carrying a one-to-one reference and sourced directly from input. Assert the resolved IR really remains Subject so the regression covers the correction above.
- Missing related row versus present row with absent Optional field; unknown required source/address values; constraint failure; actual target rows/events unchanged after failure.
- Input-guarded refusal with a missing unrelated source on the unselected accepting branch still answers the selected refusal. Related exists:false and guarded refusal must also retain their source-selected outcomes without reading the accepting branch's value expressions.
- Admitted related error payload, including one nested leaf and wrong-state Subject-via where applicable, reads the exact original referent and leaves rows/events untouched.

Run identical actual red tests first, then focused green tests plus subject-value, typed-identity, related-guard and error-payload neighbors. Strict library/changed-binary Clippy and scoped formatting; no full gate or remote publication. Rust only, own managed tree/lease and servers cache, jobs2/debug0/incremental0/external TMPDIR, 8GiB floor. Freeze for root independent review before bot commit.

ResponseField remains a separate authority unit: generate one actual response and share it with event/result, never independently mint the same-looking value. It is explicitly not authorized by this scope.

## Reviewed source integration

Implementor committed85754ad031178695fb72169a9850dc7bf60fbed8, exact frozen patch1bc0bccde4892209ef65ee8b76581910ababcba9acc187771590159966adae78, with bot author and committer. Root imported the four reviewed files as eaf7fde99a6bf3b05e94c199a6513bf2fd77b636 in batch/consumer-runtime-20261002, also verified bot author and committer. Independent review consumer-related-values records no findings and zero reviewer executions; implementor final53 unique focused tests and strict lint/format checks passed. Root integrated related tests are now running alongside312 migration group3. Owner tree ess-interpreted-related-values-20261003 is tracked clean and its own lease was released, retained pending publication.

This is local source integration only. No push, full gate, closure or release is claimed. Coordinated with the serial UI/server owner before its later282/304 interpreter changes. NextResponseField is being scoped read-only; no response implementation is authorized by this story.
