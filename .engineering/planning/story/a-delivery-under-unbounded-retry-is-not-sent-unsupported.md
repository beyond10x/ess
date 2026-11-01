---
format: aep.planning-md/3
id: story:a-delivery-under-unbounded-retry-is-not-sent-unsupported
kind: story
status: draft
title: A binding delivery under an unbounded retry is not synthesized as Unsupported
relations:
- serves: vision:O2
revision: 1
---
## Finding

Found by adversary pass 1 of wave D350 U2 (`review-result:adversary-d350-u2-pass-1`, O1, out of
scope, origin undecided): a binding delivery scenario (`raise-on-lowered/binding/delivery`) is
synthesized where the second delivery's command (`Raise`) refuses under an unbounded
`on_failure: retry`, and the model interpreter reports it `Unsupported` ("binding invocation
budget exhausted"). Raise has no related guard; not checked on a base tree.

## Acceptance

- A red case first reproduces the scenario on the base of 0.81.0 and names whether it predates
  D350.
- Synthesis does not send a delivery scenario whose retried command cannot succeed, or the
  interpreter answers it with a declared outcome; it is not reported `Unsupported`.
