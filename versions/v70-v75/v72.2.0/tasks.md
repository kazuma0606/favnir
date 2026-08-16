# v72.2.0 タスクリスト — AI エラーアシスタント（`fav ai explain` / `fav ai fix`）

Date: 2026-08-12
Status: 完了

---

## T0: 事前確認

- [x]`fav/Cargo.toml` のバージョンが `72.1.0` であることを確認
- [x]`cargo test` が 3614 tests pass（0 failures）であることを確認
- [x]`driver.rs` に `v721000_tests` モジュールが存在することを確認
- [x]`driver.rs` に `v722000_tests` が未存在であることを確認
- [x]`driver.rs` に `get_ai_hint` / `apply_ctx_migration` が未存在であることを確認
- [x]`driver.rs` 内の `"72.1.0"` 文字列（バージョンアサーション）の件数を grep で確認しておく（T4 の replace_all 前基準として記録）

---

## T1: `driver.rs` — コア関数追加

- [x]`get_ai_hint(error_code: &str) -> Option<&'static str>` を `pub fn` で追加した
  - `"E0374"` → `ctx: AppCtx` / `!IO` 関連ヒントを返す
  - `"E0001"` → 未定義変数ヒントを返す
  - その他 → `None`
- [x]`apply_ctx_migration(src: &str) -> String` を `pub fn` で追加した
  - `IO.println(` → `ctx.io.println(` に置換する
  - `IO.write_file(` → `ctx.io.write_file_raw(` に置換する
  - `IO.read_file(` → `ctx.io.read_file_raw(` に置換する
  - `!IO` → `/* ctx: AppCtx */` に置換する
- [x]`cmd_ai_explain(path: &str, error_code: &str)` を追加した
- [x]`cmd_ai_fix(path: &str) -> Result<(), String>` を追加した
- [x]`cargo build` でエラーがないことを確認

---

## T2: `main.rs` — CLI サブコマンド追加

- [x]既存の `main.rs` の引数パースロジックを読んで構造を確認した
- [x]`fav ai explain <path>` コマンドを追加した
- [x]`fav ai explain <path> --error-code <code>` コマンドを追加した
- [x]`fav ai fix <path>` コマンドを追加した
- [x]`fav check <path> --ai-explain` アームを追加した（エラー時に AI ヒントも表示）
- [x]`cargo build` でエラーがないことを確認

---

## T3: `v722000_tests` 追加（`driver.rs`）

> **前提**: T1 完了済みであること（`get_ai_hint` / `apply_ctx_migration` が `pub fn` で存在すること）。

- [x]`v721000_tests` モジュールの直後に `v722000_tests` モジュールを追加した
- [x]`use super::{apply_ctx_migration, get_ai_hint}` を追加した
- [x]`ai_explain_e0374_returns_hint` テストを実装した
  - `get_ai_hint("E0374")` が `Some(...)` を返すことを確認
  - hint が `"ctx: AppCtx"` または `"!IO"` を含むことを assert
- [x]`ai_fix_applies_ctx_migration` テストを実装した
  - `apply_ctx_migration("fn main() -> Unit !IO { IO.println(\"hello\") }")` の結果が `ctx.io.println(` を含むことを assert
- [x]`cargo build` でエラーがないことを確認

---

## T4: `fav/Cargo.toml` バージョン更新 + `driver.rs` version アサーション更新

- [x]`fav/Cargo.toml` の `version = "72.1.0"` → `version = "72.2.0"` に変更した
- [x]`driver.rs` 内の `version = \"72.1.0\"` 文字列を `version = \"72.2.0\"` に replace_all した
- [x]`driver.rs` 内のエラーメッセージ `"Cargo.toml version should be 72.1.0"` を `"72.2.0"` に replace_all した
- [x]`driver.rs` 内のエラーメッセージ `"Cargo.toml should declare version 72.1.0"` を `"72.2.0"` に replace_all した

---

## T5: 部分テスト確認

- [x]`cargo test v722000` で 2 件 pass することを確認

---

## T6: 全体テスト確認

- [x]`cargo test` 全体で 3616 tests pass（0 failures）であることを確認

---

## T7: `CHANGELOG.md` 更新

- [x]`## [v72.2.0]` エントリを先頭に追加した

---

## T8: `versions/current.md` 更新

- [x]「進行中バージョン」を `v72.2.0`（AI エラーアシスタント）に更新した
- [x]「次に切る版」を `v72.3.0` に更新した

---

## T9: 最終確認

- [x]`cargo test v722000` で 2 件 pass することを確認
- [x]`cargo test` 全体で 3616 tests pass（0 failures）であることを確認
- [x]`fav/Cargo.toml` のバージョンが `72.2.0` であることを確認
- [x]`get_ai_hint("E0374")` が `Some(...)` を返すことを確認（テストで担保）
- [x]`apply_ctx_migration` が `IO.println(` → `ctx.io.println(` に変換することを確認（テストで担保）
- [x]`versions/current.md` が正しく更新されていることを確認

---

## スコープ外（明示的除外）

- 実際の Claude API HTTP 呼び出し: 別タスク（v72.3.0 以降）
- `cmd_ai_fix` の diff プレビュー・対話的確認プロンプト: 別タスク（v72.3.0 以降）
- E0374 / E0001 以外の全エラーコードのヒント: 段階的追加
- `fav check --ai-explain` の完全統合（エラーコード自動抽出）: v72.2.0 では骨格のみ実装（v72.3.0 以降に完全化）
- `site/content/docs/cli/ai.mdx` 追加: v72.3.0 以降
- `site/` MDX 更新: 別タスク（TypeScript ビルド完了後）

---

## コードレビュー指摘対応

| 優先度 | 指摘 | 対応 |
|---|---|---|
| [HIGH] | `apply_ctx_migration` が `"!IO"` を単純置換 → `!IOError` 等の将来識別子に誤適用リスク | `" !IO"` (スペース付き) に限定。テストに `" !IO"` パターン確認を追加 |
| [HIGH] | `cmd_ai_fix` が `std::fs::write` で直接上書き → 途中失敗でファイル破損 | 一時ファイル（`.ai_fix.tmp`）に書いてから `rename` でアトミック置き換えに変更 |
| [MED] | `fav check --ai-explain` が check 結果に関わらず無条件 E0374 表示 | `eprintln!` で骨格実装（v72.2.0 制限）を明示する `note:` メッセージを追加 |
| [MED] | `fav ai explain --error-code E0001 pipeline.fav` で `path = "--error-code"` になる | `args.iter().find(!starts_with('-'))` でハイフン除外方式に変更 |
| [LOW] | v62000_tests のエラーメッセージに `"71.5.0"` 旧バージョン参照が残存 | `"72.2.0"` に修正 |
| [LOW] | tasks.md コードレビュー指摘対応テーブルが空 | 本テーブルに記録 |

---

## 完了チェックリスト

- [x]全タスク（T0〜T9）が完了している
- [x]`ai_explain_e0374_returns_hint` が pass
- [x]`ai_fix_applies_ctx_migration` が pass
- [x]テスト総数: 3616（+2）
