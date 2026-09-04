# Tasks: v99.0.0 — SAP Analytics 1.0 宣言

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `versions/v95-v100/v98.9.0/tasks.md` の Status が `COMPLETE` であることを確認する
- [x] `versions/current.md` の最新安定版が `v98.9.0` であることを確認する
- [x] `fav/src/driver.rs` に `mod v98900_tests` が存在することを確認する（v98.9.0 完了済みの証拠）
- [x] `cargo test -- --test-threads=1 2>&1 | grep "test result"` を実行し、現在のテスト数が 4,253 であることを確認する（着手前ベースライン）
- [x] `fav/Cargo.toml` の version が `98.0.0` であることを確認する（これから 99.0.0 に更新する）
- [x] `fav/tmp/hello.fav` が存在することを確認する（存在しない場合は以下の内容で作成: `fn add(a: Int, b: Int) -> Int { a + b }` / `fn main() -> Bool { add(1, 2) == 3 }`）

## T1: Cargo.toml バージョン更新

- [x] `fav/Cargo.toml` の `version = "98.0.0"` を `version = "99.0.0"` に変更する

## T2: MILESTONE.md に v99.0.0 エントリを追加

- [x] `## v98.0.0` の前に `## v99.0.0（2026-09-03）— SAP Analytics 1.0 宣言` エントリを追加する
- [x] 宣言文が含まれることを確認する
- [x] `SAP Analytics` キーワードが含まれることを確認する
- [x] SAP Analytics 1.0（v98.1〜v98.9）達成内容リストが含まれることを確認する

## T3: README.md に v99.0 セクションを追加

- [x] `## v98.0` セクションの前に `## v99.0 — SAP Analytics 1.0 宣言（2026-09-03）` を追加する
- [x] Favnir コード例（`kpi_monitor` pipeline）が含まれることを確認する
- [x] コード例が `bind` 構文・`--` コメントを使っていることを確認する（`let` / `//` 不可）
- [x] `SAP Analytics` キーワードが含まれることを確認する

## T4: CHANGELOG.md に v99.0.0 エントリを追加（T5 の前に必ず実施すること）

- [x] `CHANGELOG.md` の先頭に `[v99.0.0]` エントリを追加する
- [x] `[v99.0.0]` が含まれることを確認する（`changelog_has_v99_0_0` テストが通る前提）

## T5: driver.rs に mod v99000_tests を追加

- [x] `mod v98900_tests` の直後に `mod v99000_tests`（4 テスト）を追加する:
  - `cargo_toml_version_is_99_0_0`: `include_str!("../Cargo.toml")` で `version = "99.0.0"` を確認
  - `changelog_has_v99_0_0`: `include_str!("../../CHANGELOG.md")` で `[v99.0.0]` を確認
  - `milestone_has_sap_analytics`: `include_str!("../../MILESTONE.md")` で `SAP Analytics` を確認
  - `readme_mentions_sap_analytics`: `include_str!("../../README.md")` で `SAP Analytics` を確認
- [x] `mod v99000_tests` ブロック先頭に `// use super::* は不要（外部シンボル未使用）` という Rust コメントを 1 行追記する
- [x] Cargo.toml バージョン更新に伴い、driver.rs 全体の旧 `cargo_toml_version_is_XX` テストの assert 内バージョン文字列を `"99.0.0"` に一括更新する（`replace_all: true`）
  - `version = \"98.0.0\"` → `version = \"99.0.0\"` を一括置換（45件）
  - v97000_tests の `contains("98.0.0")` → `contains("99.0.0")` も個別に更新（1件）

## T6: cargo test で全 pass 確認（cargo clean 前）

- [x] `cargo test -- --test-threads=1 2>&1 | grep "test result"` を実行し、4,257 tests, 0 failures であることを確認する

## T7: cargo clean（★クリーンアップ）

- [x] `cargo clean` を実行する（11.5GiB 削除）
- [x] `fav/tmp/hello.fav` が存在することを確認する（cargo clean 後も残存を確認）

## T8: cargo test で全 pass 確認（cargo clean 後）

- [x] `cargo test -- --test-threads=1 2>&1 | grep "test result"` を実行し、4,257 tests, 0 failures であることを確認する（cargo clean 後の再確認）

## T9: versions/current.md 更新

- [x] `最終更新:` ヘッダーを `v99.0.0` に更新する
- [x] 最新安定版を `v99.0.0` に更新する（テスト数 4,257）

## T-last: CI 事前確認（T8 完了後・T9 完了後に実施すること。`cargo test` 再実行は不要。Clippy / fmt のみ確認する）

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
