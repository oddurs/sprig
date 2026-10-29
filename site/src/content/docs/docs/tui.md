---
title: Work the plan from one screen
description: sprig tui shows the tree, what's next and the agenda together, with vim keys, writes that never clobber an editor's, and live reload.
---

`sprig tui` opens a folder of plans on one terminal screen: the tree with its
grafts mounted, what's next, and the agenda, with a pane for the item under the
cursor.

```sh
sprig tui                  # this folder
sprig tui examples/bakery
```

```text
 sprig  1 tree 2 next up 3 agenda
›   Open the bakery               9/40  in 69d  @dana │ Open the bakery
    # Money                                       2/4 │ bakery.sprig:1
      x Business plan v3                              │
      x Bank meeting                                  │ status   doing
      ~ Equipment loan paperwork                in 6d │ owner    @dana
      - Supplier credit line                     @teo │ due      2026-12-09 (in
      - Weekly numbers to the bank              in 1d │ 69d)
    + Lease                                       2/6 │ estimate 35.5d
      x Shortlist three spaces                        │ progress 9 of 40 done
      ~ Negotiate terms                    1/3  in 4d │
        x Eight weeks rent-free for the fit-out       │
        ~ Break clause at year three                  │
        ? Who pays for the grease trap?               │
      - Change-of-use permit                          │
      - Sign the lease                                │
    + Kitchen fit-out                      1/10  @teo │
      - Design                                    1/2 │
        x Layout sketch                               │
        ~ Ventilation plan with the engineer   in 11d │
      # Equipment                                 0/3 │
        - Deck oven, three decks                  0/2 │
          - Get a third quote                   in 1d │
 10/46 done · 21 ready · 12 waiting                                      ? help
```

## Keys

| Keys | Do |
| --- | --- |
| `j` `k` | move down and up |
| `h` `l` | fold and unfold; `h` on a leaf goes to its parent |
| `g` `G` | first and last item |
| `x` | tick, or reopen; a recurring item moves its due date |
| `~` | mark as doing |
| `a` `A` | add a sibling after the item, or a child under it |
| `e` | open `$EDITOR` at the item's line |
| `u` | undo this session's last write |
| `1` `2` `3` | tree, next up, agenda |
| `/` | filter by text as you type; `Esc` clears |
| `?` | the keys |
| `q` | quit |

The next-up view lists what `sprig next` does, in the same order. The agenda
lists what `sprig agenda` does.

## Open alongside an editor or an agent

The TUI reads the files again whenever one changes, including a save that
writes a new file and renames it into place, as Vim's does. The cursor stays on
its item.

Every write is the same one-line edit `sprig tick`, `sprig mark` or
`sprig add` would make, and it reads the file again first. If the line it is
aimed at has changed since the TUI showed it, the write is refused, nothing is
written, and the screen shows the file as it is now. Nobody's change is
overwritten. A change to another line doesn't stop it.

## For scripts and docs

`--screenshot 100x30` prints one frame as text and exits. With `--keys`, it
presses keys first, through the same writes as a person's:

```sh
sprig tui --screenshot 100x30 --keys "/third quote<Enter>Gx"
```

The frames on this page, and the snapshots the tests hold the TUI to, are made
this way.
