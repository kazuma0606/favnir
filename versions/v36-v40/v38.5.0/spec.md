# v38.5.0 spec — `fav explain --verbose` LLM 拡張

## バージョン概要

| 項目 | 内容 |
|---|---|
| バージョン | v38.5.0 |
| テーマ | `fav explain --verbose` — コンテキスト付き LLM 拡張説明（スタブ実装） |
| 前提 | v38.4.0 COMPLETE — `[lsp.ai]` 設定解析実装済み |
| 完了条件 | `v38500_tests` 全テスト pass・`cargo test` 0 failures（≥ 2764 件） |

## 背景と目的

既存の `fav explain <code>` はエラーコードの概要のみを出力する。
v38.5.0 では `--verbose` フラグを追加し、コンテキスト付きの詳細説明と修正例を出力する。
実際の LLM rerank は v38.7.0 で本実装予定のため、v38.5.0 では**スタブ出力**とする。

**想定動作**:
```bash
$ fav explain --verbose E0001 main.fav:12
E0001: Undefined variable. Check for typos or missing definitions.

Context (main.fav:12): [LLM stub — v38.7.0 で本実装予定]

Fix suggestion: [LLM stub — v38.7.0 で本実装予定]
```

## 実装スコープ

### 1. `fav/src/explain_verbose.rs` — 新規作成

```rust
/// v38.5.0 — fav explain --verbose: コンテキスト付き LLM 拡張説明

pub fn explain_verbose(error_code: &str, location: &str) -> String {
    let base = base_explanation(error_code);
    let context_note = if location.is_empty() {
        String::new()
    } else {
        format!("\n\nContext ({}): [LLM stub — v38.7.0 で本実装予定]", location)
    };
    format!("{}{}\n\nFix suggestion: [LLM stub — v38.7.0 で本実装予定]\n", base, context_note)
}

fn base_explanation(error_code: &str) -> String {
    match error_code {
        "E0001" => "E0001: Undefined variable. Check for typos or missing definitions.".to_string(),
        "E0007" => "E0007: Undefined function. Ensure the function is declared before use.".to_string(),
        "E0008" => "E0008: Wrong number of arguments. Check the function signature.".to_string(),
        _ => format!("{}: No built-in explanation available.", error_code),
    }
}
```

**エクスポート関数**:
- `pub fn explain_verbose(error_code: &str, location: &str) -> String` — verbose 説明を生成

**出力構成**:
| ブロック | 内容 |
|---|---|
| 1行目 | `base_explanation(error_code)` — エラーコード概要 |
| Context 行 | `location` 非空のとき `Context (<location>): [LLM stub]` |
| Fix suggestion 行 | `Fix suggestion: [LLM stub — v38.7.0 で本実装予定]` |

### 2. `fav/src/main.rs` — `pub(crate) mod explain_verbose;` 追加 + `--verbose` 分岐追加

#### `pub(crate) mod explain_verbose;` 追加

`pub(crate) mod generate_csv;` の直後に追加:
```rust
pub(crate) mod generate_csv;
pub(crate) mod explain_verbose;
```

#### `--verbose` 分岐追加

`Some("explain")` アームの `if args.get(2).map(|s| s.as_str()) == Some("compiler")` ブロック（`return;` まで）の**直後**に追加:

```rust
if args.iter().any(|a| a == "--verbose") {
    let error_code = args.iter().skip(2)
        .find(|a| !a.starts_with('-'))
        .map(|s| s.as_str())
        .unwrap_or("E0001");
    let location = args.iter().skip(2)
        .filter(|a| !a.starts_with('-'))
        .nth(1)
        .map(|s| s.as_str())
        .unwrap_or("");
    println!("{}", explain_verbose::explain_verbose(error_code, location));
    return;
}
```

