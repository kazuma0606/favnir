# v43.11.0 タスク — Opaque type 完全化

## ステータス: COMPLETE（2026-07-13）— 2932 tests

---

## T0 — 事前確認

- [x] `cargo test` 2929 / 0 確認
- [x] `Cargo.toml` version = `43.10.0` 確認
- [x] `v431100_tests` が `fav/src/driver.rs` に存在しないことを確認
- [x] `TypeDef` に `is_opaque` フィールドが存在しないことを確認（`fav/src/ast.rs` 行 200 付近）
- [x] `parse_item` に `"opaque"` アームが存在しないことを確認（`fav/src/frontend/parser.rs`）
- [x] `E0413〜E0419: 予約` コメントが `error_catalog.rs` に存在することを確認（実エントリなし）
- [x] `get_explain_text` に `"E0413"` エントリが存在しないことを確認（`fav/src/driver.rs`）
- [x] `check_opaque_coerce_violations` が `fav/src/driver.rs` に存在しないことを確認

---

## T1 — ast.rs: `TypeDef.is_opaque` 追加

- [x] `TypeDef` 構造体に `pub is_opaque: bool,` を追加（`body` フィールドの直前）

---

## T2 — parser.rs: TypeDef 構築 4 箇所に `is_opaque: false` 追加

T1・T2 は `TypeDef` 構造体変更のため**必ず同時適用**すること（コンパイルエラー防止）。

- [x] Wrapper type 構築（`TypeBody::Wrapper` の early return）に `is_opaque: false,` 追加
- [x] Record type 構築（`TypeBody::Record` の early return）に `is_opaque: false,` 追加
- [x] Alias type 構築（`TypeBody::Alias` の early return）に `is_opaque: false,` 追加
- [x] Sum type 構築（関数末尾の最終 `Ok(TypeDef { ... body })`）に `is_opaque: false,` 追加

---

## T3 — parser.rs: `parse_item` に `"opaque"` アーム追加

- [x] `TokenKind::Type =>` の直前に `"opaque"` アームを追加
  - `TokenKind::Ident(name) if name == "opaque"` パターン（`ref` は不要）
  - `self.advance()` で "opaque" を消費
  - 返った `TypeDef` の `is_opaque` を `true` に設定して返す

---

## T4 — error_catalog.rs: E0413 エントリ追加

- [x] `// ── E0413〜E0419: 予約` コメント行を削除
- [x] E0413 実エントリを追加（`code: "E0413"` / `title: "opaque type coerce forbidden"`）
- [x] `// ── E0414〜E0419: 予約（将来拡張用）` コメントを残す

---

## T5 — driver.rs: `check_opaque_coerce_violations` + `is_bare_inner_literal` + E0413 + cmd_check 更新

- [x] `check_opaque_coerce_violations(src: &str, filename: &str) -> Vec<String>` を追加
  - `collect_explain_output` の直後、`cmd_check` の前に配置
  - `TypeExpr::Named(inner_name, params, _)` の 3 フィールドパターンを使用（Span 含む）
- [x] `fn is_bare_inner_literal(expr: &Expr, inner_type: &str) -> bool` をプライベートヘルパーとして追加
- [x] `get_explain_text` に `"E0413"` エントリを追加（`_ => None` の直前）
- [x] `cmd_check` の `if errors.is_empty()` ブランチ内、`"no errors found"` 表示の直前に opaque チェックブロックを追加

---

## T6 — driver.rs: `v431100_tests` 追加 / Cargo.toml / スタブ化

- [x] `v431000_tests` モジュールの直前に `v431100_tests` を挿入（3 件）
- [x] `v431000_tests::cargo_toml_version_is_43_10_0` をスタブ化（`// Stubbed: version bumped to 43.11.0 in v43.11.0.`）
- [x] `fav/Cargo.toml` version を `43.10.0` → `43.11.0` に更新

---

## T7 — CHANGELOG.md

- [x] v43.11.0 エントリ追加

---

## T8 — テスト実行・確認

- [x] `cargo test -j 8 -- --test-threads=8` 実行
- [x] 2932 passed; 0 failed 確認
- [x] `v431100_tests` 3 件 pass 確認

---

## T9 — バージョン管理ドキュメント更新

- [x] `versions/current.md` → v43.11.0 最新安定版（2932 tests）、次版 v43.12.0
- [x] `versions/roadmap/roadmap-v43.1-v44.0.md` → v43.11.0 を `✅ COMPLETE（2026-07-13）`、checker.fav 記述を「AST レベル MVP」に修正
- [x] `versions/v40-v45/v43.11.0/tasks.md` → COMPLETE、全チェックボックス `[x]`

---

## 実装時の知見

- **checker.rs にも TypeDef 構築箇所が 1 つ存在**: `src/middle/checker.rs` 行 2759 付近。parser.rs 4 箇所の他にこの箇所も `is_opaque: false` を追加する必要があった（コンパイルエラー E0063 で判明）
- **`TypeExpr::Named` は 3 フィールド**: `Named(String, Vec<TypeExpr>, Span)` — パターンマッチには `_` で Span を無視する（`Named(name, params, _)`）
- **opaque check の挿入位置**: `if errors.is_empty()` の内側（型エラーがある場合はスキップ、型エラーを優先）
