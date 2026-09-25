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

- **`crates/specify/`** — `ess-primitives`, `ess-domain`, `ess-compiler`, `ess-composition`,
  `ess-realization`: an authored system becomes a validated, resolved IR.
- **`crates/generate/`** — `ess-gen`, `ess-synth`, `ess-openapi`, `schema-contract`,
  `ess-deployment`: that IR becomes artifacts, and nothing here applies one to a running system.
- **`crates/verify/`** — `ess-conformance`, `ess-diff`: an implementation held to the
  specification, and one revision of a specification against another.
- **`crates/infra/`** — `infra-domain`, `infra-compiler`, `infra-analyze`, `infra-spec`,
  `infra-project`, `ess-kubernetes`: the observed cluster, a separate bounded context whose only
  dependency on the rest is `ess-primitives`.
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
setup. `build-tests` runs `test-archive`: it compiles every test binary once into two nextest
archives, the workspace and the feature-off packages. `test (<m>/2)` downloads both and runs
`test-shard` and `test-feature-off` for partition `m`, compiling nothing. The shards extract into
the checkout's own `target/`, because test binaries bake `CARGO_BIN_EXE_*` and
`CARGO_TARGET_TMPDIR` in at build time. The `Gate` job carries their joint result, and
`crates/edge/ess-xtask/tests/ci_lanes.rs` fails when a step of `check` is in no job, when the
shards leave a partition unrun, or when `Gate` misses a job. Local `task test` still runs
`cargo test` over everything.

**A pull request runs the feature-off build on number semantics only.** With `FEATURE_OFF` set to
`number-semantics` (pull requests only), the feature-off archive keeps every feature-off package
except ess-cli, plus ess-cli's unit tests and its `binary64*`, `normalization*`, `schema*` and
`execution_recovery` binaries (`FEATURE_OFF_NUMBER_SEMANTICS` in `Taskfile.yml`). The rest of
ess-cli's feature-off run — every other ess-cli integration binary compiled without
`arbitrary_precision` — has moved to the `main` push, the nightly `schedule` run and the release
gate, which archive all of it. `ci_lanes.rs` fails if an ess-cli test binary whose source names
`arbitrary_precision` falls outside the filter.

Test builds carry line tables only (`[profile.dev] debug`), and `sha2`, `flate2` and `tar` are
optimised in them. CI's Linux jobs link with lld through
`CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS` in `ci.yml`, not `.cargo/config.toml`, so local
builds and shipped release binaries keep the system linker.

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

For a pull request, CI owns the full gate. Before pushing, run only the crates the change touches
— `cargo fmt --all --check`, `cargo clippy -p <crate> --all-targets --locked -- -D warnings`,
`cargo test -p <crate> --locked`, and `task ci-lint` when a workflow, Taskfile or public API
changed — then push and read the lanes. Run the whole `task check` locally only for a release tag
(below) or to reproduce a lane that failed in CI.

The adopter-facing Docusaurus source lives under `website/`; repository-root `docs/` remains the
engineering record and is never published directly. A documentation, release, or validation
workflow change must additionally pass:

```console
task site-build
```

This check is separate because `npm ci` fetches the exact public `docs-system` Git revision and
therefore cannot be part of the offline gate. `.github/workflows/pages.yml` preserves the same
Rust/WASM, browser-lab and site-build checks without Pages authority; the unified Website publishes
the collected source and the Atlas-generated façade owns the project redirect.

Before pushing a release tag, run `task check` and `task site-lab` on the commit being tagged.
Consumer coverage is not part of that bar while it is parked (revision 3, above).
The release workflow runs the reusable gate, WASM/browser-lab correctness checks and native
packaging concurrently at that exact commit, then publishes only after all succeed. It skips the
gate only when a green `Gate` already covers the tagged bytes. "Green" means the newest `Gate`
check-run GitHub Actions recorded on a commit is a completed success. The gate is skipped when
**either** of these holds:

1. the tagged commit itself has a green `Gate`, normally from the `main` push run; or
2. all of the following hold:
   - `GET /repos/{owner}/{repo}/commits/{sha}/pulls` returns exactly one merged pull request
     whose `merge_commit_sha` is the tagged commit;
   - `git rev-parse <sha>^{tree}` equals `<head>^{tree}` for that pull request's head, which the
     release fetches;
   - the tagged commit's first parent (`main` before the merge) is an ancestor of that head;
   - that head has a green `Gate`.

Anything else runs the gate, including an API or fetch failure. The tree equality makes the tested
bytes the tagged bytes. The ancestry makes the head's `Gate` a test of those bytes: a
`pull_request` run tests the head merged into the base of its day, and that is the head's own tree
only when the base was already in the head. So tag the merge commit of an up-to-date pull request
as soon as it lands; there is no need to wait for `main`'s run. That pull-request `Gate` ran the
narrowed feature-off build (above), and `main`'s run of the full one is still in flight when
such a release publishes. `release.yml`'s `prior-gate` step is the implementation, and
`ci_lanes.rs` runs it against real Git histories.

The Intel macOS archive is cross-compiled on the Apple Silicon `macos-15` runner and smoke-run
there under Rosetta; `lipo -archs` names the architecture that was built. Archive names and
`SHA256SUMS` are unchanged. Site rendering remains a documentation-validation gate; ordinary
source releases do not wait for it. New GitHub Releases stay draft until all archives and
checksums are uploaded.

Release completion means the exact tag, required release checks, published GitHub Release and
required assets are verified. A pushed tag awaiting those checks is queued. Atlas observes release
facts and publishes documentation asynchronously: do not wait for Website or Atlas, update Website
locks or snapshots, promote consumer pins, release the shared docs runtime or redeploy façades as
part of releasing ESS. Report documentation as pending unless publication was verified. A failure
in that background work does not undo a successful ESS release.

After cutting a release, and after any release run that fails:

```console
task release-status
```

It asks the remote and GitHub whether every pushed version tag is on `origin/main` and has a
release behind it, and fails while one is not. `.github/workflows/release-record.yml` runs it after
every release run and daily.

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

## Agent plugin

The ESS agent plugin lives in `beyond10x/agentplugins` (`plugins/ess/`), with every other Beyond10x
plugin. Its `agentplugins-check tools` runs every command the plugin spells against the newest ESS
release and validates its syntax example with it, daily and on each of its pull requests, so a verb
renamed or removed here turns that check red there. The first level of `ess` is the four areas and
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
