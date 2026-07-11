# v36.7.0 実装計画 — Great Expectations 互換エクスポート

## 変更ファイル一覧

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `fav/src/driver.rs` | 追記・変更 | `export_ge_suite` 追加 / `cmd_validate` シグネチャ変更 / `v36600_tests` スタブ化 / `v36700_tests` 追加 |
| `fav/src/main.rs` | 変更 | `--export` / `--output` フラグ解析追加、`cmd_validate` 呼び出し更新 |
| `fav/Cargo.toml` | 更新 | `version = "36.6.0"` → `"36.7.0"` |
| `CHANGELOG.md` | 追記 | `[v36.7.0]` エントリ追加 |
| `versions/current.md` | 更新 | 最新安定版 v36.7.0、次バージョン v36.8.0 |
| `versions/roadmap/roadmap-v36.1-v37.0.md` | 更新 | v36.7.0 完了済みにマーク（✅）（ロードマップ記載のテスト最小値 1 件を超える 3 件を実装。ロードマップ側の件数は更新不要） |

## 実装順序

### Step 1: CHANGELOG.md に [v36.7.0] エントリ追加

`## [v36.6.0]` の `---` セパレータ直後に挿入:

```markdown
## [v36.7.0] — 2026-07-08

### Added
- `export_ge_suite(schema_name, field_names)` — Great Expectations 0.18.0 互換 Expectation Suite JSON 生成
- `fav validate --export ge --output suite.json` — GE 互換エクスポートフラグ追加

---
```

### Step 2: driver.rs — `export_ge_suite` 追加

`cmd_validate` の直前（`// ── fav validate (v36.4.0)` セクションの冒頭付近、`validate_schema_against_headers` の後）に追加。
純粋関数として実装（ファイル I/O なし）。spec.md の Rust コードスニペットをそのまま使用する。

### Step 3: driver.rs — `cmd_validate` シグネチャ変更

`pub fn cmd_validate(schema_file: Option<&str>, data_file: Option<&str>)` に
`export_fmt: Option<&str>` と `output_file: Option<&str>` を追加する。

`if !has_errors` ブロックの後にエクスポートロジックを追加:

```rust
if !has_errors {
    if export_fmt == Some("ge") {
        let out_path = output_file.unwrap_or("suite.json");
        if let Some(sd) = schema_defs.first() {
            let field_names: Vec<String> = sd.fields.iter().map(|(n, _)| n.clone()).collect();
            let json = export_ge_suite(&sd.name, &field_names);
            // write_text_file: fn(path: &Path, contents: &str) -> Result<(), String>
            write_text_file(std::path::Path::new(out_path), &json)
                .unwrap_or_else(|e| eprintln!("error writing {}: {}", out_path, e));
            println!("exported GE suite to {}", out_path);
        }
    }
}
```

> **注意**: `write_text_file` のシグネチャは `fn write_text_file(path: &Path, contents: &str) -> Result<(), String>`。
> `&str` → `Path::new()` 変換と `Result` 処理（`.unwrap_or_else`）が必要。

### Step 4: main.rs — `--export` / `--output` フラグ追加

`Some("validate") =>` アームのフラグ解析ループに `"--export"` / `"--output"` アームを追加。

変数宣言:
```rust
let mut export_fmt: Option<String> = None;
let mut output_file: Option<String> = None;
```

`cmd_validate` 呼び出しを更新:
```rust
cmd_validate(
    schema_file.as_deref(),
    data_file.as_deref(),
    export_fmt.as_deref(),
    output_file.as_deref(),
);
```

### Step 5: driver.rs — `v36600_tests::cargo_toml_version_is_36_6_0` スタブ化

ライブアサーション → `// Stubbed: version bumped to 36.7.0` に変更。

### Step 6: driver.rs — `v36700_tests` モジュール追加

`v36600_tests` の閉じ `}` の行番号を Read で特定してから Edit を実行。
`use super::export_ge_suite;` インポートで参照する（driver.rs 内 pub fn のため `super::` が正しいパターン）。

### Step 7: Cargo.toml バージョン更新

Step 2〜6 完了・コンパイルエラー解消後に `36.6.0` → `36.7.0` に更新。

## 依存関係

- `write_text_file` ヘルパーが driver.rs に既存であることを確認（T0 で確認）
- `cmd_validate` 呼び出し箇所が `main.rs` のみであることを確認（T0 で確認）
- `v36600_tests::cargo_toml_version_is_36_6_0` がライブアサーションであることを確認（T0 で確認）

## リスク

| リスク | 対処 |
|---|---|
| `cmd_validate` の呼び出し箇所が main.rs 以外にある | T0 で Grep により確認 |
| `write_text_file` が存在しない | T0 で確認、なければ `std::fs::write` を直接使用 |
| JSON のエスケープ不備（フィールド名に `"` が含まれる場合） | spec スコープ外（英数字フィールド名のみ対象とする） |
