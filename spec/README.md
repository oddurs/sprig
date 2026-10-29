# The specification

The authority on what a `.sprig` file means. When the code and these
documents disagree, the code is wrong.

| File | What it says |
| --- | --- |
| [`sprig.md`](sprig.md) | The format: lines, marks, indentation, tokens, status, people, waits, grafts and dates, every rule numbered (§1 to §10). |
| [`diagnostics.md`](diagnostics.md) | Every problem code a tool reports, its severity and what it means. A code is never reused. |
| [`tools.md`](tools.md) | What a tool built on the format must do: refs, edits, output, colour, interop and agents (T1 to T6). |
| [`tree.schema.json`](tree.schema.json) | The JSON shape of a resolved tree, as `sprig tree --json` writes it. |

The examples that prove each rule are in
[`tests/conformance/`](../tests/conformance), which any implementation can run.
How the spec changes is in [`rfcs/`](../rfcs), and what will not change after
1.0 is in [`docs/stability.md`](../docs/stability.md).

Everything in this folder is dedicated to the public domain under
[CC0](LICENSE), so anyone can implement it without asking.
