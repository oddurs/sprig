# RFC NNNN: The question, as a question

- **Status:** draft | open for review | accepted | declined | withdrawn
- **Opened:** YYYY-MM-DD
- **Review closes:** YYYY-MM-DD (14 days after opening, 7 for a clarification)
- **Decision item:** cairn NNNN, once accepted
- **Spec sections:** §N.N

## The question

What a file, or a reader, cannot tell today. One paragraph, with the smallest
`.sprig` example that shows it.

## The case for

Why the spec should change: what it would let people write, and what it would
stop going wrong. Evidence from the files people write, and from how other
plain-text formats handled the same question, counts for more than taste.

## What it adds to the card

The whole format fits on one card, at the top of `spec/sprig.md`, and that is
what keeps Sprig simple. Say what this proposal adds to it: a line, a row, a
word. The best answer is nothing. A proposal that adds a mark, or a field with
a meaning, raises a number in `crates/sprig-cli/tests/budget.rs`, and needs a
case strong enough to pay for it.

## The case against

The strongest argument you can make against your own proposal. What it costs:
in the grammar, in every tool that reads Sprig, in files that exist today. An
RFC with no case against is not ready for review.

## The ruling

What the spec will say, as close to final wording as you can get it:

- the rules that change, and the new conformance examples that pin them;
- what a 1.x file written before the change means after it, which must be
  what it meant before (`docs/stability.md`);
- any diagnostic it adds, with a new code (codes are never reused).

## What would change the answer

The evidence that would reopen the question, so a later reader knows when
it is worth asking again.
