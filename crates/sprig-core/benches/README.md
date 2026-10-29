# Benchmarks

`workspace.rs` times what an editor does on every keystroke: parse, resolve
and check a generated workspace of 20 files and about 5,000 lines, shaped like
`examples/bakery/` (one plan grafting the others, a reference into the next
file on every item). It runs each step 31 times and reports the median in
milliseconds.

```sh
scripts/task perf
```

That runs the benchmark against `baseline.json` and fails if any median is
more than twice its baseline. It also runs the language server's keystroke
test in a release build, which must recheck the same size of workspace in
under 50 ms (cairn 0049).

## The baseline

`baseline.json` holds medians measured on the CI runner (GitHub's
`ubuntu-latest`), because that is where the comparison runs; numbers from a
laptop are not comparable. To update it after a change that is meant to be
slower, or faster:

1. Run the `perf` job and read the medians from its log, or run
   `cargo bench -p sprig-core --bench workspace -- --save` on the same kind of
   machine.
2. Commit `baseline.json` in the same pull request as the change, saying why
   it moved.

A baseline is never raised just to make a slow change pass. If a change
crosses the line, find out why first.