**挿入位置の根拠**: `compiler` チェックは `args[2]` の位置一致で判定するため、`fav explain compiler --verbose` では `compiler` が先にヒットして `cmd_explain_compiler()` が呼ばれる。`--verbose` チェックを `compiler` の**後**に置くことでこの優先順位を保証する。
```

**args 解釈**:
- `fav explain --verbose E0001 main.fav:12` → `error_code = "E0001"`, `location = "main.fav:12"`
- `fav explain --verbose E0007` → `error_code = "E0007"`, `location = ""`（空）

### 3. `driver.rs` — テストモジュール追加

#### `v38400_tests::cargo_toml_version_is_38_4_0` のスタブ化

```rust
// Stubbed: version bumped to 38.5.0 — assertion intentionally removed
```

#### `v38500_tests` モジュール新規追加（4 テスト）

```rust
// ── v38500_tests (v38.5.0) — fav explain --verbose ───────────────────────────
#[cfg(test)]
mod v38500_tests {
    #[test]
    fn cargo_toml_version_is_38_5_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("38.5.0"), "Cargo.toml must contain version 38.5.0");
    }

    #[test]
    fn changelog_has_v38_5_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(src.contains("[v38.5.0]"), "CHANGELOG.md must contain [v38.5.0]");
    }

    #[test]
    fn explain_verbose_basic() {
        let result = crate::explain_verbose::explain_verbose("E0001", "");
        assert!(
            result.contains("E0001") && result.contains("Fix suggestion"),
            "explain_verbose should contain error code and fix suggestion: got {:?}", result
        );
    }

    #[test]
    fn explain_verbose_with_location() {
        let result = crate::explain_verbose::explain_verbose("E0001", "main.fav:12");
        assert!(
            result.contains("Context") && result.contains("main.fav:12"),
            "explain_verbose with location should contain Context block: got {:?}", result
        );
    }
}
```

### 4. `CHANGELOG.md` — `[v38.5.0]` エントリ追加

```
## [v38.5.0] — 2026-07-10

### Added
- `fav/src/explain_verbose.rs` — `fav explain --verbose <code> [location]` コマンド追加
- `explain_verbose`: エラーコード概要 + コンテキスト + Fix suggestion を出力（LLM stub）
- `v38500_tests` 4 テスト追加

---
```

**セパレータは `—`（全角ダッシュ U+2014）**

### 5. その他ドキュメント更新

- `fav/Cargo.toml`: `38.4.0` → `38.5.0`
- `versions/current.md`: 最新安定版 → v38.5.0、次バージョン → v38.6.0
- `versions/roadmap/roadmap-v38.1-v39.0.md`: v38.5.0 を ✅ 完了済みにマーク・テスト件数を 4 件に更新

## テスト数の計算

| バージョン | 実績 |
|---|---|
| v38.4.0 | 2760 |
| v38.5.0 追加分 | +4 |
| v38.5.0 期待値 | 2764 |

ロードマップは「Rust テスト 1 件」と記載しているが、meta 2 件 + functional 2 件の計 4 件を追加し、T7 でロードマップを 4 件に更新する。

## ロードマップとの整合

ロードマップ v38.5.0:
- コンテキスト付き説明と実際のコードに即した修正例を生成する
- Rust テスト 1 件（→ 4 件に更新）

## 注意事項

### `--verbose` フラグの挿入位置

`Some("explain")` アームで `compiler` チェック（`args[2] == "compiler"`）の**直後**に `--verbose` チェックを挿入する。
`fav explain compiler --verbose` のようなコマンドでは `compiler` チェックが先にヒットして `cmd_explain_compiler()` が呼ばれ、`--verbose` は無視される（意図的な優先順位）。

### `location` のパス traversal

v38.1.0 `suggest.rs` と異なり、`explain_verbose.rs` は `location` をファイル読み込みに使わない（スタブのため表示のみ）。
v38.7.0 で実際にファイルを読む際は `..` チェックを追加すること。

### `gen` 予約語（Rust 2024）

`explain_verbose.rs` の変数名には `error_code`・`location`・`base`・`context_note` を使用する — `gen` は使わないこと。

## 完了条件

| # | 条件 | 検証方法 |
|---|---|---|
| 1 | `explain_verbose.rs` に `pub fn explain_verbose` が含まれる | `explain_verbose_basic` テスト |
| 2 | `explain_verbose("E0001", "")` がエラーコードと Fix suggestion を含む文字列を返す | `explain_verbose_basic` テスト |
| 3 | `location` 指定時に Context ブロックが出力に含まれる | `explain_verbose_with_location` テスト |
| 4 | `CHANGELOG.md` に `[v38.5.0]` が含まれる | `changelog_has_v38_5_0` テスト |
| 5 | `Cargo.toml` バージョンが `38.5.0` | `cargo_toml_version_is_38_5_0` テスト |
| 6 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2764） | `cargo test` 実行結果 |
| 7 | `roadmap-v38.1-v39.0.md` の v38.5.0 が ✅ かつテスト件数が 4 件 | T7 後に目視確認 |
| 8 | `versions/current.md` が v38.5.0（最新安定版）に更新されている | T7 後に目視確認 |
