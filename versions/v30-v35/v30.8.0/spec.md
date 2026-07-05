# v30.8.0 仕様書 — fav new --list コマンド

## 概要

`fav new --list` でプロジェクト生成に使えるテンプレート一覧を表示できるようにする。

---

## 背景

ロードマップ v30.8 より:

**現状**: `fav new --list` を実行すると unknown argument エラーになる。

**目標**:
```
$ fav new --list
利用可能なテンプレート:

  script           シンプルなスクリプト（1ファイル）
  pipeline         基本パイプライン（seq/par）
  lib              ライブラリ（公開関数のみ）
  postgres-etl     PostgreSQL ETL（4ファイル構成）[推奨]
  etl-csv-to-db    CSV → DB ETL
  api-gateway      HTTP API ゲートウェイ
  lambda-scheduled スケジュール実行 Lambda ジョブ
  distributed-etl  分散並列 ETL パイプライン

使用例:
  fav new my-project --template postgres-etl
```

---

## スコープ

### IN SCOPE

- `cmd_new_list`（新規 `pub fn`、`driver.rs`）— テンプレート一覧を stdout に表示
- `main.rs` — `use driver::` リストへの `cmd_new_list` 追加
- `main.rs` — `fav new --list` を検出して `cmd_new_list()` を呼ぶ（行 1256 の `Some("new")` ハンドラのみ）
- `Rust テスト`（`v308000_tests` — 3 件）

> **ロードマップとのカウント差**: ロードマップは「Rust テスト 1 件」と記載しているが、
> spec では 3 件追加する（version / 実装確認 / benchmark の 3 点が最小セット）。

### OUT OF SCOPE

- `fav new --list --json` などの出力フォーマットオプション
- テンプレートの説明文のローカライズ
- `TEMPLATE_GALLERY` 定数の変更（既存の 4 件は維持）
  - `cmd_new_list` は `TEMPLATE_GALLERY` を直接利用せず 8 件をハードコードする。
    これは `TEMPLATE_GALLERY` が 4 件のみ（`etl-csv-to-db` 以降）であり、
    `script` / `pipeline` / `lib` / `postgres-etl` が含まれないため。
    将来テンプレートを追加する際は `TEMPLATE_GALLERY` と `cmd_new_list` の両方を更新すること。
- `main.rs` 2043 行付近の `Some("new")` ブロック（これは `fav notebook new` のサブコマンド — 変更不要）
- site/ MDX 更新（意図的に除外）

---

## 実装仕様

### 全テンプレートリスト（8 件）

| テンプレート名 | 説明 |
|---------------|------|
| `script` | シンプルなスクリプト（1ファイル） |
| `pipeline` | 基本パイプライン（seq/par） |
| `lib` | ライブラリ（公開関数のみ） |
| `postgres-etl` | PostgreSQL ETL（4ファイル構成）[推奨] |
| `etl-csv-to-db` | CSV → DB ETL |
| `api-gateway` | HTTP API ゲートウェイ |
| `lambda-scheduled` | スケジュール実行 Lambda ジョブ |
| `distributed-etl` | 分散並列 ETL パイプライン |

### `cmd_new_list`（`driver.rs`、`fn try_cmd_new` の直前）

```rust
pub fn cmd_new_list() {
    println!("利用可能なテンプレート:");
    println!();
    println!("  {:<17} {}", "script",          "シンプルなスクリプト（1ファイル）");
    println!("  {:<17} {}", "pipeline",         "基本パイプライン（seq/par）");
    println!("  {:<17} {}", "lib",              "ライブラリ（公開関数のみ）");
    println!("  {:<17} {}", "postgres-etl",     "PostgreSQL ETL（4ファイル構成）[推奨]");
    println!("  {:<17} {}", "etl-csv-to-db",    "CSV → DB ETL");
    println!("  {:<17} {}", "api-gateway",      "HTTP API ゲートウェイ");
    println!("  {:<17} {}", "lambda-scheduled", "スケジュール実行 Lambda ジョブ");
    println!("  {:<17} {}", "distributed-etl",  "分散並列 ETL パイプライン");
    println!();
    println!("使用例:");
    println!("  fav new my-project --template postgres-etl");
}
```

### `main.rs` 更新（2 箇所）

1. **87 行付近** — `use driver::{ ... cmd_new, ... }` に `cmd_new_list` を追加
2. **1256 行付近** — `Some("new")` ハンドラの先頭に `--list` フラグ検出を追加

```rust
Some("new") => {
    // --list フラグ: テンプレート一覧を表示して終了
    if args.get(2).map(|s| s.as_str()) == Some("--list") {
        cmd_new_list();
        return;
    }
    // ... 既存の name / --template 処理
}
```

> **2043 行の `Some("new")` は変更しない**: これは `fav notebook new` のサブコマンドハンドラであり、
> `fav new` とは無関係。

---

## テスト設計（v308000_tests — 3 件）

| # | テスト名 | 確認内容 |
|---|---------|----------|
| 1 | `cargo_toml_version_is_30_8_0` | `Cargo.toml` に `version = "30.8.0"` |
| 2 | `cmd_new_list_contains_all_templates` | `driver.rs` に `fn cmd_new_list` が存在し（先行ガード）、8 テンプレート名がすべて含まれること |
| 3 | `benchmark_v30_8_0_exists` | `benchmarks/v30.8.0.json` に `"30.8.0"` |

> テスト 2 は `fn cmd_new_list` の存在を先にガードすることで、
> `try_cmd_new` の既存 match アームにテンプレート名が含まれる誤検知を防ぐ。

---

## 完了条件

- `Cargo.toml` version = `"30.8.0"`
- `cmd_new_list` が `driver.rs` に実装されている（8 テンプレート全件表示）
- `main.rs` の `use driver::` リストに `cmd_new_list` が追加されている
- `fav new --list` が正しく一覧を表示する（`main.rs` 行 1256 の更新）
- `cargo test v308000` — 3/3 PASS
- `cargo test` — 全件 PASS（0 failures）
- `CHANGELOG.md` に `[v30.8.0]` セクション
- `benchmarks/v30.8.0.json` 存在
- `versions/current.md` を v30.8.0 に更新（「最新安定版」欄を変更）
- `tasks.md` が COMPLETE
