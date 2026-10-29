//! The conformance suite: every directory under `tests/conformance/<section>/`
//! is an example, found without registering it. See that directory's README.
//!
//! `SPRIG_BLESS=1 cargo test --test conformance` rewrites each `expected.json`
//! from the core's output, keeping only the layers the file already states.
//! A blessed file is a claim about the spec, so read the diff before
//! committing it.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;
use sprig_core::{Date, Workspace, edit, json};

/// The `today` of an example that does not set one: a fixed day, so no
/// example depends on when it runs.
const DEFAULT_TODAY: &str = "2026-10-01";

fn suite() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/conformance")
}

fn schema() -> jsonschema::Validator {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../spec/tree.schema.json");
    let schema: Value = serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap();
    jsonschema::validator_for(&schema).expect("the tree schema is valid JSON Schema")
}

/// Every example directory: `<suite>/<section>/<example>/`, in path order.
fn examples() -> Vec<PathBuf> {
    let mut out = Vec::new();
    for section in sorted_dirs(&suite()) {
        out.extend(sorted_dirs(&section));
    }
    out
}

fn sorted_dirs(dir: &Path) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("{}: {e}", dir.display()))
        .map(|e| e.unwrap().path())
        .filter(|p| p.is_dir())
        .collect();
    out.sort();
    out
}

#[derive(Default)]
struct Meta {
    today: Option<String>,
    rules: Vec<String>,
    /// Lines of main.sprig to tick, in order (§10.4); `after.sprig` holds
    /// the file that results.
    tick: Vec<usize>,
    /// Format main.sprig (§3.3, §9.4); `after.sprig` holds the result.
    format: bool,
}

fn meta(dir: &Path) -> Result<Meta, String> {
    let path = dir.join("meta.toml");
    if !path.exists() {
        return Ok(Meta::default());
    }
    let text = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let table: toml::Table = text.parse().map_err(|e| format!("meta.toml: {e}"))?;
    let mut m = Meta::default();
    for (k, v) in table {
        match (k.as_str(), v) {
            ("today", toml::Value::String(s)) => m.today = Some(s),
            ("rules", toml::Value::Array(a)) => {
                for r in a {
                    match r {
                        toml::Value::String(s) => m.rules.push(s),
                        other => return Err(format!("meta.toml: rule {other} is not a string")),
                    }
                }
            }
            ("tick", toml::Value::Array(a)) => {
                for l in a {
                    match l {
                        toml::Value::Integer(n) if n > 0 => m.tick.push(n as usize),
                        other => return Err(format!("meta.toml: tick {other} is not a line")),
                    }
                }
            }
            ("format", toml::Value::Boolean(b)) => m.format = b,
            (k, v) => return Err(format!("meta.toml: unexpected key {k} = {v}")),
        }
    }
    Ok(m)
}

/// Every `.sprig` file under `dir`, by path relative to it.
fn sources(dir: &Path) -> BTreeMap<String, String> {
    fn walk(base: &Path, dir: &Path, out: &mut BTreeMap<String, String>) {
        for e in fs::read_dir(dir).unwrap() {
            let p = e.unwrap().path();
            if p.is_dir() {
                walk(base, &p, out);
            } else if p.extension().is_some_and(|x| x == "sprig") {
                let rel: Vec<String> = p
                    .strip_prefix(base)
                    .unwrap()
                    .components()
                    .map(|c| c.as_os_str().to_string_lossy().into_owned())
                    .collect();
                // Read as bytes: an example may hold any bytes the parser
                // must survive, and git may not leave line endings alone.
                let src = String::from_utf8(fs::read(&p).unwrap()).expect("examples are UTF-8");
                // An edit example's expected result is not part of its plan.
                if rel != ["after.sprig"] {
                    out.insert(rel.join("/"), src);
                }
            }
        }
    }
    let mut out = BTreeMap::new();
    walk(dir, dir, &mut out);
    out
}

/// The core's answer for an example, as a tree document.
fn actual(dir: &Path, today: Date) -> Value {
    let mut w = Workspace::new();
    for (p, s) in sources(dir) {
        w.insert(p, s);
    }
    let r = w.resolve(today);
    json::tree_document(&r, "main.sprig", today).expect("main.sprig is in the workspace")
}

/// Diagnostics are compared on what the example states, and never on their
/// wording: a message may be improved without changing what it reports.
fn project_diagnostics(expected: &Value, actual: &Value) -> Value {
    let (Value::Array(exp), Value::Array(act)) = (expected, actual) else {
        return actual.clone();
    };
    let keys: Vec<&str> = match exp.first() {
        Some(Value::Object(m)) => m.keys().map(String::as_str).collect(),
        _ => vec!["code", "file", "line"],
    };
    let out = act
        .iter()
        .map(|d| {
            let mut m = serde_json::Map::new();
            for k in &keys {
                if *k != "message"
                    && *k != "help"
                    && let Some(v) = d.get(*k)
                {
                    m.insert((*k).to_string(), v.clone());
                }
            }
            Value::Object(m)
        })
        .collect();
    Value::Array(out)
}

