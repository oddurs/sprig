//! What can be done now.
//!
//! The same answer as the playground's Next up view: open leaves, split into
//! those that can start and those waiting on something, the first ranked by
//! in progress, then priority, then due date, then the order they are written.
//!
//! # Example
//!
//! ```
//! use sprig_core::{Date, Workspace, next};
//!
//! let mut ws = Workspace::new();
//! ws.insert("plan.sprig", "Plan\n\n- Logo  ^logo  @kim\n- Menus  after:^logo\n? Which font?  @teo\n");
//! let res = ws.resolve(Date::new(2026, 10, 1).unwrap());
//! let trees: Vec<_> = res.trees().collect();
//!
//! let n = next::next(&trees, None);
//! assert_eq!(n.ready.len(), 2);
//! assert_eq!(n.waiting[0].node.label, "Menus");
//! assert_eq!(n.questions(), 1);
//! assert_eq!(next::next(&trees, Some("kim")).ready[0].node.label, "Logo");
//! ```

use crate::date::Date;
use crate::workspace::{Resolved, Status, Tree};

/// One open leaf.
#[derive(Clone, Debug)]
pub struct Entry<'t> {
    /// The item.
    pub node: &'t Resolved,
    /// Labels from the top of its tree down to its parent; empty for an item
    /// at the top.
    pub path: Vec<&'t str>,
    /// The label of the tree's root: the plan it was found in.
    pub plan: &'t str,
    /// Its own `due:` as written, or the nearest one above it.
    pub due: Option<&'t str>,
    /// That due date read against today, when it reads as one.
    pub due_date: Option<Date>,
    /// The highest priority on it or above it.
    pub priority: u8,
}

#[derive(Clone, Debug, Default)]
/// The open leaves of a set of trees, split by whether anything is in their way.
pub struct Next<'t> {
    /// What can be done now, in progress first, then by priority and due date.
    pub ready: Vec<Entry<'t>>,
    /// What waits on something unfinished or on a start date, in the same order.
    pub waiting: Vec<Entry<'t>>,
}

impl Next<'_> {
    /// Open questions among what is ready.
    pub fn questions(&self) -> usize {
        self.ready
            .iter()
            .filter(|e| e.node.status == Status::Question)
            .count()
    }
}

/// Open leaves of `trees`, for `who` if given (§6.1: their effective owner).
///
/// An item mounted in more than one of the trees is listed once, where it is
/// first met, so a file grafted into two plans does not double its work.
/// With `who`, that is the first tree in which `who` owns it.
pub fn next<'t>(trees: &[&'t Tree], who: Option<&str>) -> Next<'t> {
    let mut out = Next::default();
    let mut seen = std::collections::HashSet::new();
    for tree in trees {
        let root = &tree.root;
        let due = root
            .tokens
            .field("due")
            .map(|f| (f.value.as_str(), root.due));
        walk(root, &root.label, &mut Vec::new(), due, 0, &mut |e| {
            // Ownership is judged in each tree, since a graft can give a
            // branch a new owner (§6.1), and only then is a repeat dropped.
            if who.is_some_and(|w| !e.node.people.iter().any(|p| p == w)) {
                return;
            }
            if !seen.insert((e.node.file.as_str(), e.node.line)) {
                return;
            }
            if e.node.blocked {
                out.waiting.push(e);
            } else {
                out.ready.push(e);
            }
        });
    }
    // Stable, so equal keys keep the order they are written in.
    out.ready.sort_by_key(|e| {
        (
            e.node.status != Status::Doing,
            std::cmp::Reverse(e.priority),
            e.due_date.map_or((1, None), |d| (0, Some(d))),
        )
    });
    out
}

fn walk<'t>(
    n: &'t Resolved,
    plan: &'t str,
    path: &mut Vec<&'t str>,
    due: Option<(&'t str, Option<Date>)>,
    priority: u8,
    f: &mut impl FnMut(Entry<'t>),
) {
    for c in &n.children {
        // The nearest due: written, whether or not it reads as a date.
        let due = c
            .tokens
            .field("due")
            .map(|f| (f.value.as_str(), c.due))
            .or(due);
        let priority = priority.max(c.tokens.priority);
        if c.is_leaf() {
            if c.status.is_open() {
                f(Entry {
                    node: c,
                    path: path.clone(),
                    plan,
                    due: due.map(|d| d.0),
                    due_date: due.and_then(|d| d.1),
                    priority,
                });
            }
        } else {
            path.push(&c.label);
            walk(c, plan, path, due, priority, f);
            path.pop();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workspace::Workspace;

    fn resolve(src: &str) -> crate::workspace::Resolution {
        let mut w = Workspace::new();
        w.insert("p.sprig", src);
        w.resolve(Date::parse_iso("2026-10-01").unwrap())
    }

    fn texts<'a>(es: &'a [Entry<'_>]) -> Vec<&'a str> {
        es.iter().map(|e| e.node.label.as_str()).collect()
    }

    #[test]
    fn in_progress_comes_first_then_priority_then_due_then_order() {
        let r = resolve(
            "P  due:2026-12-01\n\n- a\n- b  due:2026-10-05\n- c  !\n~ d\n- e  due:2026-10-03\n- f\n# G  due:2026-10-02\n  - g\n",
        );
        let n = next(&[r.tree("p.sprig").unwrap()], None);
        assert_eq!(texts(&n.ready), ["d", "c", "g", "e", "b", "a", "f"]);
    }

    #[test]
    fn waiting_items_are_set_apart_and_closed_ones_left_out() {
        let r = resolve(
            "P\n\n- Design  ^d\n- Build  after:^d\n  - Strip\nx Done\n/ Dropped\n? Ask\n  = Yes\n? Open\n",
        );
        let n = next(&[r.tree("p.sprig").unwrap()], None);
        assert_eq!(texts(&n.ready), ["Design", "Open"]);
        assert_eq!(texts(&n.waiting), ["Strip"]);
        assert_eq!(n.ready[1].path, Vec::<&str>::new());
        assert_eq!(n.waiting[0].path, ["Build"]);
        assert_eq!(n.questions(), 1);
    }

    #[test]
    fn who_filters_by_effective_owner() {
        let r = resolve("P  @dana\n\n- a\n- b  @teo\n  - c\n");
        let n = next(&[r.tree("p.sprig").unwrap()], Some("teo"));
        assert_eq!(texts(&n.ready), ["c"]);
    }
}
