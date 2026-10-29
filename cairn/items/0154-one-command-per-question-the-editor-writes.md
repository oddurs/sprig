---
id: 154
uid: 647a5ddf-9a2e-443b-b77e-200b16c11660
title: One command per question; the editor writes
type: decision
status: done
milestone: v1.0
created: 2026-10-29
updated: 2026-10-29
closed_at: 2026-10-29
priority: p0
area: cli
---

## Context

The `sprig` command grew to 22 commands, most with six or seven flags, while the format it reads stayed at nine marks. `docs/stability.md` promises that after 1.0 nothing is removed, so the surface 1.0 ships with is the surface forever. Before 1.0, cutting costs nothing.

## Options and tradeoffs

The rule proposed: one command per question a person asks of a plan, with filters and formats as flags; writing is done in an editor, because the file is the truth, and the command keeps only the one write that is fiddly by hand (ticking, which settles recurrence and dates). Each cut, with the case against it:

- **`status` → `tree --summary`.** It is the tree, one row per file. Against: a short, memorable name for the most common glance. `--oneline` stays for prompts.
- **`agenda` → `find is:open 'due:<=+2w'`.** It is a search for dated work. Against: the agenda groups by day; the TUI keeps that view, and `export --format ics` serves calendars.
- **`mark` → `tick --mark <M>`.** Setting a mark is the same act as ticking. Against: `tick ~` reads oddly; `--mark` names it.
- **`answer`, `add`, `edit` → the editor.** Each is one line typed in a file. Against: agents without an editor; they have MCP, whose `add`, `answer` and `mark` tools stay, checked and diffed.
- **`graph` → `export --format dot|mermaid`.** It is another export format. Against: discoverability; `export --help` lists it.
- **`agent` → `mcp --instructions`.** One entry point for agents, beside `mcp --config`. Against: `sprig agent` is shorter.
- **`completions`, `man` → `generate <shell|man>`.** Install-time output, one command, as ripgrep has it. Against: none worth keeping two names for.

Kept: `next`, `tree`, `why`, `check`, `fmt`, `tick`, `find`, `export`, `import`, `lsp`, `mcp`, `tui`, `generate`. Thirteen. `why` stays because "why is this blocked" is its own question and its answer is a chain, not a tree.

## Decision

Adopted as above. A removed command exits 2 and names its replacement, until 1.0, when the table goes. The simplicity budget (`crates/sprig-cli/tests/budget.rs`) pins the count, so a fourteenth command is a visible edit.

## Revisit when

A question people ask of plans has no command and no filter that answers it.

## Acceptance criteria

- [x] The choice, its evidence and its consequences are recorded.
- [x] The CLI, its tests, the docs, the site and the action use only the thirteen commands.

## 2026-10-29

Implemented in the same pull request. One finding on the way: tick now reports what a change unblocked, as mark and answer did, and the first version printed it on standard output, which broke next --plain | cut -f1 | sprig tick - by adding lines that were not refs. The report goes to standard error, where git puts its notes, and standard output stays a line per ref. The agenda's replacement is find is:open 'due:<=+2w', not find 'due:<=+2w': without is:open it lists done work too, which the agenda never did.

## Result

Thirteen commands, each answering one question; the editor writes, and tick is the one write with a command.
