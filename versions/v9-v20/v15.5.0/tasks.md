# v15.5.0 Tasks — `fav deploy`（AWS Lambda デプロイ CLI）

Date: 2026-06-14
Branch: master

---

## Phase A — Cargo バージョン更新

- [x] A-1: `fav/Cargo.toml` の `version` を `"15.5.0"` に変更

---

## Phase B — `DeployConfig` 拡張（toml.rs）

- [x] B-1: `DeployConfig` struct に `target: String` フィールド追加（default: `"aws-lambda"`）
- [x] B-2: `DeployConfig` struct に `function_name: Option<String>` フィールド追加
- [x] B-3: `Default::default()` の `runtime` を `"provided.al2023"` に更新
- [x] B-4: `parse_fav_toml` の "deploy" セクション解析に `target` / `function_name` 追加
- [x] B-5: `memory_mb` / `timeout_sec` をエイリアスとして追加（`memory` / `timeout` と同義）
- [x] B-6: `cargo build` → コンパイルエラーなし確認

---

## Phase C — `scripts/build-lambda-layer.sh` 作成

- [x] C-1: `scripts/build-lambda-layer.sh` 新規作成
  - `cross build --release --target x86_64-unknown-linux-musl --bin fav`
  - `bootstrap` シェルスクリプト（Lambda Runtime API ループ）生成
  - `function.zip` 出力

---

## Phase D — `site/content/docs/deploy.mdx` 作成

- [x] D-1: `site/content/docs/deploy.mdx` 新規作成
  - `fav.toml [deploy]` 設定リファレンス表
  - `fav deploy --dry-run` の出力例
  - `scripts/build-lambda-layer.sh` の使い方
  - 必要 IAM 権限 JSON
  - Lambda 環境変数設定例

---

## Phase E — v155000_tests 追加（driver.rs）

- [x] E-1: `fav/src/driver.rs` に `v155000_tests` モジュール追加（3 テスト）
  - `version_is_15_5_0`
  - `deploy_toml_schema_parses`
  - `deploy_cmd_exists`

---

## Phase F — テスト・コミット

- [x] F-1: `cargo test v155000` → 3/3 PASS
- [x] F-2: `cargo test` → 1572 PASS（リグレッションなし）
- [x] F-3: コミット `c1d8bdd` — feat: v15.5.0 — fav deploy 完成（AWS Lambda デプロイ CLI）

---

## 完了条件

| 確認項目 | 状態 |
|---|---|
| `Cargo.toml version == "15.5.0"` | [x] |
| `cargo test v155000` 全テストパス（3/3） | [x] |
| `cargo test` 全件パス（リグレッションなし） | [x] |
| `DeployConfig` に `target` / `function_name` が追加されている | [x] |
| `scripts/build-lambda-layer.sh` が存在する | [x] |
| `site/content/docs/deploy.mdx` が存在する | [x] |
| `fav deploy --dry-run` が正常動作 | [x] |
