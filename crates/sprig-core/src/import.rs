//! Other formats into Sprig (spec/tools.md T5): Markdown checklists,
//! todo.txt, and GitHub issues as `gh issue list --json` prints them.
//!
//! Each importer writes a file the formatter would leave alone (two spaces
//! per level, notes at their item's children's depth, one line ending at the
//! end) and that `sprig check` accepts, so the first thing a person does with
//! an import is read it, not fix it.
//!
//! # Example
//!
//! ```
//! use sprig_core::import;
//!
//! let md = import::markdown("# Move\n\n- [x] Book a van\n- [ ] Pack\n", "move");
//! assert_eq!(md, "Move\n\nx Book a van\n- Pack\n");
//! let todo = import::todotxt("(A) Call the bank +money\n", "errands");
//! assert_eq!(todo, "errands\n\n- Call the bank #money !!!\n");
//! let gh = import::github(r#"[{"number": 7, "title": "Fix it", "state": "OPEN"}]"#, "Issues").unwrap();
//! assert_eq!(gh, "Issues\n\n- Fix it  issue:7\n");
//! ```

use serde_json::Value;

/// A line that would read as something else is escaped (§2.8).
fn note(text: &str) -> String {
    let t = text.trim();
    let marked = t.chars().next().is_some_and(|c| "-~x/>?#+=".contains(c))
        && t.chars().nth(1).is_none_or(char::is_whitespace);
    if marked || t.starts_with("//") || t.starts_with('\\') {
        format!("\\{t}")
    } else {
        t.to_string()
    }
}

fn finish(mut lines: Vec<String>) -> String {
    while lines.last().is_some_and(|l| l.is_empty()) {
        lines.pop();
    }
    let mut out = lines.join("\n");
    out.push('\n');
    out
}

/// A Markdown checklist: the first heading is the title, later headings are
/// groups, list items are items (`[x]` done, anything else to do), nesting
/// follows the list's own indentation, and other text becomes notes.
pub fn markdown(src: &str, name: &str) -> String {
    let mut lines: Vec<String> = Vec::new();
    let mut title: Option<String> = None;
    // (markdown indentation, sprig depth) of the open list items.
    let mut stack: Vec<(usize, usize)> = Vec::new();
    // Depth of the current group's children, after a heading.
    let mut base = 0;
    let mut last_item_depth: Option<usize> = None;
    for raw in src.lines() {
        let indent = raw.len() - raw.trim_start().len();
        let line = raw.trim();
        if line.is_empty() {
            if lines.last().is_some_and(|l| !l.is_empty()) {
                lines.push(String::new());
            }
            continue;
        }
        if let Some(h) = heading(line) {
            stack.clear();
            last_item_depth = None;
            if title.is_none() && lines.iter().all(|l| l.is_empty()) {
                title = Some(h.to_string());
                lines.clear();
                base = 0;
            } else {
                lines.push(format!("# {h}"));
                base = 1;
            }
            continue;
        }
        if let Some((done, text)) = list_item(line) {
            while stack.last().is_some_and(|(i, _)| *i >= indent) {
                stack.pop();
            }
            let depth = stack.last().map_or(base, |(_, d)| d + 1);
            stack.push((indent, depth));
            last_item_depth = Some(depth);
            let mark = if done { 'x' } else { '-' };
            lines.push(format!("{}{mark} {}", "  ".repeat(depth), text.trim()));
            continue;
        }
        let depth = last_item_depth.map_or(base, |d| d + 1);
        // A note at the margin of a file without... it has a title, always.
        lines.push(format!("{}{}", "  ".repeat(depth), note(line)));
    }
    let title = title.unwrap_or_else(|| name.to_string());
    let mut out = vec![title, String::new()];
    while lines.first().is_some_and(|l| l.is_empty()) {
        lines.remove(0);
    }
    out.extend(lines);
    finish(out)
}

fn heading(line: &str) -> Option<&str> {
    let hashes = line.chars().take_while(|&c| c == '#').count();
    if (1..=6).contains(&hashes) && line[hashes..].starts_with(' ') {
        Some(line[hashes..].trim().trim_end_matches('#').trim())
    } else {
        None
    }
}

/// `- [x] text`, `* [ ] text`, `1. text`: whether it is done, and its text.
fn list_item(line: &str) -> Option<(bool, &str)> {
    let rest = if let Some(r) = line
        .strip_prefix("- ")
        .or_else(|| line.strip_prefix("* "))
        .or_else(|| line.strip_prefix("+ "))
    {
        r
    } else {
        let digits = line.chars().take_while(char::is_ascii_digit).count();
        let r = &line[digits..];
        if digits == 0 {
            return None;
        }
        r.strip_prefix(". ").or_else(|| r.strip_prefix(") "))?
    };
    let rest = rest.trim_start();
    for (box_, done) in [("[ ]", false), ("[x]", true), ("[X]", true)] {
        if let Some(t) = rest.strip_prefix(box_) {
            return Some((done, t));
        }
    }
    Some((false, rest))
}

