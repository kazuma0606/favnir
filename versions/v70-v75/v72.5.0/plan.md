# v72.5.0 実装プラン — Playground 2.0

Date: 2026-08-12

---

## 依存関係

```
Step 1 (PlaygroundTemplate 構造体 + PLAYGROUND_TEMPLATES)
  └─ Step 2 (playground_share_url)
       └─ Step 3 (v725000_tests)
            └─ Step 4 (バージョン更新)
                 └─ Step 5 (テスト確認)
                      └─ Step 6 (ドキュメント更新)
```

---

## Step 1: `driver.rs` — `PlaygroundTemplate` + `PLAYGROUND_TEMPLATES` 追加

対象: `fav/src/driver.rs`（v72.4.0 セクションの直後・v72.0.0 テストの前）

### 1-1. `PlaygroundTemplate` 構造体を追加

```rust
pub struct PlaygroundTemplate {
    pub name: &'static str,
    pub description: &'static str,
    pub code: &'static str,
}
```

### 1-2. `PLAYGROUND_TEMPLATES` を 5 エントリで定義

```rust
pub static PLAYGROUND_TEMPLATES: &[PlaygroundTemplate] = &[
    PlaygroundTemplate {
        name: "Hello World",
        description: "最小の Favnir プログラム",
        code: "fn main(ctx: AppCtx) -> Result<Unit, String> {\n    ctx.io.println(\"Hello, Favnir!\")\n}\n",
    },
    PlaygroundTemplate {
        name: "CSV ETL",
        description: "CSV 読み込み → スキーマ検証 → 変換",
        code: "import rune \"csv\"\n\nschema Row {\n    id: String\n    value: Float\n}\n\nfn main(ctx: AppCtx) -> Result<Unit, String> {\n    bind raw  <- ctx.io.read_file_raw(\"data.csv\")\n    bind rows <- Csv.parse_typed(raw, Row)\n    ctx.io.println(\"Loaded rows.\")\n}\n",
    },
    PlaygroundTemplate {
        name: "AI Generate",
        description: "fav ai generate で生成したパイプラインのサンプル",
        code: "import rune \"csv\"\nimport rune \"postgres\"\n\nschema OrderRow {\n    order_id: String\n    amount:   Float\n    status:   String\n}\n\nfn main(ctx: AppCtx) -> Result<Unit, String> {\n    ctx.io.println(\"AI generated pipeline\")\n}\n",
    },
    PlaygroundTemplate {
        name: "Distributed Par",
        description: "par 並列ステージのデモ",
        code: "fn main(ctx: AppCtx) -> Result<Unit, String> {\n    ctx.io.println(\"par pipeline demo\")\n}\n",
    },
    PlaygroundTemplate {
        name: "Data Quality",
        description: "Schema.validate_all を使ったデータ品質パイプライン",
        code: "import rune \"csv\"\n\nschema QualityRow {\n    id: String\n    score: Float\n}\n\nfn main(ctx: AppCtx) -> Result<Unit, String> {\n    ctx.io.println(\"Data quality check passed.\")\n}\n",
    },
];
```

### 1-3. `cargo build` で確認

---

## Step 2: `playground_share_url` 追加

```rust
pub fn playground_share_url(code: &str) -> String {
    // URL-safe hex エンコード（base64 crate 非追加）
    let encoded: String = code.as_bytes()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    format!("/playground?code={encoded}")
}
```

- `cargo build` で確認

---

## Step 3: `v725000_tests` 追加（`driver.rs`）

`v724000_tests` の直後に追加:

```rust
#[cfg(test)]
mod v725000_tests {
    use super::{playground_share_url, PLAYGROUND_TEMPLATES};

    #[test]
    fn playground2_template_gallery_has_5_entries() {
        assert!(
            PLAYGROUND_TEMPLATES.len() >= 5,
            "PLAYGROUND_TEMPLATES should have at least 5 entries, got {}",
            PLAYGROUND_TEMPLATES.len()
        );
    }

    #[test]
    fn playground2_share_url_format() {
        let url = playground_share_url("fn main() -> Unit { }");
        assert!(url.starts_with("/playground?code="), "share URL should start with /playground?code=");
        assert!(url.len() > "/playground?code=".len(), "share URL should have non-empty code part");
    }
}
```

- `cargo test v725000` で 2 件 pass することを確認（早期フィードバック）

---

## Step 4: バージョン更新

- `fav/Cargo.toml`: `version = "72.4.0"` → `version = "72.5.0"`
- `driver.rs` 内 `version = \"72.4.0\"` → `version = \"72.5.0\"`（replace_all）
- `driver.rs` 内 `"Cargo.toml version should be 72.4.0"` → `"72.5.0"`（replace_all）
- `driver.rs` 内 `"Cargo.toml should declare version 72.4.0"` → `"72.5.0"`（replace_all）
- T0 で記録した件数と置換後 grep 件数が一致することを確認する

---

## Step 5: テスト確認

- `cargo test v725000` → 2 件 pass
- `cargo test` 全体 → 3627 tests pass（0 failures）

---

## Step 6: ドキュメント更新

- `CHANGELOG.md`: `## [v72.5.0]` エントリを先頭に追加
- `versions/current.md`: 進行中バージョンを `v72.5.0`、次に切る版を `v72.6.0` に更新
