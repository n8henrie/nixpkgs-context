use std::process;

const EXE: &str = env!("CARGO_BIN_EXE_nixpkgs-context");

#[test]
fn garage() {
    let result = process::Command::new(EXE)
        .args(["pkg-config", "tests/files/garage"])
        .output()
        .unwrap();
    let stdout = str::from_utf8(&result.stdout).unwrap();
    let stderr = str::from_utf8(&result.stderr).unwrap();
    assert!(result.status.success());
    assert_eq!(stdout.trim(), "nativeBuildInputs (1)");
    assert_eq!(
        stderr.trim(),
        "parse error: tests/files/garage/default2.nix"
    );
}
