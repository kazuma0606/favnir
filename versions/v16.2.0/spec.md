# v16.2.0 Spec — 文字列補間（String Interpolation）

Date: 2026-06-14
Branch: master

---

## テーマ

データエンジニアが最もよく書く「動的文字列の組み立て」を自然に書けるようにする。
`String.concat` の連鎖を `f"..."` 構文で置き換え、コードを劇的に短縮する。

**現状（Before）:**
```fav
fn greet(name: String, count: Int) -> String {
  String.concat("Hello, ", String.concat(name, String.concat("! You have ", String.concat(Int.to_string(count), " items."))))
}
```

**目標（After）:**
```fav
fn greet(name: String, count: Int) -> String {
  f"Hello, {name}! You have {count} items."
}
```

---

## 現状分析

| コンポーネント | 状態 | 備考 |
|---|---|---|
| `$"..."` 構文（Rust パイプライン） | **実装済み** | lexer/parser/checker.rs/compiler.rs 全対応 |
| `f"..."` 構文 | **未実装** | ロードマップ指定構文、追加が必要 |
| `f"""..."""` triple-quote | **未実装** | 複数行 f-string、追加が必要 |
| checker.fav パス | **未対応** | `ast_lower_checker.rs` が `_unsupported_` にフォールバック |
| compiler.fav パス | **未対応** | compiler.fav 自身のレキサー/パーサーが `$"..."` / `f"..."` を未認識 |
| E0253（ネスト禁止） | **実装済み** | `checker.rs` |
| E0254（Show 未実装） | **実装済み** | `checker.rs`。E0322 の別名として get_help_text に追加 |

---

## スコープ

### A: バージョン更新

```toml
version = "16.2.0"
```

### B: `f"..."` 構文追加（lexer.rs）

`$"..."` に加えて `f"..."` プレフィックスを認識する。

```rust
// fav/src/frontend/lexer.rs — 追加するケース
'f' if self.peek2() == Some('"') && self.peek3() != Some('"') => {
    self.advance(); // 'f'
    self.advance(); // '"'
    TokenKind::FStringRaw(self.lex_fstring_raw(sp, sl, sc)?)
}
```

後方互換: `$"..."` は引き続き動作する（deprecation は v17.0 以降）。

### C: `f"""..."""` triple-quote 追加（lexer.rs）

```rust
// 3文字先読みで triple-quote を判定
'f' if self.peek2() == Some('"') && self.peek3() == Some('"') => {
    self.advance(); // 'f'
    self.advance(); // '"'
    self.advance(); // '"'
    self.advance(); // '"'
    TokenKind::FStringRaw(self.lex_fstring_triple(sp, sl, sc)?)
}
```

- `lex_fstring_triple` は `"""` が出現するまでスキャン（改行を含む）
- 内部の `{expr}` は通常の f-string と同様に処理

### D: checker.fav パス対応（ast_lower_checker.rs）

`Expr::FString` を `String.concat` 連鎖へデシュガーして `ECall` チェーンを生成。

```rust
// fav/src/middle/ast_lower_checker.rs
ast::Expr::FString(parts, _) => lower_fstring_to_concat(parts),
```

```rust
fn lower_fstring_to_concat(parts: &[ast::FStringPart]) -> Value {
    // 空 → 空文字列
    if parts.is_empty() {
        return v1("ELit", v1("LStr", sv("")));
    }
    // 各 part を ECall("String", "concat", [left, right]) に畳み込む
    let mut acc = lower_fstring_part(&parts[0]);
    for part in &parts[1..] {
        let right = lower_fstring_part(part);
        // ECall("String", "concat", EArgList(right, EArgList(acc, EArgNil)))
        acc = v3(
            "ECall",
            v3("ECallParts", sv("String"), sv("concat"),
               v2("EArgList", right, v2("EArgList", acc, v0("EArgNil"))))
        );
    }
    acc
}

fn lower_fstring_part(part: &ast::FStringPart) -> Value {
    match part {
        ast::FStringPart::Lit(s) => v1("ELit", v1("LStr", sv(s))),
        ast::FStringPart::Expr(expr) => lower_expr(expr),
    }
}
```

> **注意**: checker.fav の型チェック（ECall("String", "concat", ...)）は String 引数を期待する。
> Rust の checker.rs がすでに型変換を処理しているため、ここでは単純デシュガーで十分。
> Display 型チェックは Rust 側 checker.rs で行われ、E0254 として報告される。

### E: compiler.fav パス対応

compiler.fav は自分のレキサー/パーサーを持つため、f-string をそちらにも追加する必要がある。

**E-1: compiler.fav のレキサーに f-string トークン追加**

