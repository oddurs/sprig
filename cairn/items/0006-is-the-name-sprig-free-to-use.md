---
id: 6
uid: a36c7611-35fb-4bfc-a439-d95243ab2336
title: Is the name Sprig free to use?
type: spike
status: blocked
milestone: v0.1
created: 2026-09-28
updated: 2026-10-28
priority: p0
effort: s
area: community
---

## Question

Can the project publish as Sprig? Check crates.io (`sprig`, `sprig-core`), npm, the VS Code Marketplace, the GitHub organisation or repository name, and whether `.sprig` already maps to a language in GitHub Linguist. Check for conflict with Hack Club's Sprig (a JavaScript game engine and handheld console) and the Go template library Masterminds/sprig.

## Timebox

One day.

## What would change the answer

A registered software trademark on the name, or `sprig` taken on crates.io by an active crate, forces a rename.

## Answer

Checked 2026-10-28.

| Where | Name | Result |
| --- | --- | --- |
| crates.io | `sprig` | Taken: "Template functions for Go templates for Rust" (fiji-flo), last release 2021-06-07, 11,184 downloads, 457 recent. Inactive, but the name cannot be had. |
| crates.io | `sprig-core`, `sprig-cli`, `sprig-lang`, `sprig-plan` | Free. |
| npm | `sprig` | Taken: Hack Club's "standalone Sprig game engine", modified 2026-09-08. Active. |
| npm | `@sprig` scope | Taken: an organisation publishing `@sprig/event-emitter`, `@sprig/request-client` and others. |
| npm | `sprig-core`, `sprig-cli`, `sprig-lang` | Free. |
| VS Code Marketplace | "Sprig" | Taken and close: `ColinHouse.sprig-language`, display name "Sprig", "Sprig language support through the Sprig language server", v0.3.1 of 2026-10-08. It is a JVM language "written for people and for AI", whose CLI is also called `sprig` and has a `sprig check` that prints coded diagnostics with file:line:col (colinhouse.github.io/Sprig). Also: `HackClub.sprigkit-extension`, `polypixeldev.vs-sprig` (Hack Club Sprig games), and `putyourlightson.vscode-sprig-support` (Sprig, a Craft CMS plugin). |
| GitHub Linguist | `.sprig` | Not mapped to any language in `languages.yml` (main). The other Sprig language uses `.spr`. |
| GitHub | org `sprig`, hackclub/sprig, Masterminds/sprig | Not checked directly: github.com is not reachable from this session. Masterminds/sprig, the Go template library, is widely used (about 4.5k stars per ecosyste.ms and gittrend). Hack Club Sprig is an active game engine and handheld. |
| USPTO | SPRIG | Sprig Technologies Inc. (sprig.com, product research software) filed SPRIG, serial 97033231, on 2021-09-17 for downloadable software, per Justia. The current status could not be read: TSDR now needs an API key and Justia refused automated access. An older SPRIG of Sprig, Inc. is listed as abandoned in 2013. |

**Recommendation, not yet a decision: rename before v0.1 publishes anything.** Three findings decide it:

- A live software trademark application for SPRIG covers software.
- A new programming language is called Sprig and has a VS Code extension of that name, a `sprig` binary and a `sprig check` subcommand, which collides with this project's binary and its most important command.
- The plain name is taken on crates.io and npm.

The format can't keep a name whose tooling can't use it. The name belongs to the project's owner, and a trademark answer needs a person with access to TSDR or counsel, so this spike stays open (blocked) until they decide. Until then, `publish = false` stays in `Cargo.toml` and nothing is published.

## Acceptance criteria

- [x] The Answer lists every registry checked, with the result and date.
- [ ] If the answer is rename, the new name has passed the same checks and a follow-up item renames everything that says Sprig.
- [ ] The Answer section states a decision and the evidence for it.
- [ ] Follow-up items exist for everything the answer implies.

## 2026-10-28

Registries checked by script. crates.io and npm through their public APIs; the Marketplace through its extension-query API; Linguist through `languages.yml` on main. The trademark was searched on the web, because TSDR needs an API key. The remaining criteria wait on the owner's decision: if rename, the new name has to pass the same checks and a follow-up item renames everything; either way, follow-up items for whatever the decision implies.
