---
format: aep.planning-md/3
id: review-result:zi-w1-graph-adv-1
kind: review-result
status: active
title: Turn 1 adversary pass 1 on story:a-graph-editor-draws-its-edges-and-a-menu-per-node
relations:
- reviews: story:a-graph-editor-draws-its-edges-and-a-menu-per-node
revision: 2
---
unit: story:a-graph-editor-draws-its-edges-and-a-menu-per-node
verdict: broken
new failing cases: 3 (crates/ui/ess-ui-react/tests/graph_editor.rs::adversary_layers_run_from_the_nodes_no_edge_enters_whatever_the_edge_row_order, crates/ui/ess-ui-react/tests/graph_editor.rs::adversary_a_self_loop_is_drawn_outside_its_node, crates/ui/ess-ui-test/tests/adversary_graph_menu_pass1.rs::adversary_an_edge_action_opens_the_edges_menu_when_a_node_shares_its_key)
mutants survived: none

Findings cover tree `zi-t1-6` at HEAD `feab18b875` plus my uncommitted test files. Diff: `graph_editor.rs | 52 +` and the untracked `adversary_graph_menu_pass1.rs`. Both are test files. I bumped the mtime of the template and `playwright.rs` with `touch`; their content is unchanged and git shows no diff.

1. **Layering, in the acceptance (blocker).** Nodes S, A, B with edges listed B→A, A→B, S→A. The layout throws out whichever edge closes a cycle when the edges are read in row order, not when walking from the nodes no edge enters. So A→B is dropped and B lands in the first layer even though an edge enters it. Run `r4.txt` failed with: `layers run along the longest path from the source S: {"S":16,"A":248,"B":16}`. The code is in `graph_editor.tsx.tmpl`, function `layout`. Fix: find the cycle-closing edges by a depth-first walk from the source nodes, in row order.
2. **Self-loop, beside the acceptance (backlog).** An edge s2→s2 (a retry loop) is drawn as a line from the top to the bottom of its own node. The label lands inside the node's box, which is painted over the drawing, so the edge can't be seen. Run `r4.txt` failed with: `the self-loop's label at 328,44 is hidden under its node {"left":248,"top":16,"width":160,"height":64…}`. Who reaches it: any flow with a step that retries itself.
3. **Edge action opens the node's menu, in the acceptance's "address it as today" (warning).** When edges come from their own view (`edges.reads`) and a node and an edge share a key (two tables both numbered from 1), both rows are `…/rows/1`. The Playwright export uses `[…rows/1] .ui-graph-menu` `.first()`, which is the node's menu. It opens that, then clicks an edge item that was never rendered. Before this change that edge-action click worked. Run `r3.txt` failed with `assert_ne!` on two identical opener lines. Fix: make `graph_menu_row` open `.ui-graph-edge[…] .ui-graph-menu` for `edge_actions` and `.ui-graph-node[…] .ui-graph-menu` for `node_actions`.
4. **Wrong code in the shared build slot (needs-coordinator).** Runs `r1.txt` and `r2.txt` in slot B linked a stale `libess_ui_react` rlib. It still had the old template and contains paths from tree `zi-t1-3`, even after I touched the template; sccache seems to have served another tree's build. Those two runs are void. Any gate run through `build.sh` can quietly test another tree's code. Run `r4.txt` did a full rebuild, its two original tests passed, and its failures match a direct node run against the slot A project, which is byte-identical to HEAD.
5. **Counts.** Cases went 6→4: run `r4.txt` (`test -p ess-ui-react -p ess-ui-test … --test graph_editor --test graph_reads --test graph_confirm --test adversary_graph_menu_pass1`, `exit 101`, `slot B waited 131s ran 30s`) stopped after the `graph_editor` target. `graph_reads`, `graph_confirm` and my ess-ui-test file did not run in it; the 4-run budget is used up. My ess-ui-test case was red on its own in `r3.txt` (`exit 101`, `slot B waited 40s ran 17s`).
6. **What held.** Disconnected nodes, an edge to a missing node (left out of the drawing, kept in the list), duplicate edges, a menu already open (checked through `aria-expanded`), the emitted bytes being deterministic, and confirm paths not taking the menu branch. The key-with-a-slash and nested-`rows` paths fail in `graph_menu_row` the same way `row_of` already did before this change.
7. **Files I wrote outside the worktree.** All under `~/.cache/scratch/e368f6dc-c92b-425c-8fdf-280e82792de9/ess/waves/t1/a-graph-editor-draws-its-edges-and-a-menu-per-node/adv/`: `r1.txt`, `r2.txt`, `r3.txt`, `r4.txt`, `dbg-layers.cjs`, `adversary-layers.cjs`, `adversary-self-loop.cjs`. Test output also went to `~/.cache/scratch/e368f6dc-c92b-425c-8fdf-280e82792de9/ess/targets/B/tmp/ess-ui-react-graph-editor/adversary-*` and `~/.cache/scratch/e368f6dc-c92b-425c-8fdf-280e82792de9/ess/targets/B/tmp/ess-ui-test-adversary-graph-menu/`.

The coordinator's later note says `live_binding` now counts. I did not run it.

```findings
- file: crates/ui/ess-ui-react/templates/runtime/composites/graph_editor.tsx.tmpl
  category: acceptance
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: layout() drops whichever edge closes a cycle in edge row order instead of walking from the source nodes, so a node an edge enters (B in S->A->B with B->A listed first) lands in the first layer beside the source.
- file: crates/ui/ess-ui-react/templates/runtime/composites/graph_editor.tsx.tmpl
  category: boundary
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: a self-loop edge is drawn from the top to the bottom of its own node and its kind_by label sits inside the node box, which is painted over the SVG, so the edge cannot be seen.
- file: crates/ui/ess-ui-test/src/playwright.rs
  category: contract-drift
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: graph_menu_row opens the first .ui-graph-menu under rows/<key>, which is the node's when a node and an edge from edges.reads share a key, so the exported edge-action click targets an item that was never rendered.
```
