# Examples

## `bakery/`

Six plans for opening a bakery, which graft into one. It is the example every
guide, snapshot and benchmark uses, so it exercises most of the format: people,
anchors, waits across files, dates, estimates, a recurring item, questions and
answers.

| File | What it is |
| --- | --- |
| `bakery.chea` | The whole plan. It grafts the other three teams' files with `+ [[lease]]`, `+ [[kitchen]]` and `+ [[launch]]`. |
| `lease.chea` | Signing the lease, which the kitchen build and the bank wait on. |
| `kitchen.chea` | The fit-out, owned by Teo. |
| `launch.chea` | Opening week, owned by Kim. It grafts `groceries`. |
| `groceries.chea` | A weekly supply run, `every:week`. |
| `dana.chea` | One person's week: single items grafted from the other files (`+ [[lease^terms]]`), and one of Dana's own. Ticking a grafted item changes the line in its own file. |

```sh
chea next examples/bakery            # what can be done now
chea tree --summary examples/bakery  # every file on one screen
chea why Build --in examples/bakery   # what the kitchen build waits on
chea tui examples/bakery
```

The guides and tests pin the date with `--today 2026-10-01` (or
`CHEA_TODAY`), so their output does not change from one day to the next.
Changing a file here changes snapshots: run `CHEA_BLESS=1 cargo test` and
read the diff.
