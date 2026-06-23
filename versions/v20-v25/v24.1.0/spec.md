# v24.1.0 — 形式的仕様書生成（`fav spec`）

Date: 2026-06-23

## 目標

vm.fav が完成したことで Favnir の言語セマンティクスが Favnir コード自体で表現できる状態になった。
v24.1.0 では、その言語仕様を人間が読める形式の仕様書として出力する `fav spec` コマンドを追加する。

```bash
fav spec --format markdown    # → stdout（Markdown）
fav spec --format html        # → stdout（HTML）
```

---

## ロードマップとの対応

| ロードマップ | v24.1.0 での対応 |
|---|---|
| `fav spec --format markdown > SPEC.md` | `cmd_spec("markdown")` で Markdown 仕様書を生成 ✓ |
| `fav spec --format html > spec/index.html` | `cmd_spec("html")` で HTML 変換（既存 `md_to_html` 利用）✓ |
| 型システムの形式的定義 | 型推論規則（HM）+ 基本型表 ✓ |
| opcode の動作仕様 | 全 opcode の decode → execute 対応表 ✓ |
| エフェクトシステムの意味論 | エフェクト一覧 + 合成規則 + 純粋性定義 ✓ |
| パターンマッチの網羅性チェック規則 | 网羅性条件 + パターン種別表 ✓ |

---

## 既知の制限

| 制限 | 詳細 |
|---|---|
| HTML テーブル出力 | 既存 `md_to_html` はテーブル記法（`\|...\|`）を `<table>` に変換しない。テーブル行は `<p>` タグとして出力される。`cmd_spec_html_has_h1` テストは `<h1>` の存在のみを確認し、テーブル変換は確認しない。テーブル対応は Phase 2 以降。 |

---

## スコープ

### Rust（driver.rs + main.rs）

| 変更種別 | 対象 | 内容 |
|---|---|---|
| private 関数追加 | `driver.rs` | `fn build_spec_markdown() -> String` |
| 公開関数追加 | `driver.rs` | `pub fn cmd_spec(format: &str) -> String` |
| サブコマンド追加 | `main.rs` `"spec"` アーム | `--format markdown\|html` 解析と `cmd_spec` 呼び出し |

### ドキュメント

| 変更種別 | 対象 | 内容 |
|---|---|---|
| 新規作成 | `site/content/docs/tools/spec.mdx` | `fav spec` コマンド説明ページ |
| セクション追加 | `README.md` | `fav spec` CLI の説明追加 |
| エントリ追加 | `CHANGELOG.md` | v24.1.0 エントリ |
| 新規作成 | `benchmarks/v24.1.0.json` | テスト件数 |

---

## 新関数定義

### `fn build_spec_markdown() -> String`（private）

Favnir 言語仕様の Markdown 文書を生成する。以下の 4 セクションを含む:

```
# Favnir 言語仕様書

バージョン: {CARGO_PKG_VERSION}

---

## 1. 型システム（Type System）

### 1.1 基本型
| 型 | 説明 |
...（Int / Float / Bool / String / Unit / List<A> / Option<A> / Result<A,E> / Bytes）

### 1.2 型推論規則
[Var]  Γ, x : σ ⊢ x : σ
[Abs]  Γ, x : τ₁ ⊢ e : τ₂  →  Γ ⊢ fn(x) { e } : τ₁ → τ₂
[App]  Γ ⊢ f : τ₁ → τ₂, Γ ⊢ a : τ₁  →  Γ ⊢ f(a) : τ₂
[Let]  Γ ⊢ e₁ : σ, Γ, x : σ ⊢ e₂ : τ  →  Γ ⊢ (bind x <- e₁; e₂) : τ
[Gen]  Γ ⊢ e : τ, α ∉ free(Γ)  →  Γ ⊢ e : ∀α. τ

---

## 2. opcode 動作仕様

### 2.1 opcode 一覧
| opcode | バイト | オペランド | 説明 |
| Const | 0x01 | u24(idx) | 定数テーブル[idx] をプッシュ |
...（全 opcode を列挙）

### 2.2 オペランドエンコーディング
- u24(idx): 3 バイトリトルエンディアン符号なし整数
- i24(off): 3 バイトリトルエンディアン符号あり整数
- —: オペランドなし（1 バイト命令）

---

## 3. エフェクトシステム（Effect System）

### 3.1 エフェクト一覧
| エフェクト | アノテーション | 説明 |
| Pure | （なし） | 副作用なし（決定論的） |
...（Io / Http / Llm / Db / Snowflake / File / Trace）

### 3.2 エフェクト意味論
エフェクトアノテーションなし ⇔ 純粋（参照透過）:
  ∀ f : A → B. (エフェクトアノテーションなし) ⇒ f は参照透過

エフェクト合成:
  f : A → B !E₁, g : B → C !E₂  ⊢  (g ∘ f) : A → C !(E₁ ∪ E₂)

---

## 4. パターンマッチ網羅性（Pattern Match Exhaustiveness）

### 4.1 网羅性条件
  match e { pat₁ => e₁ ... patₙ => eₙ }
  条件: ∀ v ∈ type(e). ∃ i. match(patᵢ, v) = true

非網羅的なパターンマッチは E0010 としてコンパイルエラー。

### 4.2 パターン種別
| パターン | 例 | マッチ条件 |
...（リテラル / 変数 / ワイルドカード / バリアント / Or / レコード）
```