fn strip_wording(v: &Value) -> Value {
    match v {
        Value::Array(a) => Value::Array(
            a.iter()
                .map(|d| match d {
                    Value::Object(m) => Value::Object(
                        m.iter()
                            .filter(|(k, _)| *k != "message" && *k != "help")
                            .map(|(k, v)| (k.clone(), v.clone()))
                            .collect(),
                    ),
                    other => other.clone(),
                })
                .collect(),
        ),
        other => other.clone(),
    }
}

/// Paths where two JSON values differ, with both sides. Numbers compare by
/// value, so `4` and `4.0` agree.
fn diff(path: &str, expected: &Value, actual: &Value, out: &mut Vec<String>) {
    match (expected, actual) {
        (Value::Object(e), Value::Object(a)) => {
            let mut keys: Vec<&String> = e.keys().chain(a.keys()).collect();
            keys.sort();
            keys.dedup();
            for k in keys {
                let p = format!("{path}.{k}");
                match (e.get(k), a.get(k)) {
                    (Some(x), Some(y)) => diff(&p, x, y, out),
                    (Some(x), None) => out.push(format!("  {p}: expected {x}, missing")),
                    (None, Some(y)) => out.push(format!("  {p}: not expected, got {y}")),
                    (None, None) => unreachable!(),
                }
            }
        }
        (Value::Array(e), Value::Array(a)) => {
            for i in 0..e.len().max(a.len()) {
                let p = format!("{path}[{i}]");
                match (e.get(i), a.get(i)) {
                    (Some(x), Some(y)) => diff(&p, x, y, out),
                    (Some(x), None) => out.push(format!("  {p}: expected {x}, missing")),
                    (None, Some(y)) => out.push(format!("  {p}: not expected, got {y}")),
                    (None, None) => unreachable!(),
                }
            }
        }
        (Value::Number(e), Value::Number(a)) if e.as_f64() == a.as_f64() => {}
        (e, a) if e == a => {}
        (e, a) => out.push(format!("  {path}: expected {e}, got {a}")),
    }
}

/// Pretty JSON with short lists of scalars, like spans, kept on one line.
fn pretty(v: &Value, depth: usize) -> String {
    let pad = "  ".repeat(depth + 1);
    let end = "  ".repeat(depth);
    match v {
        Value::Object(m) if !m.is_empty() => {
            let items: Vec<String> = m
                .iter()
                .map(|(k, v)| {
                    format!(
                        "{pad}{}: {}",
                        Value::String(k.clone()),
                        pretty(v, depth + 1)
                    )
                })
                .collect();
            format!("{{\n{}\n{end}}}", items.join(",\n"))
        }
        Value::Array(a) if a.iter().all(|x| !x.is_object() && !x.is_array()) => {
            let items: Vec<String> = a.iter().map(|x| x.to_string()).collect();
            let line = format!("[{}]", items.join(", "));
            if line.len() <= 72 || a.len() <= 1 {
                line
            } else {
                let items: Vec<String> = items.iter().map(|i| format!("{pad}{i}")).collect();
                format!("[\n{}\n{end}]", items.join(",\n"))
            }
        }
        Value::Array(a) if !a.is_empty() => {
            let items: Vec<String> = a
                .iter()
                .map(|x| format!("{pad}{}", pretty(x, depth + 1)))
                .collect();
            format!("[\n{}\n{end}]", items.join(",\n"))
        }
        other => other.to_string(),
    }
}

const LAYERS: [&str; 3] = ["syntax", "resolved", "diagnostics"];

