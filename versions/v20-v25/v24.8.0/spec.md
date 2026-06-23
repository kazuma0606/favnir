# v24.8.0 — `fav new` テンプレートギャラリー

## テーマ

よくあるユースケースのテンプレートを `fav new --template <name>` でワンコマンド生成できるようにする。

---

## 動機

- 現状の `fav new` は `script` / `pipeline` / `lib` / `postgres-etl` の 4 テンプレートのみ
- データエンジニアがよく使うユースケース（ETL / API / スケジューラ / 分散 ETL）が未整備
- テンプレートギャラリーがなく、`fav new --template` で何が使えるか分からない
- `fav new --template list` 相当の一覧表示は v25.x でのスコープ（本バージョンは TEMPLATE_GALLERY 定数で実装）

---

## 成果物

### T1: `TEMPLATE_GALLERY` 定数（driver.rs）

```rust
pub const TEMPLATE_GALLERY: &[(&str, &str)] = &[
    ("etl-csv-to-db",    "CSV → DB ETL パイプライン"),
    ("api-gateway",      "HTTP API ゲートウェイ"),
    ("lambda-scheduled", "スケジュール実行 Lambda ジョブ"),
    ("distributed-etl",  "分散並列 ETL パイプライン"),
];
```

### T2: 4 テンプレート実装（driver.rs）

`try_cmd_new` の match アームに追加し、各 `create_*_project` 関数を実装する。

各テンプレートが生成するファイル:

| テンプレート | 生成ファイル |
|---|---|
| `etl-csv-to-db` | `pipeline.fav` / `fav.toml` / `README.md` / `.github/workflows/ci.yml` |
| `api-gateway` | `api.fav` / `fav.toml` / `README.md` / `.github/workflows/ci.yml` |
| `lambda-scheduled` | `job.fav` / `fav.toml` / `README.md` / `.github/workflows/ci.yml` |
| `distributed-etl` | `pipeline.fav` / `fav.toml` / `README.md` / `.github/workflows/ci.yml` |

`write_text_file` が `create_dir_all` で親ディレクトリを自動作成するため、`root/` および `.github/workflows/` は明示的な mkdir 不要。

### T3: エラーメッセージ更新（driver.rs）

`unknown template` エラーに 4 テンプレートを追記:

```
unknown template `foo` (expected script|pipeline|lib|postgres-etl|etl-csv-to-db|api-gateway|lambda-scheduled|distributed-etl)
```

### T4: サイトドキュメント（site/content/docs/tools/templates.mdx）

- `fav new --template` の使い方
- 4 テンプレートの概要表

---

## Rust テスト（v248000_tests、7 件）

v247000_tests にはバージョン検証テスト（`version_is_X`）がないため削除対象なし。7 件を純粋追加する。

| テスト名 | 内容 | 期待値 |
|---|---|---|
| `template_gallery_has_4_entries` | `TEMPLATE_GALLERY.len() == 4` かつ全 4 名を含む | assert_eq! |
| `fav_new_etl_csv_to_db_ok` | tempdir で `try_cmd_new` → Ok / `pipeline.fav` 存在 | assert!(ok) |
| `fav_new_api_gateway_ok` | tempdir で `try_cmd_new` → Ok / `api.fav` 存在 | assert!(ok) |
| `fav_new_lambda_scheduled_ok` | tempdir で `try_cmd_new` → Ok / `job.fav` 存在 | assert!(ok) |
| `fav_new_distributed_etl_ok` | tempdir で `try_cmd_new` → Ok / `pipeline.fav` 存在 | assert!(ok) |
| `fav_new_unknown_template_errors` | `try_cmd_new("x", "no-such")` → Err（`"etl-csv-to-db"` 含む） | assert!(err) |
| `changelog_has_v24_8_0` | `CHANGELOG.md` に `[v24.8.0]` | assert |

---

## テスト件数

- 削除: なし（v247000_tests に削除対象の `version_is_X` テストが存在しない）
- 追加: `v248000_tests`（7 件）
- 合計: **1962 + 7 = 1969 件**

---

## 完了条件

- [ ] `TEMPLATE_GALLERY` に 4 エントリ定義済み（driver.rs）
- [ ] `create_etl_csv_to_db_project` 実装済み
- [ ] `create_api_gateway_project` 実装済み
- [ ] `create_lambda_scheduled_project` 実装済み
- [ ] `create_distributed_etl_project` 実装済み
- [ ] `try_cmd_new` に 4 アーム追加済み
- [ ] エラーメッセージに 4 テンプレート名追記済み
- [ ] `site/content/docs/tools/templates.mdx` 作成済み
- [ ] `cargo test v248000 --bin fav` — 7/7 PASS
- [ ] `cargo test --bin fav` — リグレッションなし（1969 件合格）
- [ ] `CHANGELOG.md` に v24.8.0 エントリ
- [ ] `benchmarks/v24.8.0.json` 作成済み（test_count: 1969）
