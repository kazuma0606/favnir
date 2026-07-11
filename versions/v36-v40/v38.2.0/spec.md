# v38.2.0 spec — `fav generate --from sql`

## バージョン概要

| 項目 | 内容 |
|---|---|
| バージョン | v38.2.0 |
| テーマ | `fav generate --from sql` — SQL を Favnir パイプラインに変換 |
| 前提 | v38.1.0 COMPLETE — `fav suggest` 実装済み |
| 完了条件 | `v38200_tests` 全テスト pass・`cargo test` 0 failures（≥ 2749 件） |

## 背景と目的

v38.1.0 の `fav suggest` に続き、AI 支援ツールの第二弾として `fav generate --from sql` を追加する。
PostgreSQL / MySQL の SELECT / JOIN / WHERE / ORDER BY を Favnir パイプラインのスケルトンに変換することで、
既存 SQL 資産を Favnir に移植する際の出発点を自動生成する。

**想定動作**:
```bash
$ fav generate --from sql "SELECT id, name FROM users WHERE active = true"
// Generated from SQL (WHERE → List.filter)
stage LoadAndFilter -> List<String> {
    bind rows <- db.query(ctx, "SELECT id, name FROM users WHERE active = true")
    List.filter(rows, |row| True)
}

pipeline main {
    LoadAndFilter
}
```

## 実装スコープ

### 1. `fav/src/generate_sql.rs` — 新規作成

```rust
/// v38.2.0 — fav generate --from sql: SQL を Favnir パイプラインに変換する

pub fn sql_to_favnir(sql: &str) -> String {
    let up = sql.trim().to_uppercase();
    if up.contains("JOIN") {
        generate_join(sql)
    } else if up.contains("WHERE") {
        generate_filter(sql)
    } else {
        generate_load(sql)
    }
}

fn generate_load(sql: &str) -> String {
    format!(
        "// Generated from SQL\nstage LoadData -> List<String> {{\n    db.query(ctx, {:?})\n}}\n\npipeline main {{\n    LoadData\n}}\n",
        sql
    )
}

fn generate_filter(sql: &str) -> String {
    format!(
        "// Generated from SQL (WHERE → List.filter)\nstage LoadAndFilter -> List<String> {{\n    bind rows <- db.query(ctx, {:?})\n    List.filter(rows, |row| True)\n}}\n\npipeline main {{\n    LoadAndFilter\n}}\n",
        sql
    )
}

fn generate_join(sql: &str) -> String {
    format!(
        "// Generated from SQL (JOIN → List.join_on)\nstage LoadLeft -> List<String> {{ db.query(ctx, \"SELECT * FROM left_table\") }}\nstage LoadRight -> List<String> {{ db.query(ctx, \"SELECT * FROM right_table\") }}\nstage JoinTables(left: List<String>, right: List<String>) -> List<String> {{\n    List.join_on(left, right, |l, r| True)\n}}\n\npipeline main {{\n    LoadLeft, LoadRight |> JoinTables\n}}\n// Source SQL: {}\n",
        sql
    )
}
```

**エクスポート関数**: `pub fn sql_to_favnir(sql: &str) -> String`

**変換ルール**:
| SQL パターン | 生成物 |
|---|---|
| `JOIN` を含む | `List.join_on` を使った multi-stage パイプライン（`stage` + `pipeline main` ブロック含む） |
| `WHERE` を含む（JOIN なし） | `List.filter` を使った filter stage（`stage` + `pipeline main` ブロック含む） |
| それ以外の SELECT | シンプルな `db.query` stage（`stage LoadData` + `pipeline main` ブロック含む） |
| ORDER BY を含む | v38.2.0 スコープ外: SELECT パスと同じ `generate_load` で処理 |

### 2. `fav/src/main.rs` — `mod generate_sql;` 追加 + `Some("--from")` アーム追加

#### `mod generate_sql;` 追加

`mod suggest;` の直後に追加:
```rust
mod suggest;
mod generate_sql;
```

#### `Some("--from")` アーム追加

