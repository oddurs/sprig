# Security Policy

## Supported versions

Sprig has not had a release yet. Until v0.1, reports against `main` are
welcome; from v0.1 until 1.0, only the latest release receives fixes.

## Reporting a vulnerability

**Please do not open a public issue.**

Report privately through GitHub Security Advisories:

<https://github.com/oddurs/sprig/security/advisories/new>

Include the version or commit, your platform, and the smallest `.sprig` input
that reproduces the problem.

## What to expect

- **Acknowledgement within 3 days.** If you have not heard back by then, open a
  public issue saying only that you are waiting on a private report, with no
  details.
- **An assessment within 7 days**, with a severity and a plan.
- **A fix or a documented mitigation within 30 days** for anything exploitable.
- Credit in the release notes, unless you would rather not be named.

## Scope

Sprig reads plain-text plans, often from repositories somebody else wrote. The
boundaries that matter:

- **Every `.sprig` file is untrusted input.** A malformed or hostile file must
  never do more than produce a diagnostic. A panic, a hang or unbounded memory
  from a file is a real finding.
- **Grafts must not escape the workspace.** `+ [[../../somewhere]]` is refused.
  A path that reads a file outside the directory Sprig was pointed at is a
  vulnerability.
- **Nothing in a plan is executed.** No field, link or note ever reaches a shell
  or a network request.

Out of scope: anything requiring an attacker who can already run commands as
you or write to the repository you are working in.
