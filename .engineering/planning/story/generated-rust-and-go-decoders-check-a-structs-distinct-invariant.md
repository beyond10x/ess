---
format: aep.planning-md/3
id: story:generated-rust-and-go-decoders-check-a-structs-distinct-invariant
kind: story
status: active
title: Generated Rust and Go decoders check a struct's distinct invariant
summary: '0.94.0 refuses to generate Rust or Go for a workspace with distinct: in a struct, which 0.93.0 generated; the new struct invariant check does not evaluate distinct'
tags:
- backlog
- defect
- from-finding
- regression
- severity-high
relations:
- serves: vision:O2
- decomposes: epic:zero-open-issues-2026-11
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-12T22:38:41Z", actor: "human:timo", revision: 3, executor: "agent:b10x", correlation: "operator-2026-11-12-d8"}
- {from: "proposed", to: "active", at: "2026-11-12T22:38:43Z", actor: "human:timo", revision: 4, executor: "agent:b10x", correlation: "operator-2026-11-12-d8"}
---
## Defect

From 0.94.0 `ess generate synthesize --target rust` (and `--target go`) refuses a whole workspace
in which one struct declares a `distinct:` invariant. 0.93.0 generated the same workspace with
exit 0. The changelog of 0.94.0 does not say that a workspace 0.93.0 generated is now refused.

Reported by a consumer on 2026-11-12; reproduced on the 0.94.0 release binary:

```
$ ess generate synthesize --path spec --target rust --layout crate --http http.yaml --out out
rust target cannot emit this workspace
types.<domain>.Config.invariants.3: the generated decoder cannot check `<domain>.Config`: `distinct instance in instances by instance.name`: `distinct instance in instances by instance.name as string` requires distinct list members, which the generated invariant check does not evaluate
… (one line for each of six types)
$ echo $?
1
```

The reporter's specification declares `distinct:` nine times in six structs, with and without
`by:`. It cannot move past 0.93.0.

## Cause

0.94.0 added the struct invariant check to the generated Go and Rust decoders (commit
`1a35fbdc4d`, changelog: "Generated Go and Rust decoders check a struct's declared invariants").
The check refuses `Predicate::Distinct` by name
(`crates/generate/ess-synth/src/rust/invariant.rs:517-522`,
`crates/generate/ess-synth/src/go/invariant.rs:660`): "The shared invariant evaluator compares no
keys across a list's elements". A refused invariant fails the whole target
(`TargetFailureCode::MissingRepresentation`). Before 0.94.0 the decoders checked no struct
invariant, so the same declaration generated and was left unchecked.

Two smaller observations on the same output:

- A struct with four `distinct:` invariants is named once, at the first of them
  (`invariants.3` of `Config`; `invariants.4` to `.6` are not listed), so an author fixing the
  named one meets the next on the following run.
- The same refusal exists for `Predicate::Window` (a calendar window). Validation admits a window
  only in a command guard, so no struct reaches it today.

## Acceptance

1. The generated Rust and Go decoders evaluate a struct's `distinct: {in, as}` and
   `distinct: {in, as, by}`: a request whose list holds two elements with the same key is
   answered 400 with a decode error at the struct's path naming the invariant, in the words the
   other invariant kinds use. The comparison is the one the interpreted target and the published
   schema use for `distinct:` (exact or case-folded as declared, an Optional key's absent values
   as the reference states them).
2. A model holding each `distinct:` form in a struct (scalar list; `by` a text member, an enum
   member, an Optional member; a list nested in a struct of a list) generates for both targets,
   and for each form one decoder test refuses a duplicate and admits a distinct list. The
   generated servers pass the model's conformance suite, value-invariant scenarios included.
3. A workspace 0.93.0 generated generates again: the reporter's shape (six structs, nine
   `distinct:` invariants) is a regression test for both targets.
4. Where a target still cannot evaluate an invariant, every such invariant is named, one line
   each, not the first per type.
5. The changelog entry states under the release that fixes it that 0.94.0 refused such a
   workspace and that the decoders now check `distinct:`; the entry is marked breaking for a
   client that sends a list with duplicate keys, as the newtype entry of 0.62.0 and the Go entry
   were.
6. Entity Runtime's refusal of `distinct:` (`crates/generate/ess-entity-runtime/src/lib.rs:834`)
   is out of scope and unchanged.

## Evidence

Reproduction on the 0.94.0 release binary: session scratch `ess/adopter11/` (`run.sh`,
`gen-rust.log`; not in the repository).

## Decisions

**Accepted (2026-11-12).** Decided on the operator's behalf, under the delegation the operator
gave that day for decisions that are low risk or easy to revert. Reason given: a defect against
shipped behaviour, reproduced on the 0.94.0 release binary; an accepted story can be moved back.
The two transitions carry the correlation `operator-2026-11-12-d8` and the executor `agent:b10x`. No
review-result exists for this draft, so no `review_outcome` record was written.
