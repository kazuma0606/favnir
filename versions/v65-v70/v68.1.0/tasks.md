# v68.1.0 タスクリスト

Status: COMPLETE
Version: 68.1.0
Note: MDX ドキュメントは v68.9.0 で一括作成のため本バージョンでは不要
Base tests: 3519
Target tests: 3521

---

## T0: 事前確認

- [x] `cargo test -j 8 -- --test-threads=8` でベース 3519 tests passed, 0 failed を確認
- [x] `fav/Cargo.toml` の version が `"68.0.0"` であることを確認（sub-version では変更しない）
- [x] `fav/src/cluster.rs` が存在しないことを確認（新規作成）
- [x] `driver.rs` に `v68000_tests` が存在することを確認（`v68100_tests` の挿入位置）
- [x] `driver.rs` に `v68100_tests` が存在しないことを確認（新規追加）
- [x] `cargo test --bin fav v68000_tests` で 4 件 PASS することを確認（前バージョンが正常）
  - 前バージョンのテスト関数名: `cargo_toml_version_is_68_0_0`, `changelog_has_v68_0_0`, `milestone_has_dev_intelligence`, `readme_mentions_dev_intelligence`
- [x] `versions/current.md` の「最新安定版」が `v68.0.0` であることを確認

---

## T1: `fav/src/cluster.rs` 新規作成

- [x] `fav/src/cluster.rs` を新規作成
  - [x] `pub const CLUSTER_HELP: &str` を追加（`"--cluster"` / `"workers.yaml"` / `"--partition-by"` / `"--cluster-monitor"` を含む）
  - [x] `pub fn cmd_cluster_run(src: &str, cluster_file: &str, partition_by: &str) -> String` を追加
    - [x] `"--cluster"` / `"workers.yaml"` / `"--partition-by"` を含む出力を返す（`distributed_par_multi_node` テスト要件）
    - [x] `"--cluster-monitor"` / `"Rebalance"` を含む出力を返す（`distributed_work_rebalance` テスト要件）
- [x] `cargo build` でエラーなし

---

## T2: `fav/src/main.rs` 変更

- [x] `mod cluster;` を mod 宣言部に追加
- [x] `Some("cluster")` アームを追加
  - [x] `--help`/`-h` → `print!("{}", cluster::CLUSTER_HELP); return;`
  - [x] `--cluster` 省略時 → `eprintln!` + `process::exit(1)`
  - [x] `--partition-by` の値を取得（省略時は `"default"` を使用）
  - [x] `src`（pipeline.fav）省略時 → `eprintln!` + `process::exit(1)`
  - [x] `println!("{}", cluster::cmd_cluster_run(src, cluster_file, partition_by))`
- [x] `cargo build` でエラーなし

---

## T3: `driver.rs` — `v68100_tests` 追加

- [x] `grep "v68000_tests" fav/src/driver.rs` でコメント行の正確な文字列を確認
- [x] `// -- v68000_tests (v68.0.0)` の直前に `v68100_tests` を挿入
  - [x] `distributed_par_multi_node`: `cmd_cluster_run` の戻り値に `"--cluster"` / `"workers.yaml"` / `"--partition-by"` を含む
  - [x] `distributed_work_rebalance`: `cmd_cluster_run` の戻り値に `"--cluster-monitor"` / `"Rebalance"` を含む
- [x] `use super::*` は不要（`crate::cluster::` で直接参照）
- [x] `cargo build` でエラーなし

---

## T4: ビルド・テスト

- [x] `cargo test --bin fav v68100_tests` で 2 件 PASS
  - [x] `distributed_par_multi_node` PASS
  - [x] `distributed_work_rebalance` PASS
- [x] `cargo test -j 8 -- --test-threads=8` で 3521 tests passed, 0 failed を確認

---

## T5: ドキュメント・ステータス更新

- [x] `versions/roadmap/roadmap-v68.1-v69.0.md` の v68.1.0「状態」列を「未着手」→「完了」に変更
- [x] `versions/current.md` の「進行中バージョン」を v68.1.0 に更新
- [x] 本 `tasks.md` を COMPLETE に更新（T0 を含む全チェックボックスを `[x]` に）

> **sub-version ポリシー**: v68.x では Cargo.toml / CHANGELOG.md は変更しない。v69.0.0 宣言時に一括更新する。

---

## 設計上の意図的省略

- 実際のネットワーク通信・ワーカー間データ転送: 将来フェーズ
- `workers.yaml` の実際のパース: 将来フェーズ
- `--partition-by` 式の評価: 将来フェーズ
- ワーカー障害時の実際のリバランス処理: 将来フェーズ

## コードレビュー指摘と対応

| 深刻度 | 内容 | 対応 |
|---|---|---|
| [MED] | `Some("cluster")` の `src` 検出が `cluster_file` 値（`workers.yaml`）を誤検出 — `fav cluster --cluster workers.yaml pipeline.fav` で src = "workers.yaml" になる | `cluster_file` / `partition_by` を除外するフィルターを追加（`a.as_str() != cluster_file && a.as_str() != partition_by`） |
