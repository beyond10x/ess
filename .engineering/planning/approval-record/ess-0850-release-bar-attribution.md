---
format: aep.planning-md/3
id: approval-record:ess-0850-release-bar-attribution
kind: approval-record
status: approved
title: 'The 0.85.0 release bar''s 244b moves: MUT1 mutant moves are re-pinned, the pre-W2 suite moves are bisected'
summary: Orchestrator decision of 2026-11-03 on the attribution of the red 0.85.0 release bar
relations:
- decides: story:error-swap-mutates-refusals-that-carry-a-payload
- decides: story:the-mutation-audit-covers-actor-grants
revision: 4
transitions:
- {from: "draft", to: "approved", at: "2026-11-03T20:53:03Z", actor: "human:timo", revision: 3}
---
## Decision

2026-11-03, orchestrator decision under the operator's grant: the 0.85.0 release bar on merge
`b4e68f1ae8` (PR 721) was red in one test,
`adversary_244b_every_window_free_model_keeps_its_bytes_against_3b1684d1a`, with 125 moved model
labels. The other slow probes, `adversary_287_pass1` (2/0) and `adversary_w3_266_267_dump` (1/0),
passed. The tag waits for one follow-up pull request to `main` and a green bar on its merge.

## Attribution

The `adversary_244b` probe was run at `b89faaa411` (0.84.0), `3cc17ed596`, `cffb7deb3e`,
`4489c7cf8c`, `fd11ea5ca9` (SYN2 merge), `a48d077e02` (MUT1 merge) and `8dec7706dd` (MUT1 fix
round 1). Each moved line was compared field by field with its `INTENDED` pin, or with the base
digest where it had none.

- Group A, 121 labels including `@ess22` copies: only the `mutants` digest differs. IR, suite and
  refusals equal the pinned line. The first red commit is `a48d077e02`: `578fde2652` (beyond10x/ess#709,
  `story:error-swap-mutates-refusals-that-carry-a-payload`) and `ad383e41a5` (beyond10x/ess#710,
  `story:the-mutation-audit-covers-actor-grants`) add mutants through `MutantClass::ALL`.
  Synthesis outside the audit runs with no dropped grant, so its bytes stay the same. Decision:
  `INTENDED` re-pin, citing #709, #710, `ad383e41a5` and `578fde2652`.
- Group B, 4 labels: `text-length-positions.yaml` and `binding-arrangement.yaml`, each with its
  `@ess22` copy. Their suite (and, for `text-length-positions`, refusals) moved, and the first red
  commit is at or before `b89faaa411`, in `e266be529f..b89faaa411`. That range is the tail of
  0.83.0 plus W1, so the cause is not a W2 commit. Decision: bisect the range. If the story at the
  first red commit intended the move, re-pin and cite that story. If no story intended it, fix it in
  the follow-up when the fix is one unit; if it is larger, stop and report the story and its size.
- SYN2 moved no label.

## Group B bisect (2026-11-03)

The probe was run at `b81deac789` (0.83.0), `21ef8a7fae`, `bfa1aaee9a`, `0459c8708f`, `07bd42c36f`
and `86d3d194dd` (0.82.0 plus the D368 store opening). Every one of them reads the same Group B
digests as `b4e68f1ae8`. `synthesis-suite-bytes-table.tsv` names the commits that moved them, and
each move was re-pinned there by its story:

- `text-length-positions`: `915fe1fca8` (0.79.0, `story:a-binary64-value-round-trips-through-a-conformance-suite`).
  The model reaches a Binary64 and synthesizes at `ess-conformance/62`; before, it was refused.
- `binding-arrangement`: `b1a810d3d7` (0.82.0, `story:binding-site-identity-comparisons`). Binding
  scenarios compare the identities of their trigger event.

Both moves are intended by their stories, and this probe was not run against them. Decision:
`INTENDED` re-pin citing both stories, in the same follow-up.
