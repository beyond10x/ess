---
format: aep.planning-md/3
id: story:identifier-guards-get-witnesses-and-authored-inputs-meet-their-type
kind: story
status: active
title: Guards over constrained identifiers get witnesses, and authored inputs are held to their type
summary: Five synthesis and authoring gaps from one hosting-lifecycle domain (downstream gateway specification, 0.77.0); next wave.
tags:
- adopter-report
- gap
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-10-31T19:58:59Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-10-31T19:59:00Z", actor: "human:timo", revision: 3}
---
## Need

Reported by a downstream gateway specification session (2026-10-30, ESS 0.77.0 f692aab), from one
hosting-lifecycle domain whose commands refuse an identifier that is empty, holds a space, is over
256 bytes or holds a byte outside printable ASCII. Five findings, each worked around downstream:

1. An authored scenario's input is not held to its declared type: an identifier holding a space and
   an empty evidence text are admitted although the declared type excludes them.
2. A constrained input type gets no witness for a guard the type excludes: an input-guarded refusal
   over a constrained text type is refused `ESS-SYNTH-003`, so the type had to be loosened.
3. The guard grammar cannot say "every byte is printable ASCII"; the refusal for such a byte stays
   a target error.
4. An input-guarded refusal beside an exhaustive answer partition with no default branch gets no
   witness (`ESS-SYNTH-003`): synthesis asks the refusal's witness to refute every accepting
   guard although the refusal answers first (reported at `synthesize.rs:8567`, `admits_plain`).
5. `ESS-SYNTH-022` finds no input for 257-byte children of an `any:` guard; the reporter withdrew
   the refusals by authoring three scenarios that send a 257-byte value.

## Triage

Reported on 0.77.0, which is `main` (f692aab); not re-run in wave D325 (no standalone
reproduction; the reporter's specification carries the workarounds). Next wave: the fit review
rebuilds one brand-free reproduction per finding and designs them into generic features.
