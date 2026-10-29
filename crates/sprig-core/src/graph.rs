//! The dependency graph, for Graphviz and Mermaid (spec/tools.md T5): every
//! open item that waits on something through `after:`, and everything it
//! waits on, grouped by file.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use crate::token;
use crate::workspace::{Resolution, Resolved, Status, link_target};

/// An edge: the item waited on, then the item waiting.
struct Edge<'r> {
    from: &'r Resolved,
    to: &'r Resolved,
}

/// The nodes each `after:` value can name, in each file's own tree.
struct Names<'r> {
    anchors: HashMap<(&'r str, &'r str), &'r Resolved>,
    roots: HashMap<&'r str, &'r Resolved>,
}

fn names(res: &Resolution) -> Names<'_> {
    fn walk<'r>(
        file: &'r str,
        n: &'r Resolved,
        out: &mut HashMap<(&'r str, &'r str), &'r Resolved>,
    ) {
        for c in n.children.iter().filter(|c| !c.grafted) {
            if let Some(a) = c.tokens.anchor() {
                // The first item with an anchor names it (§4.5).
                out.entry((file, a.name.as_str())).or_insert(c);
            }
            walk(file, c, out);
        }
    }
    let mut anchors = HashMap::new();
    let mut roots = HashMap::new();
    for t in res.trees() {
        roots.insert(t.file.as_str(), &t.root);
        walk(&t.file, &t.root, &mut anchors);
    }
    Names { anchors, roots }
}

fn edges(res: &Resolution) -> Vec<Edge<'_>> {
    let names = names(res);
    let target = |from: &str, value: &str| -> Option<&Resolved> {
        if let Some(a) = value.strip_prefix('^') {
            let file = names.roots.get_key_value(from)?.0;
            return names.anchors.get(&(*file, a)).copied();
        }
        let (_, link) = token::links(value).into_iter().next()?;
        let path = link_target(from, &link.path)?;
        let file = *names.roots.get_key_value(path.as_str())?.0;
        match &link.anchor {
            Some(a) => names.anchors.get(&(file, a.as_str())).copied(),
            None => names.roots.get(file).copied(),
        }
    };
    fn walk<'r>(
        n: &'r Resolved,
        target: &dyn Fn(&str, &str) -> Option<&'r Resolved>,
        out: &mut Vec<Edge<'r>>,
    ) {
        for c in n.children.iter().filter(|c| !c.grafted) {
            if c.status.is_open() {
                for a in &c.tokens.after {
                    if let Some(t) = target(&c.file, &a.value) {
                        out.push(Edge { from: t, to: c });
                    }
                }
            }
            walk(c, target, out);
        }
    }
    let mut out = Vec::new();
    for t in res.trees() {
        walk(&t.root, &target, &mut out);
    }
    out
}

/// A node id that stays the same from one run to the next: the anchor where
/// there is one, the line otherwise.
fn id(n: &Resolved) -> String {
    let file = n.file.trim_end_matches(".sprig");
    let part = match n.tokens.anchor() {
        Some(a) => a.name.clone(),
        None if n.line == 0 => "root".into(),
        None => format!("l{}", n.line),
    };
    format!("{file}_{part}")
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect()
}

fn state(n: &Resolved) -> &'static str {
    match n.status {
        _ if n.blocked && n.status.is_open() => "blocked",
        Status::Done => "done",
        Status::Doing => "doing",
        Status::Dropped => "dropped",
        Status::Later => "later",
        Status::Question => "ask",
        Status::Empty => "empty",
        Status::Todo => "todo",
    }
}

/// The nodes in the graph, by file, in file and line order.
fn nodes<'r>(edges: &[Edge<'r>]) -> BTreeMap<&'r str, BTreeMap<usize, &'r Resolved>> {
    let mut by_file: BTreeMap<&str, BTreeMap<usize, &Resolved>> = BTreeMap::new();
    for e in edges {
        for n in [e.from, e.to] {
            by_file.entry(&n.file).or_default().insert(n.line, n);
        }
    }
    by_file
}

