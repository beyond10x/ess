# AGENTS.md — ess

The contract for changing this repository. Organization-wide rules live in `atlas/AGENTS.md`.

Here to **use** ESS in another repository rather than change this one? Install the `ess` plugin from
[`beyond10x/agentplugins`](https://github.com/beyond10x/agentplugins) and start with `/ess:init`. The
rest of this file does not apply to you.

## Serves

- **O2 — decisions as data, with evidence.** ESS makes system structure, semantics, projections,
  conformance requirements, and infrastructure intent executable as deterministic typed data.

## Boundaries

- ESS has no AEP dependency. Workflow evidence adaptation belongs on the AEP side of the seam.
- `EssIr` and `InfraIr` remain separate until a concrete comparison requires a shared envelope.
- Do not introduce a generic facet registry, arbitrary JSON property bags, or an `ess-ir/2` plan.
- Add concrete Rust types only when an importer or projector establishes their required semantics.
- Imports never guess. Projections never apply infrastructure or mutate an external system.
- Each adapter declares supported directions and reports coverage gaps, obligations, unresolved
  references, and refusals.
- Anything executable is Rust. Do not add Python or shell checkers.

## Crate tree

`crates/<area>/<crate>/`. The area is the directory only; a crate's name is its identity and is
never changed to follow a move.

- **`crates/specify/`** — `ess-yaml`, `ess-primitives`, `ess-domain`, `ess-compiler`,
  `ess-composition`, `ess-realization`, `ess-http`: an authored system becomes a validated,
  resolved IR. `ess-yaml` is a leaf with no ESS dependency and the only crate that names a YAML
  library; every other crate reads and writes YAML through it, and a crate depends on neither
  `serde_yaml` nor `unsafe-libyaml` (`crates/edge/ess-xtask/tests/layout.rs`,
  `docs/design/yaml-parser-replacement.md`).
- **`crates/generate/`** — `ess-gen`, `ess-synth`, `ess-openapi`, `schema-contract`,
  `ess-deployment`: that IR becomes artifacts, and nothing here applies one to a running system.
- **`crates/verify/`** — `ess-conformance`, `ess-diff`: an implementation held to the
  specification, and one revision of a specification against another.
- **`crates/infra/`** — `infra-domain`, `infra-compiler`, `infra-analyze`, `infra-spec`,
  `infra-project`, `ess-kubernetes`: the observed cluster, a separate bounded context whose only
  dependency on the rest is `ess-primitives`. It reads YAML through `ess_primitives::yaml`, which
  re-exports `ess-yaml`.
- **`crates/ui/`** — `ess-ui`: a renderer-neutral UI document (`ess-ui/1`, schema at
  `schemas/ui/ess-ui.schema.yaml`) loaded into typed Rust, with no dependency on the rest except
  the leaf `ess-yaml`.
- **`crates/edge/`** — `ess-cli`, `ess-xtask`: the `ess` binary an adopter runs, and this
  repository's own tooling.

`ess-deployment` sits under `generate/` rather than `infra/` because its dependencies are
`ess-compiler` and `ess-realization` and it has no `infra-*` dependency at all — an environment
lowered to a deployment document is a projection of the specification, not an observation of a
cluster.

## Determinism and formats

- Ordered collections only in persisted or generated data.
- Compiler-minted handles must have total lookup functions.
- Preserve canonical bytes unless a coordinated format migration explicitly changes them.
- A new format version is required when meaning, identity, references, canonicalization, names, or
  the persisted envelope changes. Internal Rust capabilities alone do not require one.
- `infra-ir/1` rejects unknown fields. Before adding any persisted field, add an old-reader
  compatibility test and decide the format consequence explicitly.

## Kubernetes credential edge

- Raw Secret `data`, `stringData`, and last-applied configuration values never reach serialized
  bytes or disk. Preserve the redaction tests and verify the guard by mutation.
- Live-cluster tests require explicit credentials and remain outside the offline gate.
- The libraries downstream of `ess-kubernetes` never select kubeconfig authority or reach a cluster.

## Gate

```console
task check
```

CI runs `task check` as a few jobs, each running Taskfile tasks as steps. `checks` runs `ci-lint`,
`ci-smoke`, `test-feature-off-doc`, `test-xtask` and `fuzz-check` one after another after one
setup. Every test binary is compiled once, into two nextest archives built side by side:
`build-tests` runs `test-archive-workspace` and `build-feature-off` runs
`test-archive-feature-off` (local `task test-archive` runs both). `test (<m>/4)` downloads both and
runs `test-shard` and `test-feature-off` for partition `slice:m/4`, compiling nothing; `slice:`
deals the whole test list out, where `count:` dealt each binary's out from shard 1 and left shard
4 with half the work of shard 1. The shards extract into
the checkout's own `target/`, because test binaries bake `CARGO_BIN_EXE_*` and
`CARGO_TARGET_TMPDIR` in at build time. The `Gate` job carries their joint result, and
`crates/edge/ess-xtask/tests/ci_lanes.rs` fails when a step of `check` is in no job, when the
shards leave a partition unrun, or when `Gate` misses a job. Local `task test` still runs
`cargo test` over everything.

**A pull request from an ordinary branch runs the feature-off build on number semantics only.**
With `FEATURE_OFF` set to `number-semantics` (a `pull_request` run whose head is not a `queue/`
branch of this repository, and a `merge_group` run), the feature-off archive keeps every feature-off package except
ess-cli, plus ess-cli's unit tests and its `binary64*`, `normalization*`, `schema*` and
`execution_recovery` binaries (`FEATURE_OFF_NUMBER_SEMANTICS` in `Taskfile.yml`). The rest of
ess-cli's feature-off run — every other ess-cli integration binary compiled without
`arbitrary_precision` — runs on the `main` push, the nightly `schedule` run, the release gate and
the pull-request run of a `queue/**` head, which archive all of it. `ci_lanes.rs` fails if an
ess-cli test binary whose source names `arbitrary_precision` falls outside the filter.

**A queue head runs CI once, as its pull request, and that run is full.** `ci.yml` does not run
on a `queue/**` push (`package.yml` still does): on 2026-10-19 one queue push started two 27-job
runs of the same commit (push 39285366577 and pull_request 39285374142) against the plan's 20
concurrent jobs, and shards waited 1,500–3,900 s. The pull-request run is the one the ruleset's
required `Gate` reads and the one whose compile cache `rust-cache` saves, so it is the one kept,
and it archives every feature-off case because the release tag reuses it (below).

Test builds carry line tables only (`[profile.dev] debug`), and `sha2`, `flate2` and `tar` are
optimised in them. CI's Linux jobs link with lld through
`CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS` in `ci.yml`, not `.cargo/config.toml`, so local
builds and shipped release binaries keep the system linker.

**Local compile cache.** On the shared development host, local ESS builds use a compile cache of
their own, because a fresh worktree otherwise recompiles every dependency. Set it per command, for
this repository's builds only: `RUSTC_WRAPPER=sccache SCCACHE_DIR=~/.cache/sccache/ess
SCCACHE_SERVER_PORT=4303 SCCACHE_CACHE_SIZE=2G`. Never the default port 4226, never a server shared
with another repository, and never in `.cargo/config.toml` or a global cargo configuration: CI and
release builds run without it.

**Consumer coverage is opt-in.** Revision 3 of the ESS evolution plan (`ESS-EVOLUTION.md`, plan
ess-evolution-20260915, 2026-09-25) parked feature-preservation accounting: `task check` runs
`consumer-check` only with `CONSUMER_CHECKS=true`, no CI lane runs it, and it is not part of the
release bar. `task consumer-check` still runs it on its own. The code,
`initial-baseline.json`, the classifications and the rebound reviewed cases stay in the tree. The
retired `SKIP_CONSUMER_CHECKS` switch means nothing; `ci_lanes.rs` refuses it in every gate file.
Re-enable the lane when a named external adopter pins a released `ess/N` and its generated
artifacts, and a change reaches that adopter undetected by conformance, `ess verify diff` or a
format refusal.

The gate is offline and runs formatting, strict Clippy, all workspace tests, rustdoc, command smoke
tests, and the dependency boundary test. Land nothing on `main` until the `Gate` job is green.
`ESS_TUTORIAL_NETWORK=1` (set on CI's workspace test shards) additionally runs the Start-here
pages' `npm` steps in `tutorial_page.rs`, which need the npm registry; unset, they print a skip. Its
`go` steps need no network and run wherever `go` is on `PATH`.

For a pull request, CI owns the full gate. Before pushing, run only the crates the change touches
— `task fmt-check` (not `cargo fmt --all --check`, which also rewrites the byte-pinned generated crates), `cargo clippy -p <crate> --all-targets --locked -- -D warnings`,
`cargo test -p <crate> --locked`, and `task ci-lint` when a workflow, Taskfile or public API
changed — then push and read the lanes. Run the whole `task check` locally only to reproduce a
lane that failed in CI.

**The package-gate subset.** The crates to gate are named by the change, not guessed:

| the change touches | gate these crates (`clippy -p` and `test -p`) |
|---|---|
| a file under `crates/<area>/<crate>/` | that crate |
| `ess-domain`, `ess-compiler` or `ess-primitives` | also `ess-cli`, whose integration tests pin validation and compile output |
| synthesis, mutation or interpretation in `ess-conformance` | also `ess-cli`; run `ess-conformance` one `--test` at a time, its full test build is the largest in the workspace |
| a `ValidationCode`, a diagnostic, a format version, `CHANGELOG.md`, `website/` or `docs/` | also `task test-xtask` (see the pins paragraph below) |
| `RawSpecFile` | also `cargo xtask schema`, then `task projection-check` |
| a workflow, `Taskfile.yml` or a public API | also `task ci-lint` |

Every push also runs `task fmt-check`. A crate the change does not touch is not gated locally; CI
covers it.

The adopter-facing Docusaurus source lives under `website/`; repository-root `docs/` remains the
engineering record and is never published directly. A documentation, release, or validation
workflow change must additionally pass:

```console
task site-build
```

This check is separate because `npm ci` fetches the exact public `docs-system` Git revision and
therefore cannot be part of the offline gate. `.github/workflows/pages.yml` preserves the same
Rust/WASM realization and site-build checks without Pages authority; the unified Website publishes
the collected source and the Atlas-generated façade owns the project redirect.

The landing page is `website/product.json`, rendered by the docs-system product-site template
(`withProductSite`). Its terminal sessions and domain graph under `website/data/` are recorded from
the real `ess` binary on `examples/billing` by `cargo xtask site-data`; never edit them by hand.
`projection-check` runs `cargo xtask site-data --check`, so a change to what `ess` prints for the
example, or to the example itself, needs `cargo xtask site-data` in the same change.

The visualise page (`website/docs/visualise.md`, outside the documentation collection, so its MDX
imports are allowed) renders `<EssPresentation>` from `@beyond10x/ess-ui-templates`, pinned by
commit in `website/package.json`. Its `website/data/billing.presentation.json` is written by that
same commit's `ess-ui data` from this `ess`'s compile, synthesize and author output, as part of
`cargo xtask site-data`. `ess-ui` depends on ESS crates, so it is never a Cargo dependency here:
`cargo xtask ess-ui` installs it once per pin under `$ESS_UI_ROOT` or `~/.cache/ess-ui` (network),
CI caches that directory, and `site-data` installs it on first use when missing. Moving the pin is
a one-line `package.json` change, `npm --prefix website install`, and `cargo xtask site-data`. A
refusal saying the IR is not the compiled model means `ess-ui`'s pinned ESS release and this
workspace compile the example differently: move the `ess-ui` pin to a commit built on this release.

**The release gate is a full CI gate on the tagged tree.** `release.yml` runs the reusable
`ci.yml` gate on the tagged commit in full unless every check the ruleset requires already passed
on that tree in a qualifying run (below); either run is the full pre-release suite. Run no local pre-release suite and no local
full gate before a release: each unit passes its package gates, the queue pull request's CI gates
the merge into `main`, and that run or the tag run gates the release. A wave holds at most 5
stories and ends in one release.

`task test-browser-product` and `task test-slow-probes` run in no CI lane (those Firefox tests and
slow probes are ignored in the CI test shards); run them locally when a change touches what they
cover. To reproduce the whole gate locally, mind that `task test` alone builds about 45 GiB into
the tree's `target/` (44.8 GiB measured on 2026-10-07 at 0.56.0), so start it only with that much
free disk beyond what other work on the machine needs, and watch free space during the step rather
than only between steps.
Set `ESS_TYPES_NODE` to a `node_modules` outside the checkout that holds `@types/node` at the version
`ci.yml` installs (`npm install --prefix <dir> @types/node@26.3.0`): without it the TypeScript cases
in `adversary_287_pass1` fail with `TS2688: Cannot find type definition file for 'node'` and
`cargo test` stops at that target, so the binaries after it never run.
Put the JavaScript tools `ci.yml` installs globally on `PATH` too (`typescript@6.0.3`,
`prettier@3.8.1`, `esbuild@0.28.2`, for example `npm install --prefix <dir>` and `<dir>/node_modules/.bin`
first on `PATH`): without `esbuild` the generated React app's `npm run dev` and `npm run preview`
cases in `ess-ui-react` exit 127 and `cargo test` stops there.
Consumer coverage is not part of that bar while it is parked (revision 3, above).
The release workflow runs the reusable gate, the WebAssembly realization check and native
packaging concurrently at that exact commit, then retains a read-only preparation artifact only
after all succeed. Its `prior-gate` step skips the gate only when every check the ruleset
requires — `Gate`, `Output ownership (macos-15)` and `Output ownership (macos-15-intel)` — already
passed on the tagged bytes in a qualifying run. Of a commit's check-runs of one name (GitHub
Actions' own, newest first, at most one page of 100), the first whose workflow run qualifies
decides, and only a completed success passes; at most the newest ten are looked up. A run
qualifies when it is this repository's `ci.yml` at that commit (`head_repository`, `head_sha`) and
archived every feature-off case: a `push` or `schedule` run of `main`, or a `pull_request` run
whose head branch is `queue/…` (a pull request from any other head or from a fork, and a
`merge_group` run, are narrowed, above, and never count). The gate is skipped when **either** of
these holds:

1. the tagged commit itself passed every required check, normally in the `main` push run; or
2. all of the following hold:
   - `GET /repos/{owner}/{repo}/commits/{sha}/pulls` returns exactly one merged pull request of
     this repository (`head.repo`) into `main` whose `merge_commit_sha` is the tagged commit, and
     that pull request's head is the tagged commit's second parent;
   - `git rev-parse <sha>^{tree}` equals `<head>^{tree}` (the merge into `main` is a new commit,
     so trees are compared, not commits);
   - the tagged commit's first parent (`main` before the merge) is an ancestor of that head;
   - that head passed every required check in a qualifying run, normally its queue pull-request
     run (a `main` push or nightly run at that commit would count too).

Anything else runs the gate: a backfill (`workflow_dispatch`), an API failure, a squash, a merge
whose base moved, a merge whose tree is not its head's, a fork's pull request or run. The tree
equality makes the tested bytes the tagged bytes. The ancestry makes the head's run a test of
those bytes: a `pull_request` run tests the head merged into the base of its day, and under the
ruleset's strict status-check policy that is the head's own tree. So tag the merge commit as soon
as it lands; there is no need to wait for `main`'s run. A call from `release.yml` passes
`checkout-ref`, which `ci.yml` never narrows, so a backfill dispatched from any branch runs it in
full. The macOS output-ownership witnesses run in a tag's gate and in every pull-request run, and
a tag that reuses a commit's checks requires both witnesses green there beside `Gate`; only a
backfill, which may gate a commit older than their test target, skips them. `ci_lanes.rs`
(`a_release_runs_its_full_gate_unless_a_full_gate_passed_the_tagged_tree`,
`a_release_reuses_a_full_gate_on_the_tagged_tree_and_otherwise_runs_it`) holds both workflows to
that and runs the step against real Git histories.

The four archives are usually built before the tag exists. `.github/workflows/package.yml` runs
on every `queue/**` push and packages the commit when its workspace version has a dated changelog
section and no tag yet. That commit is the queue head, never the merge commit a tag names, so the
release's `prebuilt` step is keyed on the head `prior-gate` resolved (rule 2's head, reported
whenever its tree is the tagged tree; keyed on the tagged commit, 0.61.0 and 0.62.0 both logged
"no successful queue package run built …" and built again). It uses that run's archives only when
all of these hold: a head was resolved, it is a successful `package.yml` push run of a `queue/`
branch at that head, and it still holds all four `release-<target>` artifacts unexpired. Anything
else, including an API failure, calls `package.yml` from the release and builds them there. Either
way the preparation job checks the four names and `SHA256SUMS` and runs the Linux x86_64 binary's
`--version` against the tag. `ci_lanes.rs` runs the `prebuilt` step against canned API answers.

Compile caches are saved only by `main` and pull-request runs (`save-if` on every
`Swatinem/rust-cache` step, held by `ci_lanes.rs`). A queue branch or a tag saves under a scope
that no later run can restore. A manual dispatch saves none: a release backfill dispatched from
`main` builds an old tag's source under `main`'s scope.

The Intel macOS archive is cross-compiled on the Apple Silicon `macos-15` runner and smoke-run
there under Rosetta; `lipo -archs` names the architecture that was built. Archive names and
`SHA256SUMS` are unchanged. Site rendering remains a documentation-validation gate; ordinary
source releases do not wait for it.

Public CI has read-only GitHub permissions and never creates or edits a Release. A successful
`Release preparation` run uploads one artifact named `ess-release-<tag>-<commit>`, containing the
four archives under `dist/`, `dist/SHA256SUMS` and `notes.md`. From a trusted operator environment,
the coordinator reads the successful run and its exact head commit, resolves the annotated tag to
that same commit, and downloads only that exact artifact through
`b10x-gates gh --repo . --repository beyond10x/ess -- ...`. Before publication, verify that the
artifact contains exactly the four target archives, checksums and notes, run
`sha256sum --check SHA256SUMS`, and smoke-run the Linux x86_64 archive's `ess --version` against the
tag. Through the same bot-authenticated wrapper, create or reconcile a draft Release on that exact
tag, upload the four archives and `SHA256SUMS`, verify every remote asset, then make the draft
public. Never use raw `gh` for these writes and never place the App credential in this repository or
its workflows.

Release completion means the exact tag, required release checks, published GitHub Release and
required assets are verified. A pushed tag awaiting those checks is queued. Atlas observes release
facts and publishes documentation asynchronously: do not wait for Website or Atlas, update Website
locks or snapshots, promote consumer pins, release the shared docs runtime or redeploy façades as
part of releasing ESS. Report documentation as pending unless publication was verified. A failure
in that background work does not undo a successful ESS release.

After cutting a release, and after any release preparation that fails:

```console
task release-status
```

It asks the remote and GitHub whether every pushed version tag is on `origin/main` and has a
release behind it, and fails while one is not. `.github/workflows/release-record.yml` runs it after
the bot publishes, after failed preparation, and daily. A successful preparation waits for bot
publication instead of immediately claiming that the final release record failed.

**A change to `RawSpecFile` is not finished until `cargo xtask schema` has run.**
`schemas/generated/ess.schema.json` is a projection of that type like any other, and
`projection-check` is the only thing that says so. `{cleared: true}` added one key to
`ExplicitPayloadSource`, the projection was not regenerated, and the published schema went on
refusing the key the model had just gained — the direction the task's own comment warns about,
because nothing downstream complains about a schema that is merely too strict. It failed the Gate
on PR 40 and nowhere else: `cargo fmt`, Clippy and every test were green.

**A version bump touches TWO lock files.** `fuzz/` is its own workspace with its own
`Cargo.lock`, and it pins the specification crates by path. The 0.26.0 bump moved the workspace
version and every crate manifest and left `fuzz/Cargo.lock` at `0.25.0` for five crates —
`ess-compiler`, `ess-domain`, `ess-gen`, `ess-primitives`, `ess-synth`. Only `fuzz-check` can see
it, because only it passes `--locked`, and the failure it prints is about the lock file rather than
about a version:

```console
cargo update --manifest-path fuzz/Cargo.toml --offline --workspace
```

Same shape as the schema projection above: a bump leaves a derived artifact behind, nothing
downstream complains, and one task in the gate is the only thing that knows.

**A new `ValidationCode` or diagnostic moves pins outside the crate that declares it.** Run
`task test-xtask` and `cargo test -p ess-compiler --test typed_diagnostics` before pushing such a
change, even when no file under `ess-xtask` or `ess-compiler` changed.
`crates/edge/ess-xtask/src/consumer_coverage/macro-guards.json` pins the digest of the
`validation_codes!` invocation, and `docs/design/review-typed-diagnostics.md` counts every
string-located `ValidationError` per file. Adding one refusal to `ess-domain/src/wire.rs` failed
both on CI (PR 502), while every package-scoped run of the touched crates was green. A test module
in `ess-cli/src/git_checkout.rs` is compiled into `ess-xtask` through `#[path]` too, so it cannot
use `ess_cli::…`.

## Agent plugin

The ESS agent plugin lives in `beyond10x/agentplugins` (`plugins/ess/`), with every other Beyond10x
plugin. Its `agentplugins-check tools` runs every command the plugin spells against the newest ESS
release and validates its syntax example with it, daily and on each of its pull requests, so a verb
renamed or removed here turns that check red there. The first level of `ess` is the five areas and
nothing else (`TOOLS` in `crates/edge/ess-cli/src/main.rs` is empty).

**Reading a failed `Gate` without the log.** The Actions log endpoint redirects to a zip and
`b10x-gates api` reports `GitHub response invalid`, so it is not a route. The **check-run
annotations** endpoint is plain JSON and carries the failing task by name:

```console
b10x-gates api --method GET \
  --path /repos/beyond10x/ess/check-runs/<check_run_id>/annotations \
  --output <file> --policy <policy> --key <key>
```

Get `<check_run_id>` from `/repos/beyond10x/ess/commits/<sha>/check-runs`. The annotation reads
`task: Failed to run task "<name>": exit status N`, which is enough to run that one task locally.
Going the other way — guessing which task broke and reproducing the whole gate — cost an hour and
found a different failure that CI does not have (see below).

**`task check` cannot pass where `$TMPDIR` sits inside a Git checkout.**
`observed_bindings::tests::an_output_created_after_preflight_is_preserved` calls
`new_output(path, outside_git: true)`, which refuses any path with a `.git` ancestor
(`crates/edge/ess-cli/src/observed_bindings.rs:511-518`), and it builds its root under
`std::env::temp_dir()`. On `ubuntu-latest` that is `/tmp` and the test passes; on a machine whose
`TMPDIR` is under a dotfiles checkout it can never pass. Run the gate with a `TMPDIR` outside every
checkout, and do not read this failure as a repository defect. The test is asserting a create-new
race, not the Git refusal, so the flag is incidental to it.

**Merge the branch, then tag.** `0.10.0` was tagged on `feat/outcome-sets-entity-fields`, published,
and `main` did not have a line of it — every other check reads the workspace and the remote's tag
list, and neither says which line a commit is on. AEP hit the same shape one version later and cut a
*newer* release that silently dropped the older one's features.

## Where work is tracked

| What | Where |
|---|---|
| The plan: epics, stories, blockers | `.engineering/planning/`, mutated only through `aep artifact` — never by editing a store file, never by writing `status:` |
| The protocol tree the store obeys | `.engineering/project.yaml` (a pinned `git+…#<40-hex>` source) |
| Binding designs | `docs/design/` — a construct is a design page before it is code |
| What shipped | `CHANGELOG.md`, and `git tag -n99` |
| Whether an adopter's request is adopted, redesigned, declined or deferred | the story's `## Fit review` and `## Decisions`, written by `.agents/skills/assessing-external-requests/SKILL.md` |

A change an adopter asks for — an issue, another session's message, a downstream review — is
assessed with `.agents/skills/assessing-external-requests/SKILL.md` before a story is written
around it. The requester's proposed syntax is evidence of a need, not a design, and a story from an
external request is not dispatched in a wave without its fit review.

**Downstream findings are designed into generic features, never filed one per finding.**

- Gaps, refusals, defects and documentation gaps from a downstream specification arrive as one
  collected report or review; the reporter's proposed ideas are input, not the request.
- Read the report against the open and closed issues, earlier reports and the design pages, and
  design generic features that each cover many findings; each feature is a local story in the
  plan store, with the findings it covers named in its fit review.
- A GitHub issue exists only for outside visibility: one per generic feature, or one batched issue
  as a last resort. Never one issue per finding, and never a one-to-one implementation of a
  request. An issue already filed for a single finding is linked to its feature's story and closed
  or merged into the feature's issue, with a comment naming it.

## Commits

Public Gates owns common security/privacy checks and generic bot delivery (Atlas ADR 0048).
Install coordinated hooks with `b10x-gates --repository beyond10x/ess install`. They scan the index,
messages, filenames, metadata, annotated tags and every outgoing commit. Private policy and local
signing keys stay outside public source. Exact adoption baselines report historical findings
separately; candidate ignore files cannot disable rules. Exceptions identify an exact rule,
bounded location and content digest. Superseded brand-exemption categories authorize no new
public association.

Use `b10x-gates bot` for direct commits/tags/pushes and `check`, `verify` and `publish` for signed
common evidence. Keep `b10x-bot[bot]`. Require the shared GitHub check before integration alongside
ESS correctness. A valid signed receipt reuses only common checks; it does not replace `task check`,
site validation or release artifacts. Commit/publish paths require no Atlas checkout, current Atlas
main or organization-wide admission. Atlas documentation validation and Website publication remain
separate from ordinary source publication.

`b10x-gates bot` runs only `commit`, `tag`, `push` and `fetch`. Merge a branch with
`git merge --no-ff --no-commit <branch>`, then `b10x-gates --repository beyond10x/ess bot -- commit
-F <message-file>`. `publish` accepts branch refs only (`--remote-ref refs/heads/<branch>`, after
`check --receipt <file>`); push a release tag with `b10x-gates --repository beyond10x/ess bot -- push
origin refs/tags/<version>`.

- Use conventional prefixes and a body explaining what changed and why.
- Use organization bot tooling outside this public repository for commits and pushes.
- Never commit credentials, tokens, kubeconfigs, or unsanitized observations.

<!-- b10x-docs-operations:start -->
## Public documentation operations

This repository owns the public source and presentation allowlist in `b10x.docs.yaml`. The generated credential-free `.github/workflows/b10x-docs-bundle.yml` passively packages only those declared files for the exact successful `main` commit; it must never run repository code. The generated `.github/workflows/b10x-docs-check.yml` runs the publisher's per-source checks on every pull request and main push, with read-only contents and no credentials; it is deliberately separate from the shared gate, which runs on `pull_request_target` with a secret and never reads candidate source. Atlas selects the latest successful bundle with every other catalog source, and Website plus Docs System own rendering, shared components, search, and feeds. Do not add a standalone docs deployer or put App credentials in this public repository. If Atlas catalogs a former Pages workflow, that file remains repository-owned validation: preserve its bespoke checks while keeping exact read-only permissions, an unconditional pull-request trigger, and no deployment primitives. Project Pages at `/ess/` serves this repository's own documentation site, deployed from `.github/workflows/b10x-docs-site.yml`. That caller fires when this repository's own credential-free `pages.yml` build finishes on `main` and hands `beyond10x/website` the artifact of that exact run, so a documentation change reaches the site without anyone dispatching anything. This repository never deploys Pages itself and never holds App credentials. Root publication does not rebuild the site and the site does not move when the root republishes.

From the complete organization workspace, verify the contract with a clean Atlas checkout at the current remote `main`. Set `B10X_ATLAS_CHECKOUT` to a managed Atlas worktree when the primary checkout is dirty or stale; never infer command availability from the primary alone.

```bash
atlas_checkout="${B10X_ATLAS_CHECKOUT:-atlas}"
atlas_head="$(git -C "$atlas_checkout" rev-parse HEAD)"
atlas_main="$(git -C "$atlas_checkout" ls-remote origin refs/heads/main | awk '{print $1}')"
test -z "$(git -C "$atlas_checkout" status --porcelain)"
test "$atlas_head" = "$atlas_main"
cargo run --manifest-path "$atlas_checkout/Cargo.toml" --locked -q -- \
  --store "$atlas_checkout/catalog/store" docs reconcile --workspace . --check
```

Keep internal plans, stories, ADRs, decisions, worklogs, security material, and research out of the public allowlist unless a repository authority explicitly declares them public.
<!-- b10x-docs-operations:end -->

<!-- b10x-release-operations:start -->
## Release completion

An ordinary release completes after this repository's exact tag, required source checks,
published release and required artifacts are verified. A pushed tag with unfinished checks or
uploads is queued; report it as released only after those requirements succeed.

Atlas reconciliation and public documentation publication run asynchronously. Do not wait for
Atlas or Website, update Website source locks or bootstrap snapshots, promote consumer pins,
release shared docs tooling, or redeploy documentation façades as part of an ordinary source
release. Report documentation as pending unless its publication was actually verified. A background
documentation failure does not invalidate a successful source release.

Keep this repository's provenance, correctness, security, compatibility and artifact verification
requirements. Shared rendering, routing or delivery-control changes still require their relevant
integration gates. A release request does not authorize deployment or downstream releases.
Repositories without a release unit retain their existing publication policy. This completion
boundary supersedes older instructions that attach synchronous documentation ceremony to each
source release.
<!-- b10x-release-operations:end -->
