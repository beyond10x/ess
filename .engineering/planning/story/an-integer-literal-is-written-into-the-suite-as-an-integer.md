---
format: aep.planning-md/3
id: story:an-integer-literal-is-written-into-the-suite-as-an-integer
kind: story
status: draft
title: An Integer literal is written into the suite as an integer, which the generated decoders accept
summary: Authored Integer literals become 1.0 in the suite; the generated Rust integer_at refuses 1.0.
tags:
- downstream
- ess-issues-3
revision: 1
---
## Problem

An authored literal for an `Integer` input is written into the suite as a float. The decoder
that ESS generates for the same field refuses it, so one ESS release's suite and its generated
contract disagree about how an integer is spelled. Reported 2026-10-29 by an adopter on ESS
0.75.0. Checked on the 0.77.0 head (84671d6, `ess 0.77.0`) against the adopter's model at a2a
d9e658f:

- `history_length: 1`, typed `Optional<Integer>`, is synthesized as
  `{"history_length":{"kind":"literal","value":1.0}, ...}`. The same happens to `page_size: 1`.
- The generated Rust `integer_at` (`crates/generate/ess-synth/src/rust/json.rs:731`) parses the
  number text as `i64` and refuses `1.0`: "expected an integer, found the number 1.0".
- The interpreted target accepts the literal. The adopter's server, built on the generated
  decoder, refuses it, and the adopter rewrites whole numbers before decoding.

## Acceptance

- A literal for an `Integer`-typed input, field or parameter is written into the suite as an
  integer (`1`, not `1.0`), for authored and synthesized scenarios alike.
- A suite step that sends such a literal decodes through the generated Rust and Go decoders.
