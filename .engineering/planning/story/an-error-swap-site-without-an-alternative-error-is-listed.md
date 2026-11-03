---
format: aep.planning-md/3
id: story:an-error-swap-site-without-an-alternative-error-is-listed
kind: story
status: draft
title: An error-swap site without an alternative error is listed
summary: A payload-less refusal in a single-error domain is skipped silently while its payload twin is listed (W2 MUT1 review finding, pre-existing, backlog)
tags:
- backlog
- from-finding
- review-finding
relations:
- informed_by: review-result:adversary-w2-mut1-pass-1
revision: 1
---
## Need

`ess verify conform mutate`'s `error-swap` lists a refusal that carries its error's payload and has
no other error that compiles as `no_compatible_error_alternative` (wave W2,
story:error-swap-mutates-refusals-that-carry-a-payload). A refusal without a payload in a domain
that declares only one error has no other error to swap in either, and it is skipped silently: no
mutant and no unmutated site (`crates/verify/ess-conformance/src/mutate.rs:2296`; the same skip is
at `mutate.rs:1742` at 4a0d58c611). Two refusals with one cause are reported differently, and the
site coverage count leaves the silent one out. Found by the W2 MUT1 adversary pass
(review-result:adversary-w2-mut1-pass-1, F5); pre-existing.

## Acceptance

- In a domain with one error, every refusal `error-swap` selects, with or without a payload, is
  listed under `unmutated_sites` with one reason, and the summary's site count includes it.
- A model with a second error keeps its error-swap ids, verdicts and report bytes.

## Schedule

Backlog: not waved; considered only after every predefined wave is released.

## Repro

The adversary's case `error_swap_lists_both_refusals_of_a_single_error_domain` in
`crates/verify/ess-conformance/tests/adversary_w2_mut1_adversary.rs` (red at ad383e41a5).
