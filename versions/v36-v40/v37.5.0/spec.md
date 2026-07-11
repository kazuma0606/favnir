# v37.5.0 spec — CDC Rune

## バージョン概要

| 項目 | 内容 |
|---|---|
| バージョン | v37.5.0 |
| テーマ | CDC Rune — Debezium JSON 形式の CDC イベント処理（MySQL / Postgres 対応） |
| 前提 | v37.4.0 COMPLETE — `List.fan_out` / `List.fan_in` 実装済み |
| 完了条件 | `v37500_tests` 全テスト pass・`cargo test` 0 failures（≥ 2723 件） |

## 背景と目的

CDC（Change Data Capture）は、データベースの変更イベントをリアルタイムで取得する仕組み。
Debezium は Kafka Connect ベースの CDC プラットフォームで、MySQL / Postgres の行変更を
JSON 形式でストリームとして出力する。

**今バージョンで行うこと（スコープ確定）:**
- `runes/cdc/cdc.fav` — Debezium JSON 形式の CDC イベント処理ユーティリティ関数群
- `runes/cdc/rune.toml` — rune メタデータ
- VM ビルトイン追加なし（`String.contains` 等の既存 primitive を使用）
- `v37500_tests` 4 テスト追加（meta 2 件 + 機能 2 件）

## Debezium JSON フォーマット（参考）

```json
{
  "payload": {
    "before": null,
    "after": { "id": 1, "name": "Alice" },
    "source": { "connector": "mysql", "table": "users" },
    "op": "c",
    "ts_ms": 1720000000000
  }
}
```

`op` フィールド: `"c"` = INSERT（create）, `"u"` = UPDATE, `"d"` = DELETE, `"r"` = READ（snapshot）

## 実装スコープ

### 1. `runes/cdc/cdc.fav` — CDC Rune 本体

```favnir
// CDC Rune — Debezium JSON 形式の CDC イベント処理（v37.5.0）
// MySQL / Postgres 対応

// op コード文字列（"c"/"u"/"d"/"r"）から人間可読な操作名を返す
fn CDC.op_name(op: String) -> String {
  if op == "c" { "insert" }
  else if op == "u" { "update" }
  else if op == "d" { "delete" }
  else { "read" }
}

// INSERT イベント（op == "c"）かどうか
fn CDC.is_insert(op: String) -> Bool {
  op == "c"
}

// UPDATE イベント（op == "u"）かどうか
fn CDC.is_update(op: String) -> Bool {
  op == "u"
}

// DELETE イベント（op == "d"）かどうか
fn CDC.is_delete(op: String) -> Bool {
  op == "d"
}

// Debezium イベント JSON 文字列から op フィールドを抽出する（文字列探索）
fn CDC.extract_op(json: String) -> String {
  if String.contains(json, "\"op\":\"c\"") { "c" }
  else if String.contains(json, "\"op\":\"u\"") { "u" }
  else if String.contains(json, "\"op\":\"d\"") { "d" }
  else { "r" }
}

// イベントリスト（JSON 文字列リスト）から INSERT のみフィルタリング
fn CDC.filter_inserts(events: List<String>) -> List<String> {
  List.filter(events, |e| String.contains(e, "\"op\":\"c\""))
}

// イベントリスト（JSON 文字列リスト）から DELETE のみフィルタリング
fn CDC.filter_deletes(events: List<String>) -> List<String> {
  List.filter(events, |e| String.contains(e, "\"op\":\"d\""))
}
```

### 2. `runes/cdc/rune.toml` — rune メタデータ

```toml
[rune]
name        = "cdc"
version     = "1.0.0"
description = "CDC Rune — Debezium JSON 形式の CDC イベント処理（MySQL / Postgres 対応）"
license     = "MIT"
authors     = ["Favnir Team"]
```

### 3. `fav/src/driver.rs` — `v37500_tests` モジュール追加

