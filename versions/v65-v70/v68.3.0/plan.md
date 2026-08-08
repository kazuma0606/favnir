# v68.3.0 実装計画

## Step 1: `fav/src/k8s.rs` 新規作成

```rust
// fav/src/k8s.rs — v68.3.0 Kubernetes-Native Orchestration

pub fn cmd_deploy_k8s(src: &str) -> String {
    // スタブ実装: 将来フェーズで実際の K8s CRD 生成を実装
    format!(
        "[generate] Kubernetes manifests → ./k8s/\n\
         [--target kubernetes] Generating Pipeline CRD for: {}\n\
         ---\n\
         apiVersion: favnir.dev/v1\n\
         kind: Pipeline\n\
         metadata:\n\
           name: pipeline\n\
           namespace: data-platform\n\
         spec:\n\
           stages:\n\
             - name: load\n\
               image: favnir/runtime:68.0.0\n\
               replicas: 1\n\
             - name: embed\n\
               image: favnir/runtime:68.0.0\n\
               replicas: 4\n\
               resources:\n\
                 requests: {{ memory: \"2Gi\", cpu: \"2\" }}\n\
                 limits:   {{ memory: \"4Gi\", gpu: \"1\" }}\n\
             - name: store\n\
               image: favnir/runtime:68.0.0\n\
               replicas: 2\n\
           checkpointing:\n\
             enabled: true\n\
             storageClass: standard\n\
         [stub] Would write manifests to ./k8s/ (source: {})",
        src, src
    )
}
```

**注意**: 出力末尾は `[stub] Would write manifests to` — 実際のファイル書き込みは行わない。

## Step 2: `fav/src/main.rs` 変更

### 2a: `mod k8s;` を mod 宣言部に追加

```rust
mod k8s;
```

`mod checkpoint;` の直後に追加。

### 2b: `Some("deploy")` アームに `--target kubernetes` ブランチ追加

挿入位置: パースループ完了後・`if let Some(ref tfile) = trigger_file` の直前

```rust
// ── v68.3.0: fav deploy --target kubernetes ───────────────────────
// 注意: --trigger と同時指定された場合は kubernetes ターゲットが優先される（--trigger は無視）
if target.as_deref() == Some("kubernetes") {
    let src = args.iter().skip(2)
        .find(|a| !a.starts_with('-') && Some(a.as_str()) != target.as_deref())
        .map(|s| s.as_str())
        .unwrap_or("pipeline.fav");
    println!("{}", k8s::cmd_deploy_k8s(src));
    return;
}
```

**注意**:
- `args` は `std::env::args().collect::<Vec<String>>()` の全引数（例: `["fav", "deploy", "--target", "kubernetes", "pipeline.fav"]`）。`skip(2)` は `args[0]`（バイナリ名）と `args[1]`（`"deploy"`）をスキップする。
- `target.as_deref()` は `Some("kubernetes")` の場合のみ分岐する
- `src` 検出時に `target.as_deref()`（= `Some("kubernetes")`）を除外し、フラグ値の誤検出を防ぐ（v68.1.0 で発生したパターンへの対策）
- `--trigger` + `--target kubernetes` 同時指定時は kubernetes が優先される（コメントで明記）

## Step 3: `driver.rs` — `v68300_tests` 追加

挿入位置: `// -- v68200_tests (v68.2.0) -- Pipeline Checkpointing（耐障害性・再開） --` の直前

```rust
// -- v68300_tests (v68.3.0) -- Kubernetes-Native Orchestration --
#[cfg(test)]
mod v68300_tests {
    #[test]
    fn k8s_pipeline_manifest_gen() {
        let result = crate::k8s::cmd_deploy_k8s("pipeline.fav");
        assert!(
            result.contains("apiVersion: favnir.dev/v1") && result.contains("kind: Pipeline"),
            "cmd_deploy_k8s should output 'apiVersion: favnir.dev/v1' and 'kind: Pipeline'"
        );
    }

    #[test]
    fn k8s_stage_replicas() {
        let result = crate::k8s::cmd_deploy_k8s("pipeline.fav");
        assert!(
            result.contains("replicas") && result.contains("resources") && result.contains("--target kubernetes"),
            "cmd_deploy_k8s should output 'replicas', 'resources', '--target kubernetes'"
        );
    }
}
```

## 注意事項

- `Some("deploy")` の既存ロジック（`cmd_deploy_strategy` / `cmd_deploy` / `cmd_deploy_trigger`）は変更しない
- `--target kubernetes` ブランチはパースループ完了後に挿入する（`target` 変数が確定した後）
- 各 Step 後に `cargo build` でエラーがないことを確認する
- Step 3 完了後に `cargo test --bin fav v68300_tests` で 2 件 PASS を確認する
- sub-version ポリシー: Cargo.toml / CHANGELOG は変更しない
