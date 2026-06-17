# v20.0.0 — Production Performance マイルストーン宣言 タスク

## ステータス: COMPLETE

---

## タスク一覧

### T1: `fav/Cargo.toml` — バージョン更新

- [x] `version = "19.8.0"` → `"20.0.0"`

---

### T2: `CHANGELOG.md` — v19.1.0〜v19.8.0 エントリ追加

- [x] v19.1.0（遅延評価パイプライン `#[streaming]`）エントリ追加
- [x] v19.2.0（AOT コンパイル Cranelift バックエンド）エントリ追加
- [x] v19.3.0（インクリメンタルコンパイル）エントリ追加
- [x] v19.4.0（並列コンパイル）エントリ追加
- [x] v19.5.0（Apache Arrow 統合）エントリ追加
- [x] v19.6.0（WASM 最適化）エントリ追加
- [x] v19.7.0（事前コンパイル `.favc`）エントリ追加
- [x] v19.8.0（プロファイリング強化 フレームグラフ）エントリ追加
- [x] v20.0.0（Production Performance マイルストーン）エントリ追加

---

### T3: `README.md` — v20.0.0 更新

- [x] 現在のバージョン表記を v20.0.0 に更新
- [x] Production Performance マイルストーン達成を記載
- [x] streaming / AOT / Arrow / precompiled 実績を記載
- [x] ベンチマーク参考値を追加（10GB CSV / Lambda コールドスタート）
- [x] バージョン履歴表に v19.1.0〜v20.0.0 エントリ追加

---

### T4: `site/content/docs/performance/` — 6 ファイル新規作成

- [x] `performance/streaming.mdx`: `#[streaming]` / `#[stateful]` 使い方、chunk 処理の仕組み
- [x] `performance/native-build.mdx`: `fav build --target native`、Cranelift バックエンド説明
- [x] `performance/incremental.mdx`: フィンガープリント、`.fav_cache/`、`FAV_NO_CACHE` / `FAV_EXPLAIN_CACHE`
- [x] `performance/arrow.mdx`: `ArrowBatch.from_list` / `to_list` / `write_parquet` / `read_parquet`
- [x] `performance/precompiled.mdx`: `fav compile` / `fav run --precompiled`、Lambda コールドスタート削減
- [x] `performance/profiling.mdx`: `--format=flamegraph/text/json`、`--runs=N`、`--stage=<name>`

---

### T5: `benchmarks/` — 3 ファイル新規作成

- [x] `benchmarks/10gb_csv.fav`: streaming パイプラインで 10GB CSV を定常メモリ処理するサンプル
- [x] `benchmarks/lambda_coldstart.sh`: `fav compile` + `fav run --precompiled` のコールドスタート計測スクリプト
- [x] `benchmarks/results.md`: ベンチマーク結果記録（参考値）

---

### T6: `fav/src/driver.rs` — `v200000_tests` 追加（5件）

- [x] `v200000_tests` モジュール追加
- [x] `version_is_19_8_0` に `#[ignore]` 追加
- [x] `version_is_20_0_0`（Cargo.toml に "20.0.0" が含まれる）
- [x] `changelog_has_v19_entries`（CHANGELOG.md に "19." エントリが含まれる）
- [x] `readme_mentions_streaming`（README.md に "streaming" が含まれる）
- [x] `readme_mentions_native_build`（README.md に "native" が含まれる）
- [x] `benchmarks_dir_exists`（`benchmarks/` ディレクトリが存在する）
- [x] `cargo test v200000` — 5/5 PASS 確認

---

## テスト（v200000_tests、5件）

| テスト名 | 内容 | 結果 |
|---|---|---|
| `version_is_20_0_0` | Cargo.toml に `"20.0.0"` が含まれる | PASS |
| `changelog_has_v19_entries` | CHANGELOG.md に v19.x エントリが含まれる | PASS |
| `readme_mentions_streaming` | README.md に "streaming" が含まれる | PASS |
| `readme_mentions_native_build` | README.md に "native" が含まれる | PASS |
| `benchmarks_dir_exists` | `benchmarks/` ディレクトリが存在する | PASS |

---

## 完了条件チェックリスト

- [x] Cargo.toml が `20.0.0` になっている
- [x] CHANGELOG.md に v19.1.0〜v19.8.0 エントリがある
- [x] README.md に Production Performance マイルストーン記述がある
- [x] `site/content/docs/performance/` に 6 ファイルが存在する
- [x] `benchmarks/` ディレクトリに 3 ファイルが存在する
- [x] `cargo test v200000` — 5/5 PASS
- [x] `cargo test` — リグレッションなし
