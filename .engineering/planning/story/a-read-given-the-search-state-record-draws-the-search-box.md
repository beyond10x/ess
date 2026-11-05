---
format: aep.planning-md/3
id: story:a-read-given-the-search-state-record-draws-the-search-box
kind: story
status: active
title: A read given the record holding the search state draws the search box
summary: read_takes counts a read of the search field but not a read given the record that holds it (0.87.0 adversary F2; from-finding)
tags:
- backlog
- from-finding
- next-wave-first
relations:
- serves: vision:O2
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-11-05T18:11:37Z", actor: "human:timo", revision: 4}
- {from: "proposed", to: "active", at: "2026-11-05T18:11:38Z", actor: "human:timo", revision: 5}
---
## Need

Found by the 0.87.0 adversary pass (case `defect_a_read_given_the_search_state_parent_draws_the_search_box` in
`crates/ui/ess-ui-react/tests/adversary_w087_ui_pass1.rs`). A filter bar draws its search box only when a read on
the page takes the search state (`read_takes` in `crates/ui/ess-ui-react/src/emit.rs`). A read given the record
that holds the search state, not the search field itself, is not counted, so the search box is not drawn there.
No document in the repository has that shape today.

## Acceptance

- The ignored adversary case runs and passes: a read given the record holding the search state draws the search box.
- A read that takes neither the field nor a record holding it still draws no box.

## Schedule

Backlog finding (from-finding).

## The next wave takes it

ESS 0.87.0 shipped with this finding pinned as an ignored test,
`defect_a_read_given_the_search_state_parent_draws_the_search_box` in
`crates/ui/ess-ui-react/tests/adversary_w087_ui_pass1.rs`, and the release check flags every added
`#[ignore]`. The release was accepted with the ignore on the condition that the next ESS wave takes
this story: it either fixes `read_takes` so the test runs green without the ignore, or rewrites the
test to assert today's behaviour and drops the ignore. Either way the next release adds no ignore
for it.
