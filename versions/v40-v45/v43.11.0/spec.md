# v43.11.0 仕様書 — Opaque type 完全化

## 概要

`opaque type Token = String` 構文を追加する。

- `opaque` contextual キーワードをパーサーで認識し、`TypeDef.is_opaque = true` としてマークする
- E0413（opaque coerce 禁止）を追加し、不正な暗黙 coerce を AST レベルで検出する
- `check_opaque_coerce_violations` で違反チェックを実装する

---

## 背景

v43.1〜v43.10 で型推論の強化（戻り値型推論・ジェネリック推論・双方向推論等）を実施した。
次の課題は**型の境界を守ること**。`opaque type` を用いることで、`Token` と `String` が別の型として扱われ、
`String` 値を `Token` 変数に代入する暗黙 coerce を静的に禁止できる。

---

## 機能仕様

### `opaque type` 構文

```favnir
opaque type Token = String   // 外部からの String → Token の暗黙 coerce を禁止
fn make_token(raw: String) -> Token { raw }  // E0413: String を Token として返すのは禁止
```

**E0413 検出条件（v43.11.0 スコープ）:**
1. `return_ty = Named("Token", [])` かつ `"Token"` が opaque alias for `"String"`
2. fn ボディが直接 `Expr::Lit(Lit::Str(_), _)` リテラルである

→ E0413: `opaque type coerce forbidden: cannot return String as Token`

**型チェックとの関係:**
- E0413 チェックは「型チェック（checker_fav_runner）が通過した後」に実施する
- 型チェックエラーが存在する場合は opaque チェックをスキップする（型エラーを優先）

---

## 実装方針

### ast.rs

`TypeDef` に `is_opaque: bool` フィールドを追加（デフォルト `false`）。

### parser.rs（`parse_item` 内）

`opaque` は現在 `TokenKind::Ident("opaque")` として lexer に渡る（contextual keyword 方式）。
`parse_item` の `TokenKind::Type =>` の直前に以下のアームを追加する：

```rust
TokenKind::Ident(name) if name == "opaque" => {
    self.advance(); // consume "opaque"
    let mut td = self.parse_type_def(vis)?;
    td.is_opaque = true;
    Ok(Item::TypeDef(td))
}
```

**注意**: `parse_type_def` 内の `TypeDef { ... }` 構築は 4 箇所あり、すべてに `is_opaque: false` を追加する必要がある。

### error_catalog.rs

`// ── E0413〜E0419: 予約` コメントを削除し、E0413 実エントリを追加:

```rust
// ── E0413: opaque type coerce (v43.11.0) ──────────────────────────────────────
ErrorEntry {
    code: "E0413",
    title: "opaque type coerce forbidden",
    ...
}
// ── E0414〜E0419: 予約（将来拡張用） ─────────────────────────────────────────
```

### driver.rs

- `pub fn check_opaque_coerce_violations(src: &str, filename: &str) -> Vec<String>` を追加
  - opaque alias map を収集し、FnDef の return_ty と body を照合して E0413 を返す
- `pub(crate) fn is_bare_inner_literal(expr, inner_type) -> bool` をプライベートヘルパーとして追加
- `get_explain_text` に `"E0413"` エントリを追加
- `cmd_check` の `if errors.is_empty()` ブランチ内に opaque チェックを追加（型チェック通過後のみ）
- `v431100_tests` テストモジュールを追加

### main.rs

変更なし。

---

## checker.fav 統合について

ロードマップには「checker.fav に追加」と記載されているが、v43.11.0 では AST レベルの静的チェック（Rust 実装）にとどめる。
理由: checker.fav への opaque 型統合は HM 型推論システム（unify / infer_hm）の大幅な改修を必要とするため、
まず AST レベルでの最小実装（E0413 の基盤）を安定させてから段階的に拡張する。
ロードマップ `roadmap-v43.1-v44.0.md` の v43.11.0 エントリを「AST レベル MVP」として修正する。

---

## テスト（`v431100_tests` — 3 件）

> **命名注意**: `v43100_tests` は v43.1.0 の既存モジュール、`v431000_tests` は v43.10.0。
> v43.11.0 のモジュール名は **`v431100_tests`**（43.11.0 = 43×10000 + 11×100 + 0）。

1. `cargo_toml_version_is_43_11_0`
2. `parser_recognizes_opaque_type_keyword` — `opaque type Token = String` を parse して `TypeDef.is_opaque == true`
3. `e0413_opaque_coerce_blocked` — `check_opaque_coerce_violations` が違反コードで E0413 を含む Vec を返す

---

## スコープ外

- checker.fav への opaque 型統合（→ 将来版）
- bind 式・関数引数・型パラメータでの opaque 強制（→ 将来版）
- `opaque` をレキサーのハードキーワードとして登録（→ 将来版。現在は contextual keyword）
- `site/content/docs/language/opaque-types.mdx` — v43.13.0（cookbook + 安定化）のスコープ
- 型チェックエラーが存在する場合の opaque チェック実行（型エラーを優先）

---

## 完了条件

- `cargo test -j 8 -- --test-threads=8` で 2932 tests passed, 0 failed
- `v431100_tests` 3 件 pass
- `Cargo.toml` version = `43.11.0`
