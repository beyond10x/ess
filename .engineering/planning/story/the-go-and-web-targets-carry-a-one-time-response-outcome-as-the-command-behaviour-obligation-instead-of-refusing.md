---
format: aep.planning-md/3
id: story:the-go-and-web-targets-carry-a-one-time-response-outcome-as-the-command-behaviour-obligation-instead-of-refusing
kind: story
status: draft
title: The Go and web targets carry a one-time response outcome as the command behaviour's obligation instead of refusing
tags:
- rust-target-follow-up
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-generated-code
- depends_on: story:the-rust-target-writes-a-one-time-response-outcome-as-an-obligation-that-carries-its-contract-instead-of-refusing-the-workspace
scope:
- confidence: inferred
  path: crates/generate/ess-synth/src/failure.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/go/obligation.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/web/catalog.rs
- confidence: inferred
  path: crates/generate/ess-synth/tests/one_time_response.rs
revision: 2
---
## Outcome

The Go and web targets write a command whose outcome declares `one_time_response:` with its
behaviour as the implementation obligation the plan holds, whose contract carries the one-time duty,
as the Rust target does after
`story:the-rust-target-writes-a-one-time-response-outcome-as-an-obligation-that-carries-its-contract-instead-of-refusing-the-workspace`,
and no longer refuse the whole workspace because of it.

## Problem

`crates/generate/ess-synth/src/failure.rs`, `fn one_time_response`, refuses every outcome with the
key for every target (`go/mod.rs:323`, `web/mod.rs:303`), and the browser catalog marks such a
command not dispatchable (`web/catalog.rs:178-189`). Decision OT-D1 of the Rust story keeps that
refusal for Go and web and files this story. The plan already carries the obligation and its
contract clause for every target (OT-D2 of the Rust story); only the Go stubs and the web app have
to stop refusing.

## Acceptance

- `crates/generate/ess-synth/tests/one_time_response.rs`:
  `all_implementation_entry_points_refuse_an_unimplemented_disclosure_policy` keeps only
  `Target::Clap`; a Go case finds the behaviour's stub carrying the contract clause and returning the
  unmet obligation, never the value.
- The web target either generates the command with the obligation or keeps a refusal that names the
  outcome, as this story's decision says; `direct_browser_catalog_refuses_dispatch_without_losing_policy_location`
  changes accordingly.
- A model without `one_time_response:` generates the bytes it generated.

## Decisions

- Origin: OT-D1 A of the Rust story. Open until that decision is taken; archived if B is taken.
- Whether the browser catalog dispatches such a command to hand code or keeps it refused: open,
  decided when this story is refined.

## Scope

- inferred: crates/generate/ess-synth/src/failure.rs
- inferred: crates/generate/ess-synth/src/go/obligation.rs
- inferred: crates/generate/ess-synth/src/web/catalog.rs
- inferred: crates/generate/ess-synth/tests/one_time_response.rs
