use std::process::Command;

fn sprig(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_sprig"))
        .args(args)
        .output()
        .expect("the sprig binary runs")
}

#[test]
fn version_names_the_binary_and_its_version() {
    let out = sprig(&["--version"]);
    assert!(out.status.success());
    assert_eq!(
        String::from_utf8_lossy(&out.stdout).trim(),
        format!("sprig {}", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn an_unknown_argument_is_a_usage_error() {
    let out = sprig(&["--frobnicate"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("--frobnicate"));
}
