---
format: aep.planning-md/3
id: story:the-rust-crate-layout-is-rustfmt-and-clippy-clean
kind: story
status: draft
title: The Rust crate layout is rustfmt and clippy clean
summary: ess generate synthesize --target rust --layout crate writes 327 rustfmt hunks in 18 files and 47 clippy warnings (0.77.0)
tags:
- defect
- ess-batch-6
- ess-next
- severity-low
- zero-open-issues
relations:
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: inferred
  path: crates/generate/ess-synth/src/rust/behaviour.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/rust/http.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/rust/items.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/layout.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/mod.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/single.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/rust/wire.rs
- confidence: inferred
  path: crates/generate/ess-synth/tests/crate_layout_clean.rs
- confidence: inferred
  path: generated/rust/billing/crates/billing-system/src/lib.rs
- confidence: inferred
  path: generated/rust/billing/crates/billing-types/src/actor.rs
- confidence: inferred
  path: generated/rust/billing/crates/billing-types/src/behaviour.rs
- confidence: inferred
  path: generated/rust/billing/crates/billing-types/src/email.rs
- confidence: inferred
  path: generated/rust/billing/crates/billing-types/src/invoice.rs
- confidence: inferred
  path: generated/rust/billing/crates/billing-types/src/lib.rs
- confidence: inferred
  path: generated/rust/billing/crates/billing-types/src/obligation.rs
- confidence: inferred
  path: generated/rust/billing/crates/billing-types/src/primitives.rs
- confidence: inferred
  path: generated/rust/billing/crates/email-service/src/lib.rs
- confidence: inferred
  path: generated/rust/billing/crates/invoice-service/src/lib.rs
- confidence: inferred
  path: generated/rust/gatepass/crates/gatepass-server/src/bin/pass-service-server.rs
- confidence: inferred
  path: generated/rust/gatepass/crates/gatepass-server/src/entry.rs
- confidence: inferred
  path: generated/rust/gatepass/crates/gatepass-server/src/http.rs
- confidence: inferred
  path: generated/rust/gatepass/crates/gatepass-server/src/json.rs
- confidence: inferred
  path: generated/rust/gatepass/crates/gatepass-server/src/lib.rs
- confidence: inferred
  path: generated/rust/gatepass/crates/gatepass-server/src/memory.rs
- confidence: inferred
  path: generated/rust/gatepass/crates/gatepass-server/src/pass_service.rs
- confidence: inferred
  path: generated/rust/gatepass/crates/gatepass-server/src/static_assets.rs
- confidence: inferred
  path: generated/rust/gatepass/crates/gatepass-server/src/wire.rs
- confidence: inferred
  path: generated/rust/gatepass/crates/gatepass-system/src/lib.rs
- confidence: inferred
  path: generated/rust/gatepass/crates/gatepass-types/src/actor.rs
- confidence: inferred
  path: generated/rust/gatepass/crates/gatepass-types/src/behaviour.rs
- confidence: inferred
  path: generated/rust/gatepass/crates/gatepass-types/src/lib.rs
- confidence: inferred
  path: generated/rust/gatepass/crates/gatepass-types/src/obligation.rs
- confidence: inferred
  path: generated/rust/gatepass/crates/gatepass-types/src/primitives.rs
- confidence: inferred
  path: generated/rust/gatepass/crates/gatepass-types/src/visit.rs
- confidence: inferred
  path: generated/rust/gatepass/crates/pass-service/src/lib.rs
revision: 12
---
## Outcome

An adopter who formats and lints its Rust workspace can keep the crate that
`ess generate synthesize --target rust --layout crate` writes inside it, without excluding or
rewriting it.

## Problem

Reported 2026-10-29 on ESS 0.77.0 by a downstream session: the crate-layout output is not
rustfmt-clean (327 hunks in 18 files) and has 47 clippy warnings.

Reproduced on 0.92.0 for the formatting half, with the repository's own examples:

```
$ ess generate synthesize --target rust --layout crate --path examples/gatepass --out crate-gatepass
29 capabilities: 28 generated, 1 obligation(s), 0 refused
$ rustfmt --check --edition 2021 crate-gatepass/src/lib.rs crate-gatepass/src/bin/pass-service-server.rs
118 hunks in 16 of 18 files (exit 1)
$ ess generate synthesize --target rust --layout crate --path examples/billing --out crate-billing
$ rustfmt --check --edition 2021 crate-billing/src/lib.rs
57 hunks in 9 of 11 files (exit 1)
```

The clippy half is not re-counted here (it needs a cargo build of the generated crate).

