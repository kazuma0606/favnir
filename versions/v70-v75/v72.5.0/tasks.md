# v72.5.0 タスクリスト — Playground 2.0

Date: 2026-08-12
Status: 完了

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `72.4.0` であることを確認
- [x] `cargo test` が 3625 tests pass（0 failures）であることを確認（v72.4.0 完了後の実測値: 計画値 3622 + code-reviewer 対応 +3 = 3625）
- [x] `driver.rs` に `v724000_tests` モジュールが存在することを確認
- [x] `driver.rs` に `v725000_tests` が未存在であることを確認
- [x] `driver.rs` に `PLAYGROUND_TEMPLATES` / `playground_share_url` が未存在であることを確認
- [x] `driver.rs` 内の `"72.4.0"` 文字列（バージョンアサーション）の件数を grep で確認しておく（29 件）

---

## T1: `driver.rs` — `PlaygroundTemplate` + `PLAYGROUND_TEMPLATES` 追加

- [x] `PlaygroundTemplate` 構造体を `pub struct` で追加した（`name`, `description`, `code` フィールド）
- [x] `PLAYGROUND_TEMPLATES: &[PlaygroundTemplate]` を `pub static` で定義した
  - エントリ数: 5 件（Hello World / CSV ETL / AI Generate / Distributed Par / Data Quality）
  - 各エントリに `name` / `description` / `code` が設定されている
- [x] `cargo build` でエラーがないことを確認

---

## T2: `playground_share_url` 追加

- [x] `playground_share_url(code: &str) -> String` を `pub fn` で追加した
  - `code` を hex エンコード（`{b:02x}` フォーマット）
  - `/playground?code={encoded}` 形式の文字列を返す
- [x] `cargo build` でエラーがないことを確認

---

## T3: `v725000_tests` 追加（`driver.rs`）

- [x] `v724000_tests` モジュールの直後に `v725000_tests` モジュールを追加した
- [x] `use super::{playground_share_url, PLAYGROUND_TEMPLATES}` を追加した
- [x] `playground2_template_gallery_has_5_entries` テストを実装した
  - `PLAYGROUND_TEMPLATES.len() >= 5` を assert
- [x] `playground2_share_url_format` テストを実装した
  - `playground_share_url("fn main() -> Unit { }")` が `/playground?code=` で始まることを assert
  - URL のコード部分が空でないことを assert
- [x] `cargo test v725000` で 2 件 pass することを確認

---

## T4: `fav/Cargo.toml` バージョン更新 + `driver.rs` version アサーション更新

- [x] `fav/Cargo.toml` の `version = "72.4.0"` → `version = "72.5.0"` に変更した
- [x] `driver.rs` 内の `version = \"72.4.0\"` 文字列を `version = \"72.5.0\"` に replace_all した
- [x] `driver.rs` 内のエラーメッセージ `"Cargo.toml version should be 72.4.0"` を `"72.5.0"` に replace_all した
- [x] `driver.rs` 内のエラーメッセージ `"Cargo.toml should declare version 72.4.0"` を `"72.5.0"` に replace_all した
- [x] 残存 72.4.0 は 3件（フィールドコメント + セクションヘッダー × 2）のみで意図的保持を確認

---

## T5: 部分テスト確認

- [x] `cargo test v725000` で 2 件 pass することを確認

---

## T6: 全体テスト確認

- [x] `cargo test` 全体で 3627 tests pass（0 failures）であることを確認

---

## T7: `CHANGELOG.md` 更新

- [x] `## [v72.5.0]` エントリを先頭に追加した

---

## T8: `versions/current.md` 更新

- [x] 「進行中バージョン」を `v72.5.0`（Playground 2.0）に更新した
- [x] 「次に切る版」を `v72.6.0` に更新した

---

## T9: 最終確認（T7・T8 完了後のドキュメント更新後リグレッション確認）

- [x] `cargo test v725000` で 2 件 pass することを確認
- [x] `cargo test` 全体で 3627 tests pass（0 failures）であることを確認
- [x] `fav/Cargo.toml` のバージョンが `72.5.0` であることを確認
- [x] `PLAYGROUND_TEMPLATES.len() >= 5` であることを確認（テストで担保）
- [x] `playground_share_url(...)` が `/playground?code=` で始まることを確認（テストで担保）
- [x] `versions/current.md` の「進行中バージョン」が `v72.5.0`、「次に切る版」が `v72.6.0` であることを確認

---

## スコープ外（明示的除外）

- サイト側（TypeScript / Next.js）の Monaco エディタ統合・AI 補完（v73.x 以降）
- 実行結果の可視化（List<Record> → テーブル表示、List<Float> → グラフ）（v73.x 以降）
- `base64` crate 追加（WASM 互換性リスクを避け本バージョンでは hex エンコードを使用）
- 共有リンクの永続化・サーバー側ストレージ（v73.x 以降）
- `site/content/playground/` MDX 更新（v73.x 以降）
- `rustyline` 統合・`~/.fav_history` 永続化・Rune メソッド補完（v72.4.0 から延期、v72.6.0 以降に実施）

---

## コードレビュー指摘対応

| 優先度 | 指摘 | 対応 |
|---|---|---|
| [HIGH] | デコード関数が存在せず hex エンコードの対称性を検証できない | `playground_decode_url` を追加 + ラウンドトリップテスト追加 |
| [MED] | `#[derive(Debug)]` 欠落 | `#[derive(Debug, Clone, Copy)]` を追加 |
| [MED] | 空文字列入力で空コード部分の URL が生成される | 戻り値を `Option<String>` に変更、空入力・8192B超で `None` を返す |
| [MED] | 大入力（>2000文字）への上限なし | 8192 バイト上限ガードを追加 |
| [MED] | `playground_share_url` のラウンドトリップテスト欠如 | `playground2_share_url_roundtrip` テスト追加 |
| [MED] | テンプレートコードが説明と乖離（Hello World 戻り型・Distributed Par に par なし・Data Quality に Schema.validate_all なし） | 全 3 テンプレートのコード文字列を修正 |
| [LOW] | `Copy` 導出が未実施 | `#[derive(Copy)]` を追加 |
| [LOW] | テンプレートフィールド空チェックのテストなし | `playground2_template_fields_non_empty` テスト追加 |
| [PERF] | バイトごとの `format!` アロケーション | `String::with_capacity` + `write!` マクロに変更 |

---

## 完了チェックリスト

- [x] 全タスク（T0〜T9）が完了している
- [x] `playground2_template_gallery_has_5_entries` が pass
- [x] `playground2_share_url_format` が pass
- [x] `playground2_share_url_empty_returns_none` が pass（コードレビュー対応で追加）
- [x] `playground2_share_url_roundtrip` が pass（コードレビュー対応で追加）
- [x] `playground2_template_fields_non_empty` が pass（コードレビュー対応で追加）
- [x] テスト総数: 3630（+5）
