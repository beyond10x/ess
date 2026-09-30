---
format: aep.planning-md/3
id: review-result:adversary-ui-docs-pass-1
kind: review-result
status: active
title: Adversary pass 1, ess-ui wave unit ui-spec-docs-generator
relations:
- reviews: story:ui-spec-docs-generator
revision: 1
---
unit: story:ui-spec-docs-generator, uncommitted working tree ~/.local/state/worktree/trees/b10x/ess/ess-ui-docs (HEAD c0f777f3c + crate crates/ui/ess-ui-docs + coordinator patches)
verdict: NEEDS-CHANGE
cases: executed 17→33, red 2
origin: introduced 2 / pre-existing 0 / undecided 0
wrote-outside-worktree: 6 paths (see part 6)
needs-coordinator: none

1. git --no-pager diff --stat (tracked files; all six are the coordinator's patches, none mine)
    Cargo.lock | 9, Cargo.toml | 1, crates/edge/ess-xtask/src/docs.rs | 1, schemas/ui/ess-ui.schema.yaml | 72, website/docs/reference/formats.md | 1, website/sidebars.ts | 1
   My only write in the tree: new untracked test file crates/ui/ess-ui-docs/tests/adversary_pass1.rs (the crate dir is untracked, so it does not show in --stat). No implementation file touched.

2. Cases added (crates/ui/ess-ui-docs/tests/adversary_pass1.rs, 16 tests)
   RED now:
   - an_empty_summary_doc_or_example_is_refused_like_a_missing_one (:582). Red output, verbatim:
       Page with `summary: ""` renders; it documents no summary
       Page with `summary: "   "` renders; it documents no summary
       Page with `doc: ""` renders; it documents no doc
       Page with `example: {}` renders; it documents no example
   - a_construct_named_filter_does_not_share_its_id_with_the_sidebar_input (:630). Red output, verbatim:
       assertion `left == right` failed: the page carries id="filter" 2 times
   GREEN now (attacked, held):
   - every_string_the_schema_declares_is_on_the_html_page / _markdown_page: every string leaf and non-structural key of the schema appears in the page text
   - every_indexed_shorthand_is_shown_on_the_construct_it_names: all 17 shorthands.index entries
   - every_html_link_lands_on_an_id_of_the_page; every_markdown_link_lands_on_a_heading_the_site_generates (github-slugger rules, with dedup)
   - rendering_twice_gives_identical_bytes; the_committed_markdown_equals_a_fresh_render_through_run (run --check, embedded schema)
   - a_highlighted_example_reads_back_as_the_construct_example (tags stripped, entities decoded, parsed = example, every construct); same for Markdown fences
   - schema_text_is_escaped_on_the_html_page; schema_text_is_mdx_safe_on_the_markdown_page (script tag, &, {y}, HTML comment, a<b in 9 positions; front matter parses)
   - a_deeply_nested_type_reads_as_words_with_links
   - a_dangling_type_inside_a_shorthand_record_is_refused; a_key_with_no_value_is_refused_naming_its_path
   Test-harness corrections before the final red (not findings): 3 first-run reds were my own false positives (label-humanised keys such as exactly_one_of, a `-` token from block YAML, Markdown link syntax). Fixed in the test. Logs adv1-red.log to adv1-red4.log.

3. Suite: CARGO_TARGET_DIR=$HOME/.cache/b10x-target/ess-ui-docs … cargo test -p ess-ui-docs --no-fail-fast → EXIT=101
     unittests 2 passed; adversary_pass1 14 passed 2 failed; reference 6 passed; refusals 9 passed
   Before = 2+6+9 = 17: the per-binary counts of this same run, leaving out adversary_pass1. After = 33.

4. Findings
   | # | file:line | verdict | origin | what was measured | what reaches it |
   | 1 | crates/ui/ess-ui-docs/src/model.rs:443 (and :458) | NEEDS-CHANGE | introduced | `summary: ""`, `summary: "   "`, `doc: ""` and `example: {}` all render; the "lacks a summary/doc/example" refusal checks only presence and type | any schema edit that blanks the text; the acceptance says a construct lacking a summary, doc or example fails. Fix: refuse a blank (trimmed-empty) string and an empty mapping or list as the example |
   | 2 | crates/ui/ess-ui-docs/src/html.rs:130 | INFEASIBLE | introduced | a construct named `Filter` gets id="filter", the same id as the sidebar search input; the uniqueness check in doc.rs:145 does not know about it, so #filter links land on the input | nothing today: the schema has no construct or heading that slugs to `filter` (there is `filter_bar`). Fix: reserve `filter` in the anchor set, or give the input a non-slug id such as `ess-ui-filter` |

   Covers: the uncommitted working tree above.

5. Attacked, could not break: silent drops (every schema string reaches both pages), type rendering for list/map/optional/one_of/record/ref/const/enum/unique and deep nesting, HTML link integrity, Markdown link integrity against site slugs, HTML escaping, MDX safety, front-matter validity, determinism, committed Markdown = fresh render, highlighting does not change the example text, refusal of a dangling type (including inside a shorthand record) and of a key with no value.

6. Written outside the worktree:
   - ~/.cache/ess-ui-wave/docs/adv1-red.log, adv1-red2.log, adv1-red3.log, adv1-red4.log, adv1-suite.log
   - ~/.cache/ess-ui-wave/docs/adv1-review.md (this file)
   - build output under the assigned $HOME/.cache/b10x-target/ess-ui-docs (shared, pre-existing)

7.
```findings
- file: crates/ui/ess-ui-docs/src/model.rs
  line: 443
  category: acceptance
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: a construct whose summary or doc is blank or whose example is an empty map renders instead of being refused as lacking it
- file: crates/ui/ess-ui-docs/src/html.rs
  line: 130
  category: boundary
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: the sidebar input's id "filter" is outside the anchor-uniqueness check, so a construct or heading slugging to filter would produce a duplicate id
```
