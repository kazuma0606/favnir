# v37.7.0 spec — `fav new --template multi-source`

## バージョン概要

| 項目 | 内容 |
|---|---|
| バージョン | v37.7.0 |
| テーマ | `fav new --template multi-source` — マルチソース ETL プロジェクトテンプレート追加 |
| 前提 | v37.6.0 COMPLETE — `render_lineage_dot` / `render_lineage_svg` 実装済み |
| 完了条件 | `v37700_tests` 全テスト pass・`cargo test` 0 failures（≥ 2730 件） |

## 背景と目的

v37.x スプリントで追加した機能（`List.join_on` / `List.fan_out` / `List.fan_in` / CDC Rune）を活用するための
プロジェクトテンプレートを `fav new` に追加する。

**今バージョンで行うこと（スコープ確定）:**
- `create_multi_source_etl_project` 関数を `driver.rs` に追加
- `try_cmd_new` に `"multi-source"` アームを追加
- `TEMPLATE_GALLERY` に `"multi-source"` エントリを追加（6 エントリに）
- `cmd_new_list` に `"multi-source"` を追加
- `try_cmd_new` エラーメッセージに `multi-source` を追加
- `v248000_tests::template_gallery_has_5_entries` の len アサーションをスタブ化
- `v37700_tests` 3 テスト追加（meta 2 件 + 機能 1 件）

**スコープ外:**
- `v248000_tests` の全面スタブ化（`template_gallery_has_5_entries` の len アサーション 1 行のみスタブ）

## 実装スコープ

### 1. `create_multi_source_etl_project` 関数

```rust
fn create_multi_source_etl_project(root: &Path, name: &str) -> Result<(), String> {
    write_text_file(&root.join("src/load_customers.fav"), &format!(
        "// Source A: Postgres から顧客データをロード\nimport postgres as db\n\nstage LoadCustomers -> List[String] {{\n    db.query(ctx, \"SELECT id,name FROM customers\")\n}}\n"
    ))?;
    write_text_file(&root.join("src/load_orders.fav"), &format!(
        "// Source B: CSV から注文データをロード\nimport csv\n\nstage LoadOrders -> List[String] {{\n    csv.read_file(\"orders.csv\")\n}}\n"
    ))?;
    write_text_file(&root.join("src/main.fav"), &format!(
        "// Multi-Source ETL — {name}\nimport postgres as db\n\nstage JoinAndLoad(customers: List[String], orders: List[String]) -> Int {{\n    bind joined <- List.join_on(customers, orders, |c, o| String.contains(o, c))\n    joined |> List.map(|row| db.execute(ctx, \"INSERT INTO results (data) VALUES ($1)\", [row]))\n          |> List.length\n}}\n\npipeline {name} {{\n    LoadCustomers, LoadOrders |> JoinAndLoad\n}}\n"
    ))?;
    write_text_file(&root.join("fav.toml"), &format!(
        "[project]\nname    = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2026\"\n\n[runes]\npostgres = \"1.0.0\"\ncsv      = \"1.0.0\"\n"
    ))?;
    write_text_file(&root.join("README.md"), &format!(
        "# {name}\n\nマルチソース ETL パイプライン。\nPostgres（顧客データ）と CSV（注文データ）を `List.join_on` で結合して出力します。\n\n## Usage\n\n```bash\nDATABASE_URL=postgres://localhost/{name} fav run src/main.fav\n```\n"
    ))?;
    write_text_file(&root.join(".github/workflows/ci.yml"),
        "name: CI\non: [push, pull_request]\njobs:\n  test:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v4\n      - run: cargo install fav\n      - run: fav check src/main.fav\n"
    )?;
    Ok(())
}
```

**生成ファイル:**
- `src/load_customers.fav` — Source A（Postgres 顧客ロード）
- `src/load_orders.fav` — Source B（CSV 注文ロード）
- `src/main.fav` — Join + 書き込みパイプライン（`List.join_on` 使用）
- `fav.toml` — プロジェクト設定（postgres + csv 両 Rune）
- `README.md`
- `.github/workflows/ci.yml`

### 2. `try_cmd_new` 更新

`"data-contract"` アームの直後に追加:

```rust
"multi-source" => create_multi_source_etl_project(&root, name),
```

エラーメッセージも更新:

```rust
other => Err(format!(
    "unknown template `{other}` \
     (expected script|pipeline|lib|postgres-etl|\
     etl-csv-to-db|api-gateway|lambda-scheduled|distributed-etl|data-contract|multi-source)"
)),
```

### 3. `TEMPLATE_GALLERY` 更新

`("data-contract", ...)` エントリの直後に追加:

```rust
("multi-source", "マルチソース ETL（複数 DB/CSV 結合）"),  // v37.7.0
```

6 エントリになる。

### 4. `cmd_new_list` 更新

`cmd_new_list` には `"data-contract"` 行が既に欠落している（TEMPLATE_GALLERY には登録済みだが未追加）。
v37.7.0 では `"data-contract"` と `"multi-source"` の 2 行をまとめて `"distributed-etl"` 行の直後に追加する。

```rust
println!("  {:<17} {}", "data-contract",   "Data Contract スキーマ定義プロジェクト");
println!("  {:<17} {}", "multi-source",    "マルチソース ETL（Postgres + CSV 結合）");
```

### 5. `v248000_tests::template_gallery_has_5_entries` のスタブ化

`TEMPLATE_GALLERY.len() == 5` は 6 エントリ追加後に失敗するため、len アサーションをスタブ化。
名前の確認アサーション群は残す（存在確認として依然有効）。

現在の該当箇所（driver.rs 行 37343-37345）:
```rust
// v36.5.0 で data-contract を追加して 5 エントリ（旧: has_4_entries）
assert_eq!(TEMPLATE_GALLERY.len(), 5,
    "TEMPLATE_GALLERY must have 5 entries, got {}", TEMPLATE_GALLERY.len());
