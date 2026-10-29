//! Plans as other programs read them: an iCalendar feed of dated work and a
//! CSV of every item (spec/tools.md T5). The JSON form is the tree format
//! itself, in [`crate::json`].

use crate::agenda::Dated;
use crate::date::Date;
use crate::workspace::{Resolution, Resolved};

/// One all-day event per dated item (RFC 5545), the same items `sprig
/// agenda` lists. `today` stamps the feed, so the same plans on the same day
/// give the same bytes.
pub fn ics(items: &[Dated<'_>], today: Date) -> String {
    let mut out = String::new();
    let mut line = |s: String| fold(&s, &mut out);
    line("BEGIN:VCALENDAR".into());
    line("VERSION:2.0".into());
    line("PRODID:-//Sprig//sprig export//EN".into());
    line("CALSCALE:GREGORIAN".into());
    line("X-WR-CALNAME:Sprig plans".into());
    let stamp = format!("{}T000000Z", compact(today));
    for d in items {
        let n = d.node;
        line("BEGIN:VEVENT".into());
        line(format!("UID:{}", uid(n)));
        line(format!("DTSTAMP:{stamp}"));
        line(format!("DTSTART;VALUE=DATE:{}", compact(d.due)));
        line(format!("DTEND;VALUE=DATE:{}", compact(d.due.add_days(1))));
        line(format!("SUMMARY:{}", text(&n.label)));
        let mut about = format!("{}:{}", n.file, n.line);
        if !d.path.is_empty() {
            about = format!("{} › {about}", d.path.join(" › "));
        }
        if let Some(rule) = d.every {
            about.push_str(&format!("\nevery:{rule}"));
        }
        line(format!("DESCRIPTION:{}", text(&about)));
        if !n.tokens.tags.is_empty() {
            let tags: Vec<String> = n.tokens.tags.iter().map(|t| text(t)).collect();
            line(format!("CATEGORIES:{}", tags.join(",")));
        }
        line("END:VEVENT".into());
    }
    line("END:VCALENDAR".into());
    out
}

/// An event's identity across edits: the anchor when the item has one, so
/// renaming it keeps its UID; otherwise its file and its words.
pub fn uid(n: &Resolved) -> String {
    match n.tokens.anchor() {
        Some(a) => format!("{}^{}@sprig", n.file, a.name),
        None => {
            let words: Vec<String> = n
                .label
                .split(|c: char| !c.is_alphanumeric())
                .filter(|w| !w.is_empty())
                .map(str::to_lowercase)
                .collect();
            format!("{}:{}@sprig", n.file, words.join("-"))
        }
    }
}

fn compact(d: Date) -> String {
    d.to_string().replace('-', "")
}

/// RFC 5545 §3.3.11 TEXT: backslash, semicolon, comma and newline escaped.
fn text(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace(';', "\\;")
        .replace(',', "\\,")
        .replace('\n', "\\n")
}

/// RFC 5545 §3.1: lines of at most 75 octets, continued with a space, and
/// never split inside a character.
fn fold(s: &str, out: &mut String) {
    let mut used = 0;
    for c in s.chars() {
        if used + c.len_utf8() > 75 {
            out.push_str("\r\n ");
            used = 1;
        }
        out.push(c);
        used += c.len_utf8();
    }
    out.push_str("\r\n");
}

/// Every item, one row each, in the file that writes it: the `--plain`
/// columns of `find`, plus status, people and the file and line, quoted
/// where RFC 4180 needs it.
pub fn csv(res: &Resolution) -> String {
    fn walk(n: &Resolved, path: &mut Vec<String>, out: &mut String) {
        for c in n.children.iter().filter(|c| !c.grafted) {
            let mark = c.mark.map(|m| m.as_char().to_string()).unwrap_or_default();
            let status = if c.blocked && c.status.is_open() {
                "blocked"
            } else {
                c.status.as_str()
            };
            let row = [
                c.file.clone(),
                c.line.to_string(),
                mark,
                c.label.clone(),
                status.to_string(),
                c.people.join(" "),
                c.due.map(|d| d.to_string()).unwrap_or_default(),
                format!("{}/{}", c.progress.done, c.progress.total),
                path.join(" › "),
            ];
            let cells: Vec<String> = row.iter().map(|v| quote(v)).collect();
            out.push_str(&cells.join(","));
            out.push_str("\r\n");
            path.push(c.label.clone());
            walk(c, path, out);
            path.pop();
        }
    }
    let mut out = String::from("file,line,mark,text,status,people,due,progress,path\r\n");
    for t in res.trees() {
        walk(&t.root, &mut Vec::new(), &mut out);
    }
    out
}

fn quote(v: &str) -> String {
    if v.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", v.replace('"', "\"\""))
    } else {
        v.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn long_lines_fold_without_splitting_a_character() {
        let mut out = String::new();
        fold(&format!("SUMMARY:{}", "é".repeat(60)), &mut out);
        for l in out.split("\r\n").filter(|l| !l.is_empty()) {
            assert!(l.len() <= 75, "{} octets: {l}", l.len());
        }
        assert_eq!(
            out.replace("\r\n ", "").trim_end(),
            format!("SUMMARY:{}", "é".repeat(60))
        );
    }

    #[test]
    fn text_values_are_escaped() {
        assert_eq!(text("a, b; c\\d\ne"), "a\\, b\\; c\\\\d\\ne");
    }
}
