---
format: aep.planning-md/3
id: story:synthesis-witnesses-a-digest-of-inputs-refusal-by-changing-each-named-input-alone-so-a-target-that-leaves-an
kind: story
status: draft
title: Synthesis witnesses a digest-of-inputs refusal by changing each named input alone, so a target that leaves an input out of the digest fails
tags:
- gap-p2
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:a-command-guard-compares-a-value-with-the-digest-of-several-named-inputs-hex-sha256-tuple-and-the-interpreted
revision: 2
---
## Need

`g15-digest-of-inputs` admits `hex(sha256(tuple(…)))` in a command guard, and the interpreted
target decides it. For one fact under `sha256`, synthesis already sends an input whose digest
matches and one whose digest does not (beyond10x/ess#584,
`website/docs/reference/predicates.md:483-487`). With several facts, one mismatching input is not
enough. A target that leaves one input out of the digest, swaps the order of two, or encodes an
absent `Optional` as an empty String still refuses a request whose key was computed from other
values altogether. The suite has to change each named input alone.

## Outcome

For a guard `key != hex(sha256(tuple(f1, …, fn)))`, the synthesized suite holds the accepting
scenario, with a key computed from the inputs it sends, and one refusal scenario per named fact,
in which only that fact differs from the inputs the key was computed from. An `Optional` fact gets
a further scenario in which it is absent on one side and an empty String on the other.

## Acceptance

- Red first: `crates/verify/ess-conformance/tests/digest_of_inputs.rs` synthesizes the
  `g15-digest-of-inputs` reproduction. Its suite passes on the interpreted target, and each of
  four mutated targets fails at least one scenario: one leaving `revision` out of the tuple, one
  swapping `source` and `revision`, one encoding an absent `profile` as `""`, and one comparing
  only the first eight hex digits.
- The refusal scenarios are named `<command>/outcome/<refusal>/<fact>`, or carry the fact in a
  name suffix the suite format already writes. No suite vocabulary moves. If one has to move, it
  is split out as its own unit before this one lands.
- A guard with a single fact keeps its suite bytes (the `ess/27` PKCE fixtures).
- `cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table` passes. A re-pinned
  row exists only for a model using `tuple`, and names "digest of named inputs witnessed".
- No new test runs over 60 s, and the report names the time of every new test over 5 s.

## Fit review

Tree `bcfc51555`. Report item 15, second half. See `g15-digest-of-inputs` § Fit review for
questions 1-5 and 7.

6. **Cost.** No format, keyword or IR change. Models using `tuple` get more scenarios. Size M.

## Decisions

**Accept, redesigned** (`P2B-D2` (a)). Open question:

- **D1 (decided: a, 2026-11-24). One refusal scenario per fact, or one scenario that changes every fact?** (a) One per
  fact. (b) One that changes all of them. **Recommended: (a).** (b) passes a target that leaves
  any one fact out.

## Scope

- inferred: `crates/verify/ess-conformance/src/input.rs` (input solving for a guard over a
  function; the existing `sha256` path)
- inferred: `crates/verify/ess-conformance/src/synthesize.rs` (refusal scenario per fact)
- inferred: `crates/verify/ess-conformance/tests/digest_of_inputs.rs` (new)
- cited: `crates/verify/ess-conformance/tests/synthesis_suite_bytes_table.rs` pins
- Size: M (about 350 changed lines).

## Depends on

- `g15-digest-of-inputs`

## Source

A downstream application, 2026-11-24, report item 15.
