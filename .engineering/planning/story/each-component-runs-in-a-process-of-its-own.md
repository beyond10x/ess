---
format: aep.planning-md/3
id: story:each-component-runs-in-a-process-of-its-own
kind: story
status: draft
title: Each network component runs in an operating-system process of its own, and a binding crosses between them (carried from feat/792-process-per-component)
tags:
- external-request
- issues-2026-11-24
refs:
- provider: github
  reference: beyond10x/ess#792
relations:
- serves: vision:O2
revision: 1
---
## Need

beyond10x/ess#792. A system whose components must not share an operating-system process has no
generated shape: the `rust` target's workspace layout emits one `<component>-server` binary per
network component, but each assembles the whole system and links every component crate, and a
binding is delivered by the in-process pump. Two binaries started side by side are two copies of
the system; an event published in one never reaches the other. Reproduced on 0.96.0 with the
issue's two-component, one-binding model (`asker`, `noter`, binding `note-on-asked`).

## The code that exists

Branch `feat/792-process-per-component`, one commit on 0.96.0:

```console
$ git log --oneline 0.96.0..4c99c6e98
4c99c6e98 feat(synthesize): --layout processes runs each network component in its own process
$ git diff --stat 0.96.0 4c99c6e98 | tail -1
 33 files changed, 3791 insertions(+), 63 deletions(-)
```

What it holds, all carried except the first item:

- `.engineering/planning/story/each-component-runs-in-a-process-of-its-own.md` (draft, 104 lines):
  not carried as a file; the plan store is written only through `aep`, and this story replaces it.
- `crates/generate/ess-synth/src/rust/processes.rs` (new, 1,464 lines): the processes layout. One
  crate per network component, `crates/<component>-server`, whose manifest names its own
  component crate and the types crate only; its server is the workspace emitter's output for an IR
  that hosts that one component, rewritten by the single-crate layout's path rewrite. A binding
  whose event and command one process holds is delivered by its pump; one that crosses is carried
  to the acceptor at `--peer`, arriving on a second listener `--inbox`, run as the system acting
  with no grant check. `where:` is decided in the publisher's process. Identities and the clock
  come from a generated `src/host.rs` (`/dev/urandom`, `SystemTime`), so no process names a
  third-party crate. Options are parsed without clap. Escalation, selections, delivery contexts,
  bounded retry and `--http` are refused by name.
- `rust/entry.rs` (126 lines changed: the binary pieces shared), `rust/http.rs` (`carried_entry`,
  49 added), `rust/context.rs` (`MINT_UUID`, `NOW_UTC`), `rust/mod.rs`, `rust/system.rs` and
  `rust/single.rs` (visibility), `src/lib.rs` (`OutputLayout::Processes`, a `BindingRefusal`
  variant).
- `crates/specify/ess-compiler/src/ir.rs`: `EssIr::hosting` (26 added; public API).
- `crates/edge/ess-cli/src/main.rs`: `--layout processes` (the layout enum at 582-591 on the
  branch head gains `Processes`).
- Tests: `crates/generate/ess-synth/tests/process_layout.rs` (new binary, 14 tests, 1,118 lines)
  and `tests/fixtures/process-layout/` (a two-component model, a minting model and the
  five-component `page-load/`); one or two lines each in `external_request_binding.rs`,
  `feasibility.rs`, `served_codec_names.rs`, `served_publication.rs`.
- `Taskfile.yml`: `process-layout-servers` (builds the servers the tests start, because a test may
  not run cargo, `no_test_builds.rs`), its variable `PROCESS_LAYOUT_SERVERS`, and the task in the
  three harness lists; `AGENTS.md`: one sentence in the harness paragraph naming it.
- `docs/design/process-per-component-synthesis.md` (new, 226 lines), `website/docs/guides/synthesize.md`
  (45 lines), `website/docs/reference/cli.md` (1 line).

Its own record: "Not run for this commit: the whole workspace gate (task test), task ci-lint, and
the rest of ess-synth's and ess-cli's test sets after the last edits."

## Conflicts with the branch head

`git merge-tree --write-tree --name-only HEAD 4c99c6e98` on `issues/all-open-2026-11-24`
(`af420a374`, 319 commits after 0.96.0):

- `AGENTS.md`: CONFLICT (content). The branch rewrites the sentence listing the harness tasks
  (`task ui-http-servers` ... ) to add `task process-layout-servers`; the head changed the same
  paragraph's neighbourhood. Resolution: the head's text, with the branch's clause added to the
  list.
- `Taskfile.yml`: CONFLICT (content). The head added harness variables (`vars:` at 14-17) and
  harness tasks in the same three `cmds:` lists. Resolution: both: every head entry stays, and
  `PROCESS_LAYOUT_SERVERS`, `ESS_PROCESS_LAYOUT_SERVERS` and `- task: process-layout-servers` are
  added beside them; the `process-layout-servers` task is added whole.