**実装方針:**
1. バージョン番号（`env!("CARGO_PKG_VERSION")`）をヘッダに埋め込む
2. 仕様コンテンツは `const SPEC_CONTENT: &str = r#"..."#` として静的定義
3. `format!("{header}{body}", ...)` で結合（`{` `}` の二重エスケープを回避するため本体は raw string に分離）

### `pub fn cmd_spec(format: &str) -> String`（public）

```rust
pub fn cmd_spec(format: &str) -> String {
    let md = build_spec_markdown();
    match format {
        "html" => md_to_html(&md),
        _ => md,
    }
}
```

- `"html"` → 既存の private `md_to_html` を呼び出し
- それ以外（`"markdown"` / 未指定）→ Markdown をそのまま返す
- `md_to_html` は driver.rs 内 private 関数（line 10244）なので同一ファイルから直接呼べる

### CLI（`main.rs`）

```
fav spec [--format markdown|html]
```

- `--format`: 出力形式（省略時: `"markdown"`）
- `cmd_spec(format)` の戻り値を `println!("{}", result)` で stdout に出力
- サブコマンド `"spec"` を `"doc"` アームの直後に追加

---

## テスト（5 件）

| テスト名 | 内容 | 期待値 |
|---|---|---|
| `version_is_24_1_0` | Cargo.toml に `version = "24.1.0"` | — |
| `cmd_spec_markdown_has_type_system` | `cmd_spec("markdown")` に `"型システム"` | — |
| `cmd_spec_markdown_has_opcodes` | `cmd_spec("markdown")` に `"0x01"` | — |
| `cmd_spec_html_has_h1` | `cmd_spec("html")` に `"<h1>"` | — |
| `changelog_has_v24_1_0` | CHANGELOG.md に `[v24.1.0]` | — |

---

## README 追加内容

既存の `fav doc` セクションの直後に `fav spec` セクションを追加:

```markdown
### fav spec

```bash
fav spec --format markdown > SPEC.md
fav spec --format html > spec/index.html
```

型システム・opcode 動作仕様・エフェクトシステム・パターンマッチ規則を
Markdown または HTML 形式の仕様書として出力する。
```

---

## 完了条件

- [ ] `fn build_spec_markdown()` が実装される（4 セクション含む）
- [ ] `pub fn cmd_spec(format: &str) -> String` が実装される
- [ ] `main.rs` に `"spec"` サブコマンドが追加される
- [ ] `v240000_tests::version_is_24_0_0` が削除済み
- [ ] `cargo test v241000 --bin fav` — 5/5 PASS
- [ ] `cargo test --bin fav` — リグレッションなし（1930 件以上合格）
- [ ] `CHANGELOG.md` に v24.1.0 エントリ
- [ ] `benchmarks/v24.1.0.json` 作成済み
- [ ] `README.md` に `fav spec` セクション追加済み
- [ ] `site/content/docs/tools/spec.mdx` 作成済み
- [ ] HTML 出力のテーブル非変換（`<p>` 出力）が既知制限として spec.md に明記済み
