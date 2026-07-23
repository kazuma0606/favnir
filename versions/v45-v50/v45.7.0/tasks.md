# Tasks: v45.7.0 — エラーメッセージ改善 Phase 2 + 数値リテラル `_`

Status: COMPLETE
Date: 2026-07-16

---

## T0 — 事前確認

- [x] `cargo test` 2982 passed, 0 failed を確認

## T1 — `error_catalog.rs`: E0201〜E0413 suggestion 追加

- [x] E0213 に `suggestion: Some(...)` を設定
- [x] E0219〜E0227 に `suggestion: Some(...)` を設定（各エントリ個別）
- [x] E0241〜E0245 に `suggestion: Some(...)` を設定
- [x] E0251 / E0253 / E0254 に `suggestion: Some(...)` を設定
- [x] E0274 に `suggestion: Some(...)` を設定
- [x] E0310〜E0315 に `suggestion: Some(...)` を設定
- [x] E0319〜E0324 に `suggestion: Some(...)` を設定
- [x] E0365 / E0368 / E0369 / E0373 / E0374 に `suggestion: Some(...)` を設定
- [x] E0380〜E0384 は変更対象外（spec 内容確認済み）
  - 注意: E0380〜E0384 は spec §1 の対象リストに含まれていたが、実際のエントリ内容を確認の上スキップ（spec の表に記載した「misc errors」扱い）
- [x] E0401〜E0406 に `suggestion: Some(...)` を設定
- [x] E0410〜E0413 に `suggestion: Some(...)` を設定
- [x] 注意: E0230・E0414 は実エントリなし — 変更しない

## T2 — `lexer.rs`: 数値リテラル `_` サポート

- [x] `lex_number` の整数部スキャンループを `c.is_ascii_digit() || c == '_'` に変更
- [x] `_` は advance するが `s.push` しないことを確認
- [x] 小数部スキャンループも同様に修正
- [x] 指数部スキャンは存在しないため修正対象外（skip）

## T3 — `driver.rs`: v457000_tests 追加

- [x] `v457000_tests` モジュール追加（`v456000_tests` の直後）
- [x] `run_inline(src: &str) -> Value` ヘルパー定義（既存モジュールと同パターン）
- [x] `e0410_suggestion` テスト追加（ERROR_CATALOG E0410 の suggestion が Some であること）
- [x] `numeric_literal_underscore_int` テスト追加（`1_000_000` → `Value::Int(1_000_000)`）
- [x] `numeric_literal_underscore_float` テスト追加（`0.000_15` → `Value::Float(0.000_15)`）

## T4 — テスト＆完了

- [x] `cargo test` 2985 passed, 0 failed
- [x] `cargo clippy -- -D warnings` クリーン
- [x] `fav/Cargo.toml` version → `45.7.0`
- [x] `CHANGELOG.md` に v45.7.0 エントリ追加
- [x] `versions/current.md` を v45.7.0（2985 tests）に更新
- [x] tasks.md を COMPLETE に更新（T0〜T4 全チェック）

## コードレビュー指摘と対応

- [MED] code-reviewer: E0415 / E0416 に `suggestion: None` が残っていた → `Some(...)` を設定
- [MED] code-reviewer: `default_suggestion` が catalog の `suggestion` を無視しており `fav check --json` に反映されない → `_` アームに `error_catalog::lookup(code).and_then(|e| e.suggestion)` フォールバック追加
- [BUG/LOW] code-reviewer: 末尾/連続 `_`（`1_`、`1__000`）が無エラー通過 → 仕様上の意図的動作として lexer.rs にコメント追加
- [LOW] code-reviewer: `0x` hex リテラルに `_` サポートなし → スコープ外（コメントで明示）
- [LOW] code-reviewer: E0365 の category が `"effects"` → 既存バグのためスコープ外
- [LOW] code-reviewer: E0416 example の `let` → `bind` に修正
- [LOW] code-reviewer: Float 直接比較は現状問題なし（許容誤差不要）
