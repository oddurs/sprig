---
id: 58
uid: c4ebdb45-d994-4fa4-a371-00bd39331a6e
title: What tools should sprig mcp expose?
type: spike
status: done
milestone: v0.3
depends_on:
- 23
created: 2026-09-28
updated: 2026-10-29
closed_at: 2026-10-29
priority: p0
effort: s
area: agents
---

## Question

Which tools, with what arguments, how writes report what they changed, and how errors look. Read the MCP specification and an existing backlog MCP server (cairn's `cairn mcp` at github.com/oddurs/cairn is a close model) before deciding.

## Timebox

Two days.

## What would change the answer

Agent clients handling many small tools badly compared with a few general ones, or a client requiring confirmation for writes.

## Answer

**Decision: ten tools, five that read and five that write, each the twin of a CLI command, over stdio, in `sprig-cli` as `sprig mcp`.** spec/tools.md T6 is the contract; `crates/sprig-cli/src/mcp.rs` implements it.

- **Few general tools, not many small ones.** Each tool is a command people already use, with the arguments that command takes. An agent that knows `sprig next` knows `next_items`. cairn's server, the closest model, has fifteen tools and its clients cope. Sprig needs fewer because one query language (T5.1) covers what cairn splits across `list_items` and `search_items`.
- **Reads return the CLI's `--json` byte for byte**, built by the same functions in `data.rs`. The test `read_tools_answer_what_the_json_commands_print` compares them, so the CLI and the server cannot drift. Lists are wrapped as `{"items": [...]}` in `structuredContent`, because the protocol requires an object there, and sent unwrapped as text.
- **Writes return what they did.** Each write returns:
  - the lines it wrote;
  - a unified diff of every file it changed;
  - `no_longer_waiting`, the open items that wait on less than before, each with `now` (`unblocked`, or what it still waits on).

  An agent that proposes a plan change to a person can show them the diff verbatim.
- **Writes are refused rather than repaired.** A write that would add an error `check` reports comes back with `isError` and each new diagnostic as `check` prints it, and nothing is written. Errors already in the plan don't block unrelated writes. The comparison is by code, file and message, since line numbers move.
- **Failures go in the result, not in the protocol.** The MCP spec says tool execution errors belong in the result with `isError`, so the model can correct itself. Only an unknown tool or a malformed request is a JSON-RPC error. cairn does the same.
- **Confirmation is the client's job.** The protocol says clients SHOULD confirm sensitive operations and that annotations are hints. Every tool carries `readOnlyHint`, `destructiveHint: false`, `idempotentHint` and `openWorldHint: false`, so a client can auto-approve reads and ask about writes. Nothing a write does is destructive: it can be undone from the diff.
- **The server keeps no copy of the plans.** It reads the files on every call, so a person editing the same plan is never overwritten by a stale buffer. On the CI runner, parsing, resolving and checking 5,078 lines takes about 7 ms (the perf job's `check_ms`).
- **Protocol revision.** It speaks 2025-11-25, and answers a client that asks for 2025-06-18 or 2025-03-26 in that revision.

Evidence:
- The MCP specification, 2025-11-25, server/tools: result shapes, `isError`, annotations, and structured content as an object.
- cairn's `src/cmd/mcp.rs`: line-delimited stdio, in-band errors, and verb_noun tool names.
- A session through the official TypeScript client (`@modelcontextprotocol/sdk` 1.32.1, `tests/mcp/session.mjs`) passes.
- The Rust tests in `crates/sprig-cli/tests/mcp.rs` pass.

Follow-ups, which already exist:
- 0059, the read tools;
- 0060, the write tools;
- 0061, the cap: `--max-new-items`, 20 per session counting every call;
- 0062, the convention and `sprig agent`;
- 0063, the end-to-end session.

### The tools, with real calls on examples/bakery

Results are shortened to their first item.

#### `list_plans`

Every plan in the folder with its progress: done and total, how many items are ready and waiting, how many are late, and its errors.

Arguments: none. Annotations: readOnlyHint true, openWorldHint false.

```json
→ {}
← [{"file": "bakery.sprig", "title": "Open the bakery", "done": 9, "total": 40, "ready": 20, "waiting": 12, "late": 0, "errors": 0}, "…"]
```

#### `show_tree`

One plan as a resolved tree: every item with its status, progress, people, dates and what it waits on, grafts mounted.

Arguments: `file` (required): string. Annotations: readOnlyHint true, openWorldHint false.

```json
→ {"file": "lease"}
← {"sprig_tree": 1, "file": "lease.sprig", "today": "2026-10-01", "syntax": "…", "resolved": {"file": "lease.sprig", "line": 1, "label": "Lease", "status": "doing", "progress": {"done": 2, "doing": 1, "total": 6}, "estimate": 120, "people": ["dana"], "children": [{"file": "lease.sprig", "line": 3, "mark": "x", "label": "Shortlist three spaces", "status": "done", "settled": true, "progress": {"done": 1, "doing": 0, "total": 1}, "people": ["dana"]}, "…"]}, "diagnostics": []}
```

#### `next_items`

Open work with nothing in its way, in progress first, then by priority and due date.

Arguments: `who`: string, `limit`: integer. Annotations: readOnlyHint true, openWorldHint false.

```json
→ {"who": "dana", "limit": 1}
← [{"ref": "lease.sprig:8", "path": ["Lease", "…"], "node": {"file": "lease.sprig", "line": 8, "mark": "~", "label": "Break clause at year three", "status": "doing", "grafted": true, "progress": {"done": 0, "doing": 1, "total": 1}, "people": ["dana"]}}]
```

#### `find_items`

Items matching every term of a query: words from the text, @who, #tag, is:open|todo|doing|done|dropped|later|ask|blocked, and due:DATE, due:<DATE, due:<=DATE, due:>DATE or due:>=DATE, where DATE may be a shortcut such as fri or +2w.

Arguments: `query` (required): string. Annotations: readOnlyHint true, openWorldHint false.

```json
→ {"query": "is:blocked #finance"}
← []
```

#### `check_plans`

Every problem in the folder's plans: broken grafts, loops, unknown references and the rest, with file, line and a stable code.

Arguments: none. Annotations: readOnlyHint true, openWorldHint false.

```json
→ {}
← []
```

#### `tick_items`

Mark items done, or reopen done ones; a recurring item's due date moves to its next date instead.

Arguments: `refs` (required): array of string. Annotations: readOnlyHint false, destructiveHint false, idempotentHint false, openWorldHint false.

```json
→ {"refs": ["^signed"]}
← {"ticked": [{"file": "lease.sprig", "line": 13, "text": "x Sign the lease  ^signed  after:^terms  after:^permit", "due_moved_to": null}], "diff": "--- a/lease.sprig\n+++ b/lease.sprig\n@@ -12,2 +12,2 @@\n   Class E covers a bakery, but seating for more than 12 needs a variation.\n-- Sign the lease  ^signed  after:^terms  after:^permit\n+x Sign the lease  ^signed  after:^terms  after:^permit\n", "no_longer_waiting": [{"file": "bakery.sprig", "line": 15, "text": "Supplier credit line", "now": "unblocked"}, "…"]}
```

#### `mark_item`

Set an item's status mark: - to do, ~ doing, x done, / dropped, > later, ? question (or the words todo, doing, done, dropped, later, ask).

Arguments: `ref` (required): string, `mark` (required): string. Annotations: readOnlyHint false, destructiveHint false, idempotentHint true, openWorldHint false.

```json
→ {"ref": "Menu boards", "mark": "~"}
← {"file": "launch.sprig", "line": 7, "text": "~ Menu boards  after:^logo  est:4h", "diff": "--- a/launch.sprig\n+++ b/launch.sprig\n@@ -6,3 +6,3 @@\n   - Window lettering  after:^logo  after:[[bakery^name]]  est:1d\n-  - Menu boards  after:^logo  est:4h\n+  ~ Menu boards  after:^logo  est:4h\n # Hiring\n", "no_longer_waiting": []}
```

#### `add_items`

Add items, in order, under an item (as its last children) or after one (as siblings), indented to match; with neither, at the end of PLAN.sprig.

Arguments: `items` (required): array of string, `under`: string, `after`: string. Annotations: readOnlyHint false, destructiveHint false, idempotentHint false, openWorldHint false.

```json
→ {"under": "^design", "items": ["Call the plumber due:fri"]}
← {"added": [{"file": "kitchen.sprig", "line": 8, "text": "- Call the plumber due:2026-10-02"}], "diff": "--- a/kitchen.sprig\n+++ b/kitchen.sprig\n@@ -7,2 +7,3 @@\n     The neighbours at no. 43 have already asked.\n+  - Call the plumber due:2026-10-02\n # Equipment  #finance\n", "no_longer_waiting": []}
```

#### `answer_question`

Settle a ? item with an answer line under it (= ...).

Arguments: `ref` (required): string, `text` (required): string. Annotations: readOnlyHint false, destructiveHint false, idempotentHint false, openWorldHint false.

```json
→ {"ref": "^name", "text": "Kiln & Crumb"}
← {"file": "bakery.sprig", "line": 25, "text": "= Kiln & Crumb", "diff": "--- a/bakery.sprig\n+++ b/bakery.sprig\n@@ -24 +24,2 @@\n   // Settle it: add a line under here that starts with \"= \".\n+  = Kiln & Crumb\n", "no_longer_waiting": [{"file": "launch.sprig", "line": 6, "text": "Window lettering", "now": "still waits on Logo, second round"}]}
```

#### `set_field`

Set a key:value field on an item, such as due, start, est, every or one of your own; an empty value removes it.

Arguments: `ref` (required): string, `key` (required): string, `value` (required): string. Annotations: readOnlyHint false, destructiveHint false, idempotentHint true, openWorldHint false.

```json
→ {"ref": "^logo", "key": "est", "value": "3d"}
← {"file": "launch.sprig", "line": 4, "text": "~ Logo, second round  ^logo  due:2026-10-14  est:3d", "diff": "--- a/launch.sprig\n+++ b/launch.sprig\n@@ -3,3 +3,3 @@\n # Brand\n-  ~ Logo, second round  ^logo  due:2026-10-14\n+  ~ Logo, second round  ^logo  due:2026-10-14  est:3d\n     Round one was too rustic. New brief: a kiln, drawn in one line.\n", "no_longer_waiting": []}
```

A refused write:

```json
→ {"ref": "^logo", "key": "after", "value": "^nowhere"}
← isError: "refused, and nothing was written: the change would add an error\nlaunch.sprig:4:56: error[S0201]: no item ^nowhere in launch.sprig"
```

## Acceptance criteria

- [x] The Answer lists every tool with its argument schema and an example call and result.
- [x] The Answer section states a decision and the evidence for it.
- [x] Follow-up items exist for everything the answer implies.

## Result

Ten tools, each the twin of a CLI command: list_plans, show_tree, next_items, find_items and check_plans read and return the CLI's --json; tick_items, mark_item, add_items, answer_question and set_field write through the edit API, refuse a change that adds a check error, and return the diff and what stopped waiting. spec/tools.md T6 is the contract.
