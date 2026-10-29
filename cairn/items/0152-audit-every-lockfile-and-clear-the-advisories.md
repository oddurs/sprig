---
id: 152
uid: 0386ba9c-96b4-4b57-9282-3fa53aae8fab
title: Audit every lockfile, and clear the advisories
type: chore
status: done
milestone: v1.0
created: 2026-10-29
updated: 2026-10-29
closed_at: 2026-10-29
priority: p1
effort: s
area: release
---

## Problem

GitHub reported 18 Dependabot advisories on main, 8 high, and nothing in the repository's own checks would have said so. Dependabot watched one of the five npm and cargo lockfiles the repository commits.

## Approach

Fix what is fixable within semver. Override a transitive dependency where the fix is only in a newer major of something deep in a build tool, and prove the tool still works. Then make the audit a `scripts/task` target, run it weekly and on dependency changes, and hold Dependabot's configuration to the list of lockfiles with a test.

## Acceptance criteria

- [x] `scripts/task audit` passes: no advisory in any committed lockfile.
- [x] A scheduled workflow runs the audit weekly and on changes to a dependency.
- [x] A test fails when a committed lockfile is not in `.github/dependabot.yml`.

## 2026-10-29

All 18 were npm; cargo audit found nothing in either Cargo.lock. The site's five (undici, vite, zod, source-map-js, postcss-selector-parser) were fixed by npm audit fix within semver, a lockfile-only change. The extension's three were one advisory in @azure/msal-node 6.0.1, three levels under @vscode/vsce. npm audit's own fix was to downgrade vsce a major version, so instead package.json overrides msal-node to 7.1.2. @azure/identity still loads, and npm run package built a VSIX. The audit workflow is not a required check: a new advisory in a build tool must not block an unrelated pull request.

## Result

No known advisories in any lockfile; the audit runs weekly and Dependabot watches every lockfile.
