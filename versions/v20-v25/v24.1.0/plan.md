# v24.1.0 実装計画 — 形式的仕様書生成（`fav spec`）

## 前提確認

v24.1.0 は Rust 2 ファイル（driver.rs + main.rs）+ ドキュメント変更のみ。

### 実装前チェック

```bash
grep -n "version = " fav/Cargo.toml
# → "24.0.0" であること

grep -n "mod v240000_tests\|mod v241000_tests" fav/src/driver.rs | head -5
# → v241000_tests が未存在であること

grep -n "cmd_spec\|\"spec\"" fav/src/driver.rs fav/src/main.rs | head -5
# → 全 0 件であること（未実装）

grep -n "fav spec" README.md | head -3
# → 0 件であること
```

---

## T0: 事前確認

```bash
# md_to_html の可視性確認（private fn のため同一ファイルから呼ぶ）
grep -n "^fn md_to_html\|^pub fn md_to_html" fav/src/driver.rs | head -3
# → line 10244 に "fn md_to_html" であること

# "doc" アームの位置確認（"spec" アームの挿入位置）
grep -n "Some(\"doc\")\|Some(\"spec\")" fav/src/main.rs | head -5
# → "spec" 未存在
```

---

## T1: `fav/src/driver.rs` — `cmd_spec` 追加

`md_to_html`（line 10244）の直後・`cmd_doc_site`（line 10335）の直前に追加する。

### T1-1: `SPEC_CONTENT` 定数と `build_spec_markdown`

