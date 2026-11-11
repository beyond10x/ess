---
format: aep.planning-md/3
id: review-result:zi-t2-datetime-sec-1
kind: review-result
status: active
title: Turn 2 security review pass 1 on story:the-date-time-agreement-oracle-refuses-non-digits
relations:
- reviews: story:the-date-time-agreement-oracle-refuses-non-digits
revision: 1
---
unit: story:the-date-time-agreement-oracle-refuses-non-digits (commit f4bc600ce9 plus my uncommitted test file)
verdict: holds
new failing cases: none
mutants survived: not run as mutants. My new comparison test with jsonschema's own checker also covers the offset-bound and leap-second lines, which the unit's own tests (`date_time_digits` and the unit test inside `date_time.rs`) only partly cover.

`git --no-pager diff --stat` is empty: no tracked file changed. The only file I added is untracked: `~/.local/state/worktree/trees/b10x/ess/zi-t2-9/crates/generate/ess-synth/tests/security_t2_datetime_conformance.rs`. It has 4 cases, all green, and I left it in place because it checks things the suite did not. The adversary's file `adversary_t2_datetime_rfc3339.rs` was not touched.

**Invariants (from 4 cargo runs):**
- **No panics:** every byte index in `date_time.rs` is checked against the length first, and `digits()` reads at most 4 bytes, so it cannot overflow. I ran 400k random strings, including multibyte characters, plus every one-byte change, deletion and truncation of a valid stamp, plus 7 inputs of 1M characters each. Nothing panicked, and `is_date_time` and `options()` gave the same answer every time.
- **Digit-only strings keep their result:** I compared against jsonschema 0.52.1's own checker (`format.rs:497`) on 300k generated stamps, every leap-second hour × minute × offset combination, and Feb 29 for every year 0000–9999. No disagreements. The new checker never accepts a string jsonschema refused, and anything it now refuses has a non-digit somewhere a digit belongs.
- **Non-digits are refused:** I put all 118 ASCII non-digits plus 6 non-ASCII characters (such as `２` and `٣`) at every digit position of 6 stamps (date, time, fraction and offset digits). All were refused, both by the function and by a validator built from `options()`.
- **The new check runs everywhere the old one did:** jsonschema 0.52.1 tries custom formats before its built-in ones (`format.rs:1697`). `validate.rs:298` turns format checking on, and `ess-cli/src/schema.rs:105` only calls `validate::validate`. The new check fires for a draft-07 schema, a schema with no `$schema`, a `$ref` into another schema file, and array items. `bundle.rs:451`, `source.rs:150` and the emitted runtimes keep format checking off, which jsonschema treats as annotation only (`format.rs:1753`), so their behaviour is unchanged. No other library code in the workspace builds jsonschema options.

**Runs:** case counts went from 52 (implementor's `g1.txt`) to 56; nothing red now.
- Run 1: my cases alone in schema-contract, exit 0.
- Run 2: the schema-contract suite went red, exit 101. My own file caused it: the existing guard `schema_unique_items.rs:347` forbids schema-contract tests from building jsonschema validators directly. The unit's code is not at fault. I moved the file to ess-synth.
- Run 3: my cases alone in ess-synth, 4 passed, exit 0.
- Run 4: schema-contract `--lib`, `bundle`, `date_time_digits` and `schema_unique_items`, 28+17+1+6 passed, exit 0.

**Findings (both side notes, not in the story's acceptance):**

| file:line | verdict | origin | what reaches it |
|---|---|---|---|
| `crates/generate/ess-synth/tests/adversary_w1_gen1_adversary.rs:76` | CONFIRMED | introduced | The doc comment says its oracle is "the validator the unit's own agreement test uses". The agreement test now builds from `uniqueness::options()`, but line 80 still builds `jsonschema::draft202012::options()`. Only tests reach it. |
| `crates/generate/ess-synth/tests/adversary_w1_gen1_security.rs:516` | CONFIRMED | pre-existing (same code at base 7623ee5ec1) | `document_admits` still compares server 400s against jsonschema's own date-time checker, which accepts non-digits in some digit positions. A future request body with such a non-digit would fail there spuriously. Only tests reach it. |

**Written outside the worktree:** `~/.cache/scratch/e368f6dc-c92b-425c-8fdf-280e82792de9/ess/waves/t2/the-date-time-agreement-oracle-refuses-non-digits/sec/run1.txt`, `run2.txt`, `run3.txt` and `run4.txt` in that same `sec/` directory.

```findings
- file: crates/generate/ess-synth/tests/adversary_w1_gen1_adversary.rs
  line: 76
  category: contract-drift
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: the comment claims this oracle is the agreement test's validator, but it still builds jsonschema's own checker while json_body_closure now uses uniqueness::options()
- file: crates/generate/ess-synth/tests/adversary_w1_gen1_security.rs
  line: 516
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: pre-existing
  message: document_admits compares the server against jsonschema's own date-time checker, which accepts non-digits in some digit positions, instead of uniqueness::options()
```
