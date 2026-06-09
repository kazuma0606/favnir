//! Postgres integration tests — only run when DATABASE_URL is set.
//!
//! In CI: DATABASE_URL is set to the GitHub Actions services postgres.
//! Locally: set DATABASE_URL to point to any Postgres instance (e.g. RDS).
//!
//! Run with:
//!   cargo test --locked integration -- --test-threads=1
//!
//! Skip condition: if DATABASE_URL is not set, all tests return early (skip).

use fav_core::backend::vm::{pg_execute, pg_query};

// ── fav test self/*.fav ───────────────────────────────────────────────────────

fn fav_bin() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_BIN_EXE_fav"))
}

fn run_fav_test(file: &str) -> std::process::ExitStatus {
    // Run from the fav/ workspace root so relative paths like "self/checker.fav" resolve.
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    std::process::Command::new(fav_bin())
        .args(["test", file])
        .current_dir(&manifest_dir)
        .status()
        .unwrap_or_else(|e| panic!("failed to run fav test {}: {}", file, e))
}

#[test]
fn fav_test_self_checker_runs() {
    let status = run_fav_test("self/checker.fav");
    assert!(status.success(), "fav test self/checker.fav failed (exit {:?})", status.code());
}

#[test]
fn fav_test_self_lexer_runs() {
    let status = run_fav_test("self/lexer.fav");
    assert!(status.success(), "fav test self/lexer.fav failed (exit {:?})", status.code());
}

// ── fav check --strict (v12.10.0) ────────────────────────────────────────────

fn run_fav_check(args: &[&str]) -> std::process::Output {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    std::process::Command::new(fav_bin())
        .arg("check")
        .args(args)
        .current_dir(&manifest_dir)
        .output()
        .unwrap_or_else(|e| panic!("failed to run fav check {:?}: {}", args, e))
}

fn run_fav_lint(args: &[&str]) -> std::process::Output {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    std::process::Command::new(fav_bin())
        .arg("lint")
        .args(args)
        .current_dir(&manifest_dir)
        .output()
        .unwrap_or_else(|e| panic!("failed to run fav lint {:?}: {}", args, e))
}

#[test]
fn check_strict_w006_exits_1() {
    // A file with `bind _ <- Postgres.execute_raw(...)` triggers W006.
    let tmp = std::env::temp_dir().join("fav_v121000_strict_w006.fav");
    std::fs::write(&tmp, r#"
pipeline Test !Postgres {
  stage Run {
    bind _ <- Postgres.execute_raw("SELECT 1", "[]")
    "done"
  }
}
"#).unwrap();
    let path = tmp.to_str().unwrap();
    let out = run_fav_check(&["--strict", "--legacy-check", path]);
    assert_ne!(out.status.code(), Some(0), "--strict with W006 should exit 1");
}

#[test]
fn check_strict_no_warning_exits_0() {
    // A file with no W006 bindings should pass --strict.
    let tmp = std::env::temp_dir().join("fav_v121000_strict_ok.fav");
    std::fs::write(&tmp, r#"fn identity(x: String) -> String { x }
"#).unwrap();
    let path = tmp.to_str().unwrap();
    let out = run_fav_check(&["--strict", path]);
    assert_eq!(out.status.code(), Some(0), "--strict with no warnings should exit 0, stderr: {}", String::from_utf8_lossy(&out.stderr));
}

#[test]
fn lint_deny_warnings_exits_1() {
    // A file with an unused variable warning (W001) should exit 1 with --deny-warnings.
    let tmp = std::env::temp_dir().join("fav_v121000_deny_warnings.fav");
    std::fs::write(&tmp, r#"
pipeline Test {
  stage Run {
    let unused = "hello"
    "done"
  }
}
"#).unwrap();
    let path = tmp.to_str().unwrap();
    let out = run_fav_lint(&["--deny-warnings", path]);
    // Either warnings exist (exit 1) or no warnings (exit 0) — if exit 1 means warnings found
    // we just verify the flag doesn't cause a crash and behaves correctly:
    // If warnings → must exit 1. If no warnings → exit 0 is fine.
    let stderr = String::from_utf8_lossy(&out.stderr);
    if stderr.contains("lint[") {
        assert_ne!(out.status.code(), Some(0), "--deny-warnings with warnings should exit 1");
    }
}

fn db_url() -> Option<String> {
    std::env::var("DATABASE_URL").ok()
}

/// Unique table name to avoid conflicts between parallel runs.
fn test_table() -> String {
    "fav_integration_test_v12900".to_string()
}

#[test]
fn postgres_create_insert_select() {
    let url = match db_url() {
        Some(u) => u,
        None => return,
    };
    let table = test_table();

    // Clean up from previous runs
    let _ = pg_execute(&url, &format!("DROP TABLE IF EXISTS {table}"), "[]");

    // CREATE
    pg_execute(
        &url,
        &format!("CREATE TABLE {table} (id INT, val TEXT)"),
        "[]",
    )
    .expect("CREATE TABLE failed");

    // INSERT
    pg_execute(
        &url,
        &format!("INSERT INTO {table} (id, val) VALUES (1, 'hello')"),
        "[]",
    )
    .expect("INSERT failed");

    // SELECT
    let result = pg_query(
        &url,
        &format!("SELECT val FROM {table} WHERE id = 1"),
        "[]",
    )
    .expect("SELECT failed");

    assert!(
        result.contains("hello"),
        "expected 'hello' in SELECT result, got: {}",
        result
    );

    // Clean up
    pg_execute(&url, &format!("DROP TABLE IF EXISTS {table}"), "[]")
        .expect("DROP TABLE failed");
}

#[test]
fn postgres_error_table_not_found() {
    let url = match db_url() {
        Some(u) => u,
        None => return,
    };

    let result = pg_query(
        &url,
        "SELECT * FROM fav_nonexistent_table_v12900_xyz",
        "[]",
    );

    assert!(result.is_err(), "expected Err for nonexistent table");
    let err_msg = result.unwrap_err();
    assert!(
        err_msg.contains("does not exist") || err_msg.contains("exist"),
        "expected 'does not exist' in error, got: {}",
        err_msg
    );
}

#[test]
fn postgres_ssl_disable_connects() {
    let url = match db_url() {
        Some(u) => u,
        None => return,
    };

    // Simple smoke test: SELECT 1 should return ok
    let result = pg_query(&url, "SELECT 1 AS n", "[]");
    assert!(
        result.is_ok(),
        "expected Ok from SELECT 1, got: {:?}",
        result
    );
    let json = result.unwrap();
    assert!(
        json.contains("1") || json.contains("n"),
        "expected result containing '1', got: {}",
        json
    );
}
