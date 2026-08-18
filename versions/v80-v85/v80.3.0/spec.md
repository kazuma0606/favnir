# Spec: v80.3.0 — `TestFixture` / `DataFactory` モックデータ生成

## Background

v80.1.0 で `TestSuite` 型基盤、v80.2.0 で `GoldenDataset` を追加した。
本バージョンでは型安全なモックデータ生成器を追加する。
`TestFixture` でスキーマとテンプレート行を宣言し、`DataFactory` が指定行数の文字列テーブルを生成する。

ロードマップ: `versions/roadmap/roadmap-v80.1-v81.0.md`（v80.3.0 セクション）

> **テスト数補足**: ロードマップ更新済み（3814 + 2 = 3816）。
> v80.2.0 コードレビュー対応で 1 件追加されたため実際のベースは **3814**。
> 本バージョンの完了条件は **3816**。

## Goals

- `FieldSpec` enum / `RowSpec` 型エイリアス / `TestFixture` 構造体 / `DataFactory` 構造体を `test_framework.rs` に追加する
- `DataFactory::from_seed` / `DataFactory::generate_rows` を実装する
- テスト 2 件を追加して **3816 tests** を達成する

## API / Type Definitions

```rust
// fav/src/test_framework.rs（既存ファイルに追記）

#[derive(Debug, Clone)]
pub enum FieldSpec {
    Str(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Null,
}

/// 1 行分のフィールド仕様。(列名, FieldSpec) のペア列。
pub type RowSpec = Vec<(String, FieldSpec)>;

#[derive(Debug)]
pub struct TestFixture {
    pub name: String,
    /// 列名の順序定義。generate_rows の出力列順と一致する。
    pub schema: Vec<String>,
    /// テンプレート行。generate_rows はこれを循環させて count 行を生成する。
    pub rows: Vec<RowSpec>,
}

#[derive(Debug)]
pub struct DataFactory {
    pub seed: u64,
}

impl DataFactory {
    /// シードを指定して DataFactory を生成する。
    pub fn from_seed(seed: u64) -> DataFactory;

    /// spec に基づいて count 行の文字列テーブルを生成する。
    ///
    /// - spec.rows をインデックス `(row_idx * seed.max(1) + row_idx) % spec.rows.len()` で参照して循環させる。
    ///   seed = 0 の場合は stride = 1 となるため `(2 * row_idx) % spec.rows.len()` で循環する。
    /// - 各 (列名, FieldSpec) を schema の列順で文字列に変換する。
    /// - schema に存在しない列名は無視し、schema の順でのみ出力する。
    /// - FieldSpec の文字列変換: Str(s) → s, Int(n) → n.to_string(), Float(f) → f.to_string(),
    ///   Bool(b) → "true"/"false", Null → ""
    /// - spec.rows が空の場合は空の Vec を返す。
    pub fn generate_rows(&self, spec: &TestFixture, count: usize) -> Vec<Vec<String>>;
}
```

### `generate_rows` の動作例

```rust
let fixture = TestFixture {
    name: "users".to_string(),
    schema: vec!["name".to_string(), "age".to_string()],
    rows: vec![
        vec![("name".to_string(), FieldSpec::Str("alice".to_string())),
             ("age".to_string(),  FieldSpec::Int(30))],
    ],
};
let factory = DataFactory::from_seed(1);
let rows = factory.generate_rows(&fixture, 2);
// rows[0] == ["alice", "30"]
// rows[1] == ["alice", "30"]  ← 1行しかないので循環
```

## Success Criteria

- `cargo test` が **3816 tests**, 0 failures
- `data_factory_generates_rows`: `DataFactory::from_seed(1)` + 2 行生成 → 出力が 2 行で各行の列数が schema と一致
- `test_fixture_schema_matches_rows`: `TestFixture` の schema 列数と generate_rows の各行の列数が一致することを確認

## Files to Modify

| ファイル | 操作 | 内容 |
|---|---|---|
| `fav/src/test_framework.rs` | 追記 | `FieldSpec` / `RowSpec` / `TestFixture` / `DataFactory` + `impl DataFactory` |
| `fav/src/driver.rs` | 追記 | `mod v80300_tests`（テスト 2 件） |

> `lib.rs` への変更は不要（`pub mod test_framework;` は v80.1.0 で宣言済み）。

## Error Codes

新規エラーコードなし。

## 注記

- テストモジュールは `#[cfg(test)] mod v80300_tests { use fav_core::test_framework::*; ... }` パターンを使用する（v80.1.0/v80.2.0 の慣例統一）。
- `FieldSpec` は `Clone` を derive する（テスト内での行コピーに必要）。
- MILESTONE.md / README.md / `site/content/docs/` の更新は v81.0.0 宣言バージョンでまとめて実施する。
