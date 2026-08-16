# v72.6.0 タスクリスト — `fav init` テンプレートギャラリー拡充

Date: 2026-08-12
Status: 完了

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `72.5.0` であることを確認
- [x] `cargo test` が 3630 tests pass（0 failures）であることを確認
- [x] `driver.rs` に `v725000_tests` モジュールが存在することを確認
- [x] `driver.rs` に `v726000_tests` が未存在であることを確認
- [x] `driver.rs` 内に `try_cmd_new` が存在し、12 アームを持つことを確認
- [x] `driver.rs` 内の `"72.5.0"` 文字列（バージョンアサーション）の件数を grep で確認しておく

---

## T1: TEMPLATE_GALLERY 拡充（5 エントリ追加）

- [x] `TEMPLATE_GALLERY` に `ai-etl` エントリを追加した（name + description）
- [x] `TEMPLATE_GALLERY` に `streaming` エントリを追加した
- [x] `TEMPLATE_GALLERY` に `enterprise` エントリを追加した
- [x] `TEMPLATE_GALLERY` に `data-quality` エントリを追加した
- [x] `TEMPLATE_GALLERY` に `distributed` エントリを追加した
- [x] `cargo build` でエラーがないことを確認

---

## T2: `make_*` + `create_*` 関数 10 件追加

各テンプレートにつき `make_<name>_main_fav(name: &str) -> String`（fs 非依存）と `create_<name>_project(name: &str) -> Result<(), String>`（fs 書き込み）の 2 関数を追加する。

- [x] `make_ai_etl_main_fav` を追加した（`llm` または `LLM` を含む文字列を返す）
- [x] `create_ai_etl_project` を追加した（`make_ai_etl_main_fav` を内部で呼ぶ）
- [x] `make_streaming_main_fav` を追加した（`kafka` または `par` を含む文字列を返す）
- [x] `create_streaming_project` を追加した
- [x] `make_enterprise_main_fav` を追加した（`TenantRow` または `tenant` を含む文字列を返す）
- [x] `create_enterprise_project` を追加した
- [x] `make_data_quality_main_fav` を追加した（`validate` を含む文字列を返す）
- [x] `create_data_quality_project` を追加した
- [x] `make_distributed_main_fav` を追加した（`par` を含む文字列を返す）
- [x] `create_distributed_project` を追加した
- [x] `cargo build` でエラーがないことを確認

---

## T3: `try_cmd_new` に 5 アーム追加

- [x] `"ai-etl"` アームを追加した（`create_ai_etl_project(name)` を呼ぶ）
- [x] `"streaming"` アームを追加した
- [x] `"enterprise"` アームを追加した
- [x] `"data-quality"` アームを追加した
- [x] `"distributed"` アームを追加した
- [x] `cargo build` でエラーがないことを確認

---

## T4: `v726000_tests` モジュール追加

- [x] `v725000_tests` モジュールの直後に `v726000_tests` モジュールを追加した
- [x] `use super::{make_ai_etl_main_fav, make_data_quality_main_fav}` を追加した
- [x] `init_template_ai_etl_valid` テストを実装した
  - `make_ai_etl_main_fav("my-project")` の返値に `llm` または `LLM` が含まれることを assert
  - `make_ai_etl_main_fav("my-project")` の返値に `AppCtx` が含まれることを assert
- [x] `init_template_data_quality_valid` テストを実装した
  - `make_data_quality_main_fav("my-project")` の返値に `validate` が含まれることを assert
  - `make_data_quality_main_fav("my-project")` の返値に `AppCtx` が含まれることを assert
- [x] `cargo test v726000` で 2 件 pass することを確認

---

## T5: `fav/Cargo.toml` バージョン更新 + `driver.rs` version アサーション更新

- [x] `fav/Cargo.toml` の `version = "72.5.0"` → `version = "72.6.0"` に変更した
- [x] `driver.rs` 内の `version = \"72.5.0\"` 文字列を `version = \"72.6.0\"` に replace_all した
- [x] `driver.rs` 内のエラーメッセージ `"Cargo.toml version should be 72.5.0"` を `"72.6.0"` に replace_all した
- [x] `driver.rs` 内のエラーメッセージ `"Cargo.toml should declare version 72.5.0"` を `"72.6.0"` に replace_all した
- [x] 残存 72.5.0 はコメント・セクションヘッダーのみで意図的保持を確認

---

## T6: 部分テスト確認

- [x] `cargo test v726000` で 2 件 pass することを確認

---

## T7: 全体テスト確認

- [x] `cargo test` 全体で 3632 tests pass（0 failures）であることを確認

---

## T8: `CHANGELOG.md` 更新

- [x] `## [v72.6.0]` エントリを先頭に追加した

---

## T9: `versions/current.md` 更新

- [x] 「進行中バージョン」を `v72.6.0`（`fav init` テンプレートギャラリー拡充）に更新した
- [x] 「次に切る版」を `v72.7.0` に更新した

---

## T10: 最終確認（T8・T9 完了後のドキュメント更新後リグレッション確認）

- [x] `cargo test v726000` で 2 件 pass することを確認
- [x] `cargo test` 全体で 3632 tests pass（0 failures）であることを確認
- [x] `fav/Cargo.toml` のバージョンが `72.6.0` であることを確認
- [x] `try_cmd_new` に 5 新アームが存在することを確認
- [x] `TEMPLATE_GALLERY` に 5 エントリが追加されていることを確認（計 17）
- [x] `versions/current.md` の「進行中バージョン」が `v72.6.0`、「次に切る版」が `v72.7.0` であることを確認

---

## スコープ外（明示的除外）

- 実際のファイルシステムへの書き込み検証（テストはコード文字列レベルで担保）
- サイト側 UI 更新（v73.x 以降）
- rustyline / `~/.fav_history` 統合（v72.7.0 以降に実施）
- WASM 対応（v73.x 以降）

---

## コードレビュー指摘対応

| 優先度 | 指摘 | 対応 |
|---|---|---|
| [BUG] | `make_streaming_main_fav` に dead import（kafka を import するが未使用） | `Kafka.consume("my-topic")` 呼び出しを追加して import を実際に使用 |
| [BUG] | `make_enterprise_main_fav` の `bind rows` が未使用（W006 相当） | `bind _ <- ctx.io.read_file_raw(...)` に変更 |
| [MED] | streaming / enterprise / distributed テンプレートのテストが存在しない | `init_template_streaming_valid` / `init_template_enterprise_valid` / `init_template_distributed_valid` テストを追加 |

---

## 完了チェックリスト

- [x] 全タスク（T0〜T10）が完了している
- [x] `init_template_ai_etl_valid` が pass
- [x] `init_template_data_quality_valid` が pass
- [x] `init_template_streaming_valid` が pass（コードレビュー対応で追加）
- [x] `init_template_enterprise_valid` が pass（コードレビュー対応で追加）
- [x] `init_template_distributed_valid` が pass（コードレビュー対応で追加）
- [x] テスト総数: 3635（+5）