```fav
// self/compiler.fav — lex関数内に追加
// 'f' の次が '"' → f-string として読む
```

`TkFStr(content: String)` トークンを追加し、`{...}` の内側をそのまま文字列として保持（後でパース）。

**E-2: compiler.fav の AST に EFString 追加**

```fav
type Expr =
  | ...
  | EFString(List<FStringPart>)  // 追加

type FStringPart =
  | FStrLit(String)
  | FStrExpr(Expr)
```

**E-3: compiler.fav の compile_expr に EFString 追加**

```fav
EFString(parts) => compile_fstring(parts, ctx, env)
```

```fav
fn compile_fstring(parts: List<FStringPart>, ctx: CodegenCtx, env: List<KVPair>) -> Result<CodegenCtx, String> {
  // 各 part を String.concat でチェーン
  // FStrLit(s) → ELit(LStr(s)) としてコンパイル
  // FStrExpr(e) → e をコンパイルし Debug.show を通す（非String型の場合）
  // ...
}
```

> **スコープ縮小オプション（v16.2.0 現実的アプローチ）**:
> compiler.fav への f-string 追加は v16.2.0 で基礎実装、
> 完全動作（全型の自動変換）は v16.3.0 以降に延期可能。
> v16.2.0 では Rust パイプライン（`--legacy` 相当）で f-string が完全動作することを優先する。

### F: E0322 エラーコード（Display 未実装）

既存の E0254 を E0322 として docs に公式化し、`get_help_text` に追加する。

```rust
// fav/src/driver.rs — get_help_text に追加
"E0254" | "E0322" => &[
    "f-string には String / Int / Float / Bool のみ使用可能",
    "型に `interface Show` を実装するか、`.to_string()` 等で文字列変換してください",
],
```

E0254 は既存コードとの互換性のため保持。E0322 は今後の推奨コード。

### G: テスト（v162000_tests — 5件）

`fav/src/driver.rs` に追加。
Rust パイプライン（`build_artifact` / `exec_artifact_main`）を使用して実行結果を検証。

1. `version_is_16_2_0`: `Cargo.toml` version == "16.2.0"
2. `fstring_basic_interpolation`: `f"Hello, {name}!"` → `"Hello, Alice!"`
3. `fstring_int_interpolation`: `f"Age: {age}"` with Int 42 → `"Age: 42"`
4. `fstring_expr_interpolation`: `f"len={List.length(xs)}"` → `"len=3"`
5. `fstring_triple_quote`: `f"""line1\nline2"""` または `f"""..."""` が正しい文字列を返す

### H: サイトドキュメント

`site/content/docs/language/string-interpolation.mdx` 新規作成:
- 基本構文（`f"..."` / triple-quote）
- 埋め込める型（String / Int / Float / Bool）
- Before/After 比較
- よくある間違い（Display 未実装型 → E0322）

---

## 完了条件

| 確認項目 | 状態 |
|---|---|
| `Cargo.toml version == "16.2.0"` | [ ] |
| `f"Hello, {name}!"` が正しく動作する | [ ] |
| `f"Age: {age}"` (Int) が自動変換される | [ ] |
| `f"len={List.length(xs)}"` (式) が動作する | [ ] |
| `f"""..."""` triple-quote が動作する | [ ] |
| `cargo test v162000` 全テストパス（5/5） | [ ] |
| `cargo test` 全件パス（リグレッションなし） | [ ] |
| `$"..."` 旧構文が引き続き動作する | [ ] |
| E0322 が get_help_text に追加されている | [ ] |
| `site/content/docs/language/string-interpolation.mdx` が存在する | [ ] |

---

## 既知の制約・スコープ外

- `{expr}` 内への f-string ネスト（E0253 禁止）は v16.2.0 でも継続禁止
- compiler.fav の f-string 完全実装は段階的実装:
  - v16.2.0: Rust パイプラインで完全動作、compiler.fav は基礎追加（または延期）
  - v16.3.0 以降: compiler.fav 完全対応
- `$"..."` の deprecation 警告は v17.0.0 以降で追加予定

---

## 参照

- `versions/roadmap-v16.1-v17.0.md` — v16.2.0 セクション
- `fav/src/frontend/lexer.rs` — `$"..."` 実装済み（`lex_fstring_raw`）
- `fav/src/frontend/parser.rs` — `parse_fstring_parts` 実装済み
- `fav/src/middle/checker.rs` — E0253/E0254 実装済み
- `fav/src/middle/compiler.rs` — `compile_fstring` 実装済み
- `fav/src/middle/ast_lower_checker.rs` — FString → `_unsupported_` フォールバック（修正対象）
- `fav/self/compiler.fav` — f-string 未対応（追加対象）
