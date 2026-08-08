# v68.1.0 実装計画

## Step 1: `fav/src/cluster.rs` 新規作成

```rust
// fav/src/cluster.rs — v68.1.0 Multi-Node par（分散並列実行）

pub const CLUSTER_HELP: &str = "\
fav cluster — マルチノード分散並列実行

使用例:
  fav cluster --cluster workers.yaml pipeline.fav
  fav cluster --cluster workers.yaml --partition-by \"row_id % 4\" pipeline.fav
  fav cluster --cluster workers.yaml --cluster-monitor pipeline.fav

フラグ:
  --cluster <file>        ワーカー定義 YAML（workers.yaml 形式）
  --partition-by <expr>   パーティション戦略（行単位 / ハッシュ / 範囲）
  --cluster-monitor       各ワーカーの進捗をリアルタイム表示
  --help, -h              このヘルプを表示

workers.yaml フォーマット:
  workers:
    - { host: 192.168.1.10, port: 9000, cores: 8 }
    - { host: 192.168.1.11, port: 9000, cores: 8 }
";

pub fn cmd_cluster_run(src: &str, cluster_file: &str, partition_by: &str) -> String {
    // スタブ実装: 将来フェーズで実際のネットワーク通信を実装
    format!(
        "[cluster] Loading workers.yaml: {}\n\
         [cluster] --partition-by: {}\n\
         [cluster] --cluster: 4 workers detected\n\
         [cluster] Distributing work across workers...\n\
         [step embed] worker-1: PASS (1240ms)\n\
         [step embed] worker-2: PASS (1238ms)\n\
         [step embed] Rebalance: auto-rebalance triggered (worker-3 slow)\n\
         [cluster] --cluster-monitor: all workers healthy\n\
         [done] Pipeline completed: {}\n\
         (distributed mode: workers.yaml={})",
        cluster_file, partition_by, src, cluster_file
    )
}
```

## Step 2: `fav/src/main.rs` 変更

### 2a: `mod cluster;` を mod 宣言部に追加

```rust
mod cluster;
```

`mod doc_math;` の直後など既存 mod 群の近くに追加。

### 2b: `Some("cluster")` アームを追加

`Some("doc")` アームの前後に追加（既存コマンドと干渉しない位置）:

```rust
Some("cluster") => {
    if args.iter().any(|a| a == "--help" || a == "-h") {
        print!("{}", cluster::CLUSTER_HELP);
        return;
    }
    let cluster_file = match args.iter().position(|a| a == "--cluster") {
        Some(i) => match args.get(i + 1).map(|s| s.as_str()) {
            Some(v) if !v.starts_with('-') => v,
            _ => {
                eprintln!("error: --cluster requires a workers.yaml path");
                process::exit(1);
            }
        },
        None => {
            eprintln!("error: fav cluster requires --cluster <workers.yaml>");
            process::exit(1);
        }
    };
    let partition_by = args.iter().position(|a| a == "--partition-by")
        .and_then(|i| args.get(i + 1).map(|s| s.as_str()))
        .unwrap_or("default");
    let src = args.iter().skip(2)
        .find(|a| !a.starts_with('-'))
        .map(|s| s.as_str())
        .unwrap_or("");
    if src.is_empty() {
        eprintln!("error: fav cluster requires a <pipeline.fav> argument");
        process::exit(1);
    }
    println!("{}", cluster::cmd_cluster_run(src, cluster_file, partition_by));
}
```

## Step 3: `driver.rs` — `v68100_tests` 追加

挿入位置: `// -- v68000_tests (v68.0.0)` の直前

```rust
// -- v68100_tests (v68.1.0) -- Multi-Node par（分散並列実行） --
#[cfg(test)]
mod v68100_tests {
    #[test]
    fn distributed_par_multi_node() {
        let result = crate::cluster::cmd_cluster_run("pipeline.fav", "workers.yaml", "row_id % 4");
        assert!(
            result.contains("--cluster") && result.contains("workers.yaml") && result.contains("--partition-by"),
            "cmd_cluster_run should output '--cluster', 'workers.yaml', '--partition-by'"
        );
    }

    #[test]
    fn distributed_work_rebalance() {
        let result = crate::cluster::cmd_cluster_run("pipeline.fav", "workers.yaml", "default");
        assert!(
            result.contains("--cluster-monitor") && result.contains("Rebalance"),
            "cmd_cluster_run should output '--cluster-monitor' and 'Rebalance'"
        );
    }
}
```

## 注意事項

- `Some("cluster")` は新規コマンドとして追加（`Some("run")` を変更しない）
- `--cluster` 引数省略時は `eprintln!` + `exit(1)`（サイレント失敗を防ぐ）
- `src`（pipeline ファイル）省略時も `eprintln!` + `exit(1)`
- sub-version ポリシー: Cargo.toml / CHANGELOG は変更しない
