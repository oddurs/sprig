# RFC 0003: Should a file declare its spec version?

- **Status:** accepted
- **Opened:** 2026-10-28
- **Review closed:** 2026-10-28, as the first RFC, from spike 0070
- **Decision item:** cairn 0150
- **Spec sections:** §4.6

## The question

Should a file say which version of the spec it was written for?

```sprig
Launch  sprig:1

- Print the menus
```

## The case for

A tool would know what a file expects, and could refuse a file written for a
version it does not read, rather than misreading it.

## The case against

- **Noise for nothing.** Every file would carry a line that means nothing
  until a breaking change, and tools would be tempted to refuse files over it.
- **Other formats chose none.** CommonMark never adopted a per-document
  version, though one was proposed, and its IANA registration always means the
  latest version. TOML, JSON and todo.txt carry none either. HTML's doctype
  ended as a fixed string.
- **Sprig already degrades safely.** An unknown key is an ordinary field and an
  unknown mark is a note, so additions in 1.x reach old tools as text, not as
  errors.

## The ruling

A file needs no version, and one without is read by the current version.
Every 1.x file means the same to every 1.x tool (`docs/stability.md`).

The title's `sprig:` key is reserved (§4.6). `sprig:1` changes nothing. A tool
that meets a version it does not read warns with S0007, then reads the file as
best it can. The reserved key costs nothing until a 2.0 needs it, and then a
1.x tool sees it and says so rather than misreading silently.

## What would change the answer

A 2.0.