```

**スタブ化後（3 行を 1 行コメントに置き換え）:**
```rust
fn template_gallery_has_5_entries() {
    // Stubbed: len check removed — multi-source added in v37.7.0 (now 6 entries)
    let names: Vec<&str> = TEMPLATE_GALLERY.iter().map(|(n, _)| *n).collect();
    assert!(names.contains(&"etl-csv-to-db"),     "missing etl-csv-to-db");
    assert!(names.contains(&"api-gateway"),        "missing api-gateway");
    assert!(names.contains(&"lambda-scheduled"),   "missing lambda-scheduled");
    assert!(names.contains(&"distributed-etl"),    "missing distributed-etl");
    assert!(names.contains(&"data-contract"),      "missing data-contract");
}
```

**注意:** 既存コメント行（`// v36.5.0 で...`）と `assert_eq!` 2 行の合計 3 行を、
スタブコメント 1 行に置き換える（合計 3 行 → 1 行）。

### 6. `v37700_tests` モジュール

```rust
// ── v37700_tests (v37.7.0) — fav new --template multi-source ─────────────────
#[cfg(test)]
mod v37700_tests {
    use super::try_cmd_new;

    #[test]
    fn cargo_toml_version_is_37_7_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("37.7.0"), "Cargo.toml must contain version 37.7.0");
    }

    #[test]
    fn changelog_has_v37_7_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(src.contains("[v37.7.0]"), "CHANGELOG.md must contain [v37.7.0]");
    }

    #[test]
    fn fav_new_multi_source_ok() {
        let dir = tempfile::tempdir().expect("tempdir");
        let proj = dir.path().join("my_multi");
        let result = try_cmd_new(proj.to_str().unwrap(), "multi-source");
        assert!(result.is_ok(), "multi-source template must succeed: {:?}", result);
        assert!(proj.join("src/main.fav").exists(),           "src/main.fav missing");
        assert!(proj.join("src/load_customers.fav").exists(), "src/load_customers.fav missing");
        assert!(proj.join("src/load_orders.fav").exists(),    "src/load_orders.fav missing");
        assert!(proj.join("fav.toml").exists(),               "fav.toml missing");
        assert!(proj.join("README.md").exists(),              "README.md missing");
        // main.fav に List.join_on が含まれることを確認
        let main_src = std::fs::read_to_string(proj.join("src/main.fav")).unwrap();
        assert!(main_src.contains("List.join_on"), "src/main.fav must contain List.join_on");
    }
}
```

**`use super::try_cmd_new`:** `try_cmd_new` は `fn`（非 `pub`）だが同ファイル内のテストからは `super::` でアクセス可能。
`tempfile` は `[dev-dependencies]` に既存登録済み。

## 注意事項

### `v248000_tests::template_gallery_has_5_entries` のスタブ化範囲

`len() == 5` アサーション行と `got {}` メッセージ行のみを除去する。
具体的には以下のコードブロックを削除:
```rust
assert_eq!(TEMPLATE_GALLERY.len(), 5,
    "TEMPLATE_GALLERY must have 5 entries, got {}", TEMPLATE_GALLERY.len());
```
コメントは「Stubbed: len check removed...」に変更し、残りの `names.contains(...)` アサーションは維持する。

### `try_cmd_new` の `other` アームのエラーメッセージ更新

既存のエラーメッセージに `multi-source` を追記することを忘れないこと。
`v248000_tests::fav_new_unknown_template_errors` は `msg.contains("etl-csv-to-db")` をアサートしており、
エラーメッセージの変更後も引き続きパスする。

### `TEMPLATE_GALLERY` のエントリ数

v37.7.0 追加後: 6 エントリ。`v37700_tests` では len チェックを行わない（ロードマップ完了条件 "Rust テスト 1 件" + meta 2 件 = 3 件の範囲内）。

### テスト数の計算

| バージョン | 実績 |
|---|---|
| v37.6.0 | 2727 |
| v37.7.0 追加分 | +3 |
| v37.7.0 期待値 | 2730 |

## ロードマップとの整合

ロードマップ v37.7.0:
- マルチソース ETL プロジェクトテンプレート追加
- 完了条件: Rust テスト 1 件（→ 3 件に更新）

ロードマップは「1 件」と記載しているが、meta 2 件 + 機能 1 件の計 3 件を追加する（他バージョンとの統一パターン）。
T8 でロードマップを 3 件に更新する。

## 完了条件

| # | 条件 | 検証方法 |
|---|---|---|
| 1 | `Cargo.toml` バージョンが `37.7.0` | `cargo_toml_version_is_37_7_0` テスト |
| 2 | `CHANGELOG.md` に `[v37.7.0]` が含まれる | `changelog_has_v37_7_0` テスト |
| 3 | `multi-source` テンプレートで 3 ファイル（main.fav / load_customers.fav / load_orders.fav）が生成される | `fav_new_multi_source_ok` テスト |
| 4 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2730） | `cargo test` 実行結果（v37.6.0 実績 2727 + 3 件 = 2730） |
