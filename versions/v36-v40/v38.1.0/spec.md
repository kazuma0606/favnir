# v38.1.0 spec — `fav suggest`

## バージョン概要

| 項目 | 内容 |
|---|---|
| バージョン | v38.1.0 |
| テーマ | `fav suggest` — エラーコードから修正案を LLM で生成 |
| 前提 | v38.0.0 COMPLETE — Multi-Source ETL Power マイルストーン宣言済み |
| 完了条件 | `v38100_tests` 全テスト pass・`cargo test` 0 failures（≥ 2744 件） |

## 背景と目的

v38.0 で Multi-Source ETL の基盤が整った。v38.x では「AI がパイプラインを補助する」フェーズに移行する。
v38.1.0 では `fav suggest` コマンドを追加し、エラーコード + ファイル位置から修正案を提示する。

**想定動作**:
```bash
$ fav check main.fav
main.fav:12:5: E0001 undefined variable `custmer_id`

$ fav suggest E0001 main.fav:12
Suggestion: Did you mean `customer_id`? (typo)
Apply fix? [y/N]
```

`ANTHROPIC_API_KEY` が設定されている場合は LLM (Claude) による提案を行う。
未設定の場合は組み込みのヒント（builtin hint）を返す。

## 実装スコープ

### 1. `fav/src/suggest.rs` — 新規作成

```rust
/// v38.1.0 — fav suggest: エラーコードから修正案を生成する

pub fn cmd_suggest(error_code: &str, location: &str) -> Result<(), String> {
    // location: "file.fav:line" 形式
    let source = read_source(location)?;
    let hint = if let Ok(key) = std::env::var("ANTHROPIC_API_KEY") {
        llm_suggest(&key, error_code, &source)
    } else {
        builtin_hint(error_code)
    };
    println!("{}", hint);
    Ok(())
}

fn read_source(location: &str) -> Result<String, String> {
    let path = location.split(':').next().unwrap_or(location);
    std::fs::read_to_string(path)
        .map_err(|e| format!("cannot read {}: {}", path, e))
}

fn builtin_hint(error_code: &str) -> String {
    match error_code {
        "E0001" => "Suggestion: Check for typos in variable names. Use `fav check` to see all defined variables.".to_string(),
        "E0007" => "Suggestion: The function may not be imported. Add the correct `import` statement at the top.".to_string(),
        "E0008" => "Suggestion: Check the number of arguments. Use `fav doc` to see function signatures.".to_string(),
        _ => format!("No built-in suggestion for {}. Set ANTHROPIC_API_KEY for LLM suggestions.", error_code),
    }
}

fn llm_suggest(_api_key: &str, error_code: &str, _source: &str) -> String {
    // v38.7.0 で実際の HTTP 呼び出しに置き換え予定
    // 現時点はスタブ: builtin_hint にフォールバック
    builtin_hint(error_code)
}
```

**エクスポート関数**:
- `pub fn cmd_suggest(error_code: &str, location: &str) -> Result<(), String>`

**テストキーワード**: `pub fn cmd_suggest`

### 2. `fav/src/main.rs` — `mod suggest;` 追加 + `Some("suggest")` アーム追加

#### `mod suggest;` 追加

`mod rune_cmd;`（行 60 付近）の直後に追加:
```rust
mod suggest;
```

#### `Some("suggest")` ディスパッチアーム

`Some("suggest")` アームを `Some("registry")` / `Some("publish")` などと同じブロックに追加する。
`args.get(1)` の `match` ブロック内に:

```rust
Some("suggest") => {
    let error_code = args.get(2).map(|s| s.as_str()).unwrap_or("E0001");
    let location   = args.get(3).map(|s| s.as_str()).unwrap_or("main.fav:1");
    if let Err(e) = suggest::cmd_suggest(error_code, location) {
        eprintln!("fav suggest error: {}", e);
        std::process::exit(1);
    }
}
```

### 3. `driver.rs` — テストモジュール追加

#### `v38000_tests::cargo_toml_version_is_38_0_0` のスタブ化