/// Runs one example; `Err` carries the report to print.
fn run(dir: &Path, validator: &jsonschema::Validator, bless: bool) -> Result<(), String> {
    let meta = meta(dir)?;
    if !dir.join("main.sprig").exists() {
        return Err("no main.sprig: every example needs an entry file".into());
    }
    let expected_path = dir.join("expected.json");
    let text = fs::read_to_string(&expected_path)
        .map_err(|_| "no expected.json: an example without one tests nothing".to_string())?;
    let expected: Value =
        serde_json::from_str(&text).map_err(|e| format!("expected.json is not JSON: {e}"))?;
    let errors: Vec<String> = validator
        .iter_errors(&expected)
        .map(|e| format!("  {} at {}", e, e.instance_path))
        .collect();
    if !errors.is_empty() {
        return Err(format!(
            "expected.json does not match spec/tree.schema.json:\n{}",
            errors.join("\n")
        ));
    }
    let today = meta.today.as_deref().unwrap_or(DEFAULT_TODAY);
    let today = Date::parse_iso(today)
        .ok_or_else(|| format!("meta.toml: today = {today} is not a date"))?;
    let actual = actual(dir, today);
    let errors: Vec<String> = validator
        .iter_errors(&actual)
        .map(|e| format!("  {} at {}", e, e.instance_path))
        .collect();
    assert!(
        errors.is_empty(),
        "{}: the core's output does not match the schema:\n{}",
        dir.display(),
        errors.join("\n")
    );

    let layers: Vec<&str> = LAYERS
        .into_iter()
        .filter(|l| expected.get(*l).is_some())
        .collect();
    if layers.is_empty() {
        return Err(
            "expected.json states no layer: give it syntax, resolved or diagnostics".into(),
        );
    }

    // An edit example: the file after the edits, byte for byte.
    let mut edited = None;
    if meta.format || !meta.tick.is_empty() {
        let mut src = String::from_utf8(fs::read(dir.join("main.sprig")).unwrap())
            .map_err(|_| "main.sprig is not UTF-8".to_string())?;
        for &line in &meta.tick {
            src = edit::tick(&src, line, today)
                .map_err(|e| format!("tick {line}: {e}"))?
                .0
                .source;
        }
        if meta.format {
            src = edit::format(&src, today).source;
        }
        edited = Some(src);
    }

    if bless {
        if let Some(src) = &edited {
            fs::write(dir.join("after.sprig"), src).unwrap();
        }
        let mut out = serde_json::Map::new();
        out.insert("sprig_tree".into(), actual["sprig_tree"].clone());
        for l in &layers {
            let v = if *l == "diagnostics" {
                strip_wording(&actual[*l])
            } else {
                actual[*l].clone()
            };
            out.insert((*l).to_string(), v);
        }
        let text = pretty(&Value::Object(out), 0) + "\n";
        fs::write(&expected_path, text).unwrap();
        return Ok(());
    }

    let mut found = Vec::new();
    if let Some(src) = &edited {
        match fs::read(dir.join("after.sprig")) {
            Ok(want) if want == src.as_bytes() => {}
            Ok(want) => found.push(format!(
                "after the edits:\n--- after.sprig\n{}\n+++ produced\n{src}",
                String::from_utf8_lossy(&want)
            )),
            Err(_) => found.push("an edit example needs after.sprig".into()),
        }
    }
    for l in layers {
        let a = if l == "diagnostics" {
            project_diagnostics(&expected[l], &actual[l])
        } else {
            actual[l].clone()
        };
        diff(l, &expected[l], &a, &mut found);
    }
    if found.is_empty() {
        Ok(())
    } else {
        let rules = if meta.rules.is_empty() {
            "none cited".to_string()
        } else {
            meta.rules.join(", ")
        };
        Err(format!("rules: {rules}\n{}", found.join("\n")))
    }
}

#[test]
fn every_example_matches_its_expected_tree() {
    let validator = schema();
    let bless = std::env::var_os("SPRIG_BLESS").is_some();
    let examples = examples();
    assert!(!examples.is_empty(), "the suite has no examples");
    let mut failures = Vec::new();
    for dir in &examples {
        if let Err(report) = run(dir, &validator, bless) {
            let shown = dir.strip_prefix(suite()).unwrap_or(dir);
            failures.push(format!("tests/conformance/{}\n{report}", shown.display()));
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {} conformance examples failed:\n\n{}",
        failures.len(),
        examples.len(),
        failures.join("\n\n")
    );
}

fn spec() -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../spec/sprig.md")).unwrap()
}

/// The spec's numbered rules, `§2.6`, with their headings, in order: every
/// `### N.M` heading of chapters 1 to 10. The appendices are not rules.
fn rules(spec: &str) -> Vec<(String, String)> {
    spec.lines()
        .filter_map(|l| l.strip_prefix("### "))
        .filter_map(|h| h.split_once(' '))
        .filter(|(n, _)| {
            n.split('.')
                .next()
                .is_some_and(|c| c.parse::<u32>().is_ok())
        })
        .map(|(n, title)| (format!("§{n}"), title.to_string()))
        .collect()
}

/// Each rule and the examples, as `section/NNN`, that cite it.
fn coverage(spec: &str, cited: &[(String, Vec<String>)]) -> Vec<(String, String, Vec<String>)> {
    rules(spec)
        .into_iter()
        .map(|(rule, title)| {
            let by: Vec<String> = cited
                .iter()
                .filter(|(_, rules)| rules.contains(&rule))
                .map(|(name, _)| name.clone())
                .collect();
            (rule, title, by)
        })
        .collect()
}

