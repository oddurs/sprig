---
id: 69
uid: a3b26da9-6e98-43e0-8ad6-92829d3832bb
title: Can a person be an email address?
type: spike
status: done
milestone: v0.3
depends_on:
- 7
created: 2026-09-28
updated: 2026-10-28
closed_at: 2026-10-28
priority: p1
effort: s
area: spec
---

## Question

Should people accept email addresses as well as handles? That helps calendar invites and org directories, but `@` inside an address complicates tokenising.

## Timebox

One day.

## What would change the answer

Adopters mapping people to identities outside the repository.

## Answer

**No: a person is a handle.** Tokenising under each outcome:

- **As ruled:** `@dana@example.org` is a word, because `@` is not a name character (§4.2). `dana@example.org` is a word. `@dana` is the person `dana`.
- **If addresses were accepted:** `@dana@example.org` would be the person `dana@example.org`. `@dana.k` would still be `dana.k`. `@dana,` would still be a word.

Accepting addresses would make the second `@` special inside one token, put personal data in every repository a plan is copied to, and tie a plan to addresses that change. Every tool would also have to treat `dana` and `dana@example.org` as maybe the same person. Handles have none of these costs, and mapping a handle to an address can live beside the plans, the way git keeps `.mailmap` and GitHub keeps CODEOWNERS.

## Acceptance criteria

- [x] The Answer includes tokenising examples for both outcomes.
- [x] The Answer section states a decision and the evidence for it.
- [x] Follow-up items exist for everything the answer implies.

## 2026-10-28

Spec §4.2 now says an email address is a word, with or without a leading `@`. `tokens/063-an-email-address-is-not-a-person` holds it. Follow-up: 0146 (map handles to identities outside plan files), filed under `later`.
