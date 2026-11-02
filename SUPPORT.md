# Getting help

Chea is maintained by one person in their own time, so the fastest answer is
usually one that is already written down.

## Read first

- [The first-plan tutorial](https://oddurs.github.io/chea/docs/first-plan/),
  five minutes from nothing to a checked plan.
- The guides for [the shell](https://oddurs.github.io/chea/docs/shell/),
  [editors](https://oddurs.github.io/chea/docs/editors/),
  [the TUI](https://oddurs.github.io/chea/docs/tui/) and
  [coding agents](https://oddurs.github.io/chea/docs/agents/).
- `chea help <command>`, and the [diagnostics](spec/diagnostics.md): every
  problem code `chea check` prints is explained there.
- [The specification](spec/chea.md), when the question is what a line means.
  Its rules are numbered, and each has an example in
  [`tests/conformance/`](tests/conformance).

## Then ask

| You have | Go to |
| --- | --- |
| A file Chea reads wrongly, or a command that misbehaves | [A bug report](https://github.com/oddurs/chea/issues/new?template=bug_report.yml), with the smallest `.chea` file that shows it |
| An idea for a command or an editor feature | [A feature request](https://github.com/oddurs/chea/issues/new?template=feature_request.yml) |
| A question the docs do not answer | [An issue](https://github.com/oddurs/chea/issues/new/choose); a question that should have been answered is a gap in the docs |
| A change to what a file means | [An RFC](rfcs/README.md), after reading the settled arguments: `cairn list --view decisions` |
| A security problem | [A private report](SECURITY.md), never a public issue |

Only security reports have a promised response time ([SECURITY.md](SECURITY.md)).
Everyone who takes part agrees to the
[code of conduct](CODE_OF_CONDUCT.md).
