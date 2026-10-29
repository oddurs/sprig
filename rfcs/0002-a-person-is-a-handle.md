# RFC 0002: Can a person be an email address?

- **Status:** accepted
- **Opened:** 2026-10-28
- **Review closed:** 2026-10-28, as the first RFC, from spike 0069
- **Decision item:** cairn 0149
- **Spec sections:** §4.2

## The question

Does `@dana@example.org` name a person?

```sprig
Lease

- Send the deposit  @dana@example.org
```

## The case for

An address is unambiguous across teams, and a tool could send a reminder to it
without a lookup.

## The case against

- **The tokeniser.** A second `@` would become special inside one token, and
  every tool would have to agree where a name ends.
- **Personal data.** Addresses would land in every repository a plan is
  copied to, and stay in its history.
- **Addresses change.** A plan would be tied to an address that outlives the
  person's use of it.
- **Two names for one person.** Every tool would have to treat `dana` and
  `dana@example.org` as maybe the same person.

Handles have none of these costs. Mapping a handle to an address can live
beside the plans, as git's `.mailmap` and GitHub's CODEOWNERS do.

## The ruling

A person is a handle: `@` followed by letters, digits, `_`, `.` or `-`, and
nothing else (§4.2). `@dana@example.org` and `dana@example.org` are both words,
since `@` is not a name character. As ruled, `@dana` is the person `dana`, and
`@dana,` is a word.

## What would change the answer

Nothing for the token itself. A file that maps handles to addresses, if one is
wanted, is a separate proposal.
