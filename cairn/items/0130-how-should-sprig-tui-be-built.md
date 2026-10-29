---
id: 130
uid: e14bfb55-4a64-433e-9683-87515a2517db
title: How should sprig tui be built?
type: spike
status: done
milestone: v0.4
depends_on:
- 43
- 119
created: 2026-09-28
updated: 2026-10-29
closed_at: 2026-10-29
priority: p0
effort: s
area: cli
---

## Question

What shape does the TUI take so that it is testable without a terminal, shares every write with the CLI, and stays safe when an editor or an agent changes a file it has open?

## Timebox

Two days.

## What would change the answer

- Whether harrow's split holds for a list-heavy UI: a pure core returns actions, and a thin ratatui and crossterm shell performs them.
- Whether `notify` file watching is reliable on macOS, Linux and Windows for files saved by rename, as Vim and many editors do.
- What happens when a write races an external save: refuse, re-resolve the ref, or merge.

## Answer

**Decision:** `sprig tui` is built as harrow is. It is a module of `sprig-cli` (`src/tui/`) with three parts:
- `app.rs`, the pure core. It holds the state and handles keys, and returns actions instead of performing them.
- `ui.rs`, one draw function from that state to a ratatui frame.
- `mod.rs`, the shell. It turns crossterm events into keys, performs the actions, and watches the folder with `notify`.

**Races are refused, not merged.** spec/tools.md T7 is the contract.

- **The split holds for a list-heavy UI.** Every key in the help is tested by driving the core, with no terminal (`tui::app::tests`). Every view has a snapshot rendered by the same draw function through ratatui's `TestBackend`, which is what `--screenshot` prints. `--keys` feeds key presses through the same core, and its writes through the same shell code. A session of ticks, a mark, an add and undos is checked against `git diff` on a copy of the bakery.
- **Writes carry what they expect.** A key that writes returns the edit, either a tick, a mark or an add, together with the text its line had when the TUI showed it. The shell reads the file again just before writing, and makes the edit only if that line still says that.
  - When the line has changed, the write is refused, nothing is written, and the screen reloads. Nothing a person or an agent saved is lost, and the next key aims at what the file says now.
  - Re-resolving the ref was rejected: a line whose text changed may no longer be the item the person meant.
  - Merging was rejected: the edits are one line each, so redoing one costs a key.
  - A change to another line doesn't stop the write: the edit is applied to the new text.
  - An undo goes further and needs the whole file unchanged since this session wrote it.
- **The watcher watches the folder, not the files.** Watching each file would lose it when a save renames a new file over the old inode, as Vim's does with `backupcopy=no`. The test `a_save_by_rename_is_seen_by_the_watcher` does exactly that, and runs in CI on Linux (inotify), macOS (FSEvents) and Windows (ReadDirectoryChangesW). Saving from Vim itself still has to be tried by hand (0133). A burst of events reloads once, on the first quiet tick. Without a watcher, `r` reloads, and the status line says why.
- **No clock in the TUI.** The core takes today's date, and the debounce counts quiet ticks rather than reading the time. The rule that only `args.rs` reads the clock holds.

Evidence:
- harrow's `app.rs` (`Action`/`Change` returned from keys) and `runtime.rs` (notify plus polling, settle before reading).
- The tests:
  - 22 core and shell unit tests, including a write raced by an edit to its line, and a reload that keeps the cursor on its item;
  - 11 integration tests in `crates/sprig-cli/tests/tui.rs`.

Follow-ups:
- 0131 to 0134, the features this answer shapes, all in this change.
- 0133's hand check of Vim on macOS stays open.

`sprig tui examples/bakery --screenshot 100x30`, with today as 2026-10-01:

```text
 sprig  1 tree 2 next up 3 agenda
›   Open the bakery                            9/40  in 69d  @dana │ Open the bakery
    # Money                                                    2/4 │ bakery.sprig:1
      x Business plan v3                                           │
      x Bank meeting                                               │ status   doing
      ~ Equipment loan paperwork                             in 6d │ owner    @dana
      - Supplier credit line                                  @teo │ due      2026-12-09 (in 69d)
      - Weekly numbers to the bank                           in 1d │ estimate 35.5d
    + Lease                                                    2/6 │ progress 9 of 40 done
      x Shortlist three spaces                                     │
      ~ Negotiate terms                                 1/3  in 4d │
        x Eight weeks rent-free for the fit-out                    │
        ~ Break clause at year three                               │
        ? Who pays for the grease trap?                            │
      - Change-of-use permit                                       │
      - Sign the lease                                             │
    + Kitchen fit-out                                   1/10  @teo │
      - Design                                                 1/2 │
        x Layout sketch                                            │
        ~ Ventilation plan with the engineer                in 11d │
      # Equipment                                              0/3 │
        - Deck oven, three decks                               0/2 │
          - Get a third quote                                in 1d │
          - Place the order                                        │
        - Spiral mixer, 40 litres                                  │
        / Proofing cabinet                                         │
        > Espresso machine                                         │
      - Build                                                  0/5 │
        - Strip out                                                │
 10/46 done · 21 ready · 12 waiting                                                          ? help
```

## Acceptance criteria

- [x] The Answer section states a decision and the evidence for it.
- [x] Follow-up items exist for everything the answer implies.
- [x] A throwaway prototype renders `examples/bakery/` as a tree with `--screenshot 100x30` and the output is committed to the answer.

## Result

A pure core (state and keys, returning actions), one draw function, and a thin crossterm and notify shell, as harrow is built. A write carries the line text it expects and is refused if the file no longer says it; the folder is watched, so a save by rename is seen.
