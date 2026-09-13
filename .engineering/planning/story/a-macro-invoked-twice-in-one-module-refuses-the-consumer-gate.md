---
format: aep.planning-md/1
id: story:a-macro-invoked-twice-in-one-module-refuses-the-consumer-gate
kind: story
status: active
title: A macro invoked twice in one module refuses the consumer gate
relations:
- serves: vision:O2
scope:
- confidence: cited
  path: crates/edge/ess-xtask/src/consumer_coverage/consumer.rs
- confidence: cited
  path: crates/edge/ess-xtask/src/consumer_coverage/entry-classifications.json
- confidence: cited
  path: crates/edge/ess-xtask/src/consumer_coverage/tests.rs
revision: 6
---
# A macro invoked twice in one module refuses the consumer gate

`task consumer-check` fails on `main`, and has since before wave 22:

```
extraction/classification refused: duplicate concrete consumer entry
ess_domain::lib(ess_domain)::binding::periodic::macro::invocation/profile_word
status: PROVISIONAL_ACCOUNTING_REFUSAL
exit status 1
```

Measured at two commits with the identical message: `bd722fa9` (wave 24's base) and `ab2bb7f1`
(wave 24's integration branch). The wave's diff touches no file under `binding/` and no
consumer-coverage input, and contains 0 occurrences of `profile_word`.

## Cause

`crates/edge/ess-xtask/src/consumer_coverage/consumer.rs:317-329`, `item_macro`, keys an unnamed
macro invocation as `<owner>::macro::invocation/<macro path><anchor>`. The `anchor` is the only thing
distinguishing two invocations of the same macro in one module, and it is computed for exactly one
hard-coded macro name:

```rust
let anchor = if matches!(
    text(&m.mac.path).as_str(),
    "checked_deserialize" | "crate :: validation :: checked_deserialize"
) {
    …
    format!("/for/{}", invocation.0)
} else {
    String::new()
};
```

So **any production macro invoked more than once in one module collides**, and `consumer.rs:572`
bails. `profile_word!` is defined at `crates/specify/ess-domain/src/binding/periodic.rs:72` and
invoked from `:92` onward, eight or more times.

`crates/edge/ess-xtask/src/consumer_coverage/tests.rs:283` pins the `checked_deserialize` shape —
`fixture::macro::invocation/checked_deserialize/for/First` — so the discriminator is tested for the
one macro that has one, and for no other.

## How it reached `main`

`binding/periodic.rs` arrived in `9572af9b`, "close the thirteen measured ESS specification and
conformance gaps". Wave 23's gate has no `consumer-check` line in its recorded exits; wave 22's
`task test` exited 201. A lane that is not run at close is how the `layout` regression reached `main`
too.

## Acceptance

Two invocations of the same macro in one module produce two distinct consumer entries, with no
macro named in the extractor's source. `task consumer-check` exits 0 on `main`.

The discriminator has to come from the invocation itself rather than a table. Whoever takes this says
what it is — a token, a span, a positional index — and what it costs: entry ids change, so
`entry-classifications.json` needs the new ids, and a positional index makes the id move when a
sibling invocation is inserted above it.

## Scope

- `crates/edge/ess-xtask/src/consumer_coverage/consumer.rs` — `cited`
- `crates/edge/ess-xtask/src/consumer_coverage/entry-classifications.json` — `cited`
- `crates/edge/ess-xtask/src/consumer_coverage/tests.rs` — `cited`
