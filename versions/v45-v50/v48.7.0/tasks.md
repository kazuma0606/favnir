# Tasks: v48.7.0 — rune.toml 標準化

Status: COMPLETE
Date: 2026-07-18

---

## T0 — 事前確認

- [x] `cargo test` 3058 passed, 0 failed を確認（ベース確認）
- [x] `toml.rs` に `validate_rune_toml` が存在しないことを確認
- [x] `parse_kv` が `toml.rs` 内に定義されていることを確認（同ファイル内から呼べる）

## T1 — `toml.rs` `validate_rune_toml` 追加

- [x] `parse_kv` 関数の直後に `validate_rune_toml(content: &str) -> Vec<String>` を追加
  - [x] `[rune]` セクション必須チェック（なければ `"missing [rune] section"` エラー）
  - [x] `name` / `version` / `entry` フィールド必須チェック（なければ各 `"missing required field: X"` エラー）
  - [x] `[connection]` セクション非標準チェック（あれば `"[connection] section is non-standard; remove it"` エラー）
  - [x] `pub` 修飾子をつけること

## T2 — `driver.rs` テスト追加

- [x] `v487000_tests` モジュールを `v486000_tests` の直前に追加（2テスト）
  - [x] `rune_toml_standard_format`: `[rune]`+`name`/`version`/`entry` で空 Vec が返ることを確認
  - [x] `rune_toml_no_connection_section`: `[connection]` セクション付きで errors に `"connection"` 含まれることを確認

## T3 — バージョン更新・完了

- [x] `fav/Cargo.toml` version → `"48.7.0"`
- [x] `CHANGELOG.md` に v48.7.0 エントリ追加
- [x] `cargo test` 3060 passed, 0 failed（3058 + 2 件）
- [x] `cargo clippy -- -D warnings` クリーン
- [x] `versions/current.md` を v48.7.0（3060 tests）に更新、進行中バージョンを `v48.8.0` に更新
- [x] `versions/roadmap/roadmap-v48.1-v49.0.md` の v48.7.0 テスト数を実績値 3060 に更新（`roadmap-v45.1-v50.0.md` への反映は v49.0.0 時・変更不要）
- [x] tasks.md を COMPLETE に更新（T0〜T3 全 `[x]`）

---

> **注記**: 全公式 rune の `rune.toml` ファイル更新は v48.7.0 のスコープ外（v48.4.0 スタブは標準フォーマット済み）
> **注記**: `cargo clean` はこのバージョンのスコープ外（v49.0.0 で実施）
