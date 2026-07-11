# v36.5.0 実装計画 — Data Contract 規約

## 実装順序

| ステップ | 対象 | 内容 |
|---|---|---|
| S1 | `CHANGELOG.md` | `## [v36.5.0]` エントリを追加（`## [v36.4.0]` の直後） |
| S2 | `fav/src/driver.rs` | `validate_contract_file` / `cmd_contract_check` を追加（`// ── fav validate` セクションの後） |
| S3 | `fav/src/driver.rs` | `create_data_contract_project` 関数を追加（`create_distributed_etl_project` の後） |
| S4 | `fav/src/driver.rs` | `TEMPLATE_GALLERY` に `data-contract` エントリを追加（5 エントリに） |
| S5 | `fav/src/driver.rs` | `try_cmd_new` に `"data-contract"` アームを追加（`other =>` の直前） |
| S6 | `fav/src/main.rs` | `cmd_contract_check` を `use driver::{ ... }` に追加 |
| S7 | `fav/src/main.rs` | `Some("contract") =>` アームを追加（`Some("validate") =>` の直後） |
| S8 | `fav/src/main.rs` | HELP 定数に `contract check` の説明を追加 |
| S9 | `fav/src/driver.rs` | `v36400_tests::cargo_toml_version_is_36_4_0` をスタブ化 |
| S10 | `fav/src/driver.rs` | `v248000_tests::template_gallery_has_4_entries` を 5 エントリ版に更新 |
| S11 | `fav/src/driver.rs` | `v36500_tests` モジュール（5 件）を追加 |
| S12 | `fav/Cargo.toml` | バージョンを `36.4.0` → `36.5.0` に更新（必ず **S2〜S11 すべて完了後**） |
| S13 | `cargo test` | 全通過確認（≥ 2681 件） |

## 各ステップの詳細

### S1: CHANGELOG.md

`## [v36.4.0]` の `---` セパレータの直後に挿入:

```markdown
## [v36.5.0] — 2026-07-08

### Added
- Data Contract 規約 — `contracts/` ディレクトリ規約策定
- `fav new --template data-contract` テンプレート追加
- `fav contract check` コマンド — contracts/ ディレクトリのスキーマ定義を検証
- `cmd_contract_check` / `validate_contract_file` — driver.rs に追加

---
```

### S2: driver.rs — `validate_contract_file` と `cmd_contract_check`

`// ── fav validate (v36.4.0)` セクションの閉じ `}` の後に新しいセクションとして追加。
spec.md §1 のコードを参照。

**配置順**: `cmd_validate` の `}` の後に `// ── fav contract check (v36.5.0) ──` セクションを追加。

### S3: driver.rs — `create_data_contract_project`

`create_distributed_etl_project` 関数の `}` の後に追加する。
spec.md §1 の `create_data_contract_project` コードを参照。

### S4: driver.rs — `TEMPLATE_GALLERY` 更新

```rust
// 変更前（4エントリ）
pub const TEMPLATE_GALLERY: &[(&str, &str)] = &[
    ("etl-csv-to-db",    "CSV → DB ETL パイプライン"),
    ("api-gateway",      "HTTP API ゲートウェイ"),
    ("lambda-scheduled", "スケジュール実行 Lambda ジョブ"),
    ("distributed-etl",  "分散並列 ETL パイプライン"),
];

// 変更後（5エントリ）
pub const TEMPLATE_GALLERY: &[(&str, &str)] = &[
    ("etl-csv-to-db",    "CSV → DB ETL パイプライン"),
    ("api-gateway",      "HTTP API ゲートウェイ"),
    ("lambda-scheduled", "スケジュール実行 Lambda ジョブ"),
    ("distributed-etl",  "分散並列 ETL パイプライン"),
    ("data-contract",    "Data Contract スキーマ定義プロジェクト"),  // v36.5.0
];
```

### S5: driver.rs — `try_cmd_new` アーム追加

`"distributed-etl"` アームの後、`other =>` の直前に追加:

```rust
"data-contract" => create_data_contract_project(&root, name),
```

`other =>` のエラーメッセージも `data-contract` を含む形に更新:

```rust
other => Err(format!(
    "unknown template `{other}` \
     (expected script|pipeline|lib|postgres-etl|\
     etl-csv-to-db|api-gateway|lambda-scheduled|distributed-etl|data-contract)"
)),
```

### S6: main.rs — import 追加

`cmd_contract_check` を既存の import ブロックに追加（`cmd_validate` と同じ行近辺）。

### S7: main.rs — ルーティング追加

`Some("validate") =>` の閉じ `}` の後・`Some("doc") =>` の前に `Some("contract") =>` を追加。
spec.md §2 のコードを参照。

### S8: main.rs — HELP 定数

`validate` コマンド説明の後に追加:
```
    contract check [dir]
                  Check that all .fav files in contracts/ (or [dir]) contain
                  at least one schema definition.
                  Default directory: ./contracts/
```

### S9: driver.rs — スタブ化

`v36400_tests::cargo_toml_version_is_36_4_0` のアサーションを空実装に:

```rust
#[test]
fn cargo_toml_version_is_36_4_0() {
    // stubbed: version bumped to 36.5.0
}
```

### S10: driver.rs — `template_gallery_has_4_entries` 更新

関数名はそのままに、内容を 5 エントリ版に更新:
- `assert_eq!(TEMPLATE_GALLERY.len(), 4, ...)` → `assert_eq!(TEMPLATE_GALLERY.len(), 5, ...)`
- コメント `// v36.5.0 で data-contract を追加したため 5 エントリ` を追加
- `assert!(names.contains(&"data-contract"), "missing data-contract");` を追加

### S11: driver.rs — `v36500_tests` モジュール追加

`v36400_tests` の閉じ `}` の後に追加。spec.md §4 のコードを参照。
5 件構成（ロードマップ最小要件 2 件を上回る）。

### S12: Cargo.toml バージョン更新

**必ず S2〜S11 すべて完了後に実行すること**（コンパイルエラー解消後）。

`version = "36.4.0"` → `version = "36.5.0"`

### S13: cargo test

期待値: T0 で実測した件数 + 5（v36500_tests）= 目標件数 pass、0 failures
（T0 実測値が 2676 の場合: 2676 + 5 = **2681 件**）

## 実装上の重要チェックポイント

### `create_distributed_etl_project` の末尾行番号を確認

`create_data_contract_project` を追加する正確な位置:
```bash
grep -n "fn create_distributed_etl_project\|TEMPLATE_GALLERY" fav/src/driver.rs | head -10
```

### `try_cmd_new` の `other =>` アームの位置を確認

```bash
grep -n "distributed-etl\|other =>" fav/src/driver.rs | head -10
```

### `v248000_tests::template_gallery_has_4_entries` の位置を確認

```bash
grep -n "template_gallery_has_4_entries\|TEMPLATE_GALLERY.len" fav/src/driver.rs | head -5
```

### `Some("validate") =>` の閉じ `}` の位置を確認（main.rs）

```bash
grep -n "Some(\"validate\")\|Some(\"contract\")" fav/src/main.rs | head -5
```

## `fav contract check` の動作フロー

```
fav contract check [./contracts/]
    ↓
[1] contracts/ ディレクトリの存在を確認
    ↓
[2] *.fav ファイルを収集（ソート済み）
    ↓
[3] 各ファイルをパースして schema 定義の存在を確認
      orders.fav: schema Orders { ... } → ok
      users.fav:  fn foo() -> Int { 1 } → ERROR: no `schema` definition found
    ↓
[4] エラーがあれば exit 1、すべて ok なら exit 0
```
