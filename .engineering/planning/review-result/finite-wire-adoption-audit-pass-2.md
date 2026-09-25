---
format: aep.planning-md/2
id: review-result:finite-wire-adoption-audit-pass-2
kind: review-result
status: active
title: Final finite wire DATA adoption audit
relations:
- reviews: task:consumer-accounting-wire-behavior-period-parity
revision: 1
---
approve

# Final finite wire DATA audit — pass 2

This is the second and final pass of the same read-only finite DATA-adoption audit. Approval is
limited to the frozen v4 ledger/index as an accurate mapping of the accepted 72-identity finite
scope to the assigned direct-file evidence. It is not code-integration approval, full
qualification, claim adoption, source freeze, or a new technical-design verdict.

## Frozen inputs and source identity

- `finite-member-reference-control-ledger-v4.candidate.json`:
  `85b01125a07614888737069166be4f4e09f1265eff400705fc13a3d0d578a650`
- `finite-control-index-v4.candidate.json`:
  `e209d673d40737cbf44b9ab214377596c1e213e52afe77a40fd3399cf9552feb`
- `active18-assigned-source-manifest.candidate.sha256`:
  `6db1406083ef46d63897228f4cece31734ce1dc63fd1a50740649b09c19a908b`
- `active18-audit-correction-handoff.md`:
  `8275421c0bb3bab7d3e8cce8f572f74754bb706581b03ec67443ff84b10e95e4`
- source worktree branch and HEAD: `plan/ess-evolution-scope-20260915` at
  `f1af8280338b97d862a6c474ec50f78d5157d71c`
- `crates/edge/ess-cli/src/load_accounting_tests.rs`:
  `7b1544ccfc78263dad29f63915d728f098bf12ffa5c1aeb377121356044524cf`

All seven paths in the manifest match the assigned tree: the loader above; `periodic.rs`
`2451848d...`; `refs.rs` `527dd20e...`; the CLI and domain manifests `6228cee4...` and
`1d733a28...`; `Cargo.lock` `92b31019...`; and generated schema `22415fc9...`. The ledger records
the independently generated provider schema as `c6f550c5...` and the corrected focused receipt at
lines 31–43.

The original pass-1 report remains byte-identical at
`a72ba65184ddd662c6dadb17c38dee4739630ff61b478ffb281bd1850032d269`; its formatting-only AEP
companion remains byte-identical at
`366c8a8ac14f13f30a809ba6586fcdd6a6722d4eee3cc981ba61c5ac4ea0d286`.

## Inspected finite coverage

I rechecked all 72 identity rows: 66 direct identities and the six stable targets `Field`,
`Naming`, `Predicate`, `QualifiedName`, `StateName`, and `TypeRef`
(`finite-member-reference-control-ledger-v4.candidate.json:4-29`). Every retained literal
`schema_node` equals the node at its decoded RFC 6901 pointer in the assigned generated schema.
For all 66 direct rows, the recorded member set, required set, branch count, and every local
reference source/target also derive exactly from that literal node. No identity or structural
payload changed from the independently checked v3 set.

I rechecked all seven case tables and all 98 member rows against the accepted completeness
contract and current 7,369-line loader source, including the shared schema, reference, naming,
field, lexical, P/A/K, and single-edit mutation helpers. The ledger and index contain the same
1,144 case/member/stage/evidence tuples in both directions: 409 C, 556 P, 175 A, and four K uses
(`finite-control-index-v4.candidate.json:13-30`). The 1,000 distinct evidence labels classify as
583 source literals, 316 finite helper-generated controls, and 101 descriptive observations;
these are mappings and observations, not 1,000 libtest cases. All 583 source-literal labels occur
as exact quoted labels in the assigned Rust source.

I inspected the initial and corrected raw receipts without running a build or test. The initial
receipt is exit 101 with six passed and one failed because `selection_inputs: null` compiled before
the case reached the second null mutation (log SHA `492c7f88...`). The corrected receipt is exit 0
with seven passed, zero failed/ignored, and 25 filtered (log SHA `5d063cf0...`). Its two null-array
controls compare the complete returned binding with the explicit-empty baseline and assert no
serialized selection plan (`load_accounting_tests.rs:5830-5854`). The external jq validation was
used only as an inventory-consistency cross-check, not as behavioral proof.

As a supplementary non-authority check, I read only the 72 wire claims in
`reviewed-model-behavior-active20.candidate.json`
(`ed0c3c9d8157743b6be7b9993888987071c115caa419aea1c63b112dba387c35`). All 66 direct claim case
sets equal the ledger's case sets, and all six stable-target case sets equal those derived from
`identity_coverage`. I did not review its unrelated 27 Rust claims, and this check adopts nothing.

## Pass-1 comparison

- **Resolved — four missing K index uses.** Both setting controls now appear at K for
  `G.RawComponentSpec.settings` and `G.RawComponentSetting.type`, and the source asserts exact
  compiler refusal shapes, codes, and locations (`finite-member-reference-control-ledger-v4.candidate.json:549-615`;
  `load_accounting_tests.rs:3162-3222`).
- **Resolved — missing executable member behavior.** The nine added controls cover binding name
  null/non-string, both selection arrays at their source-observed C behavior, host owner/authority
  wrong kinds, binding refs wrong-list-kind, command outcomes wrong-item-kind, and outcome refs
  null-item. Each is an isolated exact-once mutation at the required site; the two C controls assert
  the whole returned binding, while the other seven use the typed P oracle
  (`load_accounting_tests.rs:4780-4788,5020-5029,5830-5872,5955-5963,6133-6151`).
- **Resolved — 26 absent mappings.** All 26 previously executed controls now occur at their exact
  member and stage, alongside the four K tuples and nine new controls. Every tuple is present in
  both ledger and index (`active18-required-member-mappings.candidate.json:4-210`). Direct source
  inspection confirms the restored P/C behavior rather than relying on label presence alone.
- **Resolved — incomplete SS-null observation.** The explicit-null outcome now asserts exact
  condition `{"kind":"otherwise"}`, exact test strategy `default_branch`, and absence of the
  authored key on the mutated IR (`load_accounting_tests.rs:3876-3898`;
  `finite-member-reference-control-ledger-v4.candidate.json:703-717`).
- **Carried findings:** none.
- **New findings:** none.

Nothing in this audit establishes the pending full gates, fresh extraction qualification, claim
adoption, source freeze, aggregate closure, or submitted-code examination. Those boundaries do not
change the finite v4 DATA verdict.

```findings
[]
```
