//! `spec/tree.schema.json` is the contract for every tree a tool hands to
//! another: it must accept what the core writes, and Appendix D of the spec
//! must explain every key it allows.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;
use sprig_core::{Date, Workspace, json};

fn repo(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path)
}

fn schema() -> Value {
    serde_json::from_str(&fs::read_to_string(repo("spec/tree.schema.json")).unwrap()).unwrap()
}

fn assert_valid(v: &Value, what: &str) {
    let schema = schema();
    let validator = jsonschema::validator_for(&schema).unwrap();
    let errors: Vec<String> = validator
        .iter_errors(v)
        .map(|e| format!("{e} at {}", e.instance_path))
        .collect();
    assert!(errors.is_empty(), "{what}:\n{}", errors.join("\n"));
}

fn bakery(today: &str) -> (sprig_core::Resolution, Date) {
    let mut w = Workspace::new();
    for e in fs::read_dir(repo("examples/bakery")).unwrap() {
        let p = e.unwrap().path();
        let name = p.file_name().unwrap().to_string_lossy().into_owned();
        w.insert(name, fs::read_to_string(&p).unwrap());
    }
    let today = Date::parse_iso(today).unwrap();
    (w.resolve(today), today)
}

#[test]
fn the_schema_accepts_the_hand_checked_tree_for_lease() {
    let text = fs::read_to_string(repo("tests/schema/lease.tree.json")).unwrap();
    let tree: Value = serde_json::from_str(&text).unwrap();
    assert_valid(&tree, "tests/schema/lease.tree.json");
    // And it is what the core says, so the two cannot drift apart.
    let (r, today) = bakery("2026-10-01");
    assert_eq!(json::tree_document(&r, "lease.sprig", today).unwrap(), tree);
}

#[test]
fn the_schema_accepts_every_tree_the_core_writes_for_the_bakery() {
    let (r, today) = bakery("2026-10-01");
    for t in r.trees() {
        let doc = json::tree_document(&r, &t.file, today).unwrap();
        assert_valid(&doc, &t.file);
    }
}

#[test]
fn the_schema_rejects_an_unknown_version_and_unknown_keys() {
    let schema = schema();
    let validator = jsonschema::validator_for(&schema).unwrap();
    assert!(!validator.is_valid(&serde_json::json!({"sprig_tree": 2})));
    assert!(!validator.is_valid(&serde_json::json!({"sprig_tree": 1, "sintax": {}})));
    assert!(validator.is_valid(&serde_json::json!({"sprig_tree": 1})));
}

#[test]
fn the_syntax_layer_depends_on_neither_other_files_nor_the_date() {
    let src = fs::read_to_string(repo("examples/bakery/lease.sprig")).unwrap();
    let mut alone = Workspace::new();
    alone.insert("lease.sprig", src);
    let early = Date::parse_iso("2020-01-01").unwrap();
    let alone = alone.resolve(early);
    let alone = json::tree_document(&alone, "lease.sprig", early).unwrap();
    let (r, today) = bakery("2026-11-20");
    let together = json::tree_document(&r, "lease.sprig", today).unwrap();
    assert_eq!(alone["syntax"], together["syntax"]);
}

#[test]
fn appendix_d_documents_every_key_the_schema_allows() {
    fn keys(v: &Value, out: &mut BTreeSet<String>) {
        match v {
            Value::Object(m) => {
                if let Some(Value::Object(props)) = m.get("properties") {
                    out.extend(props.keys().cloned());
                }
                m.values().for_each(|x| keys(x, out));
            }
            Value::Array(a) => a.iter().for_each(|x| keys(x, out)),
            _ => {}
        }
    }
    let mut all = BTreeSet::new();
    keys(&schema()["properties"], &mut all);
    keys(&schema()["$defs"], &mut all);
    all.insert("sprig_tree".into());
    let spec = fs::read_to_string(repo("spec/annex.md")).unwrap();
    let appendix = &spec[spec.find("## Appendix D").expect("the spec has Appendix D")..];
    let missing: Vec<&String> = all
        .iter()
        .filter(|k| {
            !appendix.contains(&format!("`{k}`")) && !appendix.contains(&format!("\"{k}\""))
        })
        .collect();
    assert!(
        missing.is_empty(),
        "Appendix D does not mention {missing:?}"
    );
}
