# v68.1.0 — Multi-Node `par`（分散並列実行）

Date: 2026-08-07
Status: 未着手
Sprint: Distributed Favnir（v68.1〜v69.0）

---

## 概要

`par` キーワードによる並列実行をシングルマシンからマルチノードクラスタに拡張する。
`--cluster workers.yaml` で複数ワーカーに処理を分散し、スループットをスケールアウトできる。
v68.1.0 はスタブ実装。実際のネットワーク通信は将来フェーズ。

## スコープ

### IN スコープ

- `fav/src/cluster.rs` — 新規作成
  - `pub const CLUSTER_HELP: &str` — `"--cluster"` / `"workers.yaml"` / `"--partition-by"` / `"--cluster-monitor"` を含む
  - `pub fn cmd_cluster_run(src: &str, cluster_file: &str, partition_by: &str) -> String`
    - `"--cluster"` / `"workers.yaml"` / `"--partition-by"` を含む出力を返す
    - 自動リバランス / `"--cluster-monitor"` キーワードを含む出力を返す
- `fav/src/main.rs` — `mod cluster;` 追加 + `Some("run")` ではなく新規 `Some("cluster")` アームを追加
  - `--help`/`-h` → `CLUSTER_HELP` 表示
  - `--cluster <file>` → `cmd_cluster_run` 呼び出し
- `fav/src/driver.rs` — `v68100_tests` 追加（2 件）

### OUT スコープ（将来フェーズ）

- 実際のネットワーク通信・ワーカー間データ転送
- `workers.yaml` の実際のパース
- `--partition-by` 式の評価
- ワーカー障害時の実際のリバランス処理

## テスト完了条件

| テスト名 | 検証内容 |
|---|---|
| `distributed_par_multi_node` | `cmd_cluster_run` が `"--cluster"` / `"workers.yaml"` / `"--partition-by"` を含む |
| `distributed_work_rebalance` | `cmd_cluster_run` が `"--cluster-monitor"` を含み、自動リバランスを示すキーワードを含む |

ベーステスト: 3519 → 目標: **3521**

## `fav cluster` コマンド設計

```
fav cluster --cluster workers.yaml pipeline.fav
fav cluster --cluster workers.yaml --partition-by "row_id % 4" pipeline.fav
fav cluster --cluster workers.yaml --cluster-monitor pipeline.fav
fav cluster --help
```

- `--partition-by` 省略時はデフォルト値 `"default"` を使用する
- `--cluster` 省略時は `eprintln!` + `process::exit(1)`
- `<pipeline.fav>` 省略時は `eprintln!` + `process::exit(1)`

`Some("cluster")` アームとして追加し、既存の `Some("run")` には影響しない。
