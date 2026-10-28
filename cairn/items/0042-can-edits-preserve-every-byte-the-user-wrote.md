---
id: 42
uid: 5e274286-7ef2-4701-a1f3-c833f762b194
title: Can edits preserve every byte the user wrote?
type: spike
status: done
milestone: v0.2
depends_on:
- 13
created: 2026-09-28
updated: 2026-10-28
closed_at: 2026-10-28
priority: p0
effort: s
area: core
---

## Question

Ticking, setting a field, adding an item, answering and `fmt` must change only the bytes they mean to. Is splicing edits into the source by the parser's byte spans enough, or does the core need a lossless syntax tree with trivia (rowan-style)?

## Timebox

Two days.

## What would change the answer

An edit that needs to move or reindent a subtree (adding a child under a deeply nested item, `fmt`) may be awkward with spans alone.

## Answer

**Splicing by spans is enough. sprig-core does not need a lossless tree with trivia.** Decided 2026-10-28.

The evidence is in the format. In TOML, whitespace and comments sit between tokens of one logical structure, which is why toml_edit hangs a prefix and suffix of trivia off every value. Rowan, rust-analyzer's tree, likewise keeps trivia as tokens in the tree so nothing is lost. Sprig has no such trivia: every byte belongs to exactly one line, and every line is one item, note, answer, comment or blank. The source text is therefore already the lossless layer. An edit is a replacement located by the parser's spans, and everything it does not name is copied through. Indentation style, CRLF, a BOM, comments and blank lines survive by construction, with no tree to keep them in.

The worry in the question was edits that move or reindent a subtree. Neither needs a tree: adding a child inserts one line after the parent's branch, which the line indentations already bound, and `fmt` rewrites each line's leading whitespace from the depth the parser gives it.

Prototype, now `sprig_core::edit`, on `examples/bakery/`, as byte ranges in the original file:

- tick `lease.sprig:13`: bytes 498..499, `-` → `x`.
- tick `bakery.sprig:16` (`every:fri`, on 2026-10-02): bytes 630..631, `2` → `9`; the due date moves and the mark stays.
- add a child under `kitchen.sprig:9` (Deck oven): bytes 467..467, insert `    - Book the delivery  @teo
`, after the oven's last child, at its indentation.
- fmt on `kitchen.sprig` with a typed date, trailing spaces and a stray indent: bytes 58..64 `fri   ` → `2026-10-02`, and bytes 732..733 ` ` → ``. The result equals the original file with the date written out.

Property tests back the decision, in `crates/sprig-core/tests/editing.rs`. Random sequences of mark, tick, field, child, sibling and answer edits over every conformance input and example (run to 20,000 cases) leave every untouched line byte-identical. `fmt` is idempotent and keeps the tree on every input and on 50,000 generated plans. Those tests found two bugs in the prototype, both fixed: a child copying the wrong sibling's indentation, and a comment's level changing on a second format.

## Acceptance criteria

- [x] The Answer includes a prototype of tick, add child and fmt on `examples/bakery/` with byte-level diffs shown.
- [x] The Answer section states a decision and the evidence for it.
- [x] Follow-up items exist for everything the answer implies.

## 2026-10-28

Follow-ups: the edit API itself (0043), `fmt` (0044) and `tick` (0046) are done in the same pull request. The formatting and editing rules a second implementation needs are in `spec/tools.md` (T2, T3). Nothing else is implied: the language server's text edits (0053) and MCP writes (v0.3) take `Change::edits` as they are.

## Result

Span splicing, not a lossless tree: Sprig is line-oriented, so the source is its own trivia store. Prototype and property tests on the bakery.
