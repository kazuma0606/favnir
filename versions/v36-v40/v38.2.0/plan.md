# v38.2.0 実装計画 — `fav generate --from sql`

## 変更ファイル一覧

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `fav/src/generate_sql.rs` | 新規作成 | `sql_to_favnir` / `generate_load` / `generate_filter` / `generate_join` 実装 |
| `fav/src/main.rs` | 変更 | `mod generate_sql;` 追加 + `Some("--from")` アームを既存 `generate` ブロックに追加 |
| `fav/src/driver.rs` | 変更 | `v38100_tests::cargo_toml_version_is_38_1_0` スタブ化 / `v38200_tests` 追加（5 テスト）|
| `fav/Cargo.toml` | 更新 | `version = "38.1.0"` → `"38.2.0"` |
| `CHANGELOG.md` | 追記 | `[v38.2.0]` エントリ追加 |
| `versions/roadmap/roadmap-v38.1-v39.0.md` | 更新 | v38.2.0 を完了済みにマーク（✅）・テスト件数を 5 件に更新 |
| `versions/current.md` | 更新 | 最新安定版 v38.2.0、次バージョン v38.3.0 |
| `versions/v36-v40/v38.2.0/tasks.md` | 更新 | COMPLETE ステータスに更新（T0〜T9 全チェック）|

## 実装順序

### Step 1: CHANGELOG.md に [v38.2.0] エントリ追加

`## [v38.1.0]` の `---` セパレータ直後に挿入:

```markdown
## [v38.2.0] — 2026-07-10

### Added
- `fav/src/generate_sql.rs` — `fav generate --from sql <query>` コマンド追加
- `sql_to_favnir`: SELECT / JOIN / WHERE パターンを Favnir パイプラインに変換
- `v38200_tests` 6 テスト追加

---
```

**注意**: セパレータは `—`（全角ダッシュ U+2014）

### Step 2: `fav/src/generate_sql.rs` 新規作成

spec.md §1 の内容で作成。以下を含む:
- `pub fn sql_to_favnir(sql: &str) -> String`（JOIN/WHERE/SELECT の分岐）
- `fn generate_load(sql: &str) -> String`（SELECT のみ → `db.query` stage）
- `fn generate_filter(sql: &str) -> String`（WHERE → `List.filter` stage）
- `fn generate_join(sql: &str) -> String`（JOIN → `List.join_on` multi-stage）

**キーワード確認**（テストが検索する文字列）:
- `pub fn sql_to_favnir` — `generate_sql_fn_exists` テストが確認
- `stage` または `Load` — `sql_select_to_stage` テストが確認（`generate_load` の出力に含む）
- `join` または `Join` または `join_on` — `sql_join_to_stage` テストが確認（`generate_join` の出力に含む）

### Step 3: `fav/src/main.rs` — `mod generate_sql;` 追加

Read で `mod suggest;` の行番号を確認 → 直後に `mod generate_sql;` を追加。

**注意**: Step 2 で `generate_sql.rs` を作成してから Step 3 の `mod generate_sql;` を追加すること（コンパイルが通る順序）。

### Step 4: `fav/src/main.rs` — `Some("--from")` アーム追加

Read で `Some("generate") => match args.get(2)` ブロック（line 2385 付近）の `other =>` catch-all（line 2428 付近）の行番号を確認。
`other =>` の**直前**に `Some("--from")` アームを追加:

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

### Step 5: `driver.rs` — `v38100_tests::cargo_toml_version_is_38_1_0` スタブ化

Read で `cargo_toml_version_is_38_1_0` の行番号を確認 → ライブアサーションを:
```rust
// Stubbed: version bumped to 38.2.0 — assertion intentionally removed
```
に変更。

**注意**: `changelog_has_v38_1_0` / `suggest_fn_exists` はスタブ化しない。

### Step 6: `driver.rs` — `v38200_tests` モジュール追加（Step 1・2 完了後）

`v38100_tests` の閉じ `}` の行番号（v38.1.0 実装後の実態: 行 43649、実装前に Read で再確認）を特定してから Edit。
spec.md §3 のコードブロックに従い 6 テストを追加:
- `cargo_toml_version_is_38_2_0`
- `changelog_has_v38_2_0`
- `generate_sql_fn_exists`（`include_str!("generate_sql.rs")`）
- `sql_select_to_stage`（`crate::generate_sql::sql_to_favnir("SELECT id, name FROM users")`）
- `sql_join_to_stage`（`crate::generate_sql::sql_to_favnir("SELECT u.id FROM users u JOIN orders o ON u.id = o.user_id")`）
- `sql_where_to_stage`（`crate::generate_sql::sql_to_favnir("SELECT id FROM users WHERE active = true")`）

### Step 7: Cargo.toml バージョン更新

Step 1〜6 完了後に `38.1.0` → `38.2.0` に更新。

### Step 8: `cargo test` 実行・全通過確認

`cd /c/Users/yoshi/favnir/fav && cargo test 2>&1 | grep "test result"`

期待: ≥ 2750 passed, 0 failed

### Step 9: ドキュメント更新

- `versions/roadmap/roadmap-v38.1-v39.0.md` の v38.2.0 を ✅ にマーク・テスト件数を 6 件に更新
- `versions/current.md` を v38.2.0（最新安定版）・v38.3.0（次バージョン）に更新
- `versions/v36-v40/v38.2.0/tasks.md` を COMPLETE ステータスに更新

## 依存関係

```
Step 1 (CHANGELOG) ─────────────────────────────► Step 6 (driver tests, changelog_has_v38_2_0)
Step 2 (generate_sql.rs) ───────────────────────► Step 3 (main.rs mod generate_sql;)
                         ───────────────────────► Step 6 (driver tests, generate_sql_fn_exists)
                         ───────────────────────► Step 6 (driver tests, sql_select / sql_join / sql_where)
Step 3 (main.rs mod) ───────────────────────────► Step 4 (main.rs --from arm)
Step 4 (main.rs arm) ───────────────────────────► Step 8 (cargo test)
Step 5 (stub v38100) ───────────────────────────► Step 8 (cargo test)
Step 6 (v38200_tests) ──────────────────────────► Step 7 (Cargo.toml bump)
Step 6 (v38200_tests) ──────────────────────────► Step 8 (cargo test)
Step 7 (Cargo.toml) ────────────────────────────► Step 8 (cargo test)
Step 8 (all pass) ──────────────────────────────► Step 9 (docs)
```

## リスク

| リスク | 対処 |
|---|---|
| `crate::generate_sql::sql_to_favnir` が driver.rs からアクセスできない | `generate_sql.rs` は main.rs の `mod generate_sql;` で binary crate に追加される。driver.rs も同 binary crate のため `crate::generate_sql` でアクセス可能（v38.1.0 の `crate::suggest` と同構造） |
| `Some("--from")` を挿入する位置を誤り catch-all より後になる | Read で `other =>` 行番号を確認してから Edit を実行 |
| `gen` 予約語（Rust 2024） | v38.2.0 では `sql_result`・`up`・`fmt` 等を使用 — `gen` は不使用 |
| `mod generate_sql;` を先に追加すると `generate_sql.rs` がなくてコンパイルエラー | Step 2 → Step 3 の順序を厳守 |
