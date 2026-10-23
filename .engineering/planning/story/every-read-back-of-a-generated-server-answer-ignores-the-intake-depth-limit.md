---
format: aep.planning-md/3
id: story:every-read-back-of-a-generated-server-answer-ignores-the-intake-depth-limit
kind: story
status: draft
title: Every read-back of a generated server's own answer ignores the intake depth limit
summary: Rust single() and the Go envelope read-backs read the server's own output at any depth, like entry::read.
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 1
---
## Need

Found while landing `story:a-generated-rust-server-reads-its-own-answer-without-the-intake-depth-limit`
(0.69.0). Two more places in the generated servers read back output the server wrote itself with
the depth-limited intake reader. Neither panics; both give a wrong answer for a deep value:

- Rust `single` in `crates/generate/ess-synth/src/rust/http/bound.rs` (~592) reads a view's rows
  with `parse(rows).unwrap_or(Null)`, so a deep `Json` column gives an empty answer.
- The generated Go target refuses past `jsonDepth` (`crates/generate/ess-synth/src/go/http/bound.rs`
  ~353), and `readEnvelope` (~933) and the read at ~884 read back the server's own output, so a deep
  envelope becomes nil or `null`.

## Acceptance

- A view row whose `Json` column nests as deep as intake admits is answered with that value by the
  generated Rust `single`, not `null`; a test drives it.
- The generated Go server answers the same deep command and view as the Rust one; a test drives both
  read-backs.
- Intake depth and its refusal body stay unchanged in both targets.
