---
format: aep.planning-md/3
id: story:the-refusal-of-a-component-that-owns-accepts-and-publishes-nothing-names-the-actor-as-the-home-of-a-caller
kind: story
status: draft
title: The refusal of a component that owns, accepts and publishes nothing names the actor as the home of a caller that serves nothing
tags:
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
revision: 1
---
## Need

A system's design names callers that only call it: a dashboard that reads a stream, or an admin who
calls health routes. Neither serves anything. An author writes each as a component and gets:

- `EmptyDeclaration` at `component <name>`: "`<name>` owns nothing, accepts nothing and publishes
  nothing", with the hint "a component that does nothing is a name; delete it or give it a domain"
  (`crates/specify/ess-domain/src/component.rs:812-825`).

The hint offers two wrong ways out. Inventing a domain gives the caller ownership it does not have,
and deleting the caller loses it. The right home exists: a domain's `actors:` entry with `may:`
grants. The generated index graph draws actors and their grants ("a solid edge out of an actor is a
grant", the generated domain index of every model). One adopter found it by itself. A second one
reading the hint would not.

## Outcome

The hint names the idiom: "a caller that serves nothing is an actor: declare it under a domain's
`actors:` and grant it what it calls with `may:`; a component that does nothing else is a name, so
delete it or give it a domain". Same code, same site, same message.
`website/docs/guides/specify/bindings-and-components.md` § "Three layers above the domains" gets
one sentence saying the same.

## Acceptance

1. Red today: a unit test in `component.rs` validates a component `dashboard` with nothing and
   asserts that the hint names `actors:` and `may:`. This fails.
2. After: it passes. Code and message are unchanged (the existing `EmptyDeclaration` assertions in
   `component.rs` pass unmodified).
3. The guide sentence is present. `task test-xtask` passes (no diagnostic pin holds this hint:
   `git grep` finds it only at `component.rs:823`).

## Scope

- cited: `crates/specify/ess-domain/src/component.rs` (812-825)
- cited: `website/docs/guides/specify/bindings-and-components.md`

## Owned files

`crates/specify/ess-domain/src/component.rs` (the hint string and one unit test),
`website/docs/guides/specify/bindings-and-components.md`.

## Not yours

Everything else. The stream subscription of such an actor belongs to the server-sent events stories.

## Tests first

Acceptance 1.

## Gate

Each test stays under 60 s, the step under 120 s, and each job under 10 minutes.

```console
task fmt-check
cargo clippy -p ess-domain --all-targets --locked -- -D warnings
cargo test -p ess-domain --locked --lib component
cargo test -p ess-compiler --locked --test owner_of
cargo test -p ess-cli --locked --test validate_format_hint
task test-xtask
```

`ess-compiler`'s `owner_of` holds the message text (`git grep "owns nothing"`), and its message is
unchanged. `ess-cli` is gated because `ess-domain` changed (`AGENTS.md`, package-gate table).
`validate_format_hint` is the target that prints a refusal with its hint.

## Depends on

Nothing.

## Changelog

- The refusal of a component that owns, accepts and publishes nothing says that a caller which
  serves nothing is an actor with `may:` grants.

## Fit review

1. **Need.** A caller that serves nothing has a home in the model. The report modelled its two
   such callers as actors after meeting this refusal.
2. **Class: convenience.** Already expressible.
3. **Expressible today?** Yes: `actors:` with `may:`. Components are units of ownership
   (`component.rs:1-6`), and actors are who invokes. The index graph draws both.
4. **Fit.** No new surface, only a refusal text. It follows the story precedent of a hint that names
   the form ESS wants (`story:the-refusal-of-an-input-whose-identity-type-several-entities-share-on-a-branch-with-no-subject-names-the-forms`).
5. **Second adopter.** A mobile client that only calls a public API.
6. **Cost.** One string. No code, no format.
7. **Considered.** (a) Change nothing (AE-D10 B). (b) A component kind `caller` (AE-D10 C): a
   second home for "who calls", beside actors. (c) Chosen: the hint.

## Decisions

Decline the construct, with the idiom, and make the refusal point at it. **AE-D10**, default A.

## Size

S, about 40 lines.
