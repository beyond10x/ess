---
format: aep.planning-md/3
id: story:a-read-given-the-search-state-record-draws-the-search-box
kind: story
status: draft
title: A read given the record holding the search state draws the search box
summary: read_takes counts a read of the search field but not a read given the record that holds it (0.87.0 adversary F2; from-finding)
tags:
- backlog
- from-finding
revision: 1
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