/// Graphviz: one cluster per file, edges from what is waited on to what
/// waits, done work filled and blocked work dashed.
pub fn dot(res: &Resolution) -> String {
    let edges = edges(res);
    let mut out = String::from(
        "digraph sprig {\n  rankdir=LR;\n  node [shape=box, style=rounded, fontname=\"Helvetica\"];\n",
    );
    for (i, (file, ns)) in nodes(&edges).iter().enumerate() {
        out.push_str(&format!(
            "  subgraph cluster_{i} {{\n    label=\"{}\";\n",
            dot_text(file)
        ));
        for n in ns.values() {
            let style = match state(n) {
                "done" => ", style=\"rounded,filled\", fillcolor=\"#d9ead3\"",
                "blocked" => ", style=\"rounded,dashed\"",
                "dropped" | "later" => ", fontcolor=\"#777777\"",
                "doing" => ", penwidth=2",
                _ => "",
            };
            out.push_str(&format!(
                "    {} [label=\"{}\"{style}];\n",
                id(n),
                dot_text(&label(n))
            ));
        }
        out.push_str("  }\n");
    }
    let mut seen = BTreeSet::new();
    for e in &edges {
        if seen.insert((id(e.from), id(e.to))) {
            out.push_str(&format!("  {} -> {};\n", id(e.from), id(e.to)));
        }
    }
    out.push_str("}\n");
    out
}

/// Mermaid, which GitHub renders in Markdown: the same graph as [`dot`].
pub fn mermaid(res: &Resolution) -> String {
    let edges = edges(res);
    let mut out = String::from("flowchart LR\n");
    for (file, ns) in nodes(&edges) {
        out.push_str(&format!(
            "  subgraph {}[\"{}\"]\n",
            id_text(file),
            mermaid_text(file)
        ));
        for n in ns.values() {
            out.push_str(&format!(
                "    {}[\"{}\"]:::{}\n",
                id(n),
                mermaid_text(&label(n)),
                state(n)
            ));
        }
        out.push_str("  end\n");
    }
    let mut seen = BTreeSet::new();
    for e in &edges {
        if seen.insert((id(e.from), id(e.to))) {
            out.push_str(&format!("  {} --> {}\n", id(e.from), id(e.to)));
        }
    }
    out.push_str(
        "  classDef done fill:#d9ead3,stroke:#6aa84f\n  classDef blocked stroke-dasharray:4 3\n  classDef doing stroke-width:2px\n  classDef dropped color:#777777\n  classDef later color:#777777\n",
    );
    out
}

fn label(n: &Resolved) -> String {
    let mark = n
        .mark
        .map(|m| format!("{} ", m.as_char()))
        .unwrap_or_default();
    format!("{mark}{}", n.label)
}

fn dot_text(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

fn mermaid_text(s: &str) -> String {
    s.replace('"', "#quot;")
}

fn id_text(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Date, Workspace};

    fn res() -> Resolution {
        let mut ws = Workspace::new();
        ws.insert(
            "lease.sprig",
            "Lease\n\n- Sign  ^signed  after:^terms\nx Terms  ^terms\n- Paint\n",
        );
        ws.insert(
            "bakery.sprig",
            "Bakery\n\n- Credit  after:[[lease^signed]]\n",
        );
        ws.resolve(Date::parse_iso("2026-10-01").unwrap())
    }

    #[test]
    fn edges_run_from_what_is_waited_on_to_what_waits() {
        let d = dot(&res());
        assert!(d.contains("lease_signed -> bakery_l3;"), "{d}");
        assert!(d.contains("lease_terms -> lease_signed;"), "{d}");
        assert!(
            !d.contains("Paint"),
            "an item with no edge is left out: {d}"
        );
        let m = mermaid(&res());
        assert!(m.contains("  lease_signed --> bakery_l3\n"), "{m}");
    }

    #[test]
    fn the_output_is_the_same_on_every_run() {
        assert_eq!(dot(&res()), dot(&res()));
        assert_eq!(mermaid(&res()), mermaid(&res()));
    }
}
