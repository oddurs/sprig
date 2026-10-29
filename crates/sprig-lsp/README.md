# sprig-lsp

The language server for [Sprig](https://oddurs.github.io/sprig/), a plain-text
format for plans. Run it as `sprig lsp`; this crate is the library behind that
command.

It shows problems as you type, including ones a change in one file causes in
another, completes `@people`, `^anchors`, `[[file]]` links and field keys,
follows links and anchors, finds references, shows progress and what an item
waits on in its hover, offers code actions to tick an item or cycle its mark,
and formats the way `sprig fmt` does.

It is a plain loop over messages on one thread, on `lsp-server` and
`lsp-types`, the way rust-analyzer is built. The folder the editor opened is
the workspace root, an open document's text wins over what is on disk, and
nothing outside the root is read.

Editor setup is in the [editor guide](https://oddurs.github.io/sprig/docs/editors/).

## License

MIT.
