# Sprig diagnostics

Every problem a Sprig tool reports about a file has a code from this catalogue.
The code is the problem's identity: search for `S0103` and you find the same
problem in every version of every tool. The wording of a message may improve;
the code may not move.

This document is dedicated to the public domain under
[CC0 1.0](https://creativecommons.org/publicdomain/zero/1.0/), like the
specification it accompanies.

## Rules for the catalogue

- **Codes are never reused.** A code that is retired keeps its row, marked
  retired, and no later problem takes its number.
- **A code names one problem.** Two problems that need different fixes get two
  codes, even when one message could describe both.
- **Severity is part of the code.** An *error* means the file does not say what
  its author meant, and `sprig check` exits 1. A *warning* means a tool kept
  going on a best reading, and `sprig check` exits 1 only with `--strict`.
- **Ranges.** `S00xx` are problems one line has on its own, `S01xx` are grafts,
  and `S02xx` are references.

A diagnostic is printed the way compilers print one, with a 1-based line and a
column counted in characters:

```
lease.sprig:13:31: error[S0201]: no item ^permt in lease.sprig
```

## Catalogue

| Code | Severity | Rule | Problem |
| --- | --- | --- | --- |
| S0001 | warning | §9.5 | A date field that is not a YYYY-MM-DD date |
| S0002 | warning | §10.1 | An estimate that is not a number and a unit |
| S0003 | warning | §4.5 | An item with more than one anchor |
| S0004 | warning | §4.5 | An anchor already used in the same file |
| S0005 | error | §8.1 | A graft line with no link |
| S0101 | error | §8.5 | A graft naming a file that does not exist |
| S0102 | error | §8.5 | A graft naming an anchor its file does not write |
| S0103 | error | §8.4 | A graft that would loop |
| S0104 | error | §8.2 | A graft whose path leaves the workspace |
| S0105 | error | §8.8 | A graft whose content is already in the tree |
| S0201 | error | §7.2 | A reference to an anchor the file does not write |
| S0202 | error | §7.2 | A reference to a file that does not exist |
| S0203 | error | §7.2 | A reference to an anchor its file does not write |
| S0204 | error | §8.2 | A reference whose path leaves the workspace |
| S0205 | error | §7.2 | An after: value that is not a reference |

## Notes on each code

**S0001.** `due:` and `start:` hold dates (§9.1). A value in neither the stored
form nor a typed shortcut (§9.3) is kept as text and does not count as a date:
it sets no deadline and blocks nothing. Typed shortcuts are not read before
v0.2, so until then they are reported here too.

**S0002.** An estimate is a number and `h`, `d` or `w` (§10.1). Anything else
is ignored when estimates are summed.

**S0003.** An item names one anchor. When it writes several, the last one names
it (Appendix B.3), and the others reach nothing.

**S0004.** Anchors are unique within a file (§4.5). References to a repeated
anchor reach the first item that writes it (Appendix B.4).

**S0005.** A `+` line grafts the file or branch its link names (§8.1); without
a link it grafts nothing. Reported on the line.

**S0101, S0102.** A graft whose target does not exist mounts nothing (§8.5).
Reported on the link.

**S0103.** A graft to a file that is already being mounted above it would
never end (§8.4). Files are walked in path order and each file's grafts in
document order, and the graft that closes the loop is the one reported, so a
loop through three files is one error on one line, wherever a tool starts.

**S0104, S0204.** A link's path is resolved from the file that writes it
(§4.8). A path that climbs above the directory a tool was asked to read, or is
absolute, is refused and nothing outside is read (§8.2).

**S0105.** Within one tree an item appears once (§8.8). Reported on the later
graft line in document order.

**S0201, S0202, S0203, S0205.** A reference that names no existing file or
anchor is unknown (§7.2), and the item that waits on it is blocked until it is
fixed (§7.3).
