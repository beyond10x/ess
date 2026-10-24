---
format: aep.planning-md/3
id: story:identity-minting-and-view-ordering-are-public
kind: story
status: draft
title: Identity minting and view row ordering are public for consumers
summary: 'Adopter report row 28: minting and view ordering are private, so consumers copy them.'
tags:
- ess-0.70.0
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 1
---
## Problem

The interpreted target mints identities and orders view rows with functions consumers cannot call: `fn minted` (`crates/verify/ess-conformance/src/interpret/execute.rs:2827`) is private and `compare` (`crates/verify/ess-conformance/src/interpret/views.rs:340`) is `pub(crate)`. A downstream renderer that must agree with the interpreter copies both; before its copy it measured 86 mismatches against the interpreter (adopter report, row 28, ess 0.67.0). A copy drifts silently when either function changes.

## Acceptance

- `a_consumer_mints_the_identity_the_interpreter_mints`: a public function of `ess-conformance` (or a crate it depends on) mints, for a declared identity type and prefix, the same value the interpreted target mints for the same inputs.
- `a_consumer_orders_view_rows_as_the_interpreter_does`: a public function orders view rows by a view's declared ordering exactly as the interpreted target answers them.
- `the_interpreter_calls_the_public_functions`: the interpreted target uses those functions itself, so there is one implementation of each.
