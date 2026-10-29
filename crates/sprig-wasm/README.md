# sprig-wasm

[`sprig-core`](../sprig-core) compiled to WebAssembly, for the website's
playground, its landing page and its build-time renderer. The site carries no
parser of its own, so what the playground shows is what the `sprig` command
would say.

The interface is one call: a JSON request in, a JSON answer out. There is no
bindings generator, and nothing for the browser to trust beyond bytes in and
out. Offsets are in UTF-16 code units, the way JavaScript indexes a string.
The answer is shaped for the site's renderer; for interchange, use
`sprig tree --json`, which `spec/tree.schema.json` describes.

```sh
scripts/task wasm    # build it into site/src/generated/
```

Not published: it is part of the site, not a library.
