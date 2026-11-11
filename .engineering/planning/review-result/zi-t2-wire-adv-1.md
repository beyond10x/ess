---
format: aep.planning-md/3
id: review-result:zi-t2-wire-adv-1
kind: review-result
status: active
title: Turn 2 adversary pass 1 on story:the-specification-checks-command-response-wire-names
relations:
- reviews: story:the-specification-checks-command-response-wire-names
revision: 1
---
unit: story:the-specification-checks-command-response-wire-names
verdict: holds
new failing cases: none
mutants survived: none

- **One test run, all passing.** Command: `build.sh adv/run1.txt <tree> test -p ess-domain --locked --test adversary_t2_response_wire_names`. Result: `test result: ok. 5 passed; 0 failed`, `exit 0`, `slot B waited 271s ran 16s`. The outfile is `~/.cache/scratch/e368f6dc-c92b-425c-8fdf-280e82792de9/ess/waves/t2/the-specification-checks-command-response-wire-names/adv/run1.txt`.
- **New test file, uncommitted:** `~/.local/state/worktree/trees/b10x/ess/zi-t2-10/crates/specify/ess-domain/tests/adversary_t2_response_wire_names.rs`. I left it in place because it covers things the unit's suite did not:
  - the field-level `wire:` spelling (not just `naming: {wire:}`) collides in the response;
  - two renamed `Optional` fields on one wire name are refused, and the message names `response[0]`;
  - when there are several collisions, each later one is reported separately (`response[2]` and `response[3]`), and each names the first field;
  - `foo_bar`, `fooBar` and `FooBar` are separate wire names and still validate;
  - a repeated response field name is reported the same number of times as a repeated input field name.
- **Response shapes the check might not see:** none. `CommandSpec.response` is always a `Vec<Field>`; there is no single-type-reference shorthand. A struct type used as a field is already checked by the `types.*.fields` pass. Every generator works out the wire name the same way, as `naming.wire` or else the field name, with no casing applied. The Go and Rust served answers put the response under its own `response` member, so it cannot collide with `outcome`, `error` or `published`.
- **Commands from other documents:** none escape. `wire::validate` goes through `spec.commands()` after assembly, so it does not matter which document declared the command. I dropped the multi-document test because I was unsure of the document layout, so this is from reading the code, not from a test.
- **Committed models that might now fail:** none found. I scanned every YAML file, markdown code block and raw-string Rust model in the tree with `adv/scan.py`: 115 commands with a response list, and the only collision is the story's own repro. Limit: `format!` templates with placeholders such as `{input}` probably did not parse, so models built that way were not scanned.
- **Side note, not a defect of this unit (backlog at most):** two response fields with the same name get two `duplicate_declaration` errors, one from `field_shape_at` and one from the wire check. A repeated input name gets the same pair, so the behaviour is consistent; the parity test pins that.
