---
id: 12
uid: 711fe65d-42b2-4081-ae90-91fce2a03e58
title: Diagnostics with stable codes
type: feature
status: done
milestone: v0.1
depends_on:
- 8
created: 2026-09-28
updated: 2026-10-28
closed_at: 2026-10-28
priority: p0
effort: s
area: core
---

## Problem

`sprig check`, the language server and the MCP server all report problems. If each formats its own, codes and wording drift and nobody can search for an error.

## Proposal

One `Diagnostic` type in the core: code (`S0001`...), severity, file, span, message, optional help. The core returns diagnostics and never panics on user input. The catalogue lives in `spec/diagnostics.md`.

## Acceptance criteria

- [x] Every diagnostic the core can emit has a code listed in `spec/diagnostics.md`, checked by a test that compares the two.
- [x] Display renders `file:line:col: error[S0003]: message`.
- [x] Codes are never reused; the catalogue says so.

## 2026-10-28

`Diagnostic` in `sprig-core/src/diagnostic.rs`: code, file, byte span, 1-based line, character column, message, optional help. 15 codes, from S0001 to S0205, in three ranges: line problems, grafts and references. (1) `tests/catalogue.rs` compares `Code::ALL` with the table in `spec/diagnostics.md` both ways, including severity and the rule each code cites. (2) `Display` renders `file:line:col: severity[code]: message`, tested in `diagnostic::tests` and end to end in the CLI's check tests. (3) The catalogue's first rule says codes are never reused and a retired code keeps its row; the catalogue test refuses a row the core lacks, so a retirement has to be explicit.

## Result

One `Diagnostic` type and a 15-code catalogue in `spec/diagnostics.md`, held to each other by a test. User input never panics: see 0014.
