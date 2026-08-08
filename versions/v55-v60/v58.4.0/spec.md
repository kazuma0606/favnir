# Spec — v58.4.0 — Data Catalog 統合（`fav catalog`）

## 概要

`fav catalog push` と `fav catalog search` コマンドを追加し、
データカタログ（DataHub / Apache Atlas）へのパイプラインメタデータ登録と検索を提供する。
`cmd_catalog_push` / `cmd_catalog_search` 関数を driver.rs に実装し、
`Some("catalog")` arm を main.rs に追加する。
Rust テスト 2 件（`cmd_catalog_push` / `cmd_catalog_search`）で動作を検証する。

---

## ロードマップ参照

- `versions/roadmap/roadmap-v58.1-v59.0.md` — v58.4.0 セクション
- ベーステスト数: **3286**（v58.3.0 完了時点の実績値、code-review 対応込み）
- 目標テスト数: **3288**（+2）、かつ `cargo test` failures=0
- rolling チェック更新: **5 件**（v56300 / v56900 / v57000 / v57900 / v58000）を `58.3.0` → `58.4.0`

---

## スコープ外項目

| 項目 | 備考 |
|---|---|
| `!Catalog` エフェクトの AST/IR 追加 | v58.x パターン（driver.rs スタブ）に合わせて見送り。将来版で実装 |
| DataHub / Atlas への実 HTTP 通信 | 出力文字列モックで検証 |
| `--catalog` URL のバリデーション | フラグ値欠落のみ検証（スキームチェック等は将来）。`--catalog ""` のような空文字列値はバリデーション対象外（空文字列のまま関数に渡る） |
| パイプラインメタデータの実解析 | ロードマップ例示の固定文字列を出力するスタブ |

---

## 実装内容

### 1. `fav/Cargo.toml` バージョン更新

```toml
version = "58.4.0"
```

### 2. `fav/src/driver.rs` — `cmd_catalog_push` 追加

`fav catalog push --catalog <url>` を処理する関数。
ロードマップ例示の出力フォーマットに準拠する。

```rust
pub fn cmd_catalog_push(catalog_url: &str) -> i32 {
    println!("Registering pipeline: OrderIngestion");
    println!("  stage Parse:    RawOrder → Order");
    println!("  stage Validate: Order → Result<ValidOrder>");
    println!("  stage Store:    ValidOrder → Unit  (Snowflake: orders_v2)");
    println!("Catalog push: OK  [catalog: {}]", catalog_url);
    0
}
```

`catalog_url` はロードマップ出力例が `Snowflake: orders_v2` を固定文字列として示すため、
データ宛先ではなく登録先カタログ URL を `Catalog push: OK` 行に表示する形式とする。

`cmd_schema_migrate`（v58.3.0 追加）の直後に追加する。

### 3. `fav/src/driver.rs` — `cmd_catalog_search` 追加

`fav catalog search <query>` を処理する関数。
ロードマップ例示の出力フォーマットに準拠する。

```rust
pub fn cmd_catalog_search(query: &str) -> i32 {
    println!("Catalog search: \"{}\"", query);
    println!("OrderIngestion  pipeline  last_run: 2026-07-23T10:00:00Z");
    0
}
```

`cmd_catalog_push` の直後に追加する。

### 4. `fav/src/main.rs` — `Some("catalog")` arm 追加

`Some("schema")` arm の後（`Some("doc")` arm の前）に追加する。

```rust
Some("catalog") => {
    match args.get(2).map(|s| s.as_str()) {
        Some("push") => {
            let has_catalog = args.iter().any(|a| a == "--catalog");
            let catalog_url = args.windows(2).find(|w| w[0] == "--catalog")
                .map(|w| w[1].as_str());
            if has_catalog && catalog_url.is_none() {
                eprintln!("error: --catalog requires a value (e.g., --catalog datahub://localhost:8080)");
                process::exit(1);
            }
            std::process::exit(driver::cmd_catalog_push(
                catalog_url.unwrap_or("datahub://localhost:8080"),
            ));
        }
        Some("search") => {
            let query = args.get(3).map(|s| s.as_str()).unwrap_or("");
            std::process::exit(driver::cmd_catalog_search(query));
        }
        sub => {
            eprintln!(
                "error: unknown subcommand `{}` for `fav catalog`\n  usage: fav catalog push --catalog <url>\n         fav catalog search <query>",
                sub.unwrap_or("(none)")
            );
            process::exit(1);
        }
    }
}
```

`cmd_catalog_push` / `cmd_catalog_search` は `driver::` プレフィックス経由でアクセスするため、
use インポートへの追加は不要（他の driver 関数と同様）。

### 5. `fav/src/driver.rs` — `v58400_tests` 追加

`v58300_tests` の直前に挿入する。

```rust
// -- v58400_tests (v58.4.0) -- Data Catalog 統合 --
#[cfg(test)]
mod v58400_tests {
    use super::{cmd_catalog_push, cmd_catalog_search};

    #[test]
    fn cmd_catalog_push_test() {
        let code = cmd_catalog_push("datahub://localhost:8080");
        assert_eq!(code, 0, "catalog push should succeed");
    }

    #[test]
    fn cmd_catalog_search_test() {
        let code = cmd_catalog_search("order");
        assert_eq!(code, 0, "catalog search should succeed");
    }
}
```

**注意**: ロードマップのテスト名は `cmd_catalog_push` / `cmd_catalog_search` だが、
関数名と衝突するため `cmd_catalog_push_test` / `cmd_catalog_search_test` とする
（v58.3.0 の `cmd_schema_migrate_test` と同じ慣例）。

### 6. Rolling バージョンチェック更新（5 件）

```
v56300_tests  : "58.3.0" → "58.4.0"
v56900_tests  : "58.3.0" → "58.4.0"
v57000_tests  : "58.3.0" → "58.4.0"
v57900_tests  : "58.3.0" → "58.4.0"
v58000_tests  : "58.3.0" → "58.4.0"
```

---

## テスト仕様

| テスト名 | 検証内容 |
|---|---|
| `cmd_catalog_push_test` | `cmd_catalog_push("datahub://localhost:8080")` が 0 を返す |
| `cmd_catalog_search_test` | `cmd_catalog_search("order")` が 0 を返す |

---

## 完了条件

- `cargo build` コンパイルエラーなし
- `cargo test` 全通過（**3288 tests passed, 0 failed**、ベース 3286 + 2）
- `cargo clippy -- -D warnings` クリーン
- `v58400_tests` 2 件全 pass
- `fav catalog push` / `fav catalog search` が `Some("catalog")` arm で処理される
- rolling チェック 5 件が `"58.4.0"` になっている

---

## 備考

- `cmd_catalog_push` / `cmd_catalog_search` は関数名との衝突を避けるため `_test` サフィックスを付ける（v58.3.0 慣例）
- `Some("catalog")` arm は driver:: プレフィックス経由のため use インポート不要
- `!Catalog` エフェクトの AST/IR 統合はスコープ外（v58.x パターン踏襲）
- rolling チェックは全バージョンで 5 件全件更新が必要（v58.1〜v58.3 で確認済みパターン）