/// A todo.txt file: `x` lines are done, `(A)` to `(C)` become `!!!` to `!`,
/// `+project` and `@context` become tags (todo.txt's `@` is a place, not a
/// person), dates of creation and completion are dropped, and `due:` and
/// other `key:value` pairs are kept.
pub fn todotxt(src: &str, name: &str) -> String {
    let mut out = vec![name.to_string(), String::new()];
    for raw in src.lines() {
        let mut words: Vec<&str> = raw.split_whitespace().collect();
        if words.is_empty() {
            continue;
        }
        let done = words[0] == "x";
        if done {
            words.remove(0);
        }
        let mut priority = 0;
        if let Some(p) = words.first().and_then(|w| todo_priority(w)) {
            priority = p;
            words.remove(0);
        }
        // A completion date and a creation date may follow, in that order.
        while words.first().is_some_and(|w| is_date(w)) {
            words.remove(0);
        }
        let mut text = Vec::new();
        for w in words {
            if let Some(p) = w.strip_prefix('+').filter(|p| !p.is_empty()) {
                text.push(format!("#{p}"));
            } else if let Some(c) = w.strip_prefix('@').filter(|c| !c.is_empty()) {
                text.push(format!("#{c}"));
            } else if let Some(p) = w
                .strip_prefix("pri:")
                .and_then(|p| todo_priority(&format!("({p})")))
            {
                priority = priority.max(p);
            } else {
                text.push(w.to_string());
            }
        }
        if priority > 0 {
            text.push("!".repeat(priority));
        }
        let mark = if done { 'x' } else { '-' };
        out.push(format!("{mark} {}", text.join(" ")));
    }
    finish(out)
}

fn todo_priority(w: &str) -> Option<usize> {
    let b = w.as_bytes();
    if b.len() == 3 && b[0] == b'(' && b[2] == b')' && b[1].is_ascii_uppercase() {
        Some(match b[1] {
            b'A' => 3,
            b'B' => 2,
            _ => 1,
        })
    } else {
        None
    }
}

fn is_date(w: &str) -> bool {
    crate::Date::parse_iso(w).is_some()
}

/// GitHub issues, from `gh issue list --json
/// number,title,state,stateReason,labels,assignees,milestone` (or pull
/// requests, from `gh pr list` with the same fields): milestones become
/// groups, labels tags, assignees people, closed issues done, and each item
/// carries `issue:NUMBER`. Issues without a milestone come first.
pub fn github(json: &str, name: &str) -> Result<String, String> {
    let v: Value = serde_json::from_str(json).map_err(|e| format!("not JSON from gh: {e}"))?;
    let issues = v
        .as_array()
        .ok_or("expected a JSON array of issues, as gh issue list --json prints")?;
    let mut groups: Vec<(Option<String>, Vec<String>)> = vec![(None, Vec::new())];
    for i in issues {
        let number = i["number"].as_u64().ok_or("an issue has no number")?;
        let title = i["title"].as_str().unwrap_or("").trim();
        // `gh pr list` says MERGED where `gh issue list` says CLOSED; an
        // issue closed as not planned was dropped, not done.
        let state = i["state"].as_str().unwrap_or("").to_ascii_uppercase();
        let mark = match state.as_str() {
            _ if i["stateReason"].as_str() == Some("NOT_PLANNED") => '/',
            "CLOSED" | "MERGED" => 'x',
            _ => '-',
        };
        let mut words = vec![title.to_string()];
        for a in i["assignees"].as_array().into_iter().flatten() {
            if let Some(login) = a["login"].as_str() {
                words.push(format!("@{login}"));
            }
        }
        for l in i["labels"].as_array().into_iter().flatten() {
            if let Some(tag) = l["name"].as_str().map(tag)
                && !tag.is_empty()
            {
                words.push(format!("#{tag}"));
            }
        }
        words.push(format!("issue:{number}"));
        let line = format!("{mark} {}", words.join("  "));
        let milestone = i["milestone"]["title"].as_str().map(str::to_string);
        match groups.iter_mut().find(|(m, _)| *m == milestone) {
            Some((_, items)) => items.push(line),
            None => groups.push((milestone, vec![line])),
        }
    }
    let mut out = vec![name.to_string(), String::new()];
    for (milestone, items) in groups {
        match milestone {
            None => out.extend(items),
            Some(m) => {
                out.push(format!("# {m}"));
                out.extend(items.into_iter().map(|l| format!("  {l}")));
            }
        }
    }
    Ok(finish(out))
}

/// A label as a tag (§4.3): letters, digits, `_` and `-`, with spaces as `-`.
fn tag(label: &str) -> String {
    label
        .trim()
        .chars()
        .filter_map(|c| match c {
            c if c.is_alphanumeric() || c == '_' || c == '-' => Some(c),
            ' ' | '/' | ':' | '.' => Some('-'),
            _ => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markdown_nesting_and_done_items_become_sprig() {
        let md = "# Launch\n\n- [x] Logo\n- [ ] Menu\n  - [ ] Prices\n    Check with Teo.\n## Later\n* Window lettering\n";
        assert_eq!(
            markdown(md, "x"),
            "Launch\n\nx Logo\n- Menu\n  - Prices\n    Check with Teo.\n# Later\n  - Window lettering\n"
        );
    }

    #[test]
    fn todotxt_priorities_projects_and_dates_carry_over() {
        let t = "(A) 2026-09-01 Call the bank +money @phone due:2026-10-02\nx 2026-09-20 2026-09-01 Pick a name +launch\n";
        assert_eq!(
            todotxt(t, "Todo"),
            "Todo\n\n- Call the bank #money #phone due:2026-10-02 !!!\nx Pick a name #launch\n"
        );
    }

    #[test]
    fn a_note_that_would_read_as_an_item_is_escaped() {
        assert_eq!(note("- not an item"), "\\- not an item");
        assert_eq!(note("x marks"), "\\x marks");
        assert_eq!(note("x-ray"), "x-ray");
    }
}
