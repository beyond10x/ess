---
format: aep.planning-md/3
id: review-result:adversary-w1-syn1-pass-1
kind: review-result
status: active
title: 'W1 SYN1 adversary pass 1: grant/denied beside a ranked view and every ungranted actor'
summary: APPROVE, 226 to 234 executed, 0 red, no findings; eight cases kept
relations:
- reviews: story:grant-denied-is-synthesized-beside-a-ranked-view
- reviews: story:a-denied-scenario-covers-every-ungranted-actor-of-the-domain
revision: 1
---
unit: W1 SYN1 (story:grant-denied-is-synthesized-beside-a-ranked-view, story:a-denied-scenario-covers-every-ungranted-actor-of-the-domain), e5905e8e6 and 509854bee merged as 7835e2196, reviewed in tree ess-w1-syn1 at 509854bee
verdict: APPROVE
cases: executed 226→234, red 0
origin: introduced 0 / pre-existing 0 / undecided 0
wrote-outside-worktree: the wave scratch directory syn1/adversary-scratch (cargo log); the Go parity helper's temporary directory under the harness scratch, removed by the helper
needs-coordinator: none

**Findings.** None.

**Cases kept.** Eight cases in `tests/adversary_w1_syn1_adversary.rs`, committed with the unit as da9a26af1:
an updating command and an update of the ranking field beside a ranked view, a moving command beside two
ranked views, each refused send its command's own and last; a target leaking only the second ungranted actor
of the domain fails exactly the denied read, with Go parity; system and attributed ungranted actors keep the
suite passing.

**Attacked, not broken.** The template's first pass keeps its bytes (bytes table green, no row re-pinned);
no repository model gains a `grant/denied`; `denied_from` drops every later send; the fallback path and
`ReadGrantedToEveryActor` are unchanged; the multi-actor denied read runs in the Rust and Go runners; the
three documents and the CHANGELOG match the code. Peak target/ 6G, lowest / 89G.