```rust
// ── v37500_tests (v37.5.0) — CDC Rune ────────────────────────────────────────
#[cfg(test)]
mod v37500_tests {
    // include_str! のみ使用のため use super::* 不要

    #[test]
    fn cargo_toml_version_is_37_5_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("37.5.0"), "Cargo.toml must contain version 37.5.0");
    }
    #[test]
    fn changelog_has_v37_5_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(src.contains("[v37.5.0]"), "CHANGELOG.md must contain [v37.5.0]");
    }
    #[test]
    fn cdc_rune_file_exists() {
        let src = include_str!("../../runes/cdc/cdc.fav");
        assert!(
            src.contains("CDC.extract_op"),
            "runes/cdc/cdc.fav must contain CDC.extract_op"
        );
    }
    #[test]
    fn cdc_rune_toml_exists() {
        let src = include_str!("../../runes/cdc/rune.toml");
        assert!(
            src.contains("cdc"),
            "runes/cdc/rune.toml must contain 'cdc'"
        );
    }
}
```

**重要:** このテストモジュールは `include_str!` のみ使用するため `use super::*` / `use super::{...}` は不要。
（`v37400_tests` は `run()` ヘルパーを持つため `use super::{build_artifact, exec_artifact_main}` が必要だが、
`v37500_tests` は `run()` を使わないため imports 一切不要。）

## 注意事項

### `= expr` 構文 vs `{ body }` 構文

cdc.fav では **すべての関数を `{ body }` ブロック構文に統一** する。
理由: `CDC.op_name` / `CDC.extract_op` は `else if` が必要なため `{ body }` 必須。
`CDC.is_insert` 等の単純比較も `= expr` で書けるが、ファイル内統一のために `{ body }` を使用する。

### `String.contains` の引数順序

`String.contains(haystack, needle)` — 第 1 引数が検索対象文字列、第 2 引数が検索パターン。

### cdc.fav の `String.contains` 使用

`List.filter(events, |e| String.contains(e, "\"op\":\"c\""))` のように
クロージャ内で `String.contains` を呼ぶパターンは、既存 stdlib（`driver.rs` 行 17242 等）で確認済み。

### スコープ外（v37.6 以降）

- Debezium JSON の深いフィールド抽出（`payload.after.id` 等）
- MySQL / Postgres 接続からのリアルタイム CDC ストリーム取得
- Kafka コンシューマとの統合
- `site/content/docs/runes/cdc.mdx` の作成（v37.8.0 Multi-Source cookbook 追加時にまとめて対応）

## ロードマップとの整合

ロードマップ v37.5.0:
- `runes/cdc/cdc.fav` — Debezium JSON 形式の CDC イベント処理
- MySQL / Postgres 対応
- Rust テスト 2 件

**実際のスコープ（ロードマップを T8 で更新）:**
- ロードマップは「Rust テスト 2 件」と記載しているが、本バージョンでは meta 2 件 + 機能 2 件の計 4 件が必要と確定。
  理由: `include_str!` ベーステストは「バージョン確認（meta）」と「ファイル内容確認（機能）」をセットで追加するのが他 rune バージョン（v29.2.0 mlflow 等）との統一パターン。ロードマップは tasks.md T8 で 4 件に更新する。
- `CDC.extract_op` / `CDC.filter_inserts` / `CDC.filter_deletes` / `CDC.op_name` / `CDC.is_*` 関数群

## 完了条件

| # | 条件 | 検証方法 |
|---|---|---|
| 1 | `Cargo.toml` バージョンが `37.5.0` | `cargo_toml_version_is_37_5_0` テスト |
| 2 | `CHANGELOG.md` に `[v37.5.0]` が含まれる | `changelog_has_v37_5_0` テスト |
| 3 | `runes/cdc/cdc.fav` に `CDC.extract_op` が含まれる | `cdc_rune_file_exists` テスト |
| 4 | `runes/cdc/rune.toml` に `cdc` が含まれる | `cdc_rune_toml_exists` テスト |
| 5 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2723） | `cargo test` 実行結果（v37.4.0 実績 2719 + 4 件 = 2723） |