```rust
// ── v24.1.0: fav spec — 形式的仕様書 ────────────────────────────────────
const SPEC_CONTENT: &str = r#"## 1. 型システム（Type System）

### 1.1 基本型

| 型 | 説明 |
|---|---|
| `Int` | 64-bit 符号付き整数 |
| `Float` | 64-bit 浮動小数点数 |
| `Bool` | 真偽値（`true` / `false`） |
| `String` | UTF-8 文字列 |
| `Unit` | 単一値型（副作用のみの式の戻り値） |
| `List<A>` | 同質リスト（共変） |
| `Option<A>` | 省略可能な値（`some(a)` / `none`） |
| `Result<A, E>` | 成功または失敗（`ok(a)` / `err(e)`） |
| `Bytes` | バイト列（v23.1.0） |

### 1.2 型推論規則（Hindley-Milner ベース）

Favnir は Hindley-Milner 型推論を基礎とし、エフェクトアノテーションを拡張する。

```
[Var]  Γ, x : σ ⊢ x : σ
[Abs]  Γ, x : τ₁ ⊢ e : τ₂  →  Γ ⊢ fn(x) { e } : τ₁ → τ₂
[App]  Γ ⊢ f : τ₁ → τ₂, Γ ⊢ a : τ₁  →  Γ ⊢ f(a) : τ₂
[Let]  Γ ⊢ e₁ : σ, Γ, x : σ ⊢ e₂ : τ  →  Γ ⊢ (bind x <- e₁; e₂) : τ
[Gen]  Γ ⊢ e : τ, α ∉ free(Γ)  →  Γ ⊢ e : ∀α. τ
```

---

## 2. opcode 動作仕様

### 2.1 opcode 一覧

| opcode | バイト | オペランド | 説明 |
|---|---|---|---|
| Const | 0x01 | u24(idx) | 定数テーブル[idx] をプッシュ |
| ConstUnit | 0x02 | — | Unit をプッシュ |
| ConstTrue | 0x03 | — | true をプッシュ |
| ConstFalse | 0x04 | — | false をプッシュ |
| LoadLocal | 0x10 | u24(idx) | ローカル変数[idx] をプッシュ |
| StoreLocal | 0x11 | u24(idx) | スタックトップ → ローカル変数[idx] |
| LoadGlobal | 0x12 | u24(idx) | グローバル[idx] をプッシュ |
| Pop | 0x13 | — | スタックトップを破棄 |
| Dup | 0x14 | — | スタックトップを複製 |
| Call | 0x15 | u24(argc) | 関数呼び出し（argc 引数） |
| Return | 0x16 | — | 関数から戻る |
| Add | 0x20 | — | 加算 |
| Sub | 0x21 | — | 減算 |
| Mul | 0x22 | — | 乗算 |
| Div | 0x23 | — | 除算 |
| Eq | 0x24 | — | 等値比較 |
| Ne | 0x25 | — | 非等値比較 |
| Lt | 0x26 | — | 小なり |
| Le | 0x27 | — | 以下 |
| Gt | 0x28 | — | 大なり |
| Ge | 0x29 | — | 以上 |
| And | 0x2A | — | 論理積 |
| Or | 0x2B | — | 論理和 |
| Jump | 0x30 | i24(off) | 無条件ジャンプ（pc += off） |
| JumpIfFalse | 0x31 | i24(off) | false のときジャンプ |
| MatchFail | 0x32 | — | パターンマッチ失敗 |
| GetField | 0x40 | u24(idx) | フィールドアクセス（namespace.field） |
| BuildRecord | 0x41 | u24(n) | レコード構築 |
| MakeClosure | 0x42 | u24(idx) | クロージャ生成 |
| CollectBegin | 0x50 | — | コレクション収集開始 |
| CollectEnd | 0x51 | — | コレクション収集終了 |

### 2.2 オペランドエンコーディング

- `u24(idx)`: 3 バイトリトルエンディアン符号なし整数
- `i24(off)`: 3 バイトリトルエンディアン符号あり整数（符号拡張）
- `—`: オペランドなし（1 バイト命令）

---

## 3. エフェクトシステム（Effect System）

### 3.1 エフェクト一覧

| エフェクト | アノテーション | 説明 |
|---|---|---|
| Pure | （なし） | 副作用なし（決定論的） |
| Io | `!Io` | 標準入出力 |
| Http | `!Http` | HTTP リクエスト |
| Llm | `!Llm` | LLM API（Claude / OpenAI） |
| Db | `!Db` | データベース（汎用） |
| Snowflake | `!Snowflake` | Snowflake DWH |
| File | `!File` | ファイルシステム |
| Trace | `!Trace` | OpenTelemetry トレーシング |

### 3.2 エフェクト意味論

エフェクトアノテーションなし ⇔ 純粋（参照透過）:

```
∀ f : A → B. (エフェクトアノテーションなし) ⇒ f は参照透過
```

エフェクト合成（合成関数のエフェクトは構成要素の和集合）:

```
f : A → B !E₁,  g : B → C !E₂  ⊢  (g ∘ f) : A → C !(E₁ ∪ E₂)
```

---

## 4. パターンマッチ網羅性（Pattern Match Exhaustiveness）

### 4.1 网羅性条件

```
match e { pat₁ => e₁ ... patₙ => eₙ }
条件: ∀ v ∈ type(e). ∃ i. match(patᵢ, v) = true
```

非網羅的なパターンマッチは E0010 としてコンパイルエラー。

### 4.2 パターン種別

| パターン | 例 | マッチ条件 |
|---|---|---|
| リテラル | `0`, `"hi"`, `true` | 等値 |
| 変数 | `x` | 常にマッチ（束縛） |
| ワイルドカード | `_` | 常にマッチ（束縛なし） |
| バリアント | `some(x)`, `ok(v)` | バリアント一致 + ペイロードマッチ |
| Or パターン | `0 \| 1 \| 2` | いずれかにマッチ |
| レコード | `Point { x, y }` | フィールド全マッチ |
"#;

fn build_spec_markdown() -> String {
    let header = format!(
        "# Favnir 言語仕様書\n\nバージョン: {}\n\n---\n\n",
        env!("CARGO_PKG_VERSION")
    );
    format!("{}{}", header, SPEC_CONTENT)
}

pub fn cmd_spec(format: &str) -> String {
    let md = build_spec_markdown();
    match format {
        "html" => md_to_html(&md),
        _ => md,
    }
}
```

