//! Dated work: every open item with its own due date, the walk that `sprig
//! agenda` and the iCalendar export share, so they always list the same
//! items (spec/tools.md T5).
//!
//! # Example
//!
//! ```
//! use sprig_core::{Date, Workspace, agenda, export};
//!
//! let mut ws = Workspace::new();
//! ws.insert("plan.sprig", "Plan\n\n- Call  due:2026-10-02\n- Pay  due:2026-09-20\n- Bins  every:fri  due:2026-10-02\n");
//! let today = Date::new(2026, 10, 1).unwrap();
//! let res = ws.resolve(today);
//!
//! let w = agenda::window(agenda::dated(&res), today, 14);
//! assert_eq!(w.late[0].node.label, "Pay");
//! let friday = &w.days[&Date::new(2026, 10, 2).unwrap()];
//! assert_eq!(friday.len(), 2);
//! assert_eq!(friday[1].every, Some("fri"));
//!
//! // The same items as a calendar, each with a UID that survives edits.
//! let ics = export::ics(&agenda::dated(&res), today);
//! assert!(ics.starts_with("BEGIN:VCALENDAR\r\n"));
//! assert_eq!(export::uid(friday[0].node), "plan.sprig:call@sprig");
//! assert!(export::csv(&res).starts_with("file,line,mark,text,status"));
//! ```

use std::collections::BTreeMap;

use crate::date::Date;
use crate::workspace::{Resolution, Resolved};

/// An open item and the day it is due.
#[derive(Clone, Debug)]
pub struct Dated<'r> {
    /// The item, as its own file writes it.
    pub node: &'r Resolved,
    /// Its own `due:`, read against today; for a recurring item, the next occurrence.
    pub due: Date,
    /// The `every:` rule of a recurring item (§10.3). Its due date is the
    /// next occurrence, and it is listed on that day only.
    pub every: Option<&'r str>,
    /// The labels of the items above it in its file.
    pub path: Vec<&'r str>,
}

/// Every open item with a readable due date of its own, each once, read in
/// the file that writes it, ordered by date, then file, then line.
pub fn dated(res: &Resolution) -> Vec<Dated<'_>> {
    fn walk<'r>(n: &'r Resolved, path: &mut Vec<&'r str>, out: &mut Vec<Dated<'r>>) {
        for c in n.children.iter().filter(|c| !c.grafted) {
            if c.status.is_open()
                && let Some(due) = c.due
            {
                out.push(Dated {
                    node: c,
                    due,
                    every: c.tokens.field("every").map(|f| f.value.as_str()),
                    path: path.clone(),
                });
            }
            path.push(&c.label);
            walk(c, path, out);
            path.pop();
        }
    }
    let mut out = Vec::new();
    for t in res.trees() {
        walk(&t.root, &mut Vec::new(), &mut out);
    }
    out.sort_by(|a, b| (a.due, &a.node.file, a.node.line).cmp(&(b.due, &b.node.file, b.node.line)));
    out
}

/// The items due before `today`, and those due on each day from `today`
/// for `days` days.
pub struct Window<'r> {
    /// Open items due before today, earliest first.
    pub late: Vec<Dated<'r>>,
    /// The days of the window that have something due, each with its items in file order.
    pub days: BTreeMap<Date, Vec<Dated<'r>>>,
}

/// Sorts dated items into what is late and what falls on each of the `days`
/// days from `today`, today included. Items due later are left out.
pub fn window<'r>(items: Vec<Dated<'r>>, today: Date, days: i64) -> Window<'r> {
    let end = today.add_days(days);
    let mut w = Window {
        late: Vec::new(),
        days: BTreeMap::new(),
    };
    for d in items {
        if d.due < today {
            w.late.push(d);
        } else if d.due < end {
            w.days.entry(d.due).or_default().push(d);
        }
    }
    w
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Workspace;

    #[test]
    fn a_recurring_item_appears_once_on_its_next_due_date() {
        let mut ws = Workspace::new();
        ws.insert(
            "p.sprig",
            "P\n\n- Numbers  every:fri  due:2026-10-02\n- Old  due:2026-09-01\nx Done  due:2026-10-03\n",
        );
        let today = Date::parse_iso("2026-10-01").unwrap();
        let r = ws.resolve(today);
        let w = window(dated(&r), today, 30);
        assert_eq!(w.late.len(), 1);
        let shown: Vec<_> = w
            .days
            .values()
            .flatten()
            .map(|d| d.node.label.as_str())
            .collect();
        assert_eq!(shown, ["Numbers"], "once, and the done item not at all");
        assert_eq!(w.days.values().flatten().next().unwrap().every, Some("fri"));
    }
}
