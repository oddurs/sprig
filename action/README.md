# Chea plans action

Checks the `.chea` plans in a repository, and writes a job summary saying
what a pull request did to them: each file's done count on the base and on
the head, and the items the change unblocked. Problems become annotations on
the lines they are about.

```yaml
on: pull_request

jobs:
  plans:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: oddurs/chea/action@main
        with:
          paths: plans
          strict: true
```

| Input | Default | |
| --- | --- | --- |
| `paths` | `.` | Plans to check: files or directories, separated by spaces. |
| `strict` | `false` | Fail on warnings as well as errors. |

The job fails when `chea check` does: on errors, or on warnings with
`strict`. The summary is written first, so it is there either way.

There is no release to download yet, so the action builds `chea` from the
same commit as the action, which takes a minute or two on a fresh runner.
Put a `chea` on `PATH` before the step to skip that.

`summary.sh` can be run anywhere, given two checkouts:

```sh
action/summary.sh ../base . plans
```
