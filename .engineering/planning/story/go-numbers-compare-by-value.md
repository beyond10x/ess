---
format: aep.planning-md/3
id: story:go-numbers-compare-by-value
kind: story
status: active
title: Go conformance compares JSON numbers by value
refs:
- provider: github
  reference: beyond10x/ess#101
relations:
- serves: vision:O2
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-09-26T02:30:42Z", actor: "human:timo", revision: 2, imported: true}
- {from: "proposed", to: "active", at: "2026-09-26T02:31:36Z", actor: "human:timo", revision: 3, imported: true}
---
## Outcome

Go conformance compares JSON numbers by value.

## Why

GitHub issue beyond10x/ess#101; its Observed and Expected sections are the contract and are not restated here.

## Acceptance

- Every expectation in beyond10x/ess#101 holds, each with a red-first test.
- Where the issue offers alternatives, the design page or unit report names the one taken.

## Reconciliation 2026-10-02

Implementation is released in0.51.0; historical red-first output has not been located. Commit4a521d9e0 is an ancestor of the public release. Go runtime equality uses numberValue (src/go/runtime.go:3041/:3141); ordering does likewise at:3569. tests/primitive_corpus.rs::the_go_runtime_compares_every_number_carrier_by_the_value_rust_decides covers all13 carriers. tests/fixtures/number-carriers.go::TestSuiteExpectationsMatchWhateverTypeCarriesTheNumber reaches the issue's actual suite-expectation seam. The current complete conformance package primitive_corpus lane passed9 tests, including this case.

The chosen design compares numeric values, not a README restriction to float64. Source/regression behavior and release containment are verified. The original acceptance also asked for red-first chronology; current passes cannot manufacture that historical evidence. No new implementation is justified by this evidence gap. Lifecycle remains active pending reconciliation of that process acceptance; no claim that the behavior is still unimplemented.
