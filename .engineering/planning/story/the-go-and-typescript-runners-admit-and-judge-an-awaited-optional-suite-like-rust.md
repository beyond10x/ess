---
format: aep.planning-md/3
id: story:the-go-and-typescript-runners-admit-and-judge-an-awaited-optional-suite-like-rust
kind: story
status: implemented
title: The Go and TypeScript runners admit and judge a suite with an awaited Optional input the way the Rust runner does
summary: Go refuses admission (fresh original pair) and TS fails grant/denied on the row 7.1 fixture suite; pre-existing, found in 0.69.0.
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-verdicts
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-11-09T01:00:48Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-11-09T01:00:48Z", actor: "human:timo", revision: 4}
- {from: "active", to: "implemented", at: "2026-11-09T01:01:05Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"test_result":1}}}
---
## Need

Found while landing `story:an-optional-input-a-system-command-awaits-is-compared-only-when-issued`
(0.69.0). Replaying the whole synthesized suite of that story's fixture (an entity `Job` with
`note: Optional<Note>`, an agent command `Finish(job_id, note: Optional<Note>)` that
`sets: {note: input.note}`, and a system command awaiting `Finish`) through the generated runners
gives verdicts the Rust runner does not:

- The Go runner refuses to admit the suite: `jobs.job.Job/state/Done/refuses/jobs.job.Finish:
  complete subject comparison lacks its fresh original pair`.
- The TypeScript runner fails the `grant/denied` scenarios that pass in the Rust runner.

Both exist without the 0.69.0 change (the unit's tests replay only the `/moved` scenarios for that
reason). Whether the TypeScript failure is the one
`story:a-refused-send-keeps-the-earlier-read-token-for-a-denied-grant` describes is not checked.

## Acceptance

- The whole synthesized suite of that fixture gets the same verdict per scenario from the Rust, Go
  and TypeScript runners against the interpreted target; one test replays it through all three.
- The Go admission refusal above either stops, or is shown to be right and the Rust runner refuses
  the same suite too.

## Reconcile note (2026-11-02)

Store hygiene H0 of wave W1, section 2i: Re-run the TypeScript half before W3; if TypeScript passes, the story narrows to Go.
