# Conformance suite

Each directory `<section>/<NNN-slug>/` is one example, found and run by
`scripts/task test` with no registration. To add one:

1. Make the directory, e.g. `lines/056-what-it-shows/`.
2. Write `main.sprig`, the entry file, and any other `.sprig` files it grafts.
3. Write `meta.toml`: `rules = ["§2.6"]` and, if dates matter, `today = "2026-10-01"`.
4. Write `expected.json`: `{"sprig_tree": 1, "syntax": {...}}` with any of
   `syntax`, `resolved` and `diagnostics` (spec Appendix D); only those are compared.
5. Or write `{"sprig_tree": 1, "syntax": {}}`, run `SPRIG_BLESS=1 cargo test -p sprig-core --test conformance`,
   and check every line of what it wrote against the spec before committing.

Diagnostics are compared on the keys an example states, never on `message`.
An example without `expected.json` fails the run. Inputs are byte-exact
(`.gitattributes`), so CRLF and BOM examples stay as written.
