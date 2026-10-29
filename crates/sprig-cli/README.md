# sprig-cli

The `sprig` command for [Sprig](https://oddurs.github.io/sprig/), a plain-text
format for plans. It reads a folder of `.sprig` files and tells you what can be
done now and what is broken.

```sh
cargo install --locked --git https://github.com/oddurs/sprig sprig-cli
```

```console
$ sprig next examples/bakery     # what can be done now, most urgent first
$ sprig check --strict .         # every problem, with a code; exit 1 on any
$ sprig tick van                 # tick an item by anchor, text or file:line
$ sprig tui                      # the tree, next up and the agenda on one screen
$ sprig lsp                      # the language server, for any editor
$ sprig mcp                      # the plans, served to a coding agent
```

Every command takes `--plain`: tab-separated, with columns that stay stable.
`next`, `check`, `find` and `tree --summary` also take `--json`, and
`sprig export` writes the whole tree as JSON. `sprig help <command>` has the
rest, and `sprig generate <shell>` and `sprig generate man` print the shell
completions and the manual page.

Thirteen commands, each answering one question about a plan. Changing a plan
is a line typed in an editor; `tick` is the one write with a command, because
it settles recurrence and dates.

The guides are on the website: [the shell](https://oddurs.github.io/sprig/docs/shell/),
[the TUI](https://oddurs.github.io/sprig/docs/tui/) and
[coding agents](https://oddurs.github.io/sprig/docs/agents/).

## License

MIT.
