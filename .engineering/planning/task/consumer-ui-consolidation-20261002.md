---
format: aep.planning-md/3
id: task:consumer-ui-consolidation-20261002
kind: task
status: active
title: Consolidate existing UI PRs into one locally verified candidate
relations:
- decomposes: task:consumer-backlog-20261002
- serves: vision:O2
- decomposes: story:ui-tui-app-generator
- informed_by: story:ui-react-live-binding
- informed_by: story:ui-tui-live-binding
- informed_by: story:ui-spec-style-tokens
- informed_by: story:feature-request-323
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-10-02T09:20:40Z", actor: "human:timo", revision: 2, executor: "agent:codex-ess-backlog", correlation: "consumer-backlog-20261002"}
- {from: "proposed", to: "active", at: "2026-10-02T09:20:40Z", actor: "human:timo", revision: 3, executor: "agent:codex-ess-backlog", correlation: "consumer-backlog-20261002"}
---
## Outcome

One UI delivery candidate replaces separately gated overlapping PRs without losing their behavior, tests or history. Use existing PR 381 as the published carrier after local integration and verification.

## Authority

The operator explicitly requested reconciliation of which existing PRs to keep and fewer full gate runs on 2026-10-02. Preserve one managed tree and one PR for the coherent UI batch; no unconditional reruns of unchanged blocked candidates.

## Exact source inventory

- Carrier PR 381: 28aeddddfc86c4a92c48aba98d5d70091ba4219d (21 fixes; ESS Gate green, common security blocked).
- Live UI PR 377: 88bd4f3aa9a661e525e36dc285f9d37946f1a855. It includes exact ancestor PR 375 head 7e3b6b0986f7023a2daaed66d0f8e5e565753bdb and original React head 2ec7c005b. PR 380 squash ebdd72d7fa43f569f77cb9a68c6492160342a6f0 has the same complete-diff stable patch ID fec76f14d9d4ca4c082e2721518f378309819fe4 as original React diff 5f53c5a4b..2ec7c005b. PRs 375 and 380 closed as superseded through the App; neither closure claims delivery.
- Overlay row fix PR 344: 78323c15daa7d6888190bfc1aeb1005c60077e27.
- TUI dotted fields PR 345: 3403df9e4ae27b6494c081aea130c200c68aaf9b.
- UI style document model PR 368: 87c3ff22e7e8c75e9e024d4d185609932d03597a; model/check/schema step only, do not claim renderer token support.
- Literal expression PR 369: 0d4ac7dd36d4ae8392f74b64be739438a5914fb7. Its patch is not equivalent to 381's alternative #353 fix. Preserve its regression cases and resolve behavior explicitly, not by deleting the patch as duplicate.
- Current main: c211314afc75f9a4282e552e61406d782839513a. Preserve it; 377's older complete tree must not replace this base.

## Known gate failures

Run 36958395688 on 377: doc-check fails a private rust_string intra-doc link at crates/ui/ess-ui-tui/src/generate.rs:19. Use a code span, preserving private visibility. Shard 3 reports 1683 passed and one failed: the_generated_crate_is_clippy_clean_for_any_app cannot find cargo-clippy under toolchain 1.98.1. The shard toolchain step in .github/workflows/ci.yml lacks components: clippy. Install the required test dependency; retain the test and lints. These are required integration corrections, not permission to waive CI.

## Acceptance

All included heads are exact ancestors of the final candidate or have explicit equivalent-patch proof and retained unique regression coverage. Resolve 381/377's known conflicts in React actions/confirm, Playwright tests and TUI app/view with both features intact. Integrate 344/345/368/369 once; retain tests for overlay row params, nested typed fields, token model/check/schema, literal text and live React/TUI generation. Capture actual local affected-package, strict lint, formatting, workflow-contract and site-build results before one grouped publication. Full gate remains required on final combined bytes. Never use either component's green gate as combined evidence.

## Execution

Reuse managed tree batch-0-51 (resolve its machine-local path with worktree inspect), originally clean tracked source on release/0.52.0-squash at 28aeddddf. Its registry had no live lease; ignored target evidence is preserved. Acquire a new task lease before source changes. One integration worker owns source; coordinator owns AEP. Scope is included PR paths plus the two proven CI corrections. Worker may make bot-authored local integration commits through normal hooks, with exact parents retained. No push while preparing. No AEP edits by worker; route any store conflict to coordinator. Resource floor 8 GiB; defer large builds until source stable and root allocates disk.

## Keep and retire

Keep 381 and 386 as final existing delivery carriers, plus one future synthesis PR. Close 344/345/368/369/377 only once their unique changes are preserved in the published combined UI candidate. Do not delete source branches/worktrees during reconciliation. Source issue closure waits for actual accepted integration/release evidence.

## Typed scope record

The CLI refuses scope on tasks: `scope` is a field of `story`; a task inherits the surface of the story it decomposes. Recorded primary integration/CI scope on ui-tui-app-generator, with additional participating story scopes on ui-react-live-binding, ui-tui-live-binding and ui-spec-style-tokens. Exact changed-file manifests from the seven source heads remain the authoritative merge scope; the task does not invent another executable surface.

## Local integration checkpoint

Local candidate010e6c06b062411047b5fb2746f72d61958f5f70 has all seven recorded source heads as exact ancestors, verified by git merge-base --is-ancestor. Six integration commits have verified bot author and committer. No source PR has been closed merely on this local preservation; publication remains pending. Source verification, confirm/refusal composition checks, issue323 red/green regression and the two demonstrated CI corrections are underway before a grouped gate. Local ancestry proves preservation, not combined correctness.
