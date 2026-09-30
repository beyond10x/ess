---
format: aep.planning-md/3
id: story:served-committed-command-answers-its-outcome
kind: story
status: draft
title: A committed command is answered with its outcome, not 501
relations:
- serves: vision:O2
- decomposes: epic:language-consistency-after-adoption
revision: 1
---
## Outcome

A command whose effect was committed is answered with its own declared outcome, and a failed delivery is reported beside it; 501 means only that the port did not run (review S24, worst five #2).

## Acceptance

- the served answer of a committed command whose delivery failed is its outcome status and body plus `undelivered: [{binding, event}]`, declared in the contract
- 501 is answered only when the port did not run; its contract description says so
- `Refused::Undelivered` / `BridgeError::Undelivered` are removed or reduced to one breaking change, batched with any other breaking change of the same release
- Rust, Go and the web bridge agree; a client retry test shows the committed case is never answered as an error

## Origin

Evidence: `docs/design/review-external-requests-2026-09.md` (served-committed-command-answers-its-outcome). Drafted, not scheduled; a fit review precedes dispatch.
