# v69.0.0 実装計画 — Distributed Favnir 宣言

Status: DRAFT
Version: 69.0.0

---

## 実装ステップ

### Step 1: `fav/Cargo.toml` バージョン更新

`fav/Cargo.toml` の `version = "68.0.0"` を `version = "69.0.0"` に変更。

### Step 2: `MILESTONE.md` 更新

先頭に以下を追加:

```
## v69.0.0 — Distributed Favnir（2026-08-07）

`par` がクラスタを越え、チェックポイントが失敗を無効にする。
Kubernetes が AI ステージのスケールを決め、コスト見積もりが LLM 呼び出しの予算を守る。
型安全な AI パイプラインが、大規模でも壊れない。

- Multi-Node `par`（分散並列実行）
- Pipeline Checkpointing（耐障害性・再開）
- Kubernetes-Native Orchestration
- Stage Retry Policies
- Distributed Incremental Cache
- Cost-Aware Scheduling
- Multi-Cloud AI Routing
- Distributed Observability
```

### Step 3: `README.md` 更新

"Distributed Favnir" キーワードを含む v69.0.0 宣言セクションを追加。

### Step 4: `CHANGELOG.md` 更新

先頭に v69.0.0 エントリを追加（"v69.0.0" キーワード必須）:

```
## [v69.0.0] — 2026-08-07 — Distributed Favnir 宣言

### Added
- Multi-Node `par` 分散並列実行（--cluster workers.yaml / --partition-by）
- Pipeline Checkpointing（--checkpoint / --resume）
- Kubernetes-Native Orchestration（fav deploy --target kubernetes）
- Stage Retry Policies（ExponentialBackoff / LinearBackoff）
- Distributed Incremental Cache（--distributed-cache redis://...）
- Cost-Aware Scheduling（fav cost-estimate --provider --scale）
- Multi-Cloud AI Routing（fav.toml [ai] セクション / --env dev/prod/test）
- Distributed Observability（--otel-endpoint / OpenTelemetry）
- Stabilization & Code Freeze v68.9.0（distributed.mdx 作成・全機能統合確認）

### Changed
- Cargo.toml version: 68.0.0 → 69.0.0
```

### Step 5: `driver.rs` — `v69000_tests` 追加

`v68900_tests` ブロックの直前に挿入（driver.rs は降順配置）。

テスト 4 件:
- `cargo_toml_version_is_69_0_0`: `include_str!("../Cargo.toml")` → `"version = \"69.0.0\""` assert
- `changelog_has_v69_0_0`: `include_str!("../../CHANGELOG.md")` → `"v69.0.0"` assert
- `milestone_has_distributed`: `include_str!("../../MILESTONE.md")` → `"Distributed Favnir"` assert
- `readme_mentions_distributed`: `include_str!("../../README.md")` → `"Distributed Favnir"` assert

### Step 6: `cargo clean`（★クリーンアップ）

```bash
cd /c/Users/yoshi/favnir/fav && cargo clean
```

### Step 7: テスト実行

```bash
cargo test --bin fav v69000_tests  # 4 件 PASS
cargo test -j 8 -- --test-threads=8  # 3541 tests PASS
```

---

## ファイルパス参照（include_str! 基準）

`driver.rs` の include_str! は `fav/src/driver.rs` から見たパスで:
- Cargo.toml: `"../Cargo.toml"` — `fav/Cargo.toml`（`../` で `fav/` に上がる）
- CHANGELOG.md: `"../../CHANGELOG.md"` — `CHANGELOG.md`（リポジトリルート）
- MILESTONE.md: `"../../MILESTONE.md"` — `MILESTONE.md`（リポジトリルート）
- README.md: `"../../README.md"` — `README.md`（リポジトリルート）

> 注: v68.0.0 / v67.0.0 の実績パターンを踏襲（driver.rs 内で確認済み）
