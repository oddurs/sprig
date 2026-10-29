# Requests for comment

A change to what a `.sprig` file means goes through an RFC: a short document
with the same shape as the arguments the format already settled (`cairn list
--view decisions`). It states the question, the case for, the case against, and
the ruling.

## When one is needed

- A new mark, token, key with a defined meaning, or rule in `spec/sprig.md`.
- A change to what any valid file means. After 1.0 that is only possible in a
  2.0 (`docs/stability.md`), and the RFC says why it is worth one.
- A new diagnostic code, or a change to when an existing one is reported.

Not for: a clarification that changes no conformance example (open a pull
request), the tools' behaviour (`spec/tools.md` changes go through ordinary
review), or anything in the implementation.

## How

1. Copy `0000-template.md` to `NNNN-short-title.md` with the next number, and
   open a pull request with it. The status is **open for review**, and the
   pull request is where discussion happens.
2. Review stays open for **14 days** from the pull request's opening, or **7**
   for a clarification. It closes early only if the proposer withdraws.
3. The maintainer decides, in a comment on the pull request that gives the
   reasons. The ruling is the RFC's last section, edited to say what was decided.
   With more than one maintainer, a ruling needs a second maintainer's approval
   of the pull request.
4. An accepted RFC lands with everything it changes in the same pull request:
   - the spec text;
   - new conformance examples;
   - the implementation;
   - a cairn `decision` item that records the ruling and links the RFC.

   A declined RFC is merged as declined, so the argument is not had twice.

## The RFCs

| RFC | Question | Status |
| --- | --- | --- |
| [0001](0001-dates-only.md) | Dates only, or times of day too? | accepted |
| [0002](0002-a-person-is-a-handle.md) | Can a person be an email address? | accepted |
| [0003](0003-no-version-in-files.md) | Should a file declare its spec version? | accepted |