- Auto-merged, to be read after the merge because the head changed them too: `ess-cli/src/main.rs`
  (head +623 lines since 0.96.0), `ess-synth/src/rust/http.rs` (+24), `rust/mod.rs` (+4),
  `ess-compiler/src/ir.rs` (+14), `tests/external_request_binding.rs`, `tests/feasibility.rs`,
  `website/docs/guides/synthesize.md`, `website/docs/reference/cli.md`.

## The carry, in order

One re-application, three commits on `issues/all-open-2026-11-24`:

1. `git cherry-pick -n 4c99c6e98`; remove the `.engineering/` file from the index and the tree;
   resolve `AGENTS.md` and `Taskfile.yml` as above; build `ess-synth`, `ess-compiler`, `ess-cli`
   and fix what the head's API changes broke, without changing behaviour. Commit through the bot:
   `feat(synthesize): --layout processes runs each network component in its own process (#792)`,
   the body naming the source commit `4c99c6e98` and the two resolved conflicts.
2. Move `tests/process_layout.rs` to `tests/process_layout/mod.rs` and declare it from the
   existing target `crates/generate/ess-synth/tests/synthesis.rs`
   (`#[path = "process_layout/mod.rs"] mod process_layout;`), so the crate gains no test binary
   (one integration-test binary per crate; no new binary). Fix the `AGENTS.md` sentence to name
   the host target. Commit: `test(synthesize): the processes layout's tests join an existing target (#792)`.
3. The Gate below, with the measured time of every test over 5 s; commit only what the gate moves
   (none expected).

## Acceptance

1. Red today: on the branch head, `ess generate synthesize --target rust --layout processes` is
   refused by the argument parser (`processes` is not a value of `--layout`); the moved test
   `the_plan_names_the_processes_layout` fails to compile or fails on the head without the carry.
2. `--layout processes` emits one crate per network component, `crates/<component>-server`, whose
   manifest names its own component crate and the types crate and no other component's crate; no
   crate assembles every component (`each_component_process_links_its_own_component_and_the_types_crate_only`,
   `no_crate_assembles_the_whole_system`).
3. A binding whose event one process publishes and whose command another accepts is carried to the
   acceptor: `at_most_once` makes one attempt; `at_least_once` attempts a lost or unmet invocation
   again; `on_failure:` answers a declared refusal as the in-process pump does; a failed delivery
   answers the publishing request `501` with `committed: true`
   (`two_processes_carry_a_binding_across_the_boundary_with_its_delivery_kept`); the five-process
   page load is carried end to end (`five_processes_carry_one_page_load`).
4. A binding with an end no process hosts, or a shape the layout does not carry yet, is refused by
   name before anything is written; `go`, `web` and `clap` refuse the layout; `--http` is refused
   with it (`a_binding_with_an_end_no_process_hosts_is_refused_by_name`,
   `every_other_target_refuses_the_processes_layout_by_name`).
5. The workspace and crate layouts' bytes do not move (`the_workspace_layout_is_unchanged`, and the
   byte pins under Gate unchanged).
6. The crate has no new test binary, and each of the 14 tests ends within 60 s, the target's step
   within 120 s; every wait in them gives up after 30 s (as on the branch).
7. `docs/design/process-per-component-synthesis.md`, the synthesize guide and the CLI reference
   describe the layout as built; the guide no longer says the workspace layout lets "one
   component be taken alone" without saying the processes layout is the shape that does.
8. The delivery-guarantees page says, for the process layout, in one sentence, that a lost answer
   under `at_most_once` is not routed into `on_failure:` (D16-2 A).

## Scope

As listed under "The code that exists", minus the `.engineering/` file, plus
`crates/generate/ess-synth/tests/synthesis.rs` (one `mod` line) and the moved test file.

## Owned files

- `crates/generate/ess-synth/src/rust/processes.rs`, `rust/entry.rs`, `rust/http.rs`,
  `rust/context.rs`, `rust/mod.rs`, `rust/system.rs`, `rust/single.rs`, `src/lib.rs`
- `crates/specify/ess-compiler/src/ir.rs` (`EssIr::hosting` only)
- `crates/edge/ess-cli/src/main.rs` (the layout enum only)
- `crates/generate/ess-synth/tests/process_layout/mod.rs` (moved), `tests/fixtures/process-layout/`,
  `tests/synthesis.rs` (one line), and the one-line edits of `external_request_binding.rs`,
  `feasibility.rs`, `served_codec_names.rs`, `served_publication.rs`
- `Taskfile.yml` (the task, its variable, three list entries), `AGENTS.md` (one sentence)
- `docs/design/process-per-component-synthesis.md`, `website/docs/guides/synthesize.md`,
  `website/docs/reference/cli.md`
- the delivery-guarantees page (one sentence, acceptance 8)
- `crates/verify/ess-conformance/tests/fixtures/synthesis-sources.digest` (`ess-compiler/src` is
  hashed by the stamp; rewritten by the command under Gate)
- `CHANGELOG.md` (one bullet)

## Not yours

Item 3 of the request, local IPC (Unix domain sockets for `--listen`, `--inbox` and `--peer`), is
`story:the-process-layout-s-listen-inbox-and-peer-take-a-unix-domain-socket-address-unix-path` (D16-1 C). The rest of increments 2 to 4 of the branch's design page (the
conformance suite across processes; escalation, selections, delivery contexts, bounded retry,
`--http`; the Go target) is not asked for by the issue and is not in this pull request.