> **注意**: `SPEC_CONTENT` は raw string `r#"..."#` のため内部の `{` `}` はそのまま。
> `format!("{}{}", header, SPEC_CONTENT)` で安全に結合できる。SPEC_CONTENT は `format!` の第 2 引数（位置引数）として渡されるため、内部の `{` `}` がフォーマット文字列として再解釈されることはない。
> NG: `format!(r#"...{SPEC_CONTENT}..."#)` のような**フォーマット文字列**として埋め込む場合は干渉する。

- [ ] **事後確認**: `cargo check --bin fav` — エラー 0

---

## T2: `fav/src/main.rs` — `"spec"` サブコマンド追加

`"doc"` アームの直後（`Some("transpile")` の前）に追加する。

```rust
        Some("spec") => {
            // ── v24.1.0: fav spec [--format markdown|html] ───────────────
            let format = if let Some(pos) = args.iter().position(|a| a == "--format") {
                args.get(pos + 1).map(|s| s.as_str()).unwrap_or_else(|| {
                    eprintln!("error: --format requires markdown or html");
                    process::exit(1);
                })
            } else {
                "markdown"
            };
            println!("{}", driver::cmd_spec(format));
        }
```

**挿入位置:** `Some("doc") => { ... }` ブロックの閉じ括弧の直後、`Some("transpile")` の前。

- [ ] **事後確認**: `cargo check --bin fav` — エラー 0

---

## T1 + T2 事後確認

```bash
cargo check --bin fav
# → エラー 0 であること

# 後方互換性確認
cargo test v240000 --bin fav
# → 5/5 PASS（version_is_24_0_0 は削除前なので 5 件）
```

---

## T3: `fav/src/driver.rs` — v241000_tests 追加

### T3-1: `v240000_tests::version_is_24_0_0` を削除（T4-1 より前に必須）

```rust
    #[test]
    fn version_is_24_0_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("version = \"24.0.0\""), "Cargo.toml should have version 24.0.0");
    }
```

この関数ごと削除する。

### T3-2: `v241000_tests` モジュールを `v240000_tests` の直後に追加

```rust
// ── v241000_tests (v24.1.0) — 形式的仕様書生成（fav spec） ──────────────
#[cfg(test)]
mod v241000_tests {
    use super::*;

    #[test]
    fn version_is_24_1_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("version = \"24.1.0\""), "Cargo.toml should have version 24.1.0");
    }

    #[test]
    fn cmd_spec_markdown_has_type_system() {
        let out = cmd_spec("markdown");
        assert!(out.contains("型システム"), "spec markdown should contain 型システム");
    }

    #[test]
    fn cmd_spec_markdown_has_opcodes() {
        let out = cmd_spec("markdown");
        assert!(out.contains("0x01"), "spec markdown should contain opcode 0x01 (Const)");
    }

    #[test]
    fn cmd_spec_html_has_h1() {
        let out = cmd_spec("html");
        assert!(out.contains("<h1>"), "spec html should contain <h1> tag");
    }

    #[test]
    fn changelog_has_v24_1_0() {
        let cl = include_str!("../../CHANGELOG.md");
        assert!(cl.contains("[v24.1.0]"), "CHANGELOG.md should have [v24.1.0] entry");
    }
}
```

```bash
cargo test v241000 --bin fav
# → 5/5 PASS を確認

cargo test --bin fav
# → リグレッションなし（1930 件以上合格）を確認
```

---

## T4: Cargo.toml + CHANGELOG + README + benchmarks

> **注意**: T3-1 の `version_is_24_0_0` 削除完了後に Cargo.toml を更新すること。

### T4-1: `fav/Cargo.toml` バージョン更新

```
version = "24.0.0" → "24.1.0"
```

### T4-2: `CHANGELOG.md` 先頭に v24.1.0 エントリ追加

```markdown
## [v24.1.0] — 2026-06-23 — 形式的仕様書生成（fav spec）

### Added
- `driver::cmd_spec(format: &str) -> String` — Favnir 言語仕様書を Markdown / HTML で生成する公開 API
- `fav spec [--format markdown|html]` CLI サブコマンド — 型システム・opcode・エフェクト・パターンマッチ規則を仕様書として出力

### Notes
- 仕様書は 4 セクション構成: 型システム（HM 推論規則）/ opcode 動作仕様 / エフェクトシステム意味論 / パターンマッチ網羅性
- HTML 変換は既存 `md_to_html`（v21.7.0 実装）を再利用
```

