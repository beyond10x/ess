---
format: aep.planning-md/3
id: story:the-go-and-typescript-runners-admit-and-judge-an-awaited-optional-suite-like-rust
kind: story
status: draft
title: The Go and TypeScript runners admit and judge a suite with an awaited Optional input the way the Rust runner does
summary: Go refuses admission (fresh original pair) and TS fails grant/denied on the row 7.1 fixture suite; pre-existing, found in 0.69.0.
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 1
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
