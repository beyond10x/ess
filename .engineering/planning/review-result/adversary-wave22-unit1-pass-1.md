---
format: aep.planning-md/2
id: review-result:adversary-wave22-unit1-pass-1
kind: review-result
status: active
title: Adversary pass 1 against literal representation walk exhaustion
relations:
- reviews: story:literal-representation-walk-exhaustion
revision: 1
---
## Pass

`aep-drive:adversary`, pass 1, against `wt-8fb8a43f6a81` on `impl/literal-representation-walk-exhaustion`
over base `c80230671162c512405b95b9c8596ec37d2f762e`. Verdict **NEEDS-CHANGE**.
Cases executed 418 → 423, red 3. Origin: introduced 3, pre-existing 0, undecided 1 (five findings;
two share the `introduced` classification without a red case).

Five cases added in `crates/specify/ess-domain/tests/literal_representation_adversary.rs`, written
before anything was run. Two green (the acceptance asserted through the public document surface at
31/32/33 links, and the `Cyclic` arm reached from a real YAML document), three red.

## The gate the package lanes cannot see

```
$ RUSTDOCFLAGS='-D warnings' cargo doc --package ess-domain --no-deps --locked
error: public documentation for `binding` links to private item `representation`
error: unresolved link to `BindingCheck::check_literal`
error: public documentation for `WRAPPER_LIMIT` links to private item `representation`
EXIT: 101
```

`cargo fmt --package ess-domain -- --check` and
`cargo clippy --locked --package ess-domain --all-targets -- -D warnings` are both exit 0 with the
adversary's file in the tree, so the red belongs to the change and not to the new cases.

## Attacked and could not break

The walk is linear, so the visited set is the path set and cannot produce a false `Cyclic`;
termination is bounded by registry size. A chain re-entering a name by a different path, an
`Optional`/newtype mixture at `MAX_TYPE_DEPTH` on top of a 33-link chain, and cycles through
`Optional`, `List`, `Map`, struct and union bodies all terminate and classify correctly. A union
short-circuits to `Structured` before any variant is inspected. `Undeclared` still stays silent and
`build_registry` really does report the unresolved reference. The new `TypeMismatch` is
source-addressed on both consumers and deterministic. No valid enum or String literal that passed
before now refuses.

```findings
- file: crates/specify/ess-domain/src/binding.rs
  line: 1702
  category: contract-drift
  severity: blocker
  verdict: CONFIRMED
  origin: introduced
  message: three new doc links do not resolve — `BindingCheck` names no type in the workspace and the public `WRAPPER_LIMIT` and module docs link the private `representation` — so `task check`'s doc-check lane fails with exit 101 while the package gate stays green.
- file: crates/specify/ess-compiler/src/ir.rs
  line: 761
  category: contract-drift
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: the public IR docs at :761 and :1050 still tell consumers that literal checking is bounded by WRAPPER_LIMIT and that exhausting it establishes no guarantee, which this change made false.
- file: crates/specify/ess-domain/src/binding.rs
  line: 1588
  category: judgement
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: the Cyclic arm claims nothing else reports the case, but `Ring = newtype of Ring` is already refused as self_reference by check_inhabitation, so one mistake now yields two diagnostics — the exact double-report the Undeclared arm exists to avoid.
- file: crates/edge/ess-xtask/src/consumer_coverage/entry-classifications.json
  category: contract-drift
  severity: blocker
  verdict: CONFIRMED
  origin: introduced
  message: the new `Resolution` enum and its three variants mint four consumer-coverage entries with no classification row, so `consumer-check` bails "unclassified concrete consumer entry"; GitHub CI skips that lane, the integration `task check` does not.
- file: crates/generate/ess-gen/src/docs.rs
  line: 1765
  category: boundary
  severity: note
  verdict: INFEASIBLE
  origin: undecided
  message: the docs renderer still stops at WRAPPER_LIMIT and prints "establishes no additional value constraints" for a literal the compiler now verifies past 32 wrappers — an under-claim, and I found no document in the repository with a chain that long.
```
