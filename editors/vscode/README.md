# Chea for VS Code

[Chea](https://oddurs.github.io/chea/) is a plain-text format for plans: one item per line, a mark at the start, indentation for structure. This extension adds:

- highlighting for marks, people, tags, anchors, fields and links;
- problems as you type, including the ones a change in one file causes in another;
- completion for `@people`, `^anchors`, `[[file]]` links and field keys;
- go to definition and find references for anchors and links;
- hover with progress, work left, owner and what an item waits on;
- code actions to tick an item, cycle its mark and write out a typed date;
- formatting, the same as `chea fmt`.

The language server is bundled, so it works offline with nothing else to install. To use a different `chea`, set `chea.server.path`.
