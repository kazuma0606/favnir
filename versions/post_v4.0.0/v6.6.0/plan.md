# Favnir v6.6.0 実装計画 — T.validate 完成

作成日: 2026-05-27

---

## 実装順序

```
Phase A (one_of 制約追加)
  → Phase B (TypeName.validate VM dispatch)
    → Phase C (db.query<T> 自動バリデーション)
      → Phase D (統合テスト)
        → Phase E (ドキュメント更新)
```

A → B は依存関係なく並列可能だが、C は B の `validate_record` 共通関数に依存する。

---

## Phase A — `one_of` 制約

### 変更ファイル

1. `fav/src/schemas.rs`
2. `fav/src/backend/vm.rs`

### schemas.rs 変更手順

1. `FieldConstraints` に `pub one_of: Option<Vec<String>>` を追加
2. `Default` 実装に `one_of: None` を追加
3. `load_schemas` の YAML パース部分で `one_of` キーを読む

### vm.rs 変更手順

`Validate.run_raw` の `Some(ref val_str)` ブロック末尾（`pattern` チェックの後）に
`one_of` チェックを追加。

### リスク

- `FieldConstraints::default()` を使っている既存のテストコードが
  コンパイルエラーにならないか確認（`one_of: None` がデフォルトなので問題なし）

---

## Phase B — `TypeName.validate` VM dispatch

### 変更ファイル

- `fav/src/backend/vm.rs`

### 実装手順

1. `call_builtin` 内の `match name` の末尾 fallback（`_ =>` の直前）に動的 dispatch を追加
2. `Validate.run_raw` のロジックを `fn validate_record_inner(type_name, raw, schemas)` に抽出
3. `Validate.run_raw` と `TypeName.validate` dispatch の両方がこの共通関数を呼ぶ
4. checker が期待する `Result<T, List<ValidationError>>` 形式で返す

### checker の確認

`checker.rs:5535` の dispatch:
```rust
(type_name, "validate") if self.schemas.contains_key(type_name) => Some(Type::Result(...))
```
これは `schemas.contains_key(type_name)` が前提条件。
スキーマが存在しない型に `.validate` を呼ぶと checker エラーになる設計で正しい。

### リスク

- `match name` での文字列マッチは O(n)。スキーマ型の数が多い場合は
  `strip_suffix(".validate")` で先に絞る。
- `TypeName.validate` のコンパイラ lowering を確認:
  checker は `(type_name, "validate")` を認識するが、compiler.rs が
  `CallBuiltin("TypeName.validate", [arg])` を発行しているか要確認。

---

## Phase C — `db.query<T>` 自動バリデーション

### 変更ファイル

- `fav/src/backend/vm.rs`

### 対象ビルトイン

| ビルトイン | 箇所 |
|-----------|------|
| `DB.query_raw` | ~l.9906 |
| `DB.query_raw_params` | ~l.9972 |
| `DuckDb.query_raw` | ~l.10528 |

`aws.s3.read_csv_raw` も対象だが、既存の行マッピング構造を確認してから実装する。

### 実装方針

Phase B の `validate_record_inner` 関数を再利用。
各 `query_raw` の **行リスト構築後・return 前** に挿入:

```rust
let schemas = SCHEMA_REGISTRY.with(|s| s.borrow().clone());
if let Some(type_schema) = schemas.get(&inferred_type_name) {
    for row in &rows {
        let errs = validate_row_inner(row, type_schema);
        if !errs.is_empty() {
            return Ok(err_vm(VMValue::List(FavList::new(errs))));
        }
    }
}
```

`inferred_type_name` は `query_raw` に渡された型パラメータ文字列から取得する（既存の仕組みを利用）。

### 注意

自動バリデーションはスキーマが存在する型のみ有効。
スキーマなし型への `db.query<T>` は従来通り動作する（後方互換）。

---

## Phase D — 統合テスト

### 変更ファイル

- `fav/src/backend/vm_stdlib_tests.rs`

### テスト実装の注意

- `set_schema_registry` / `set_schema_registry(HashMap::new())` を各テストの前後で呼ぶ
  （テスト間で registry が汚染されないよう）
- `FieldConstraints` を直接構築してスキーマを設定

---

## Phase E — ドキュメント更新

### 変更ファイル

- `site/content/docs/language/schema.mdx`
- `site/content/docs/stdlib/infer.mdx`

### 変更内容

`schema.mdx` の `T.validate` セクション:
- `> **Note**: T.validate は v6.6.0 で完全実装予定です。` の Note を削除
- `Validate.run_raw` ではなく `TypeName.validate` 構文を使った例に更新

`infer.mdx` の「生成型をコードで使う」セクション:
- スキーマが存在する型を `db.query<T>` に渡すと自動バリデーションされる旨を追記

---

## ファイル変更一覧

| ファイル | 変更種別 |
|---------|---------|
| `fav/src/schemas.rs` | `one_of` フィールド追加・YAML パース追加 |
| `fav/src/backend/vm.rs` | `one_of` チェック追加・`validate_record_inner` 抽出・`TypeName.validate` dispatch 追加・`query_raw` 自動バリデーション追加 |
| `fav/src/backend/vm_stdlib_tests.rs` | 統合テスト 10 件追加 |
| `site/content/docs/language/schema.mdx` | T.validate preview Note 削除・コード例更新 |
| `site/content/docs/stdlib/infer.mdx` | 自動バリデーション説明追記 |

---

## テスト戦略

```bash
# A/B フェーズ後
cargo test validate -- --nocapture

# C フェーズ後
cargo test validate_duckdb -- --nocapture

# 全テスト
cargo test
```

目標: 既存 1033 件 + 新規 10 件以上 = 1043 件以上通過