The default workspace layout is not clean either: `rustfmt --check --edition 2021` exits 1 on each
of the eight committed `generated/rust/*/crates/*/src/lib.rs` (0.92.0 tree). Both layouts come from
one emitter: `crates/generate/ess-synth/src/rust/mod.rs` builds either through
`feasibility::checked_shaped(…, single_crate)` and `Layout::shaped` in `layout.rs`.

## Acceptance

The two named tests are in a new target, run with
`cargo test -p ess-synth --locked --test crate_layout_clean`.

- `the_crate_layout_is_rustfmt_clean`: for every example under `examples/` that synthesizes for
  Rust, `ess generate synthesize --target rust --layout crate` output passes
  `rustfmt --check --edition 2021` on each crate root, exit 0.
- `the_crate_layout_is_clippy_clean`: the same output passes
  `cargo clippy --all-features -- -D warnings`, exit 0. A lint the generator cannot meet is allowed
  by name at the crate root, each with its reason in the generator source.
- The committed workspace layout is clean too: `rustfmt --check --edition 2021` exits 0 on every
  `.rs` file under `generated/rust/`, and the CI comparison of `generated/rust/` with what the
  specifications determine passes on the regenerated tree.
- `CHANGELOG.md` names the byte change of both layouts' generated Rust under D135 (the coordinator
  writes the sentence the implementor proposes).

## Decisions

D135, decided: both layouts' default bytes change, with a release notice, and there is no opt-in.
The emitter is shared by both layouts (`mod.rs`, `single_crate`), the change moves layout only and
no behaviour, and an opt-in would keep the defect as every adopter's default. The workspace layout
is therefore in scope, by the condition this section set before ("unless the same emitter is
shared").

## Scope

This story moves the `CHANGELOG.md` shared surface and re-pins any byte-pin row over generated
Rust.

- cited: crates/generate/ess-synth/src/rust/single.rs
- cited: crates/generate/ess-synth/src/rust/mod.rs
- cited: crates/generate/ess-synth/src/rust/layout.rs
- inferred: crates/generate/ess-synth/src/rust/behaviour.rs
- inferred: crates/generate/ess-synth/src/rust/http.rs
- inferred: crates/generate/ess-synth/src/rust/wire.rs
- inferred: crates/generate/ess-synth/src/rust/items.rs
- inferred: crates/generate/ess-synth/tests/crate_layout_clean.rs
- inferred: generated/rust/billing/crates/billing-system/src/lib.rs
- inferred: generated/rust/billing/crates/billing-types/src/actor.rs
- inferred: generated/rust/billing/crates/billing-types/src/behaviour.rs
- inferred: generated/rust/billing/crates/billing-types/src/email.rs
- inferred: generated/rust/billing/crates/billing-types/src/invoice.rs
- inferred: generated/rust/billing/crates/billing-types/src/lib.rs
- inferred: generated/rust/billing/crates/billing-types/src/obligation.rs
- inferred: generated/rust/billing/crates/billing-types/src/primitives.rs
- inferred: generated/rust/billing/crates/email-service/src/lib.rs
- inferred: generated/rust/billing/crates/invoice-service/src/lib.rs
- inferred: generated/rust/gatepass/crates/gatepass-server/src/bin/pass-service-server.rs
- inferred: generated/rust/gatepass/crates/gatepass-server/src/entry.rs
- inferred: generated/rust/gatepass/crates/gatepass-server/src/http.rs
- inferred: generated/rust/gatepass/crates/gatepass-server/src/json.rs
- inferred: generated/rust/gatepass/crates/gatepass-server/src/lib.rs
- inferred: generated/rust/gatepass/crates/gatepass-server/src/memory.rs
- inferred: generated/rust/gatepass/crates/gatepass-server/src/pass_service.rs
- inferred: generated/rust/gatepass/crates/gatepass-server/src/static_assets.rs
- inferred: generated/rust/gatepass/crates/gatepass-server/src/wire.rs
- inferred: generated/rust/gatepass/crates/gatepass-system/src/lib.rs
- inferred: generated/rust/gatepass/crates/gatepass-types/src/actor.rs
- inferred: generated/rust/gatepass/crates/gatepass-types/src/behaviour.rs
- inferred: generated/rust/gatepass/crates/gatepass-types/src/lib.rs
- inferred: generated/rust/gatepass/crates/gatepass-types/src/obligation.rs
- inferred: generated/rust/gatepass/crates/gatepass-types/src/primitives.rs
- inferred: generated/rust/gatepass/crates/gatepass-types/src/visit.rs
- inferred: generated/rust/gatepass/crates/pass-service/src/lib.rs

confidence: low

Every emitter writes text by hand, so the hunks are spread over every module; formatting the
emitted text once, before it is written, is the likely single point, and a decision for the
implementor.