### T4-3: `site/content/docs/tools/spec.mdx` 新規作成

`fav spec` コマンドの説明ページを追加:

```mdx
---
title: fav spec
description: 形式的仕様書生成コマンド
---

# fav spec

Favnir 言語の形式的仕様書を Markdown または HTML 形式で出力します。

## 使い方

```bash
fav spec --format markdown > SPEC.md
fav spec --format html > spec/index.html
```

## オプション

| オプション | 説明 |
|---|---|
| `--format markdown` | Markdown 形式で出力（デフォルト） |
| `--format html` | HTML 形式で出力 |

## 出力セクション

1. **型システム** — 基本型一覧と Hindley-Milner 型推論規則
2. **opcode 動作仕様** — 全 31 opcode のデコード・実行対応表
3. **エフェクトシステム** — エフェクト一覧と純粋性・合成の意味論
4. **パターンマッチ網羅性** — 網羅性条件とパターン種別

> **既知の制限**: HTML 出力のテーブル行は `<table>` ではなく `<p>` として出力されます（`md_to_html` の現状実装）。
```

### T4-4: `README.md` に `fav spec` セクション追加

既存の `fav doc` セクションの直後に追加:

```markdown
### fav spec

```bash
fav spec --format markdown > SPEC.md
fav spec --format html > spec/index.html
```

型システム・opcode 動作仕様・エフェクトシステム・パターンマッチ規則を
Markdown または HTML 形式の仕様書として出力する。
```

### T4-4: `benchmarks/v24.1.0.json` 作成

```json
{
  "version": "24.1.0",
  "date": "2026-06-23",
  "test_count": 0,
  "feature": "形式的仕様書生成（fav spec）",
  "metrics": {
    "spec_sections": 4,
    "opcode_count": 31,
    "new_pub_fns": 1
  }
}
```

> `test_count` は最終 `cargo test --bin fav` 後に実件数で更新。0 のままコミットしないこと。

---

## 実装順序

```
T0（事前確認）
T1（driver.rs: SPEC_CONTENT / build_spec_markdown / cmd_spec 追加）
T2（main.rs: "spec" サブコマンド追加）
cargo check → エラー 0 確認
T3-1（version_is_24_0_0 削除）← T4-1 より前に必須
T3-2（v241000_tests 追加）
cargo test v241000 → 5/5 PASS 確認
T4-1（version 更新）← T3-1 完了後
T4-2〜4（CHANGELOG / README / benchmarks）
cargo test --bin fav → リグレッションなし確認（test_count 更新）
```

---

## リスク対応表

| リスク | 検出方法 | 対応 |
|---|---|---|
| `SPEC_CONTENT` 内の backtick コードブロックが raw string を閉じる | cargo check / `rustc` 構文エラー | raw string delimiter を `r#####"..."#####` に変更（バッククォートと `#` の混在なし） |
| `build_spec_markdown` の `format!` で `{}` が SPEC_CONTENT 内の `{` `}` と干渉 | テスト失敗 | ヘッダのみ `format!` で生成し、SPEC_CONTENT は別変数で管理（format! 引数に渡さない） |
| `cmd_spec("html")` で `md_to_html` が private のためコンパイルエラー | cargo check | `cmd_spec` と `md_to_html` を同一ファイル（driver.rs）に配置する |
| `md_to_html` が `# Favnir 言語仕様書` を `<h1>` に変換しない | テスト `cmd_spec_html_has_h1` 失敗 | `md_to_html` の実装確認（`# ` → `<h1>` 変換が行われていることを確認） |
| `version_is_24_0_0` 削除順序 | 誤った順序で Cargo.toml を更新すると旧テストが失敗 | Cargo.toml 更新（T4-1）前に必ず T3-1 を完了させること |
