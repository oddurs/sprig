---
id: 145
uid: abee43c0-cba8-49b2-b835-91f967cdcf14
title: 'Spec draft 0.5: a ruling on every known ambiguity'
type: docs
status: done
milestone: v0.2
created: 2026-10-28
updated: 2026-10-28
closed_at: 2026-10-28
priority: p1
area: spec
effort: m
---

## Reader and question

Anyone writing a second Sprig parser asks what each corner of the format means, and draft 0.4 answered fifteen of those questions with "the reference parser does one thing, sprig-core another" (Appendix B). Writing the conformance suite found more places where the text was silent.

## Change

- **Rulings on B.1 to B.15.** They are folded into the numbered rules, and each Appendix B entry keeps its number and records its resolution.
- **Behaviour made explicit**, with conformance examples for each:
  - how quoted runs split, and what an unclosed quote does;
  - `key:""` and `key:a:b`;
  - letters of any script;
  - a link must name a file and holds no whitespace;
  - an escaped title;
  - `X` is not a mark;
  - answers attach to the root, take nothing beneath them, and an empty `=` answers;
  - tabs and other whitespace in indentation;
  - shortcuts ignore case, and `+0d` is today;
  - the `est:` and `every:` grammars;
  - month-end recurrence;
  - due dates and priority reach down (§9.6, §4.4);
  - `§1.2`'s example named the retired `target` field.
- **One change of meaning:** a dropped item no longer holds up what waits on it (§5.7, decision 0141).
- **Decisions** 0141 to 0144: dropped references, recurring checklists (B.6), empty is never finished (B.5), and loops found once (B.10).
- **New warnings:**
  - S0006, an `every:` that is not a recurrence;
  - S0206, an `after:` naming something with nothing to do.
- **Fix:** an item's plain text drops brackets only from real links (B.16).

## Acceptance criteria

- [x] Every Appendix B entry records a resolution, and its rule says it.
- [x] Each rule added or changed has a conformance example.
- [x] Every example in the spec parses without an error, held by a test.

## 2026-10-28

Every Appendix B entry now ends in a resolution that names its rule and its conformance example, and a sixteenth records the bracket fix. There are 17 new examples: four in `lines/`, ten in `tokens/` and three in `graph/`. Two existing examples were re-blessed for the change of meaning: `graph/016` gains S0206, and `graph/023` no longer waits on the dropped item.

Two rules can't show in the tree format, which records an item's own date and priority: effective priority (§4.4) and effective due dates (§9.6). They are held by `next.rs`'s `in_progress_comes_first_then_priority_then_due_then_order`. Month-end recurrence (§10.4) is held by `date.rs`.

`crates/sprig-core/tests/spec_examples.rs` reads every `sprig` block in the spec. It fails if any block has an error, or if §5.7's dropped example waits.

Spikes 0068, 0069 and 0070 were answered in the same draft:
- §9.2: a date has no time of day.
- §4.2: an email address is a word.
- §4.6: the title's `sprig:` key is reserved, with S0007.

## Result

Draft 0.5 rules on all fifteen open ambiguities and on the cases the suite found silent. One meaning changed: a dropped item finishes references. Two warnings were added for silent traps, and three spikes were answered.
