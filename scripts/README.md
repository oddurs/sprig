# Scripts

POSIX `sh`, so they run unchanged on Linux, macOS and Windows' Git Bash.

| Script | What it does |
| --- | --- |
| [`task`](task) | The one seam all automation goes through. CI, the git hooks and people all run its targets, so they cannot disagree. Run with no argument, it lists them. |
| [`agent`](agent) | The item, branch, worktree and pull request loop in [CONTRIBUTING.md](../CONTRIBUTING.md): `start`, `check`, `commit`, `pr`, `sync`, `done`, `list` and `doctor`. |
| [`setup`](setup) | Once per clone: points git at `.githooks/`, checks the toolchain, and registers cairn's merge driver. |

A `cargo`, `npm` or `node` command belongs in `task`, once, and nowhere else:
not in a workflow, a hook or another script.
