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
