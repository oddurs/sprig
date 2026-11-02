# chea-cli

The `chea` command for [Chea](https://oddurs.github.io/chea/), a plain-text
format for plans. It reads a folder of `.chea` files and tells you what can be
done now and what is broken.

```sh
cargo install --locked --git https://github.com/oddurs/chea chea-cli
```

```console
$ chea next examples/bakery     # what can be done now, most urgent first
$ chea check --strict .         # every problem, with a code; exit 1 on any
$ chea tick van                 # tick an item by anchor, text or file:line
$ chea tui                      # the tree, next up and the agenda on one screen
$ chea lsp                      # the language server, for any editor
$ chea mcp                      # the plans, served to a coding agent
```

Every command takes `--plain`: tab-separated, with columns that stay stable.
`next`, `check`, `find` and `tree --summary` also take `--json`, and
`chea export` writes the whole tree as JSON. `chea help <command>` has the
rest, and `chea generate <shell>` and `chea generate man` print the shell
completions and the manual page.

Thirteen commands, each answering one question about a plan. Changing a plan
is a line typed in an editor; `tick` is the one write with a command, because
it settles recurrence and dates.

The guides are on the website: [the shell](https://oddurs.github.io/chea/docs/shell/),
[the TUI](https://oddurs.github.io/chea/docs/tui/) and
[coding agents](https://oddurs.github.io/chea/docs/agents/).

## License

MIT.