## Gate

One at a time; no job in the build pool over 10 minutes (a build that needs longer names
`SLOT_TIMEOUT`, at most 30 minutes). Every test under 60 s, the test step under 120 s.

- `task fmt-check`
- `cargo clippy -p ess-synth --all-targets --locked -- -D warnings`; the same for `ess-compiler`
  and `ess-cli`
- `slot task process-layout-servers` (builds the servers; `ESS_PROCESS_LAYOUT_SERVERS` set for the
  next line)
- `cargo test -p ess-synth --locked --test synthesis` (hosts the 14 tests; time named)
- `cargo test -p ess-synth --locked --test external_request_binding`, `--test served_publication`,
  each alone
- byte pins, each alone (the table: a change to a generator): `cargo test -p ess-synth --locked
  --test served_codec_names`, `--test go`, `--test feasibility`, `--test
  adversary_related_guard_behaviour_pass1`, `--test http_binding_rust`
- `cargo test -p ess-compiler --locked` (the crate whose file changed; it is not one of the two
  crates that must name one target), and `cargo test -p ess-cli --locked --test tutorial_page`
- `task ci-lint` (`Taskfile.yml`, a public API and a CLI value changed; it runs `ci_lanes.rs`)
- `task test-xtask` (`docs/`, `website/` and `CHANGELOG.md` changed)
- the sources stamp, rewritten in this change:
  `SYNTHESIS_SOURCES_STAMP_WRITE="$PWD/crates/verify/ess-conformance/tests/fixtures/synthesis-sources.digest" cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table the_synthesis_sources_digest_matches_the_committed_stamp`;
  re-run after every rebase, never merge the stamp by hand
- conformance byte pins, each alone, none may move (`ess-compiler` is read by synthesis):
  `--test synthesis_suite_bytes_table`, `--test synthesis_suite_bytes_table_cli_read`,
  `--test adversary_related_values_all_bytes`, `--test external_beside_held_guard`

## Depends on

Nothing in the plan. Lands in ORDER's slot 16; touches no file another numbered step owns except
`Taskfile.yml`, `AGENTS.md` and `CHANGELOG.md` (serial in the one pull request).

## Changelog

Under `Added`:

- `ess generate synthesize --target rust --layout processes` runs each network component in an
  operating-system process of its own, and a binding between two of them crosses the boundary
  keeping `delivery:` and `on_failure:` (#792, first part).

## Fit review

The branch draft's fit review (seven questions with citations) is carried unchanged in substance:

1. Need: a component that is an isolation boundary runs without the code of the components it is
   isolated from, and a binding between two such components crosses the boundary keeping
   `delivery:` and `on_failure:`. The requester proposes no syntax.
2. Class: gap, with a documentation defect: the synthesize guide says the workspace layout exists
   "so one component can be taken alone", and its binaries do not.
3. Already expressible: no (`ess-runtime` places workloads but synthesis does not read it;
   `ess-transport` gives a publisher with no consumer side; the design page's table).
4. Fit: no authored surface; which components share a process is a deployment choice recorded
   where `--layout crate` records its choice (`plan.json` `scope.layout`). Every target emits the
   layout (Rust) or refuses it by name (Go, web, clap).
5. A second adopter: a payments system whose card-data component runs in its own sandboxed
   process, invoked only through a binding from the order component.
6. Cost: no format, no key, no diagnostic code; one `OutputLayout` value, one `BindingRefusal`
   variant, one CLI value, one public `EssIr::hosting`.
7. Considered: point at `ess-runtime` (does not change what a binary links); a fourth
   `reached_by` word (moves every target and the format for a deployment fact); a process-grouping
   key in the specification (deployment, like replicas); carrying events instead of invocations
   (needs a route no OpenAPI document declares).

## Decisions

- Accept, redesigned: items 1 and 2 of the request as a synthesis layout, not a specification
  construct; item 3 (local IPC) as the address form `unix:<path>` in its own story,
  `story:the-process-layout-s-listen-inbox-and-peer-take-a-unix-domain-socket-address-unix-path`.
- D16-1 (decided: C, 2026-11-24): #792 closes in the one pull request. This step carries increment 1
  as written; the story named above builds item 3 of the request; everything else of increments 2
  to 4 is not asked for by the issue and stays out of this pull request. The changelog bullet below
  drops "first part" once that story is in.
- D16-2 (decided: A, 2026-11-24): a lost answer under `at_most_once` is not routed into
  `on_failure:`, as built (the in-process reading); the delivery-guarantees page says so for the
  process layout (acceptance 8).
- D16-3 (open: security, with the owner as S14): a carried invocation runs as the system acting,
  with no grant check, and whoever reaches `--inbox` can run the carried commands.
  - A: as built, the inbox address left to deployment.
  - B: a declared service actor (a specification construct and a format).
  - This story is not accepted and not built until S14 is answered.
