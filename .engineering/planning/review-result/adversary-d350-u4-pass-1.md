---
format: aep.planning-md/3
id: review-result:adversary-d350-u4-pass-1
kind: review-result
status: active
title: 'D350 U4 adversary pass 1: the published schema admits the name aliases'
relations:
- reviews: story:the-published-schema-admits-the-name-aliases-the-parser-reads
revision: 1
---
unit: D350 U4 (story:the-published-schema-admits-the-name-aliases-the-parser-reads), unit commit 4198b2241 on unit/ess-d350-u4, reviewed against bf5cf1873
verdict: CONFIRMED (1 warning, 1 note)
cases: executed 0→3 (run at "go" by the implementor at 813ee8838 after the F1 fix), red 0
origin: introduced 2 / pre-existing 0 / undecided 0
wrote-outside-worktree: the wave scratch directory u4/adv-scratch (read-only pass under the cargo hold)
needs-coordinator: the cases could not run under the 13:26Z hold; the implementor ran them at go

**Cases** in `crates/specify/ess-domain/tests/adversary_d350_u4_pass1.rs` (3): a whole-repository case over every committed spec document; case 2, an unknown key beside each name spelling, which only the repeated `additionalProperties: false` at `name.rs:229` refuses; a third alias case. Run at go: 3 passed beside published_name_aliases 5 passed. One lint fix by the implementor (`Path::extension()` instead of `ends_with`), the cases unchanged.

**Findings (from reading, confirmed by the run after the fix):**
- F1 (warning, introduced, acceptance), `crates/specify/ess-domain/tests/published_name_aliases.rs:101` (walk at :80): the corpus test walked only `crates/` and `examples/` and skipped dot directories, leaving 148 committed spec documents (models/, docs/design/, fuzz/, .engineering/repro/) unchecked, 20 of them spelling names `id:`/`component:`. Fixed at 813ee8838: `git ls-files`, 612 documents read (was 474), only the two named exceptions refused.
- F2 (note, introduced, mutant), `crates/specify/ess-domain/src/name.rs:229`: the repeated `additionalProperties: false` survived every ess-domain test; case 2 now guards it.

**Not broken:** the alias charsets, the exactly-one-spelling rule, parsing.

```findings
[
{"file":"crates/specify/ess-domain/tests/published_name_aliases.rs","line":101,"category":"acceptance","severity":"warning","verdict":"CONFIRMED","origin":"introduced","message":"the corpus test walks only crates/ and examples/ and skips dot directories, so 148 committed spec documents (models/, docs/design/, fuzz/, .engineering/repro/) go unchecked"},
{"file":"crates/specify/ess-domain/src/name.rs","line":229,"category":"mutant","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"deleting the repeated additionalProperties false line leaves every ess-domain test green"}
]
```
