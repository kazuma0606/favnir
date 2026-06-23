# v24.8.0 — `fav new` テンプレートギャラリー タスク

## ステータス: COMPLETE

---

## タスク一覧

### T0: 事前確認

- [x] `grep -n "version = " fav/Cargo.toml` — `"24.7.0"` であること
- [x] `cargo test --bin fav 2>&1 | grep "test result: ok"` — 1962 件であること
- [x] `grep -n "mod v248000_tests" fav/src/driver.rs | head -3` — 未存在
- [x] `grep -n "etl-csv-to-db\|TEMPLATE_GALLERY" fav/src/driver.rs | head -5` — 全 0 件

---

### T1: `TEMPLATE_GALLERY` 定数 + `create_*_project` 関数追加（driver.rs）

- [x] **T1-1**: `pub const TEMPLATE_GALLERY: &[(&str, &str)]` を追加（4 エントリ）
- [x] **T1-2**: `create_etl_csv_to_db_project(root, name)` 実装
- [x] **T1-3**: `create_api_gateway_project(root, name)` 実装
- [x] **T1-4**: `create_lambda_scheduled_project(root, name)` 実装
- [x] **T1-5**: `create_distributed_etl_project(root, name)` 実装
- [x] **事後確認**: `cargo check --bin fav` — エラー 0

---

### T2: `try_cmd_new` match アームに 4 テンプレート追加（driver.rs）

- [x] `"etl-csv-to-db"` / `"api-gateway"` / `"lambda-scheduled"` / `"distributed-etl"` アームを追加
- [x] `unknown template` エラーメッセージに 4 テンプレート名を追記
- [x] **事後確認**: `cargo check --bin fav` — エラー 0

---

### T3: `fav/src/driver.rs` — v248000_tests 追加

- [x] **T3-1**: `v248000_tests` モジュールを `v247000_tests` の直後に追加（7 件）
  - `template_gallery_has_4_entries`
  - `fav_new_etl_csv_to_db_ok`
  - `fav_new_api_gateway_ok`
  - `fav_new_lambda_scheduled_ok`
  - `fav_new_distributed_etl_ok`
  - `fav_new_unknown_template_errors`
  - `changelog_has_v24_8_0`
- [x] `cargo test v248000 --bin fav` — 7/7 PASS を確認
- [x] `cargo test --bin fav` — リグレッションなし（1969 件合格）

---

### T4: サイトドキュメント

- [x] `site/content/docs/tools/templates.mdx` を新規作成

---

### T5: Cargo.toml + CHANGELOG + benchmarks

- [x] `fav/Cargo.toml` の `version = "24.7.0"` → `"24.8.0"` に変更
- [x] `CHANGELOG.md` 先頭に v24.8.0 エントリを追加
- [x] `benchmarks/v24.8.0.json` を新規作成（test_count: 1969、duration_ms: 17400）
- [x] `cargo test v248000 --bin fav` — 最終確認 7/7 PASS
- [x] `cargo test --bin fav` — リグレッションなし（1969 件合格）

---

## テスト一覧（v248000_tests、7 件）

| テスト名 | 内容 | 期待値 |
|---|---|---|
| `template_gallery_has_4_entries` | `TEMPLATE_GALLERY.len() == 4` かつ全 4 名を含む | assert_eq! |
| `fav_new_etl_csv_to_db_ok` | tempdir で `try_cmd_new` → Ok / `pipeline.fav` 存在 | assert!(ok) |
| `fav_new_api_gateway_ok` | tempdir で `try_cmd_new` → Ok / `api.fav` 存在 | assert!(ok) |
| `fav_new_lambda_scheduled_ok` | tempdir で `try_cmd_new` → Ok / `job.fav` 存在 | assert!(ok) |
| `fav_new_distributed_etl_ok` | tempdir で `try_cmd_new` → Ok / `pipeline.fav` 存在 | assert!(ok) |
| `fav_new_unknown_template_errors` | `try_cmd_new("x", "no-such")` → Err（`"etl-csv-to-db"` 含む） | assert!(err) |
| `changelog_has_v24_8_0` | `CHANGELOG.md` に `[v24.8.0]` | assert |

---

## 完了条件チェックリスト

- [x] `TEMPLATE_GALLERY` に 4 エントリ定義済み（driver.rs）
- [x] `create_etl_csv_to_db_project` 実装済み
- [x] `create_api_gateway_project` 実装済み
- [x] `create_lambda_scheduled_project` 実装済み
- [x] `create_distributed_etl_project` 実装済み
- [x] `try_cmd_new` に 4 アーム追加済み
- [x] エラーメッセージに 4 テンプレート名追記済み
- [x] `cargo test v248000 --bin fav` — 7/7 PASS
- [x] `cargo test --bin fav` — リグレッションなし（1969 件合格）
- [x] `site/content/docs/tools/templates.mdx` 作成済み
- [x] `CHANGELOG.md` に v24.8.0 エントリ
- [x] `benchmarks/v24.8.0.json` 作成済み（test_count: 1969）

---

## コードレビュー対応（実施済み）

spec-reviewer 指摘（9 件）:
- [HIGH] #1 テスト件数実測手順 → plan.md Step 0 に `cargo test` 実測コマンド追加
- [HIGH] #2 tempfile 二重登録 → plan.md 注意書きを正確な説明に修正
- [HIGH] #3 try_cmd_new 可視性説明 → 「同一ファイル内 mod から super::* でアクセス可能」に修正
- [HIGH] #4 ディレクトリ自動作成 → write_text_file の create_dir_all 動作を spec/plan に明記
- [MED] #5 format!/raw string 注意書き → 1 案に統一（`format!(r#"..."#)` で {name} 展開可能）
- [MED] #6 テストカバレッジ不足 → lambda-scheduled / distributed-etl のテストを追加（7 件）
- [MED] #7 削除理由なし → 削除廃止（v247000_tests に削除対象なし、純粋 +7）
- [LOW] #8 fav new --template list → v25.x スコープと spec.md に明記
- [LOW] #9 完了条件粒度 → spec.md の完了条件を tasks.md と 1 対 1 に統一
