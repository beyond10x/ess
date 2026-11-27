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
never changed to follow a move. The areas and every crate in them are listed in
[README.md § The crate tree](README.md#the-crate-tree); the dependency rules are here.

- `crates/specify/ess-yaml` is a leaf with no ESS dependency and the only crate that names a YAML
  library; every other crate reads and writes YAML through it, and a crate depends on neither
  `serde_yaml` nor `unsafe-libyaml` (`crates/edge/ess-xtask/tests/layout.rs`,
  `docs/design/yaml-parser-replacement.md`).
- `crates/infra/` is a separate bounded context whose only dependency on the rest is
  `ess-primitives`. It reads YAML through `ess_primitives::yaml`, which re-exports `ess-yaml`.
- `crates/ui/ess-ui` (schema at `schemas/ui/ess-ui.schema.yaml`) has no dependency on the rest
  except the leaf `ess-yaml`.
- Nothing under `crates/generate/` applies an artifact to a running system.

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
setup. Every test binary is compiled once, in jobs built side by side: `build-tests` runs
`test-archive-workspace`, which writes two workspace archives from one build, `workspace-a`
(`WORKSPACE_GROUP_A`: ess-conformance binaries `a`..`m`, 20.4 GB extracted) and `workspace-b` (its
complement, 25.2 GB), and `build-feature-off` runs `test-archive-feature-off` (local
`task test-archive` runs both). Each of the 20 `test (<m>/20)` jobs runs `test-shard GROUP=<a|b>
PART=<k>/10` (shards 1–10 group a, 11–20 group b): it extracts its group's archive, deletes both
archives and runs `slice:k/10` from the extracted metadata, because all 45.6 GB extracted did not
fit a 52 GB hosted runner. It then runs `test-feature-off` for `slice:m/20`, compiling nothing;
`slice:` deals the whole test list out, where `count:` dealt each binary's out from shard 1 and
left the last shard with half the work of shard 1. The shards extract into
the checkout's own `target/`, because test binaries bake `CARGO_BIN_EXE_*` and
`CARGO_TARGET_TMPDIR` in at build time. The `Gate` job carries their joint result, and
`crates/edge/ess-xtask/tests/ci_lanes.rs` fails when a step of `check` is in no job, when the
shards leave a partition unrun, or when `Gate` misses a job. Local `task test` still runs
`cargo test` over everything.

**The `msrv` job holds the declared `rust-version`.** Downstream repositories compile ESS with the
`rust-version` in `Cargo.toml`, so on every pull request `msrv` reads it from there (`1.85` installs
`1.85.0`) and runs `cargo check --workspace --all-targets --locked` on it, and `Gate` carries the
result. A line only a newer rustc accepts fails it, and so does a `Cargo.lock` entry whose own
`rust-version` is newer. Reproduce it with `cargo +<rust-version>.0 check --workspace --all-targets
--locked`. `ci_lanes.rs` (`the_msrv_job_checks_every_target_on_the_declared_rust_version`) fails
when the job names the version a second time or checks less than every target.

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

**A pull request that changes only the plan store runs no test job.** The `diff` job of `ci.yml`
lists the paths the pull request's merge commit changes against its base. When every one lies
under `.engineering/` and none under `.engineering/repro/` (tests in `ess-conformance`, `ess-domain`
and `ess-synth` read those), `build-tests`, `build-feature-off` and the 20 test jobs are skipped,
and `Gate` passes them skipped for that reason only and says so in its log; `Checks`, `msrv`, the
transport job and the macOS witnesses still run. Only a pull request from an ordinary head or a
fork decides: a `main` push, the nightly run, a `queue/**` head, a `merge_group` run and a call
from `release.yml` never skip, and an error decides "not plan-only". Such a run never counts for a
release, because `prior-gate` qualifies only runs that cannot skip. The path patterns are written
once, as `plan` and `tested` at the top of the `plan-only` step. `ci_lanes.rs` runs the decision
and `Gate`'s step against canned inputs, and fails when an archived test reads the plan store
outside `.engineering/repro/`.

**Every test-running step has a limit, and so has every test** (the organization rule `Tests stay
fast`; story:the-test-step-meets-the-test-budget). Each task that runs tests in a lane runs its
test command as `{{.TEST_STEP}} <task> <seconds> <command>` (`TEST_STEP` in `Taskfile.yml`), locally
and on CI: the command is killed at its limit and the step fails with `test step <task> ran <n> s,
over its limit of <m> s`; within it the step prints `test step <task> ran <n> s of its <m> s`.

The nextest profiles `ci` and `slow` kill a test at 60 s (`terminate-after = 1`) and count it
failed; before, `slow` killed at 1,200 s (`period = "120s", terminate-after = 10`). The nextest
steps are `test-shard`, `test-feature-off`, `test-packages`, `test-xtask` and `fuzz-check` (under
`fuzz/.config/nextest.toml`, the same 60 s with no exception: `fuzz/` is a workspace of its own,
and nextest refuses an override naming a binary the workspace lacks). Doc-tests cannot
run under nextest, so `test-doc` and `test-feature-off-doc` have a step limit of 60 s instead (CI
measured about 22 s and 7 s): a doc-test of 61 s fails its step.

Each test seen over 60 s in 14 CI runs of 2026-11-26 and 2026-11-27 (13 successful, and run
46063361909, the first under these limits, red on one test killed at its override) has an override
in `.config/nextest.toml` at 1.2 times its longest time there, and so has each test whose longest
time there was 50 to 60 s (1.2 times it, at most 72 s), since tests that close cross 60 s on some
runs: 174 under `ci` (the longest `adversary_u22_pass3`'s wide-inputs case, 366 s, 33 of them the
tests of 50 to 60 s) and 15 under `slow` (`adv_e_u6_byte_identity_probe`, 337 s; 2 of them the
tests of 50 to 60 s, from the slow job's 12 runs). `ci_lanes.rs` pins both lists (`CI_EXCEPTIONS`,
`SLOW_EXCEPTIONS`), so an override may only be dropped or lowered, and refuses any spelling of
`.config/nextest.toml` but the one its reader understands: the workspace has no TOML parser.

The steps' limits: `test-xtask` and `fuzz-check` 120 s, `test-doc` and `test-feature-off-doc` 60 s.
Three steps are known to be over 120 s and have a ceiling of 1.2 times the longest time seen in
the 13 successful runs; a ceiling may only fall (`STEP_CEILINGS` in `ci_lanes.rs`). In run
46063361909 every step stayed under its ceiling (`test-shard` 919 s, `test-feature-off` 250 s):

- `test-shard`: 1,235 s (run 45879426222, shard 16/20; 977 s at most in the other twelve), ceiling
  1483 s;
- `test-feature-off`: 556.9 s (run 45965268982, shard 3/20: 546.2 s of tests and 10.7 s of
  listing; the listing took up to 11.9 s), ceiling 669 s; it extracts its archive (up to 345 s)
  before the step, in `test-feature-off-extract`;
- `test-packages` (a patch tag's targeted gate): not measured, no patch tag has run it; estimated
  at 837 s from the 3,346 test-seconds of every crate it may select over 4 threads, ceiling 1004 s.

What a step builds runs before it, outside the limit (`cargo nextest run --no-run`, `cargo test
--no-run`, an archive extraction), except in the doc-test steps: `cargo test --doc` refuses
`--no-run`, so they build the libraries first (`cargo build --lib`), and the dev-dependencies and
the crates whose features those change still compile inside the step (16 crates for `test-doc`,
6 for `test-feature-off-doc`, about 12 s on the shared host).

Every test-running step prints its ten slowest tests through `test-slowest` from a JUnit report,
also when it fails, and removes the report it prints before it runs, so a run that writes none
prints `test-slowest: no JUnit report at <path>` and never an earlier step's. One exception: stable
libtest prints no time per test, so the doc-test steps print their ten slowest crates, read from
the `finished in` line of each crate's doc-tests (`DOC_TEST_TIMES`); a crate still running when
its step was killed is listed with the step's time. `test-packages` prints its
own before it runs `test-xtask`, which prints its own.

`ci_lanes.rs` fails when a test-running task runs outside `TEST_STEP` or over its ceiling, when a
step prints no ten slowest or does not remove its report first, and when an override is added or
raised. Outside the test step: local `test`, `test-slow` (its tests are held by the `slow`
profile), `test-browser-product` and `typescript-check` (in no lane), `test-transport-network` and
the macOS witness job. Tests run slower on the shared host than on CI, so there a test under 60 s
on CI can be killed.

Test builds carry line tables only (`[profile.dev] debug`), and `sha2`, `flate2` and `tar` are
optimised in them. CI's Linux jobs link with lld through
`CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS` in `ci.yml`, not `.cargo/config.toml`, so local
builds and shipped release binaries keep the system linker.

**Consumer coverage is opt-in.** Revision 3 of the ESS evolution plan (plan
ess-evolution-20260915, 2026-09-25) parked feature-preservation accounting, as recorded in
`decision-blocker:consumer-accounting-parked` and the 2026-09-25 amendment to
`architecture-decision-record:ess-evolution-01-evolve-without-feature-loss`: `task check` runs
`consumer-check` only with `CONSUMER_CHECKS=true`, no CI lane runs it, and it is not part of the
release bar. `task consumer-check` still runs it on its own. The code,
`initial-baseline.json`, the classifications and the rebound reviewed cases stay in the tree. The
retired `SKIP_CONSUMER_CHECKS` switch means nothing; `ci_lanes.rs` refuses it in every gate file.
Re-enable the lane when a named external adopter pins a released `ess/N` and its generated
artifacts, and a change reaches that adopter undetected by conformance, `ess verify diff` or a
format refusal.

`task check` runs formatting, strict Clippy, all workspace tests (`test`), the slow lane
(`test-slow`), rustdoc, command smoke tests, and the dependency boundary test. Its test step is
offline. Its slow lane is not: `task test-slow` sets `ESS_TUTORIAL_NETWORK=1`, which makes
`the_walkthrough_runs_the_npm_and_go_blocks_too` run the Start-here pages' `npm` and `go` steps, and
those need the npm registry and `go`; the test step's walkthrough runs every `ess` command and
skips those blocks. Land nothing on `main` until the `Gate` job is green.

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
| anything that changes a committed example tree under `generated/` or a projection (`ess-gen`, `ess-synth`, schema or OpenAPI output) | also `ess-synth`'s whole test set: `feasibility`, `adversary_related_guard_behaviour_pass1` (`adv319`) and `http_binding_rust` (its `go` module) compare the committed example trees byte for byte |

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

**A gate of named test targets includes the repository-wide byte pins.** A change to synthesis,
to a generator or to generated wording moves bytes that only these targets compare, and a gate
that names targets instead of building everything misses them (0.87.0's queue run found nine
red there):

- `ess-conformance`: `synthesis_suite_bytes_table`, `synthesis_suite_bytes_table_cli_read`,
  `adversary_related_values_all_bytes`, `external_beside_held_guard`;
- `ess-synth`: `served_codec_names`, `go`;
- `ess-openapi`: `adversary_ess_issues_2_u6`;
- `ess-compiler`: `string_alphabet_and_length`;
- `ess-cli`: `tutorial_page`.

Re-pin a moved row only where the change claims it, and name the cause beside the pin.

These targets are gated by name as before. In the test step `synthesis_suite_bytes_table` compares a
digest of the sources of synthesis and of every model with
`crates/verify/ess-conformance/tests/fixtures/synthesis-sources.digest` and synthesizes nothing; the
byte comparison itself (the two tables, and the whole-corpus `enum_presence_guard` case) runs in the
slow lane. A change under `src/` of `ess-conformance`, `ess-gen`, `ess-transport`, `ess-http`,
`ess-compiler`, `ess-domain`, `ess-primitives` or `ess-yaml`, or to a model file, re-stamps with the
command the failing `the_synthesis_sources_digest_matches_the_committed_stamp` prints, in the same
change.

`task test-browser-product` runs in no CI lane (those Firefox tests are ignored in the CI test
shards); run it locally when a change touches what it covers. The slow lane, `task test-slow`, runs
every test marked `#[ignore = "slow lane: <reason>"]` (`cargo xtask slow-list` names them). The CI
job `slow` carries the lane and `Gate` carries the job: a pull request runs it unless every changed
path is in the `skippable` list of the `diff` job's `slow` step, and the `main` push, the nightly
run, a `queue/**` head, a merge group and the release gate always run it, so a run that skipped it
never counts for a release. The full release gate and a reused qualifying run include the whole
lane; the targeted gate of a patch tag (§ Hotfix, step 5) runs the slow-lane tests of the crates it
tests.
`test-browser-product` builds its wasm host once into one shared target and took 1120 s for 30 tests on 2026-10-22 at debuginfo 0 (about 6.8 h before, when each test built its own host on one core). To reproduce the whole gate locally, mind that `task test` alone builds about 45 GiB into
the tree's `target/` (44.8 GiB measured on 2026-10-07 at 0.56.0), so start it only with that much
free disk beyond what other work on the machine needs, and watch free space during the step rather
than only between steps.
Set `ESS_TYPES_NODE` to a `node_modules` outside the checkout that holds `@types/node` at the version
`ci.yml` installs (`npm install --prefix <dir> @types/node@26.3.0`): without it the TypeScript cases
in `adversary_287_pass1` fail with `TS2688: Cannot find type definition file for 'node'` and
`cargo test` stops at that target, so the binaries after it never run.
`ess-synth`'s `integer_spellings` runs a Rust harness built once by `task integer-spellings-harness`
(no test builds with cargo, `no_test_builds.rs`); `task test` and the CI shards set
`ESS_INTEGER_SPELLINGS_HARNESS` to it, and a bare `cargo test -p ess-synth` needs that task run and
the variable set, or its Rust cases fail naming the task. The same holds for `task
append-set-harness` (`append_set`, through `ESS_APPEND_SET_HARNESS`), `task binary64-harness`
(`binary64_rust`, through `ESS_BINARY64_HARNESS`), `task json-body-harnesses` (`json_body_closure` and `adversary_w1_gen1_security`, through
`ESS_JSON_BODY_CLOSURE_HARNESS` and `ESS_JSON_BODY_SECURITY_HARNESS`), `task ui-http-servers`
(`ess-cli`'s `generate_ui_http`, through `ESS_UI_HTTP_SHELF_SERVER` and `ESS_UI_HTTP_ITEMS_SERVER`),
`task body-cap-rust-server` (`ess-synth`'s `body_cap_rust` cases, through
`ESS_BODY_CAP_RUST_SERVER`) and `task body-cap-go-server` (`ess-synth`'s `body_cap_go` cases,
through `ESS_BODY_CAP_GO_SERVER`).
Each generates its server from a committed model and `http.yaml.in` binding beside the test; a
change to either needs the task run again.
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

**The GitHub release body is `release-notes/<version>.md`, written by hand.** Whoever prepares
the release writes it in the release-preparation commit, beside the version's `CHANGELOG.md`
section, which stays the full record. The file is exactly the body, in this shape: a lead, a
blank line, `**Breaking**`, a blank line, its list, a blank line, `**Changed**`, a blank line, its
list, a blank line, and last the line
`Full entry: [CHANGELOG.md, <version>](https://github.com/beyond10x/ess/blob/<version>/CHANGELOG.md#<anchor>)`,
`<anchor>` being GitHub's anchor of the section heading. The tool refuses: more than 900
characters before that last line (1,200 when `**Breaking**` has more than 3 bullets); a lead of
more than one line, one sentence or 25 words, or one that counts entries; a list of more than 5
bullets; a bullet over 110 characters after `- `, ending with a period, or holding "so that", a
parenthesis, a `U+` code point or a semicolon (one is allowed in a `**Breaking**` bullet);
"now", "again" and "its own" anywhere outside code; a word in backticks that the section does not
contain; and `- None.`, the only no-breaking form, when the section has an entry starting
`**Breaking`. What no program can check is the writer's duty: each bullet is one clause, no
second clause after "and", no list of three or more examples, no internals or rationale.
`cargo xtask release notes <version>` validates and prints it, and `release.yml` publishes that
output as `notes.md`. `cargo xtask release verify`, and with it `task release-check` and CI,
refuses the workspace version without a valid file; `cargo xtask release notes --check`, in
`release-check` too, validates every file present. The rules are `release_notes.rs` in
`ess-xtask`.

Public CI has read-only GitHub permissions and never creates or edits a Release. A successful
`Release preparation` run uploads one artifact named `ess-release-<tag>-<commit>`, containing the
four archives under `dist/`, `dist/SHA256SUMS` and `notes.md`. From a trusted operator environment,
the coordinator reads the successful run and its exact head commit, resolves the annotated tag to
that same commit, and downloads only that exact artifact through
`b10x-gates gh --repo . --repository beyond10x/ess -- ...`. The wrapper runs `gh` in the `--repo` directory and refuses an argument holding a home-directory path, so run it from an empty download directory outside every checkout with `--repo .` and relative paths; pointed at a checkout, `run download -D <dir>` writes the release files into that checkout. Before publication, verify that the
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
| The plan: epics, stories, blockers | `.engineering/planning/`, mutated only through `aep plan artifact` — never by editing a store file, never by writing `status:` |
| The protocol tree the store obeys | `.engineering/project.yaml` (a pinned `git+…#<40-hex>` source) |
| Binding designs | `docs/design/` — a construct is a design page before it is code |
| What shipped | `CHANGELOG.md`, and `git tag -n99` |
| Whether an adopter's request is adopted, redesigned, declined or deferred | the story's `## Fit review` and `## Decisions`, written by `.agents/skills/assessing-external-requests/SKILL.md` |

Where the open work stands, what was released last and what waits on the operator: the first
section of `.engineering/waves/zero-open-issues.md`. Read it before starting anything; the copy
on the branch `integration/ess-zero-issues-4` is the later one whenever that branch is ahead of
`main`.

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

## Hotfix

A defect an adopter reports, once reproduced, is pending work, never new work: it starts at once,
in parallel with any running wave, and needs no operator confirmation. The target is a pushed tag
within one hour of the reproduction.

1. Branch from the latest release tag, or from the newest release head not yet released.
2. Write the failing regression test first, then the fix.
3. Bump the patch version and add its `CHANGELOG.md` section and its `release-notes/<version>.md`.
4. Before the tag, run only the regression test and the touched crates' tests and Clippy: no local
   full gate.
5. Commit with `b10x-gates bot`, create an annotated tag with `b10x-gates bot -- tag -a`, and push
   the tag. The tag's release workflow is the gate. No wave. For a patch tag it runs a targeted
   gate instead of the 20 test shards: `Hotfix checks` (`task ci-lint` and `task ci-smoke`) and
   `Hotfix tests` (the crates changed since the previous tag and their direct dependents, chosen by
   `.github/scripts/hotfix-scope.sh`), beside the package builds; `Hotfix tests` then runs `task test-slow` on the same crates. A change the script cannot map to
   crates, such as a workflow, the Taskfile or a dependency, runs the full gate. `CHANGELOG.md` and
   `release-notes/` count as prose, and `schemas/ui/ess-ui.schema.yaml` as code of `ess-ui`.
   A change that selects `ess-cli`, `ess-conformance` or `ess-synth` runs the full gate: their
   tests do not fit the targeted job's limit.
6. In the same minute, push the branch and open its pull request to `main`. The main ruleset needs
   its required checks on the tagged commit itself, and the tag's release run does not count; a pull
   request opened only after the release costs a second full CI run (0.91.2: 50 extra minutes).
7. While a hotfix release runs, start no other release or release-candidate run in this repository.
   The runners are shared; 0.91.2's test shards queued for up to 72 minutes behind the 0.92.0 runs.
8. The release publishes itself on Gate pass: `publish.yml` publishes the prepared artifact as the
   organization bot when the release run succeeds. Check it within five minutes: the release
   exists, its author is `b10x-bot[bot]`, it has four archives and `SHA256SUMS`. The session that
   pushed the tag owns this check.

A tagged hotfix commit reaches main through a merge commit, never a squash, so the tag stays on main.

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

## Public documentation operations

The site at `https://beyond10x.github.io/ess/` is this repository's documentation site and the only
one. This repository's credential-free `pages.yml` builds it, and `.github/workflows/b10x-docs-site.yml`
fires when that build finishes on `main` and hands `beyond10x/website` the artifact of that exact
run for deployment, so a documentation change reaches the site without anyone dispatching anything.
This repository never deploys Pages itself and never holds App credentials. There is no
documentation manifest, no bundle workflow and no per-source documentation check any more, and the
unified `/docs/ess/` route is deprecated.

Keep internal plans, stories, ADRs, decisions, worklogs, security material, and research out of
public documentation.

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
