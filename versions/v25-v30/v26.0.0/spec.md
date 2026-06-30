# v26.0.0 — Rune Foundation マイルストーン宣言

## テーマ

v25.1〜v25.9 で達成した「コア 8 Rune の完全実装 + vm.fav Phase 6（CallNamed）」を、
**v26.0.0 = Rune Foundation** として正式に宣言する。

> 「Favnir で書いたパイプラインが実際の本番データを動かせる」
> — `fav run examples/full_etl.fav` が postgres → 集計 → s3 → kafka 通知を実際に実行する

---

## 達成済みコンポーネント（v25.1〜v25.9 時点）

| バージョン | Rune | 実装内容 | テスト |
|---|---|---|---|
| v25.1.0 | postgres | connect / query / execute / execute_many / transaction / Pool | 5 件 |
| v25.2.0 | s3 | get_object / put_object / list_objects / delete_object / presign_url / stream_get | 5 件 |
| v25.3.0 | redis | get / set / del / incr / lpush / rpop / publish / subscribe | 5 件 |
| v25.4.0 | mysql | connect / query / execute / transaction（DbConn interface 統一） | 4 件 |
| v25.5.0 | mongodb | find / find_one / insert_one / insert_many / update_one / delete_one / aggregate | 5 件 |
| v25.6.0 | dynamodb | get_item / put_item / delete_item / query / scan / batch_write / transact_write | 5 件 |
| v25.7.0 | kafka | produce / consume / consume_batch / commit / seek | 5 件 |
| v25.8.0 | elasticsearch | index / index_with_id / search / bulk / delete / knn_search / create_index | 5 件 |
| v25.9.0 | vm.fav Phase 6 | CallNamed(0x56) opcode / vm_run_program / build_vm_program_json | 7 件 |

---

## 成果物

### T1: `MILESTONE.md` 更新

`MILESTONE.md`（v25.0.0 で作成済み）に **「Rune Foundation」セクション** を追加:

- `"Rune Foundation"` を含む（テスト要件）
- コア 8 Rune がすべて「動く Rune の 5 条件」をクリアした旨
- vm.fav Phase 6 完了の宣言
- `fav run examples/full_etl.fav` デモの説明

### T2: `examples/` — ETL デモファイル群（新規作成）

| ファイル | 内容 |
|---|---|
| `examples/postgres_etl.fav` | postgres → 集計 → postgres 書き込み |
| `examples/s3_csv_to_parquet.fav` | s3 CSV 取得 → Parquet 変換 → s3 書き込み |
| `examples/full_etl.fav` | postgres → 集計 → s3 保存 → kafka 通知（結合デモ） |

各ファイルは Favnir コードとして有効であり、`fav run` で実行可能（ローカル Docker 環境あり）。

### T3: `README.md` 更新

- `"v26.0"` / `"Rune Foundation"` を追記（テスト要件: `"v26.0"`）
- コア 8 Rune が動作することを示す実行例を追記

### T4: `site/content/docs/rune-foundation.mdx` — 新規作成

- `"Rune Foundation"` を含む（テスト要件）
- 8 Rune の 5 条件クリア状況表
- `full_etl.fav` のコード例と実行手順（Docker Compose）
- vm.fav Phase 6 の達成サマリー

### T5: `versions/roadmap/roadmap-v25.1-v26.0.md` 更新

- v25.1〜v25.9 を「完了」に更新
- v26.0.0 を「宣言済み」に更新

---

## Rust テスト（v260000_tests、5 件）

| テスト名 | 内容 | 期待値 |
|---|---|---|
| `milestone_md_has_rune_foundation` | `MILESTONE.md` に `"Rune Foundation"` が含まれる | assert |
| `readme_mentions_v26_0` | `README.md` に `"v26.0"` が含まれる | assert |
| `site_rune_foundation_page_exists` | `site/content/docs/rune-foundation.mdx` に `"Rune Foundation"` が含まれる | assert |
| `examples_full_etl_exists` | `examples/full_etl.fav` が存在し `"postgres"` を含む | assert |
| `changelog_has_v26_0_0` | `CHANGELOG.md` に `[v26.0.0]` が含まれる | assert |

---

## テスト件数

- 削除: なし（v259000_tests に `version_is_` テストは存在しない — T0 で確認）
- 追加: `v260000_tests`（5 件）
- v25.9.0 完了時: 2035 件
- **目標**: 2035 + 5 = **2040 件**（固定）

---

## スコープ外（v26.x 以降）

以下はロードマップ §v26.0「最終テスト」に記載があるが、この宣言バージョンでは Docker E2E の自動化は必須としない（デモファイルの存在とコードの正しさを確認するにとどまる）:

- LocalStack / Redpanda / Postgres Docker コンテナを自動起動する CI 統合テスト
- `fav run --vm=self/vm.fav self/compiler.fav -- hello.fav` の E2E 自動テスト（bootstrap）
- クロージャのキャプチャ（vm.fav Phase 7）
- Rune Registry の実装（v28.x 予定）

---

## 完了条件

- [ ] `fav/Cargo.toml` が `version = "26.0.0"` であること
- [ ] `MILESTONE.md` に `"Rune Foundation"` が含まれること
- [ ] `examples/postgres_etl.fav` が存在すること
- [ ] `examples/s3_csv_to_parquet.fav` が存在すること
- [ ] `examples/full_etl.fav` が存在し `"postgres"` を含むこと
- [ ] `README.md` に `"v26.0"` が含まれること
- [ ] `site/content/docs/rune-foundation.mdx` が存在し `"Rune Foundation"` を含むこと
- [ ] `versions/roadmap/roadmap-v25.1-v26.0.md` が v25.1〜v26.0 を反映済みであること
- [ ] `cargo test v260000 --bin fav` — 5/5 PASS
- [ ] `cargo test --bin fav` — リグレッションなし（2040 件）
- [ ] `CHANGELOG.md` に `[v26.0.0]` エントリが存在すること
- [ ] `benchmarks/v26.0.0.json` が存在すること（test_count: 2040）