```rust
// Stubbed: version bumped to 38.1.0 — assertion intentionally removed
```

#### `v38100_tests` モジュール新規追加

```rust
// ── v38100_tests (v38.1.0) — fav suggest ─────────────────────────────────────
#[cfg(test)]
mod v38100_tests {
    // include_str! のみ使用のため imports 不要

    #[test]
    fn cargo_toml_version_is_38_1_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("38.1.0"), "Cargo.toml must contain version 38.1.0");
    }

    #[test]
    fn changelog_has_v38_1_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(src.contains("[v38.1.0]"), "CHANGELOG.md must contain [v38.1.0]");
    }

    #[test]
    fn suggest_fn_exists() {
        // driver.rs と suggest.rs は同じ fav/src/ ディレクトリに置かれるため
        // "suggest.rs" の相対パスで解決される（T2 で fav/src/suggest.rs に作成すること）
        let src = include_str!("suggest.rs");
        assert!(src.contains("pub fn cmd_suggest"), "suggest.rs must contain pub fn cmd_suggest");
    }
}
```

**`include_str!` のみ使用のため `use super::*` / imports 不要。**

### 4. `CHANGELOG.md` — `[v38.1.0]` エントリ追加

`## [v38.0.0]` の `---` セパレータ直後に挿入:

```
## [v38.1.0] — 2026-07-10

### Added
- `fav/src/suggest.rs` — `fav suggest <error-code> <file:line>` コマンド追加
- `builtin_hint`: E0001 / E0007 / E0008 の組み込みヒント
- `ANTHROPIC_API_KEY` 設定時は LLM 提案（v38.7.0 で本実装予定、現在スタブ）
- `v38100_tests` 3 テスト追加

---
```

**セパレータは `—`（全角ダッシュ U+2014）**

### 5. その他ドキュメント更新

- `fav/Cargo.toml`: `38.0.0` → `38.1.0`
- `versions/current.md`: 最新安定版 → v38.1.0、次バージョン → v38.2.0
- `versions/roadmap/roadmap-v38.1-v39.0.md`: v38.1.0 を ✅ 完了済みにマーク・テスト件数を 3 件に更新

## テスト数の計算

| バージョン | 実績 |
|---|---|
| v38.0.0 | 2741 |
| v38.1.0 追加分 | +3 |
| v38.1.0 期待値 | 2744 |

ロードマップは「Rust テスト 2 件」と記載しているが、meta 2 件 + 機能 1 件の計 3 件を追加する。
T8 でロードマップを 3 件に更新する。

## ロードマップとの整合

ロードマップ v38.1.0:
- `fav suggest <error-code> <file>` が動作する
- Rust テスト 2 件（→ 3 件に更新）

## 注意事項

### `suggest.rs` の LLM スタブ

v38.7.0「Llm Rune 強化」で実際の HTTP 呼び出しを実装する。
v38.1.0 では LLM パスも `builtin_hint` にフォールバックするスタブ実装とする。

### `main.rs` dispatch アーム挿入位置

`Some("suggest")` アームは `Some("registry")` / `Some("search")` などがある `match` ブロック内に挿入する。
Read で `Some("registry")` の行番号を確認してから Edit を実行すること。

### `gen` 予約語

Rust 2024 edition では `gen` は予約語。テスト内の変数名に `gen` を使用しないこと
（今回のテストでは `suggest` 系のみのため問題なし）。

## 完了条件

| # | 条件 | 検証方法 |
|---|---|---|
| 1 | `suggest.rs` に `pub fn cmd_suggest` が含まれる | `suggest_fn_exists` テスト |
| 2 | `CHANGELOG.md` に `[v38.1.0]` が含まれる | `changelog_has_v38_1_0` テスト |
| 3 | `Cargo.toml` バージョンが `38.1.0` | `cargo_toml_version_is_38_1_0` テスト |
| 4 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2744） | `cargo test` 実行結果（v38.0.0 実績 2741 + 3 件 = 2744） |
| 5 | `roadmap-v38.1-v39.0.md` の v38.1.0 が ✅ かつテスト件数が 3 件 | T9 後に目視確認 |
