---
format: aep.planning-md/3
id: review-result:adversary-ui-docs-pass-2
kind: review-result
status: active
title: Adversary pass 2, ess-ui wave unit ui-spec-docs-generator
relations:
- reviews: story:ui-spec-docs-generator
revision: 1
---
unit: story:ui-spec-docs-generator, uncommitted working tree ~/.local/state/worktree/trees/b10x/ess/ess-ui-docs (HEAD c0f777f3c + crate crates/ui/ess-ui-docs after correction 1 + coordinator patches)
verdict: NEEDS-CHANGE
cases: executed 34→47, red 9
origin: introduced 9 / pre-existing 0 / undecided 0
wrote-outside-worktree: 3 paths + 1 fixture dir (see part 6)
needs-coordinator: none

1. git --no-pager diff --stat (tracked files; all six are the coordinator's patches, unchanged by me)
    Cargo.lock | 9, Cargo.toml | 1, crates/edge/ess-xtask/src/docs.rs | 1, schemas/ui/ess-ui.schema.yaml | 72, website/docs/reference/formats.md | 1, website/sidebars.ts | 1
   My only write in the tree: new file crates/ui/ess-ui-docs/tests/adversary_pass2.rs (inside the untracked crate dir, so absent from --stat). No implementation file touched. rustfmt ran on that one file only.

2. Cases added (crates/ui/ess-ui-docs/tests/adversary_pass2.rs, 13 tests). Red output below is from the first run of this binary alone (`cargo test -p ess-ui-docs --test adversary_pass2`, log adv2-red.log, EXIT=101, 4 passed / 9 failed).
   RED:
   | test | red output, verbatim |
   |---|---|
   | a_blank_string_example_is_refused_like_an_empty_map | Page with `example: ""` renders; it documents no example |
   | a_blank_format_is_refused_like_a_missing_one | `format: ""` renders; the page title is `#  reference` |
   | a_shorthand_without_an_expansion_is_refused_not_shown_as_null | a shorthand with no `expands_to` renders; its "It means" cell reads: … <td><code>title</code></td><td><code>string</code></td><td><code>null</code></td> |
   | an_absent_foundation_text_never_prints_null | the page prints `null` for text the schema never wrote: ["no type_rule.summary", "a layer without a note", "an expression form without a note"] |
   | a_type_that_admits_nothing_is_refused | {one_of: []}: \| `title` \| one of:  \|  …; {enum: []}: \| `title` \| one of:  \| …; {record: {}}: \| `title` \| record \{  \} \| … |
   | a_name_with_no_slug_is_refused | "🙂" -> id=""; "" -> id=""; "   " -> id="---"; group title "???" -> id="" |
   | a_chapter_title_with_underscores_links_to_the_id_the_site_derives | heading `## The _core_ document` gets the site id `the-core-document`; the link says `#the-_core_-document` |
   | front_matter_carries_the_format_verbatim | "ess-ui: 1": front matter is not YAML: mapping values are not allowed in this context at line 1 column 14; "\"quoted\" ui": front matter is not YAML: did not find expected key at line 1 column 17; "ui #1": title reads "ui" |
   | a_document_given_as_the_schema_is_refused_without_blaming_a_comma | 3 of 7 problems tell the user to quote text in a file that is not a schema: pages.deals.saved.header: `filters` has no value; an unquoted comma in a flow mapping split the text before it (quote that text) … (then) the schema has no `type_rule.primitives` … |
   GREEN (attacked, held):
   - names_that_collide_after_slugging_are_refused: "Page🙂", "PAGE", "page", "ess-ui-filter", "Ess ui filter" all refused beside Page / the reserved filter id
   - a_long_unicode_name_links_to_its_own_section: 407-char non-ASCII name, section id and href agree
   - every_html_id_is_unique_and_the_filter_script_finds_its_input: fixture and real schema; one id="ess-ui-filter", script targets it
   - check_accepts_only_the_exact_bytes_and_writes_nothing: CRLF, missing and extra trailing newline all fail `--check` with "not a fresh render"; the file is byte-unchanged; exact bytes pass

3. Suite: CARGO_TARGET_DIR=$HOME/.cache/b10x-target/ess-ui-docs CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 cargo test -p ess-ui-docs --no-fail-fast → EXIT=101 (log adv2-suite.log)
     unittests 2 ok; adversary_pass1 16 ok; adversary_pass2 4 passed 9 failed; reference 6 ok; refusals 10 ok; doctests 0
   Before = 2+16+6+10 = 34: this same run with adversary_pass2 left out. After = 47.
   clippy -p ess-ui-docs --all-targets -D warnings: Finished (clean). fmt -p ess-ui-docs --check: clean.

4. Findings (tree above; origin: the crate is new in this unit, absent at c0f777f3c)
   | # | file:line | verdict | what was measured | what reaches it |
   |---|---|---|---|---|
   | 1 | src/model.rs:524 | NEEDS-CHANGE | `example: ""` / `"   "` renders; `is_empty` treats only null, {} and [] as empty. Residue of correction 1 | a schema edit; acceptance names "lacks an example". Fix: a blank string counts as empty |
   | 2 | src/markdown.rs:16 | INFEASIBLE | `format` interpolated raw into front matter: `:` and leading `"` make it invalid YAML, ` #` truncates the title | only `--schema` with a non-`ess-ui/1` format. Fix: write title/sidebar_label/description as YAML-quoted scalars |
   | 3 | src/markdown.rs:118 | INFEASIBLE | `heading()` leaves `_x_` unescaped; the site renders emphasis and derives `the-core-document`, links say `#the-_core_-document` (label text is escaped, heading is not) | a group title with `_word_`; none today. Fix: escape `_` (and `*`) in headings as in `escape` |
   | 4 | src/doc.rs:82 | INFEASIBLE | `prose_of(Null)` prints `<code>null</code>` for absent type_rule.summary/doc, layer note, expression-form note | none today; all present. Fix: refuse, or omit the paragraph/cell |
   | 5 | src/doc.rs:386 | INFEASIBLE | a shorthand without `expands_to` renders "It means: null" | none today. Fix: refuse a shorthand lacking `expands_to` |
   | 6 | src/model.rs:70 | INFEASIBLE | blank `format` accepted (title "  reference") | only `--schema`. Fix: `text_of` like every other required text |
   | 7 | src/model.rs:241 | INFEASIBLE | `{one_of: []}`, `{enum: []}`, `{record: {}}` pass and render "one of: " / "record {  }" | none today. Fix: refuse an empty alternative list / field map |
   | 8 | src/doc.rs:149 | INFEASIBLE | a name or title with no slug characters gets id="" (or "---" for blank), links `href="#"`; a blank construct name is not refused | none today. Fix: refuse an empty slug and a blank construct name |
   | 9 | src/model.rs:69 | CONFIRMED | `--schema examples/partner-portal/ui.yaml` is refused without panic, but its first 3 of 7 problems say "unquoted comma … quote that text" for the document's deliberate `{filters: null}` | the documented `--schema` flag given the example document, the scenario named in the brief. Fix: check the schema shape (type_rule, groups, constructs) first and stop before `split_text` when absent |

   Brief question, notes on non-required fields: missing/blank notes are refused on every field. The schema's own header (line 5-6: "fields (each with a one-line `note`)") says that, so this is not a finding.

5. Attacked, could not break:
   - Slug collisions after unicode, emoji and case folding: all refused.
   - The reserved id: `ess-ui-filter` and "Ess ui filter" are refused, and every HTML id is unique.
   - Long non-ASCII anchors: ids and links agree.
   - `--check`: strict about bytes (CRLF, trailing newline) and never writes.
   - `--schema` with a document: no panic, and no output is written on refusal, in both write mode and `--check` mode.
   - Blank values nested in a record/one_of type: already refused as "neither a primitive nor a construct". Record fields carry types, not notes.
   - Blank field note and blank group text are refused (refusals.rs, correction 1).

6. Written outside the worktree:
   - ~/.cache/ess-ui-wave/docs/adv2-red.log
   - ~/.cache/ess-ui-wave/docs/adv2-suite.log
   - ~/.cache/ess-ui-wave/docs/adv2-review.md (this file)
   - ~/.cache/b10x-target/ess-ui-docs/tmp/ess-ui-docs-adversary-pass2/ (test fixtures via CARGO_TARGET_TMPDIR, inside the assigned build dir)

7.
```findings
- file: crates/ui/ess-ui-docs/src/model.rs
  line: 524
  category: acceptance
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: a blank string example renders instead of being refused as lacking an example
- file: crates/ui/ess-ui-docs/src/markdown.rs
  line: 16
  category: boundary
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: the format is written unquoted into front matter, so a colon or leading quote breaks the YAML and " #" truncates the title
- file: crates/ui/ess-ui-docs/src/markdown.rs
  line: 118
  category: contract-drift
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: underscores in a heading render as emphasis on the site, so its id differs from the slug every link uses
- file: crates/ui/ess-ui-docs/src/doc.rs
  line: 82
  category: boundary
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: an absent foundation text (type_rule summary, layer note, expression-form note) is printed as null
- file: crates/ui/ess-ui-docs/src/doc.rs
  line: 386
  category: boundary
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: a shorthand without expands_to renders with the expansion null instead of being refused
- file: crates/ui/ess-ui-docs/src/model.rs
  line: 70
  category: boundary
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: a blank format is accepted while every other required text refuses blank
- file: crates/ui/ess-ui-docs/src/model.rs
  line: 241
  category: boundary
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: an empty one_of, enum or record type is accepted and renders with no alternatives
- file: crates/ui/ess-ui-docs/src/doc.rs
  line: 149
  category: boundary
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: a heading with no slug characters gets an empty id and links to the page top
- file: crates/ui/ess-ui-docs/src/model.rs
  line: 69
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: a document passed as --schema is refused with comma-split advice before the missing-schema problems
```
