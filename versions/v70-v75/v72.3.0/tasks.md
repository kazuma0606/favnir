# v72.3.0 タスクリスト — `fav ai generate`（自然言語 → Favnir パイプライン）

Date: 2026-08-12
Status: 完了

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `72.2.0` であることを確認
- [x] `cargo test` が 3616 tests pass（0 failures）であることを確認
- [x] `driver.rs` に `v722000_tests` モジュールが存在することを確認
- [x] `driver.rs` に `v723000_tests` が未存在であることを確認
- [x] `driver.rs` に `cmd_ai_generate` / `infer_schema_from_description` が未存在であることを確認
- [x] `driver.rs` 内の `"72.2.0"` 文字列（バージョンアサーション）の件数を grep で確認しておく（T4 の replace_all 前基準として記録）— 28 件

---

## T1: `driver.rs` — コア関数追加

- [x] `infer_schema_from_description(description: &str) -> Vec<(&'static str, &'static str)>` を `pub fn` で追加した
  - "order" / "注文" を含む → `[("order_id", "String"), ("amount", "Float"), ("status", "String")]`
  - それ以外 → `[("id", "String"), ("value", "String")]`
- [x] `cmd_ai_generate(description: &str) -> String` を `pub fn` で追加した
  - キーワードベースで import rune ブロックを生成（csv / postgres / s3 / json）
  - `infer_schema_from_description` を呼んで schema ブロックを生成
  - `fn main(ctx: AppCtx) -> Result<Unit, String>` の雛形を生成
  - 3 ブロックを結合した文字列を返す
- [x] `cargo build` でエラーがないことを確認

---

## T2: `main.rs` — CLI サブコマンド追加

- [x] 既存の `Some("ai")` アームの `"generate"` ブランチを追加した
- [x] `args.iter().skip(3).cloned().collect::<Vec<_>>().join(" ")` で description を構築した
- [x] `driver::cmd_ai_generate(desc)` を呼び出し結果を `println!` した
- [x] `cargo build` でエラーがないことを確認

---

## T3: `v723000_tests` 追加（`driver.rs`）

> **前提**: T1 完了済みであること（`cmd_ai_generate` / `infer_schema_from_description` が `pub fn` で存在すること）。

- [x] `v722000_tests` モジュールの直後に `v723000_tests` モジュールを追加した
- [x] `use super::{cmd_ai_generate, infer_schema_from_description}` を追加した
- [x] `ai_generate_returns_valid_fav_code` テストを実装した
  - `cmd_ai_generate("csv pipeline")` の結果が `"fn main"` を含むことを assert
  - `cmd_ai_generate("csv pipeline")` の結果が `"ctx: AppCtx"` を含むことを assert
- [x] `ai_generate_schema_inferred_from_description` テストを実装した
  - `cmd_ai_generate("注文データのETL")` の結果が `"order_id"` を含むことを assert
- [x] `cargo build` でエラーがないことを確認

---

## T4: `fav/Cargo.toml` バージョン更新 + `driver.rs` version アサーション更新

- [x] `fav/Cargo.toml` の `version = "72.2.0"` → `version = "72.3.0"` に変更した
- [x] `driver.rs` 内の `version = \"72.2.0\"` 文字列を `version = \"72.3.0\"` に replace_all した
- [x] `driver.rs` 内のエラーメッセージ `"Cargo.toml version should be 72.2.0"` を `"72.3.0"` に replace_all した
- [x] `driver.rs` 内のエラーメッセージ `"Cargo.toml should declare version 72.2.0"` を `"72.3.0"` に replace_all した
- [x] T0 で記録した 28 件と置換後の `"72.3.0"` grep 件数（28 件）が一致することを確認した

---

## T5: 部分テスト確認

- [x] `cargo test ai_generate` で 2 件 pass することを確認

---

## T6: 全体テスト確認

- [x] `cargo test` 全体で 3618 tests pass（0 failures）であることを確認

---

## T7: `CHANGELOG.md` 更新

- [x] `## [v72.3.0]` エントリを先頭に追加した

---

## T8: `versions/current.md` 更新

- [x] 「進行中バージョン」を `v72.3.0`（`fav ai generate`）に更新した
- [x] 「次に切る版」を `v72.4.0` に更新した

---

## T9: 最終確認（T7・T8 完了後のドキュメント更新後リグレッション確認）

- [x] `cargo test ai_generate` で 2 件 pass することを確認
- [x] `cargo test` 全体で 3618 tests pass（0 failures）であることを確認
- [x] `fav/Cargo.toml` のバージョンが `72.3.0` であることを確認
- [x] `cmd_ai_generate("csv pipeline")` が `"fn main"` と `"ctx: AppCtx"` を含む文字列を返すことを確認（テストで担保）
- [x] `cmd_ai_generate("注文データのETL")` が `"order_id"` を含むことを確認（テストで担保）
- [x] `versions/current.md` が正しく更新されていることを確認

---

## スコープ外（明示的除外）

- 実際の Claude / OpenAI API HTTP 呼び出し（v73.x 以降）
- 生成コードのファイルへの書き込み・エディタ起動（v72.4.0 以降）
- `fav check` による生成コードの自動検証（v72.4.0 以降 — サブプロセス実行が必要なため）
- CSV / Postgres / S3 / JSON 以外のすべての Rune 推論（段階的追加）
- スキーマフィールド推論パターンの拡張（段階的追加）
- `site/content/docs/cli/ai.mdx` への `fav ai generate` 追記（v72.4.0 以降）

---

## コードレビュー指摘対応

| 優先度 | 指摘 | 対応 |
|---|---|---|
| [BUG] | `infer_schema_from_description` / `cmd_ai_generate` で `description.contains("注文")` と `lower.contains("注文")` が混在 | 両箇所を `lower.contains("注文")` に統一 |
| [BUG] | ユーザー入力が Favnir 文字列リテラルに無エスケープで埋め込まれる | `replace('\\', "\\\\").replace('"', "\\\"")` で事前エスケープ + テスト追加 |
| [MED] | テストがデフォルトスキーマ（`Row`）ケースをカバーしていない | `ai_generate_default_schema_for_unknown_description` テスト追加 |
| [MED] | `"` 含む入力のテストがない | `ai_generate_escapes_quotes_in_description` テスト追加 |
| [LOW] | `infer_schema_from_description` が `pub fn`（crate 外公開不要） | `pub(crate)` に変更 |
| [LOW] | `cargo_toml_version_is_72_0_0` 関数名がアサーション内容（72.3.0）と不一致 | `cargo_toml_version_is_current` にリネーム |

---

## 完了チェックリスト

- [x] 全タスク（T0〜T9）が完了している
- [x] `ai_generate_returns_valid_fav_code` が pass
- [x] `ai_generate_schema_inferred_from_description` が pass
- [x] `ai_generate_default_schema_for_unknown_description` が pass（コードレビュー対応で追加）
- [x] `ai_generate_escapes_quotes_in_description` が pass（コードレビュー対応で追加）
- [x] テスト総数: 3620（+4）
