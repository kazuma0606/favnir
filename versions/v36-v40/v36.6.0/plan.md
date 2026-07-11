# v36.6.0 実装計画 — E0380〜E0384 スキーマ不整合エラーコード

## 変更ファイル一覧

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `fav/src/error_catalog.rs` | 追記 | E0380〜E0384 エントリを `ERROR_CATALOG` 末尾に追加 |
| `fav/src/driver.rs` | 追記 | `v36500_tests::cargo_toml_version_is_36_5_0` スタブ化 + `v36600_tests` モジュール追加 |
| `fav/Cargo.toml` | 更新 | `version = "36.5.0"` → `"36.6.0"` |
| `CHANGELOG.md` | 追記 | `[v36.6.0]` エントリ追加 |
| `versions/current.md` | 更新 | 最新安定版 v36.6.0、次バージョン v36.7.0 |
| `versions/roadmap/roadmap-v36.1-v37.0.md` | 更新 | v36.6.0 完了済みにマーク |

## 実装順序

### Step 1: CHANGELOG.md に [v36.6.0] エントリ追加

`## [v36.5.0]` の `---` セパレータ直後に挿入:

```markdown
## [v36.6.0] — 2026-07-XX

### Added
- `error_catalog.rs` に E0380〜E0384（スキーマ不整合エラーコード）を追加
  - `E0380` `schema_field_missing`: 必須フィールドがデータに存在しない
  - `E0381` `schema_type_mismatch`: フィールド型がデータ値と一致しない
  - `E0382` `schema_constraint_violated`: `where` 制約をデータ値が満たさない
  - `E0383` `schema_duplicate_key`: スキーマ定義にフィールド名が重複している
  - `E0384` `schema_extra_field`: データにスキーマ未定義のフィールドが含まれている

---
```

### Step 2: error_catalog.rs — E0380〜E0384 エントリ追加

`ERROR_CATALOG` 末尾エントリ（`E0903`）の `},` の後、配列の閉じ `];` の前に追加:

```rust
// ── E038x: スキーマ不整合 (v36.6.0) ────────────────────────────────────
ErrorEntry { code: "E0380", title: "schema_field_missing", category: "schema", ... },
ErrorEntry { code: "E0381", title: "schema_type_mismatch", category: "schema", ... },
ErrorEntry { code: "E0382", title: "schema_constraint_violated", category: "schema", ... },
ErrorEntry { code: "E0383", title: "schema_duplicate_key", category: "schema", ... },
ErrorEntry { code: "E0384", title: "schema_extra_field", category: "schema", ... },
```

全フィールド（code / title / category / description / example / fix）を埋める。
spec.md の Rust コードスニペットをそのまま使用する。

### Step 3: driver.rs — スタブ化と v36600_tests 追加

1. `v36500_tests::cargo_toml_version_is_36_5_0` のライブアサーション → スタブコメントに変更
2. `v36500_tests` の `}` の後に `v36600_tests` モジュールを追加（spec.md のコードスニペットどおり）

テストインポート:
```rust
use crate::error_catalog::{lookup, ERROR_CATALOG};
```

### Step 4: Cargo.toml バージョン更新

```toml
version = "36.5.0"
↓
version = "36.6.0"
```

（Step 2〜3 完了・コンパイルエラー解消後に実施）

### Step 5: ドキュメント更新

- `versions/current.md`: 最新安定版 → v36.6.0、次バージョン → v36.7.0
- `versions/roadmap/roadmap-v36.1-v37.0.md`: v36.6.0 を ✅ にマーク
- `versions/v36-v40/v36.6.0/tasks.md`: COMPLETE に更新

## 依存関係

- `error_catalog.rs` の `ErrorEntry` struct と `lookup` 関数が既存であることを確認（v35.x 以前に実装済み）
- `ERROR_CATALOG` が `pub static` または `pub const` で定義されていることを確認
- driver.rs の `error_catalog` import が存在することを確認（テストで `use crate::error_catalog::*` を利用）

## リスク

| リスク | 対処 |
|---|---|
| `ErrorEntry` の struct フィールド順が変わっている | `error_catalog.rs` を実装前に Read して確認する |
| `ERROR_CATALOG` の閉じ `];` の直前に別のエントリが追加されている | 実装前に末尾を確認してから Edit する |
| `lookup` 関数が存在しない | spec-reviewer・実装前に確認 |
