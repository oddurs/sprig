fn main() {
    let src = std::path::Path::new("src");
    let mut c = cc::Build::new();
    c.std("c11").include(src);
    #[cfg(target_env = "msvc")]
    c.flag("-utf-8");
    c.file(src.join("parser.c"));
    c.warnings(false);
    c.compile("tree-sitter-sprig");
    println!("cargo:rerun-if-changed=src/parser.c");
}
