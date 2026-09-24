---
format: aep.planning-md/2
id: review-result:adversary-wave22-unit1-pass-2
kind: review-result
status: active
title: Adversary pass 2 against the Cyclic/Uninhabited split
relations:
- reviews: story:literal-representation-walk-exhaustion
revision: 1
---
## Pass

`aep-drive:adversary`, pass 2, against `wt-8fb8a43f6a81` on `impl/literal-representation-walk-exhaustion`
over base `c80230671162c512405b95b9c8596ec37d2f762e`, attacking correction round 1's
`Cyclic`/`Uninhabited` split. Verdict **CONFIRMED**. Cases executed 426 → 433, red 1.
Origin: introduced 2, pre-existing 1, undecided 0.

Seven cases added in `crates/specify/ess-domain/tests/literal_representation_adversary_pass2.rs`;
pass 1's file was not edited and still passes 5/5.

## The matrix

Case 1 drives **all 512 three-name registries** over `{bare name, Optional<name>, String, enum}`
through `RawSpecFile::parse` + `Specification::assemble` and asserts which of three claims each
shape draws. Case 2 replays the same 512 through an outcome `payload:` and compares. Case 5 is the
teeth: the obvious mutant — counting every `Optional` rather than only those inside the ring —
reclassifies at least one shape between two classes case 1 asserts opposite things about. Case 1
also fails as vacuous if any of its five classes was not populated by a real document.

The three other base cases `check_inhabitation`'s own hint names — `List`, `Map`, a union with a
terminating variant — terminate the walk as `Structured` before a ring can close, so the uncounted
base case cannot produce a wrong answer.

## Attacked and could not break

`Uninhabited` is never silence nobody owns, checked on all 512 shapes and structurally: in a ring
the walk can close on, every member is a newtype whose `of` is a bare name in the ring, so the
fixpoint marks none of them. `Cyclic` is never a double report for a newtype/`Optional` ring — one
`Optional` inside the loop makes every member inhabited and `check_inhabitation` silent. Payload and
binding made identical claims on all 512 shapes. `Established` unchanged. `Undeclared` reported by
the resolver whether reached bare, under an `Optional`, or at the end of a 33-link chain. The five
consumer-coverage rows are complete in both directions. Pass 1's finding 1 is repaired: the doc lane
that exited 101 exits 0.

```findings
- file: crates/specify/ess-domain/src/binding.rs
  line: 1620
  category: judgement
  severity: warning
  verdict: CONFIRMED
  origin: pre-existing
  message: the Structured arm does not apply the ownership rule the unit adopted for rings, so a ring closing through a struct or union draws two self_reference errors from check_inhabitation and a third refusal from the literal check for one mistake.
- file: crates/specify/ess-domain/src/binding.rs
  line: 99
  category: contract-drift
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: the new module-doc table row states that a type whose wrappers resolve through themselves is refused here, which the prose seven lines below and the code both contradict for the half that has no base case.
- file: crates/specify/ess-domain/src/types.rs
  line: 149
  category: contract-drift
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: MAX_TYPE_DEPTH still justifies its 32 by saying WRAPPER_LIMIT chose 32 for the same reason, and this unit rewrote WRAPPER_LIMIT's doc to give a different reason and to say it no longer bounds validation.
- file: crates/specify/ess-compiler/src/ir.rs
  line: 761
  category: contract-drift
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: pass 1's finding 2 is still live in this tree — the public IR docs at :761 and :1050 still bound literal checking by WRAPPER_LIMIT — but ess-compiler is outside the unit's assignment and the coordinator owns the patch, so it cannot be settled here.
```