既存の `Some("generate") => match args.get(2).map(|s| s.as_str())` ブロック内の
`other =>` catch-all アームの**直前**に追加する:

```rust
Some("--from") => {
    let fmt = args.get(3).map(|s| s.as_str()).unwrap_or("");
    match fmt {
        "sql" => {
            let sql = args.get(4).map(|s| s.as_str()).unwrap_or("");
            let output = generate_sql::sql_to_favnir(sql);
            println!("{}", output);
        }
        _ => {
            eprintln!("error: unsupported --from format {:?}. Supported: sql", fmt);
            process::exit(1);
        }
    }
}
```

**注意**: `Some("generate")` の `match args.get(2)` ブロック内に挿入する（行 2385 付近）。
`other =>` catch-all（行 2428 付近）の直前に配置すること。

**args インデックス根拠**（v38.1.0 の `Some("suggest")` パターンと同様）:
- `args[0]` = バイナリパス
- `args[1]` = `"generate"`（外側の match で分岐済み）
- `args[2]` = `"--from"`（内側 `match args.get(2)` でマッチ）
- `args[3]` = `"sql"`（format 種別）
- `args[4]` = SQL クエリ文字列

よって `args.get(3)` で format、`args.get(4)` でクエリを取得するのは正しい。
v38.1.0 の `Some("suggest")` では `args.get(2)` = error_code、`args.get(3)` = location で同様のパターンを採用済み。

### 3. `driver.rs` — テストモジュール追加

#### `v38100_tests::cargo_toml_version_is_38_1_0` のスタブ化

```rust
// Stubbed: version bumped to 38.2.0 — assertion intentionally removed
```

#### `v38200_tests` モジュール新規追加（5 テスト）

```rust
// ── v38200_tests (v38.2.0) — fav generate --from sql ─────────────────────────
#[cfg(test)]
mod v38200_tests {
    // include_str! のみ使用のため imports 不要（crate:: 直接参照）

    #[test]
    fn cargo_toml_version_is_38_2_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("38.2.0"), "Cargo.toml must contain version 38.2.0");
    }

    #[test]
    fn changelog_has_v38_2_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(src.contains("[v38.2.0]"), "CHANGELOG.md must contain [v38.2.0]");
    }

    #[test]
    fn generate_sql_fn_exists() {
        // driver.rs と generate_sql.rs は同じ fav/src/ ディレクトリに置かれる
        let src = include_str!("generate_sql.rs");
        assert!(src.contains("pub fn sql_to_favnir"), "generate_sql.rs must contain pub fn sql_to_favnir");
    }

    #[test]
    fn sql_select_to_stage() {
        let result = crate::generate_sql::sql_to_favnir("SELECT id, name FROM users");
        assert!(
            result.contains("stage") || result.contains("Load"),
            "SELECT SQL should generate a stage or Load: got {:?}", result
        );
    }

    #[test]
    fn sql_join_to_stage() {
        let result = crate::generate_sql::sql_to_favnir(
            "SELECT u.id FROM users u JOIN orders o ON u.id = o.user_id"
        );
        assert!(
            result.contains("join") || result.contains("Join") || result.contains("join_on"),
            "JOIN SQL should reference join or join_on: got {:?}", result
        );
    }

    #[test]
    fn sql_where_to_stage() {
        let result = crate::generate_sql::sql_to_favnir(
            "SELECT id FROM users WHERE active = true"
        );
        assert!(
            result.contains("filter") || result.contains("Filter") || result.contains("Where"),
            "WHERE SQL should generate filter: got {:?}", result
        );
    }
}
```

**注意**: `sql_select_to_stage` / `sql_join_to_stage` / `sql_where_to_stage` は `crate::generate_sql::sql_to_favnir` を直接呼び出す。
これが可能なのは `generate_sql.rs` が main.rs の `mod generate_sql;` により binary crate のモジュールとして宣言され、
`driver.rs` も同じ binary crate に属しているため（v38.1.0 の `suggest.rs` / `crate::suggest` と同構造）。

