---
format: aep.planning-md/1
id: review-result:finite-wire-adoption-audit-pass-1
kind: review-result
status: active
title: Finite wire behavior DATA adoption audit, pass 1
relations:
- reviews: task:consumer-accounting-wire-behavior-period-parity
revision: 1
---
needs-revision

Owner: `task:consumer-accounting-wire-behavior-period-parity`.

Formatting adaptation: original `high` maps to AEP `blocker`; original `medium` maps to AEP `warning`. Finding reasons and verdict are unchanged.

Source audit: `finite-wire-adoption-audit-pass-1.md`, SHA256 `a72ba65184ddd662c6dadb17c38dee4739630ff61b478ffb281bd1850032d269`.

task:consumer-accounting-wire-behavior-period-parity — The index is not a complete index of the ledger. The ledger assigns G-entity-setting-type and G-undeclared-setting-type at K to both G.RawComponentSpec.settings and G.RawComponentSetting.type (four uses), but the index has no entries for either label and declares only 1,102 uses. Counting C/P/A/K in the ledger gives 1,106. The frozen source really executes both compile refusals with exact diagnostic codes and locations, so these are lost adoption records rather than speculative controls. The recorded consistency method checked only c/p/a and therefore could not detect the omission. — finite-control-index-v3.candidate.json:13

task:consumer-accounting-wire-behavior-period-parity — Required member behavior is absent from the executable seven-case suite and therefore cannot support the affected ledger rows. P.RawBindingSpec.name has no null or non-string P control; P.RawBindingSpec.selection_arrays has no explicit-null P control for either array; P.HostInputContract.owner and authority have no wrong-kind P control; RawBindingSpec.refs has no non-null wrong-list-kind P control; Q.RawCommandSpec.outcomes has no wrong-item-kind P control; and Q.RawOutcome.refs has no null-item P control. These are explicit required/wrong-kind/null or item/list boundaries in the accepted contract. The existing source tables cover omission, lexical errors, scalar/list alternatives, or item errors selectively, but none executes these listed conditions. — crates/edge/ess-cli/src/load_accounting_tests.rs:4722

task:consumer-accounting-wire-behavior-period-parity — The seven member tables omit 26 required controls that the frozen source does execute, so the candidate is not an accurate finite source-to-member mapping. The omitted controls are: N-qualified-type-name-missing/null; G-qualified-owned-domain-null and G-qualified-view-null; E-duplicate-wire-current-acceptance; P-anchor/first/cadence/overlap/missed/lifetime/cancellation/eligibility-null, P-host-kind, P-periodic-closure, P-context-field-type-null and P-read-field-type-null; Q-outcome-missing-name, Q-outcome-emits-item-kind, Q-outcome-refs-kind and Q-source-both-null; and S-mapping-selection-kind, S-first-in-kind, S-first-present-kind, S-first-excluding-kind and S-predicate-number-kind. None appears in the index either. These cover required-name/null, current accepted duplicate-wire behavior, one-word null refusals, object closure/kind, Field null type, RawOutcome member shape, explicit payload pair, and Selection/First2/Predicate kind boundaries; they cannot be replaced by a helper name or a broader descriptive label. — finite-member-reference-control-ledger-v3.candidate.json:49

task:consumer-accounting-wire-behavior-period-parity — SS-null compiles an outcome with explicit `when_subject_state: null` but then checks only that the serialized IR outcome has no member literally named `when_subject_state`. EssIr represents this through `condition`, so that assertion does not establish the actual resulting condition or test strategy. The base outcome is asserted elsewhere, but the mutated `null_ir` is not compared with it. This does not meet the accepted requirement for the explicit-null site to assert its exact returned behavior. — crates/edge/ess-cli/src/load_accounting_tests.rs:3884

What I read: the same first audit covering the frozen ledger/index, all 72 identity rows, all seven case tables and 98 member rows, the accepted contracts, assigned source vector, and retained focused receipt; no new review was performed.

What I could not establish: no new uncertainty beyond the original audit; this formatting adaptation does not reassess findings or qualification.

```findings
- file: finite-control-index-v3.candidate.json
  line: 13
  category: data
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: >-
    The index is not a complete index of the ledger. The ledger assigns
    G-entity-setting-type and G-undeclared-setting-type at K to both
    G.RawComponentSpec.settings and G.RawComponentSetting.type (four uses), but the index has no
    entries for either label and declares only 1,102 uses. Counting C/P/A/K in the ledger gives
    1,106. The frozen source really executes both compile refusals with exact diagnostic codes and
    locations, so these are lost adoption records rather than speculative controls. The recorded
    consistency method checked only c/p/a and therefore could not detect the omission.
- file: crates/edge/ess-cli/src/load_accounting_tests.rs
  line: 4722
  category: data
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: >-
    Required member behavior is absent from the executable seven-case suite and therefore cannot
    support the affected ledger rows. P.RawBindingSpec.name has no null or non-string P control;
    P.RawBindingSpec.selection_arrays has no explicit-null P control for either array;
    P.HostInputContract.owner and authority have no wrong-kind P control; RawBindingSpec.refs has
    no non-null wrong-list-kind P control; Q.RawCommandSpec.outcomes has no wrong-item-kind P
    control; and Q.RawOutcome.refs has no null-item P control. These are explicit
    required/wrong-kind/null or item/list boundaries in the accepted contract. The existing source
    tables cover omission, lexical errors, scalar/list alternatives, or item errors selectively,
    but none executes these listed conditions.
- file: finite-member-reference-control-ledger-v3.candidate.json
  line: 49
  category: data
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: >-
    The seven member tables omit 26 required controls that the frozen source does execute, so the
    candidate is not an accurate finite source-to-member mapping. The omitted controls are:
    N-qualified-type-name-missing/null; G-qualified-owned-domain-null and
    G-qualified-view-null; E-duplicate-wire-current-acceptance; P-anchor/first/cadence/overlap/
    missed/lifetime/cancellation/eligibility-null, P-host-kind, P-periodic-closure,
    P-context-field-type-null and P-read-field-type-null; Q-outcome-missing-name,
    Q-outcome-emits-item-kind, Q-outcome-refs-kind and Q-source-both-null; and
    S-mapping-selection-kind, S-first-in-kind, S-first-present-kind,
    S-first-excluding-kind and S-predicate-number-kind. None appears in the index either. These
    cover required-name/null, current accepted duplicate-wire behavior, one-word null refusals,
    object closure/kind, Field null type, RawOutcome member shape, explicit payload pair, and
    Selection/First2/Predicate kind boundaries; they cannot be replaced by a helper name or a
    broader descriptive label.
- file: crates/edge/ess-cli/src/load_accounting_tests.rs
  line: 3884
  category: data
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: >-
    SS-null compiles an outcome with explicit `when_subject_state: null` but then checks only that
    the serialized IR outcome has no member literally named `when_subject_state`. EssIr represents
    this through `condition`, so that assertion does not establish the actual resulting condition
    or test strategy. The base outcome is asserted elsewhere, but the mutated `null_ir` is not
    compared with it. This does not meet the accepted requirement for the explicit-null site to
    assert its exact returned behavior.
```
