---
id: 70
uid: 9be5c4a5-e2f9-4f76-8887-9ce188f88a96
title: Should a file declare its spec version?
type: spike
status: done
milestone: v0.3
depends_on:
- 7
created: 2026-09-28
updated: 2026-10-28
closed_at: 2026-10-28
priority: p1
effort: s
area: spec
---

## Question

Should a file be able to say which spec version it follows (for example a `sprig:1` field on the title line)? It helps a future 2.0; it costs noise in every file.

## Timebox

One day.

## What would change the answer

Any change planned after 1.0 that a 1.0 parser would misread.

## Answer

**No version in 1.x files, and one reserved key for a future major version.**

- A file without a version is read by the current version of the spec.
- Every 1.x file means the same to every 1.x tool, so a file never needs to say which 1.x it follows.
- The title's `sprig:` field is reserved (§4.6). `sprig:1` changes nothing. A tool that meets a version it does not read warns (S0007) and reads the file as best it can.

Evidence:
- CommonMark never adopted a per-document version declaration, though one was proposed, and its IANA registration always means the latest version.
- TOML, JSON and todo.txt carry no version in files either.
- HTML's doctype ended as a fixed string.

Compatibility comes from never changing what a valid file means. Sprig's grammar already degrades safely: an unknown key is an ordinary field and an unknown mark is a note, so additions in 1.x reach old tools as text, not as errors. The reserved key costs nothing until a 2.0 needs it, and then a 1.x tool sees it and says so rather than misreading silently.

## Acceptance criteria

- [x] The Answer states what a parser must do with a file that has no version and one with an unknown version.
- [x] The Answer section states a decision and the evidence for it.
- [x] Follow-up items exist for everything the answer implies.

## 2026-10-28

Spec §4.6 reserves `sprig`. Diagnostic S0007 is added and catalogued, and `tokens/064` and `tokens/065` hold it. Follow-up: the stability policy (0073) should promise that 1.x never changes what a valid file means; a note there records it.
