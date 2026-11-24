---
format: aep.planning-md/3
id: story:the-generated-servers-split-and-the-generated-apps-join-a-list-the-binding-declares-comma-joined
kind: story
status: draft
title: The generated servers split, and the generated apps join, a list the binding declares comma-joined
tags:
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:a-query-form-or-view-parameter-list-may-travel-as-one-comma-joined-value-validated-compiled-and-projected
revision: 2
---
## Need

After `ae12a-a-query-list-may-travel-comma-joined`, a binding may declare a list comma-joined. The
generated Rust and Go servers read a list as repeated keys
(`crates/generate/ess-synth/src/rust/http/bound.rs`, `go/http/bound.rs`). They would read
`?kinds=opened,closed` as one element, `"opened,closed"`, or refuse the request. The generated apps
(`crates/ui/ess-ui-react/templates/runtime/data.ts.tmpl`, `crates/ui/ess-ui-tui/src/http.rs`) would
send repeated keys the binding no longer states.

## Outcome

Both servers split a `comma` list on `,` after percent-decoding, read the empty value as the empty
list, and refuse a repeated key as `malformed`. Both apps join the elements with `,`
(percent-encoding each element). They refuse, by name, an element holding `,`, which the form
cannot carry.

## Acceptance

1. Red today: a fixture binding in `crates/generate/ess-synth/tests/http_binding/` declares
   `lists: {kinds: comma}`. The Rust and Go cases request `?kinds=opened,closed` and assert the view
   saw two elements. This fails.
2. After, both servers alike:
   - two elements for `?kinds=opened,closed`;
   - the empty list for `?kinds=`;
   - absent for no key;
   - `malformed` for `?kinds=a&kinds=b`.
3. Both apps write `kinds=opened,closed` for a two-element list (cases beside
   `crates/ui/ess-ui-react/tests/bare_body.rs` and `crates/ui/ess-ui-tui/tests/bare_body.rs`). A
   binding without `lists` writes today's bytes.
4. Every byte pin passes unmodified (`http_binding_rust`, `served_codec_names`, `go`).

## Scope

- inferred: `crates/generate/ess-synth/src/rust/http/bound.rs`, `go/http/bound.rs`
- cited: `crates/ui/ess-ui-react/templates/runtime/data.ts.tmpl`, `crates/ui/ess-ui-tui/src/http.rs`

## Owned files

The four files above, one fixture in `crates/generate/ess-synth/tests/http_binding/`, and cases in
the two `bare_body.rs` targets.

## Not yours

Validation and OpenAPI (`ae12a-a-query-list-may-travel-comma-joined`).

## Tests first

Acceptance 1 (Rust, then Go), then 3. Shared fixture build, built once per run.

## Gate

Each test stays under 60 s, the step under 120 s, and each job under 10 minutes.

```console
task fmt-check
cargo clippy -p ess-synth -p ess-ui-react -p ess-ui-tui --all-targets --locked -- -D warnings
cargo test -p ess-synth --locked --test http_binding_rust
cargo test -p ess-synth --locked --test served_codec_names
cargo test -p ess-synth --locked --test go
cargo test -p ess-ui-react --locked --test bare_body
cargo test -p ess-ui-tui --locked --test bare_body
```

## Depends on

`ae12a-a-query-list-may-travel-comma-joined`. Same server files as `ae1c`, `ae1e` and `ae3b`: one
after another.

## Changelog

- The generated servers read, and the generated apps write, a list declared comma-joined.

## Fit review

As `ae12a-a-query-list-may-travel-comma-joined`. Question 4, targets: every reader of the key
handles it.

## Decisions

**AE-D13**, default A.

Asked again by a ninth report, 2026-11-24; open with the owner.

## Size

M, about 300 lines with tests.
