# Tasks: v49.7.0 — セキュリティ審査 2.0

Status: COMPLETE
Date: 2026-07-18

---

## T0 — 事前確認

- [x] `cargo test` 3081 passed, 0 failed を確認（ベース確認）
- [x] `driver.rs` に `validate_import_path` / `validate_rune_name` が存在しないことを確認
- [x] `v496000_tests` モジュールが `driver.rs` に存在することを確認（挿入位置の前提）

## T1 — ヘルパー関数追加

- [x] `pub fn validate_import_path(path: &str) -> Result<(), String>` を `driver.rs` に `pub fn` として追加
  - [x] 空パスを拒否
  - [x] `\` を拒否
  - [x] `..` コンポーネントを拒否（split('/') によるコンポーネント単位検査）
- [x] `pub fn validate_rune_name(name: &str) -> Result<(), String>` を `driver.rs` に `pub fn` として追加
  - [x] 空名前を拒否
  - [x] `a-z`, `A-Z`, `0-9`, `-` 以外を拒否（`chars().all(...)` で網羅）
  - [x] 先頭・末尾 `-` を拒否
  - [x] `--` を拒否

## T2 — `v497000_tests` 追加

- [x] `v497000_tests` モジュールを `v496000_tests` の直前に追加（2 テスト）
- [x] 挿入後 `grep -n v497000_tests src/driver.rs` で存在確認
  - [x] `import_path_traversal_rejected`:
    - [x] `validate_import_path("../../etc/passwd")` が `Err` を返す
    - [x] `validate_import_path("./stages/validate")` が `Ok` を返す
  - [x] `install_invalid_name_rejected`:
    - [x] `validate_rune_name("my-rune")` が `Ok` を返す
    - [x] `validate_rune_name("../evil")` が `Err` を返す
    - [x] `validate_rune_name("my rune")` が `Err` を返す

## T3 — バージョン更新・完了

- [x] `fav/Cargo.toml` version → `"49.7.0"`
- [x] `cargo test` 3083 passed, 0 failed
- [x] `cargo clippy -- -D warnings` クリーン
- [x] `CHANGELOG.md` に v49.7.0 エントリ追加（セキュリティ審査 2.0 を明記）
- [x] `versions/current.md` を v49.7.0（3083 tests）に更新、進行中バージョンを `v49.8.0` に更新
- [x] `versions/roadmap/roadmap-v49.1-v50.0.md` の v49.7.0 実績を 3083 に記入
- [x] tasks.md を COMPLETE に更新（T0〜T3 全 `[x]`）
