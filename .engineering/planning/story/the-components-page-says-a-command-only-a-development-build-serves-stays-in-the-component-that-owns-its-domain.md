---
format: aep.planning-md/3
id: story:the-components-page-says-a-command-only-a-development-build-serves-stays-in-the-component-that-owns-its-domain
kind: story
status: active
title: The components page says a command only a development build serves stays in the component that owns its domain
tags:
- external-request
- issues-2026-11-24
refs:
- provider: github
  reference: beyond10x/ess#816
relations:
- serves: vision:O2
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-11-24T03:24:46Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-11-24T03:24:48Z", actor: "human:timo", revision: 3}
---
## Need

beyond10x/ess#816 item 6, decided NQ2 A (decline, with the idiom). A development-only command (a
local sign-in compiled into debug builds only) was moved to a component of its own and refused:
"`<component>` accepts `<command>`, and `<owner>` owns `<domain>`" (ESS-COMPONENT-004,
`conflicting_declaration`, `crates/specify/ess-domain/src/component.rs:1395-1407`; its hint: "§9
gives a command one handler, and it is the component owning its domain"). The reporter then dropped
the command, and the rows it creates became unreachable: 11 outcomes of another command fell to
ESS-SYNTH-003.

The specification describes what the system means, not which build serves what: the component
layer says "whether it is a process or a module" is not its business
(`website/docs/guides/specify/bindings-and-components.md:13-18`). A command a release build does
not serve is still a command of the owning component; which builds serve it is the
implementation's. The page does not say so.

## Reproduction

Reproduced on 0.98.0, both models under `run/` (generic names, `format: ess/36`):

- `run/devonly/system.yaml`: domain `devonly.access`, entity `Session` (`Active` to `Ended`),
  commands `LocalSignIn(user)` (creates a session; a comment says development builds only) and
  `SignOut(session_id)` (`unknown-session`, `signed-out`, `already-ended`); one component
  `access-service` owns the domain and accepts both.

  ```console
  $ ess specify validate --path devonly
  devonly v1 — 1 file(s), valid
  $ ess verify conform synthesize --path devonly --out devonly-suite.json
  5 scenario(s) (0 authored), 0 refusal(s), written to devonly-suite.json
  $ ess verify conform run --path devonly --suite devonly-suite.json --target interpreted --report-format 2
    5 scenarios: 5 passed, 0 failed, 0 error, 0 unsupported
  ```

  `SignOut`'s scenarios arrange their session through `LocalSignIn`.
- `run/devonly-split/system.yaml`: the same, with `LocalSignIn` accepted by a second component
  `dev-sign-in` that owns no domain:

  ```console
  $ ess specify validate --path devonly-split
    - error[ESS-COMPONENT-004]: `dev-sign-in` accepts `devonly.access.LocalSignIn`, and `access-service` owns `devonly.access`
  ```

## Outcome

`bindings-and-components.md` gains a subsection under "Three layers above the domains" (after the
paragraph ending at 36), `### A command only some builds serve`: the command stays in the
component that owns its domain; the specification does not say which build serves it; a release
build that leaves it out still serves the other commands, and the scenarios that arrange state
through it (as `SignOut`'s arrange a session) run against a build that serves it. It shows the
one-component model and names the refusal a second component gets (ESS-COMPONENT-004) and the
cost of dropping the command (its rows unreachable, ESS-SYNTH-003 for the commands that need them).

## Acceptance

1. Red today: a test asserts that `bindings-and-components.md` has a subsection `### A command only
   some builds serve` whose first fenced `yaml` model validates; it fails on the branch (no such
   subsection).
2. That model synthesizes with 0 refusals, the scenarios of the command that needs the development
   command's rows arrange them through it, and the suite passes on the interpreted target.
3. The same model with the development command accepted by a second component that owns no domain
   is refused as ESS-COMPONENT-004 (`conflicting_declaration`) at
   `component <second>.accepts.commands`.
4. The subsection names ESS-COMPONENT-004 and ESS-SYNTH-003, and says the build choice is the
   implementation's, in words the test finds.

## Scope

- `website/docs/guides/specify/bindings-and-components.md`: one `###` subsection after line 36.
- `crates/verify/ess-conformance/tests/binding_stored_read_idiom.rs`: new `#[test]`s only (the
  existing target that holds the binding guide's declined-with-idiom case; its page reader at
  17-30 reads one heading's fenced model, the pattern to copy for this page).

## Owned files

- `website/docs/guides/specify/bindings-and-components.md`
- `crates/verify/ess-conformance/tests/binding_stored_read_idiom.rs` (new tests only)
- `CHANGELOG.md` (one bullet)

## Not yours

- ESS-COMPONENT-004 itself (NQ2 B, not taken) and a build profile on commands (NQ2 C, not taken).
- `crates/specify/ess-domain/tests/binding_stored_read_idiom.rs`, which also reads this page
  (`BINDINGS`, line 12): it must stay green; it is not changed.

## Tests first

In `crates/verify/ess-conformance/tests/binding_stored_read_idiom.rs` (existing target):

- `components_page_keeps_a_development_only_command_in_its_owner` (acceptance 1, 4).
- `development_only_model_synthesizes_and_passes` (acceptance 2).
- `a_second_component_accepting_the_development_command_is_refused` (acceptance 3).

Each is one validation and at most one synthesis and interpreted run of a two-command model, well
under 1 s (bound 60 s per test, 120 s per test step). Time them and name any over 5 s.

## Gate

One at a time; no job in the build pool over 10 minutes.

- `task fmt-check`
- `cargo clippy -p ess-conformance --all-targets --locked -- -D warnings`
- `cargo test -p ess-conformance --locked --test binding_stored_read_idiom`
- `cargo test -p ess-domain --locked --test binding_stored_read_idiom` (reads the same page)
- `task test-xtask` (`website/` changed)
- the sources stamp, checked unchanged:
  `cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table the_synthesis_sources_digest_matches_the_committed_stamp`
- byte pins: none reachable (no synthesis source changes)
- `task site-build` where the network is available; otherwise CI's `pages.yml` lane

## Depends on

Nothing.

## Changelog

Under `Changed` (documentation):

- The components guide says a command only a development build serves stays in the component that
  owns its domain; which build serves it is the implementation's (#816).

## Fit review

1. Need: a command some builds of a component serve and others do not, kept in the model so the
   rows it creates stay reachable. The reporter's proposal, theirs: a component of its own for the
   command.
2. Class: local policy (which build serves a command is a build decision of one implementation),
   with a documentation gap.
3. Already expressible: yes, run above on 0.98.0; the suite checks every figure, against a build
   that serves the command.
4. Fit: no key. NQ2 B would reopen the one-handler rule (`component.rs:1404`); NQ2 C would add a
   build profile to commands and a suite selector, a source format for a build fact.
5. A second adopter: a test-data seeding command that only a staging build serves.
6. Cost: one page subsection and three tests.
7. Considered: NQ2 B, NQ2 C, the idiom (chosen, NQ2 A).

## Decisions

- Decision: decline, with the idiom (NQ2 A, taken on its default). The idiom is the page subsection
  and its tested model; reply to the requester with it.
