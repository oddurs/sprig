---
title: Plans for coding agents
description: Where the plan lives, how an agent should behave around it, and the sprig mcp server that lets it read and change plans safely.
---

A coding agent loses the thread between sessions; a plan in the repository is
how the next session, or the person reviewing the work, picks it up. Sprig keeps
that plan in plain text, where an agent can read it, write to it, and be held to
it in review like any other file.

## The convention

- **`PLAN.sprig` at the repository root** is the plan for the work in progress.
  Larger projects graft other files into it (`+ [[plans/api]]`).
- **Read it before starting.** `sprig next` lists what is ready, in progress
  first.
- **Plan before changing code.** The agent adds the steps it means to take,
  marks the one it starts `~`, and ticks each when it is done, in the same
  commit as the work. A reviewer sees the plan change next to the code.
- **Questions for a person are `?` items that name them:**
  `? Roof or wall for the outlet?  @teo`. The agent doesn't answer its own
  questions. It carries on with work the question doesn't block, and `sprig
  next` puts the question in front of the person it names.
- **`sprig check` passes before the agent says it has finished.**

`sprig agent` prints these instructions for an agent's instruction file.
`--write` keeps them in a marked block, so running it again updates the block
and running it twice changes nothing:

```sh
sprig agent --write AGENTS.md     # or CLAUDE.md
```

## The MCP server

`sprig mcp` serves a folder's plans over the
[Model Context Protocol](https://modelcontextprotocol.io), on standard input and
output. `sprig mcp --config` prints the snippet that registers it:

```json
{
  "mcpServers": {
    "sprig": { "command": "/usr/local/bin/sprig", "args": ["mcp", "--in", "/home/you/project"] }
  }
}
```

Put it in `.mcp.json` at the project root for Claude Code, `.cursor/mcp.json`
for Cursor, or under `[mcp_servers.sprig]` in `~/.codex/config.toml` for Codex.

### Tools

| Tool | Does | Same as |
| --- | --- | --- |
| `list_plans` | Each plan's progress: done, total, ready, waiting, late, errors | `sprig status --json` |
| `show_tree` | One plan as a resolved tree | `sprig export --format json` |
| `next_items` | Ready work, optionally for one person, at most `limit` | `sprig next --json` |
| `find_items` | Items matching a query: words, `@who`, `#tag`, `is:`, `due:` | `sprig find --json` |
| `check_plans` | Every problem, with a stable code | `sprig check --json` |
| `tick_items` | Tick items done, or reopen them | `sprig tick` |
| `mark_item` | Set a mark: `-` `~` `x` `/` `>` `?` | `sprig mark` |
| `add_items` | Add items under or after one, or to `PLAN.sprig` | `sprig add` |
| `answer_question` | Settle a `?` with an `=` line | `sprig answer` |
| `set_field` | Set or remove a `key:value` field | |

The read tools return exactly what the matching command prints. The write tools
are held to three rules:

- **A write changes only the bytes it means to.** Every one goes through the
  same edit API as `sprig tick`, so indentation, line endings and comments
  stay as they were.
- **A write never breaks a plan.** One that would add an error `sprig check`
  reports is refused, with the diagnostic in the message, and nothing is
  written.
- **A write shows what it did.** It returns the unified diff it applied, and
  the items that stopped waiting because of it.

An agent can add items faster than a person can read them. `add_items` accepts
at most **20 new items per session** by default, counting every call, and
refuses a call that would pass the limit. The person running the server can
change it with `sprig mcp --max-new-items N`.

The server reads the files afresh on every call, so a person editing the same
plan, or another agent, is never overwritten by a stale copy.

## A session, from start to finish

Teo asks an agent to get the kitchen's ventilation moving. These are the calls
the agent makes on the [bakery example](https://github.com/oddurs/sprig/tree/main/examples/bakery),
with their results, shortened where they repeat.

The agent finds Teo's work first:

```json
→ next_items {"who": "teo", "limit": 2}
[
  { "ref": "kitchen.sprig:5", "path": ["Kitchen fit-out", "Design"],
    "node": { "label": "Ventilation plan with the engineer", "status": "doing",
              "due": "2026-10-12", "estimate": 16, "people": ["teo"] } },
  { "ref": "kitchen.sprig:11", "path": ["Kitchen fit-out", "Equipment", "Deck oven, three decks"],
    "node": { "label": "Get a third quote", "status": "todo", "due": "2026-10-02" } }
]
```

The ventilation plan is in progress. `show_tree {"file": "kitchen"}` gives the
whole plan, including the note under it: *Extraction has to exit through the
roof, not the back wall.* Before touching anything, the agent writes down its
steps, and asks Teo the one thing it can't decide:

```json
→ add_items {"under": "Ventilation plan",
             "items": ["Measure the roof run  est:1h", "? Roof or wall for the outlet?  @teo"]}
{
  "added": [
    { "file": "kitchen.sprig", "line": 8, "text": "- Measure the roof run  est:1h" },
    { "file": "kitchen.sprig", "line": 9, "text": "? Roof or wall for the outlet?  @teo" }
  ],
  "diff": "--- a/kitchen.sprig\n+++ b/kitchen.sprig\n@@ -7,2 +7,4 @@\n     The neighbours at no. 43 have already asked.\n+    - Measure the roof run  est:1h\n+    ? Roof or wall for the outlet?  @teo\n # Equipment  #finance\n",
  "no_longer_waiting": []
}
```

The new lines took the indentation of the notes around them. The agent does the
work it can, then ticks it:

```json
→ tick_items {"refs": ["Measure the roof run"]}
{
  "ticked": [{ "file": "kitchen.sprig", "line": 8, "text": "x Measure the roof run  est:1h" }],
  "diff": "--- a/kitchen.sprig\n+++ b/kitchen.sprig\n@@ -7,3 +7,3 @@\n     The neighbours at no. 43 have already asked.\n-    - Measure the roof run  est:1h\n+    x Measure the roof run  est:1h\n     ? Roof or wall for the outlet?  @teo\n",
  "no_longer_waiting": []
}
```

A ref can be words from an item's text, as long as they match one item. A
mistake comes back as an error the agent can act on. Here it wired the outlet
to an anchor that doesn't exist:

```json
→ set_field {"ref": "Roof or wall", "key": "after", "value": "^survey"}
refused, and nothing was written: the change would add an error
kitchen.sprig:9:43: error[S0201]: no item ^survey in kitchen.sprig
```

Before it reports back, the agent checks the plans, and lists what is waiting
on a person:

```json
→ check_plans {}
[]
→ find_items {"query": "is:ask @teo"}
[{ "ref": "kitchen.sprig:9", "node": { "label": "Roof or wall for the outlet?", "status": "question" } }]
```

Teo reads the question in the diff, or in `sprig next --who teo`, and answers
it in the file (`  = Roof, as the engineer advised`) or with
`sprig answer "Roof or wall" "Roof, as the engineer advised"`. The next
session starts from there.

CI runs a session like this one through the official MCP client on every change
to Sprig, and then runs `sprig check --strict` on the result.