ロードマップの「SELECT / JOIN / WHERE 変換テスト」は SELECT / JOIN / WHERE それぞれに対応する
3 機能テストを追加することで満たす（meta 2 + functional 4 = 6 件）。

### 4. `CHANGELOG.md` — `[v38.2.0]` エントリ追加

```
## [v38.2.0] — 2026-07-10

### Added
- `fav/src/generate_sql.rs` — `fav generate --from sql <query>` コマンド追加
- `sql_to_favnir`: SELECT / JOIN / WHERE パターンを Favnir パイプラインに変換
- `v38200_tests` 5 テスト追加

---
```

**セパレータは `—`（全角ダッシュ U+2014）**

### 5. その他ドキュメント更新

- `fav/Cargo.toml`: `38.1.0` → `38.2.0`
- `versions/current.md`: 最新安定版 → v38.2.0、次バージョン → v38.3.0
- `versions/roadmap/roadmap-v38.1-v39.0.md`: v38.2.0 を ✅ 完了済みにマーク・テスト件数を 5 件に更新

## テスト数の計算

| バージョン | 実績 |
|---|---|
| v38.1.0 | 2744 |
| v38.2.0 追加分 | +6 |
| v38.2.0 期待値 | 2750 |

ロードマップは「Rust テスト 3 件（SELECT / JOIN / WHERE 変換テスト）」と記載しているが、
meta 2 件（cargo_toml + changelog）+ functional 4 件（existence + SELECT + JOIN + WHERE）の計 6 件を追加し、T9 でロードマップを 6 件に更新する。

## ロードマップとの整合

ロードマップ v38.2.0:
- PostgreSQL / MySQL の SELECT / JOIN / WHERE / ORDER BY を Favnir パイプラインに変換
- Rust テスト 3 件（→ 6 件に更新）

## 注意事項

### ORDER BY スコープ外

ロードマップは「SELECT / JOIN / WHERE / ORDER BY」と列挙しているが、ORDER BY → `List.sort` への変換は v38.2.0 スコープ外とする。
ORDER BY を含む SQL は `generate_load`（SELECT パス）と同じ `db.query` stage として扱い、ソート変換は行わない。
ORDER BY 対応は v38.3.0 以降で検討する。

### `crate::generate_sql::sql_to_favnir` のアクセス権

`generate_sql.rs` は `main.rs` の `mod generate_sql;` を経由して binary crate に追加される。
`driver.rs` は同じ binary crate のモジュールのため `crate::generate_sql` にアクセス可能。
v38.1.0 の `crate::suggest` と同一構造（ただし v38.1.0 の `suggest_fn_exists` は include_str! のみで関数呼び出しは行わなかった）。

### `Some("--from")` アームの挿入位置

既存の `Some("generate")` ブロックは line 2385 から始まり、`other =>` catch-all が line 2428 付近にある。
`Some("--from")` は `other =>` の**直前**に挿入すること。`Some("api")` アームの後ろが正しい位置。

### `gen` 予約語（Rust 2024）

今バージョンの変数名には `sql_result`、`up`、`fmt` 等を使用する — `gen` は使わないこと。

## 完了条件

| # | 条件 | 検証方法 |
|---|---|---|
| 1 | `generate_sql.rs` に `pub fn sql_to_favnir` が含まれる | `generate_sql_fn_exists` テスト |
| 2 | SELECT SQL が stage を含む出力に変換される | `sql_select_to_stage` テスト |
| 3 | JOIN SQL が join_on を含む出力に変換される | `sql_join_to_stage` テスト |
| 4 | WHERE SQL が filter を含む出力に変換される | `sql_where_to_stage` テスト |
| 5 | `CHANGELOG.md` に `[v38.2.0]` が含まれる | `changelog_has_v38_2_0` テスト |
| 6 | `Cargo.toml` バージョンが `38.2.0` | `cargo_toml_version_is_38_2_0` テスト |
| 7 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2750） | `cargo test` 実行結果 |
| 8 | `roadmap-v38.1-v39.0.md` の v38.2.0 が ✅ かつテスト件数が 6 件 | T9 後に目視確認 |
