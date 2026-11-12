---
format: aep.planning-md/3
id: story:an-append-onto-an-absent-optional-list-is-asserted
kind: story
status: draft
title: An append onto an absent Optional list is asserted
summary: 'Follow-up of story:a-present-optional-element-is-appended: an append onto an Optional<List<…>> the creator leaves absent is asserted in no view expectation, so a target appending null or nothing passes (adversary finding, pre-existing, backlog)'
tags:
- backlog
- defect
- from-finding
- review-finding
- severity-medium
- zero-open-issues
relations:
- decomposes: epic:zero-open-issues-2026-11
- serves: vision:O2
revision: 1
---
## Need

A suite synthesized for `sets: {f: {append: input.x}}` asserts the whole list in the view row, so a
target that appends the wrong element, or nothing, fails. Over an `Optional<List<…>>` that the
arranging creator leaves absent, it asserts nothing about `f`: the `ExpectView` of the appending
scenario holds only the key, in the run that sends `x` and in the run that leaves it out. A target
that appends `null` whatever it is sent, or never appends, passes.

## Problem

`crates/verify/ess-conformance/src/synthesize.rs` `appended_at` reads the list held before through
`before_literal_at(before, target_location)?`. A field the arrangement never stored is missing from
`before`, so the function returns `None` and the expectation drops the field. Its own doc comment
says an absent list is read as empty. The committed `jobs` model's `AddNote` append onto `notes`
reaches it, and is asserted nowhere.

Found by the adversary pass over `story:a-present-optional-element-is-appended`. It is pre-existing
(base `4086c925a4` has the same line).

## Acceptance

1. `adv_an_element_appended_to_an_absent_optional_list_is_present_and_absent` and
   `adv_two_appends_in_one_branch_are_each_present_and_absent` in
   `crates/verify/ess-conformance/tests/append_set.rs` lose their
   `#[ignore = "backlog: absent-optional-list-append"]` and pass with their assertions unchanged.
2. The suite of an append onto an absent `Optional<List<…>>` expects the list holding only the sent
   element (or `[null]` in the leave-out run), and a target that appends `null` whatever it is sent
   fails the present-element scenario.
3. `synthesis_suite_bytes_table` moves only rows whose model appends onto an `Optional<List<…>>` its
   creator leaves absent.
