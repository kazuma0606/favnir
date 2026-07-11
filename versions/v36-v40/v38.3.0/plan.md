# v38.3.0 実装計画 — `fav generate --from csv` 強化

## 変更ファイル一覧

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `fav/src/generate_csv.rs` | 新規作成 | `csv_to_favnir` / `csv_to_favnir_from_str` / `parse_headers` / `generate_from_headers` 実装 |
| `fav/src/main.rs` | 変更 | `pub(crate) mod generate_csv;` 追加 + `"csv"` 分岐追加 + `_ =>` エラーメッセージ更新 |
| `fav/src/driver.rs` | 変更 | `v38200_tests::cargo_toml_version_is_38_2_0` スタブ化 / `v38300_tests` 追加（4 テスト） |
| `fav/Cargo.toml` | 更新 | `version = "38.2.0"` → `"38.3.0"` |
| `CHANGELOG.md` | 追記 | `[v38.3.0]` エントリ追加 |
| `versions/roadmap/roadmap-v38.1-v39.0.md` | 更新 | v38.3.0 を完了済みにマーク（✅）・テスト件数を 4 件に更新 |
| `versions/current.md` | 更新 | 最新安定版 v38.3.0、次バージョン v38.4.0 |
| `versions/v36-v40/v38.3.0/tasks.md` | 更新 | COMPLETE ステータスに更新（T0〜T9 全チェック）|

## 実装順序

### Step 1: CHANGELOG.md に [v38.3.0] エントリ追加

`## [v38.2.0]` の `---` セパレータ直後に挿入:

```markdown
## [v38.3.0] — 2026-07-10

### Added
- `fav/src/generate_csv.rs` — `fav generate --from csv <file>` コマンド追加
- `csv_to_favnir`: CSV ヘッダーから `type Row` + `schema` + `expect` ブロックを生成
- `v38300_tests` 4 テスト追加

---
```

**注意**: セパレータは `—`（全角ダッシュ U+2014）

### Step 2: `fav/src/generate_csv.rs` 新規作成

spec.md §1 の内容で作成。以下を含む:
- `pub fn csv_to_favnir(csv_path: &str) -> Result<String, String>` — ファイルから生成（パス traversal ガード含む）
- `pub fn csv_to_favnir_from_str(csv_str: &str) -> Result<String, String>` — 文字列から生成（テスト用）
- `fn parse_headers(csv: &str) -> Result<Vec<String>, String>`
- `fn generate_from_headers(headers: &[String]) -> String`

**キーワード確認**（テストが検索する文字列）:
- `pub fn csv_to_favnir` — `generate_csv_fn_exists` テストが確認
- `type Row` + `schema` + `expect` — `csv_to_favnir_basic` テストが確認

**パス traversal ガード**: `csv_path.contains("..")` の場合 `Err` を返す。

### Step 3: `fav/src/main.rs` — `pub(crate) mod generate_csv;` 追加

Read で `pub(crate) mod generate_sql;` の行番号を確認 → 直後に `pub(crate) mod generate_csv;` を追加。

**注意**: Step 2 で `generate_csv.rs` を作成してから Step 3 を実施（コンパイルが通る順序）。

### Step 4: `fav/src/main.rs` — `"csv"` 分岐追加 + `_ =>` メッセージ更新

Read で `match fmt` ブロック内 `_ =>` catch-all（line 2441 付近）の行番号を確認。
`_ =>` の直前に `"csv"` アームを追加（spec.md §2 のコードブロックに従う）:

```rust
"csv" => {
    let csv_path = args.get(4).map(|s| s.as_str()).unwrap_or_else(|| {
        eprintln!("error: `fav generate --from csv` requires a CSV file path");
        eprintln!("usage: fav generate --from csv <file.csv>");
        process::exit(1)
    });
    match generate_csv::csv_to_favnir(csv_path) {
        Ok(output) => println!("{}", output),
        Err(e) => {
            eprintln!("fav generate error: {}", e);
            process::exit(1);
        }
    }
}
```

`_ =>` catch-all のメッセージを `"Supported: sql"` → `"Supported: sql, csv"` に更新:

```rust
_ => {
    eprintln!("error: unsupported --from format {:?}. Supported: sql, csv", fmt);
    process::exit(1);
}
```

### Step 5: `driver.rs` — `v38200_tests::cargo_toml_version_is_38_2_0` スタブ化

Read で `cargo_toml_version_is_38_2_0` の行番号を確認（T0 で記録） → ライブアサーションを:
```rust
// Stubbed: version bumped to 38.3.0 — assertion intentionally removed
```
に変更。

**注意**: `changelog_has_v38_2_0` / `generate_sql_fn_exists` / `sql_*` テストはスタブ化しない。

### Step 6: `driver.rs` — `v38300_tests` モジュール追加（Step 1・2 完了後）

`v38200_tests` の閉じ `}` の行番号（T0 で記録、参考値: 43704）を Read で特定してから Edit。
spec.md §3 のコードブロックに従い 4 テストを追加。

### Step 7: Cargo.toml バージョン更新

Step 1〜6 完了後に `38.2.0` → `38.3.0` に更新。

### Step 8: `cargo test` 実行・全通過確認

`cd /c/Users/yoshi/favnir/fav && cargo test 2>&1 | grep "test result"`

期待: ≥ 2754 passed, 0 failed

### Step 9: ドキュメント更新

- `versions/roadmap/roadmap-v38.1-v39.0.md` の v38.3.0 を ✅ にマーク・テスト件数を 4 件に更新
- `versions/current.md` を v38.3.0（最新安定版）・v38.4.0（次バージョン）に更新
- `versions/v36-v40/v38.3.0/tasks.md` を COMPLETE ステータスに更新

## 依存関係

```
Step 1 (CHANGELOG) ─────────────────────────────► Step 6 (driver tests, changelog_has_v38_3_0)
Step 2 (generate_csv.rs) ───────────────────────► Step 3 (main.rs mod generate_csv;)
                         ───────────────────────► Step 4 (main.rs "csv" arm)
                         ───────────────────────► Step 6 (driver tests, generate_csv_fn_exists)
                         ───────────────────────► Step 6 (driver tests, csv_to_favnir_basic)
Step 3 (main.rs mod) ───────────────────────────► Step 4 (main.rs "csv" arm, コンパイル通過)
                     ───────────────────────────► Step 8 (cargo test, mod 宣言がコンパイルに必須)
Step 4 (main.rs arm) ───────────────────────────► Step 8 (cargo test)
Step 5 (stub v38200) ───────────────────────────► Step 8 (cargo test)
Step 6 (v38300_tests) ──────────────────────────► Step 7 (Cargo.toml bump)
Step 6 (v38300_tests) ──────────────────────────► Step 8 (cargo test)
Step 7 (Cargo.toml) ────────────────────────────► Step 8 (cargo test)
Step 8 (all pass) ──────────────────────────────► Step 9 (docs)
```

## リスク

| リスク | 対処 |
|---|---|
| `csv_to_favnir_from_str` に空文字を渡した場合 `parse_headers` が `Err` を返す | `csv_to_favnir_basic` テストは有効な CSV 文字列を渡すため問題なし |
| `pub(crate) mod generate_csv;` を先に追加すると `generate_csv.rs` がなくてコンパイルエラー | Step 2 → Step 3 の順序を厳守 |
| `_ =>` catch-all のメッセージ更新を忘れる | Step 4 のチェックリストに明示（T4 の `[x]` 確認項目）|
| `gen` 予約語（Rust 2024） | `generate_csv.rs` では `headers`・`fields`・`first_col` を使用 — `gen` は不使用 |
