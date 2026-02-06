use std::fs::{self, File};
use std::process::Command;
use tempfile::TempDir;

fn vfv_binary() -> Command {
    Command::new(env!("CARGO_BIN_EXE_vfv"))
}

fn setup_test_dir() -> TempDir {
    let temp_dir = TempDir::new().unwrap();
    let base = temp_dir.path();

    fs::create_dir_all(base.join("src")).unwrap();
    fs::create_dir_all(base.join("tests")).unwrap();

    File::create(base.join("src/main.rs")).unwrap();
    File::create(base.join("src/lib.rs")).unwrap();
    File::create(base.join("README.md")).unwrap();

    temp_dir
}

#[test]
fn test_find_basic() {
    let temp_dir = setup_test_dir();

    let output = vfv_binary()
        .args(["find", "main", temp_dir.path().to_str().unwrap()])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("main.rs"));
}

#[test]
fn test_find_json_output() {
    let temp_dir = setup_test_dir();

    let output = vfv_binary()
        .args(["find", "main", temp_dir.path().to_str().unwrap(), "--json"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should be valid JSON
    let parsed: Result<Vec<serde_json::Value>, _> = serde_json::from_str(&stdout);
    assert!(parsed.is_ok(), "Output should be valid JSON: {}", stdout);
}

#[test]
fn test_find_compact_json() {
    let temp_dir = setup_test_dir();

    let output = vfv_binary()
        .args([
            "find",
            "main",
            temp_dir.path().to_str().unwrap(),
            "--json",
            "--compact",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Compact JSON should be on a single line
    assert_eq!(
        stdout.lines().count(),
        1,
        "Compact JSON should be single line"
    );
}

#[test]
fn test_find_no_results_exits_with_code_1() {
    let temp_dir = setup_test_dir();

    let output = vfv_binary()
        .args([
            "find",
            "nonexistent_file_xyz",
            temp_dir.path().to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn test_find_dir_only() {
    let temp_dir = setup_test_dir();

    let output = vfv_binary()
        .args([
            "find",
            "src",
            temp_dir.path().to_str().unwrap(),
            "--dir",
            "--json",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let results: Vec<serde_json::Value> = serde_json::from_str(&stdout).unwrap();

    // All results should be directories
    for result in results {
        assert!(result["is_dir"].as_bool().unwrap_or(false));
    }
}

#[test]
fn test_find_limit() {
    let temp_dir = setup_test_dir();

    let output = vfv_binary()
        .args([
            "find",
            "r",
            temp_dir.path().to_str().unwrap(),
            "--json",
            "--limit",
            "1",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let results: Vec<serde_json::Value> = serde_json::from_str(&stdout).unwrap();

    assert!(results.len() <= 1);
}

#[test]
fn test_find_first_flag() {
    let temp_dir = setup_test_dir();

    let output = vfv_binary()
        .args([
            "find",
            "r",
            temp_dir.path().to_str().unwrap(),
            "--json",
            "-1",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let results: Vec<serde_json::Value> = serde_json::from_str(&stdout).unwrap();

    assert_eq!(results.len(), 1);
}

#[test]
fn test_find_exact_match() {
    let temp_dir = setup_test_dir();

    let output = vfv_binary()
        .args([
            "find",
            "main.rs",
            temp_dir.path().to_str().unwrap(),
            "--exact",
            "--json",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let results: Vec<serde_json::Value> = serde_json::from_str(&stdout).unwrap();

    // All results should have name "main.rs"
    for result in results {
        assert_eq!(result["name"].as_str().unwrap(), "main.rs");
    }
}

#[test]
fn test_find_query_too_long() {
    let temp_dir = setup_test_dir();
    let long_query = "a".repeat(1001);

    let output = vfv_binary()
        .args(["find", &long_query, temp_dir.path().to_str().unwrap()])
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Query too long"));
}

#[test]
fn test_find_timeout() {
    let temp_dir = setup_test_dir();

    // Very short timeout
    let output = vfv_binary()
        .args([
            "find",
            "x",
            temp_dir.path().to_str().unwrap(),
            "--timeout",
            "0",
        ])
        .output()
        .expect("Failed to execute command");

    // With timeout=0, no timeout is applied, so it should complete normally
    // Either success or no results (exit 1)
    assert!(output.status.code() == Some(0) || output.status.code() == Some(1));
}

#[test]
fn test_version_flag() {
    let output = vfv_binary()
        .arg("--version")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("vfv"));
}

#[test]
fn test_help_flag() {
    let output = vfv_binary()
        .arg("--help")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("fuzzy search"));
}

#[test]
fn test_man_page() {
    let output = vfv_binary()
        .arg("man")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains(".TH vfv"));
    assert!(stdout.contains("SYNOPSIS"));
}

#[test]
fn test_init_creates_config() {
    // We can't easily test init with custom path, but we can verify
    // that --help shows the init command
    let output = vfv_binary()
        .args(["init", "--help"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("config") || stdout.contains("completions"));
    assert!(stdout.contains("--force"));
}

#[test]
fn test_subcommands_in_help() {
    let output = vfv_binary()
        .arg("--help")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    // All subcommands should be listed
    assert!(stdout.contains("find"));
    assert!(stdout.contains("init"));
    assert!(stdout.contains("man"));
}