/// Every example as `section/NNN`, with the rules its meta.toml cites.
fn citations() -> Vec<(String, Vec<String>)> {
    examples()
        .iter()
        .map(|dir| {
            let section = dir.parent().unwrap().file_name().unwrap().to_string_lossy();
            let name = dir.file_name().unwrap().to_string_lossy();
            let m = meta(dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display()));
            (format!("{section}/{}", &name[..3]), m.rules)
        })
        .collect()
}

/// The rule headings of some chapters that no example in one section of the
/// suite cites.
fn uncited(section: &str, chapters: &[&str]) -> Vec<String> {
    let in_section: Vec<(String, Vec<String>)> = citations()
        .into_iter()
        .filter(|(name, _)| name.starts_with(&format!("{section}/")))
        .collect();
    let cov = coverage(&spec(), &in_section);
    assert!(cov.len() > 4, "the spec's rule headings were not found");
    cov.into_iter()
        .filter(|(rule, _, _)| {
            chapters
                .iter()
                .any(|c| rule[2..].split('.').next() == Some(c))
        })
        .filter(|(_, _, by)| by.is_empty())
        .map(|(rule, _, _)| rule)
        .collect()
}

#[test]
fn the_lines_section_cites_every_line_and_indentation_rule() {
    let missing = uncited("lines", &["1", "2", "3"]);
    assert!(missing.is_empty(), "no lines example cites {missing:?}");
}

#[test]
fn the_tokens_section_cites_every_token_date_estimate_and_recurrence_rule() {
    let missing = uncited("tokens", &["4", "9", "10"]);
    assert!(missing.is_empty(), "no tokens example cites {missing:?}");
}

#[test]
fn the_graph_section_cites_every_status_people_blocking_and_graft_rule() {
    let missing = uncited("graph", &["5", "6", "7", "8"]);
    assert!(missing.is_empty(), "no graph example cites {missing:?}");
}

#[test]
fn every_rule_in_the_spec_has_an_example() {
    let missing: Vec<String> = coverage(&spec(), &citations())
        .into_iter()
        .filter(|(_, _, by)| by.is_empty())
        .map(|(rule, title, _)| format!("{rule} {title}"))
        .collect();
    assert!(
        missing.is_empty(),
        "no example cites:\n  {}",
        missing.join("\n  ")
    );
}

#[test]
fn a_rule_with_no_example_is_reported() {
    // The check above, on a spec with a rule nothing cites, as a new rule
    // would be until its example is written.
    let spec = "### 1.1 Old\n\n### 1.2 New\n\n## Appendix A\n\n### A.1 Not a rule\n";
    let cited = vec![("lines/001".to_string(), vec!["§1.1".to_string()])];
    let cov = coverage(spec, &cited);
    let missing: Vec<&str> = cov
        .iter()
        .filter(|c| c.2.is_empty())
        .map(|c| c.0.as_str())
        .collect();
    assert_eq!(missing, ["§1.2"]);
    assert_eq!(cov.len(), 2, "appendix headings are not rules");
}

const START: &str = "<!-- coverage:start -->";
const END: &str = "<!-- coverage:end -->";

/// The coverage report as the conformance README holds it.
fn report() -> String {
    let cov = coverage(&spec(), &citations());
    let total = examples().len();
    let mut out = format!(
        "{START}\n\n{} rules, {total} examples. Generated by the conformance test; `SPRIG_BLESS=1` rewrites it.\n\n| Rule | | Examples |\n| --- | --- | --- |\n",
        cov.len()
    );
    for (rule, title, by) in cov {
        out.push_str(&format!(
            "| {rule} | {title} | {}: {} |\n",
            by.len(),
            by.join(", ")
        ));
    }
    out.push_str(&format!("\n{END}"));
    out
}

#[test]
fn the_readme_holds_the_current_coverage_report() {
    let path = suite().join("README.md");
    let readme = fs::read_to_string(&path).unwrap().replace("\r\n", "\n");
    let (s, e) = (readme.find(START), readme.find(END));
    let want = report();
    if std::env::var_os("SPRIG_BLESS").is_some() {
        let new = match (s, e) {
            (Some(s), Some(e)) => format!("{}{want}{}", &readme[..s], &readme[e + END.len()..]),
            _ => format!("{}\n\n## Coverage\n\n{want}\n", readme.trim_end()),
        };
        fs::write(&path, new).unwrap();
        return;
    }
    let (Some(s), Some(e)) = (s, e) else {
        panic!("the conformance README has no coverage report; run with SPRIG_BLESS=1");
    };
    assert_eq!(
        &readme[s..e + END.len()],
        want,
        "the coverage report is out of date; run with SPRIG_BLESS=1"
    );
}
