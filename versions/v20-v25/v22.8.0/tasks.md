# v22.8.0 — `fav deploy` 強化（ECS / K8s / Fly.io 対応）タスク

## ステータス: COMPLETE

実装完了: 2026-06-21
テスト結果: 1882 passed / 0 failed（v228000_tests 5/5 PASS）

---

## タスク一覧

### T1: `fav/src/toml.rs` — DeployConfig 拡張

- [x] **事前確認**: `grep -n "pub role_arn\|pub region\|pub cpu\|impl Default for DeployConfig\|fn parse_deploy" fav/src/toml.rs | head -20` で挿入位置確認
- [x] **事前確認**: `grep "tempfile" fav/Cargo.toml` で tempfile が既に native-only deps / dev-dependencies に存在することを確認（追加不要）
- [x] `DeployConfig` struct に `cpu`/`cluster`/`namespace`/`schedule`/`app`/`out_dir` を追加（plan.md T1-1）
- [x] `impl Default for DeployConfig` に新フィールドの初期値（`None`）を追加（plan.md T1-2）
- [x] `parse_deploy_config` に新フィールドのパース行を追加（plan.md T1-3）
- [x] `cargo check --bin fav` でコンパイルエラーが 0 であることを確認

---

### T2: `fav/src/driver.rs` — deploy 関数追加・拡張

- [x] **事前確認**: `grep -n "cmd_deploy_trigger\|fn cmd_deploy\b\|fn deploy_upload_to_s3" fav/src/driver.rs | head -10` で挿入位置を確認
- [x] **事前確認**: `grep -n "cmd_deploy(" fav/src/driver.rs fav/src/main.rs | grep -v "fn cmd_deploy\|cmd_deploy_"` で呼び出し箇所をリストアップ
- [x] ヘルパー関数 `generate_dockerfile` を `cmd_deploy_trigger` の直前に追加（`#[cfg(not(target_arch = "wasm32"))]` 付き、plan.md T2-1）
- [x] ヘルパー関数 `generate_ecs_task_def` を追加（`#[cfg]` 付き、plan.md T2-1）
- [x] ヘルパー関数 `generate_k8s_cronjob` を追加（`#[cfg]` 付き、plan.md T2-1）
- [x] ヘルパー関数 `generate_fly_toml` を追加（`#[cfg]` 付き、plan.md T2-1）
- [x] ヘルパー関数 `write_deploy_file` を追加（`#[cfg(not(target_arch = "wasm32"))]` 付き、plan.md T2-2）
- [x] `cmd_deploy_ecs` を追加（plan.md T2-3）
- [x] `cmd_deploy_k8s` を追加（plan.md T2-4）
- [x] `cmd_deploy_fly` を追加（plan.md T2-5）
- [x] `cmd_deploy` に `target: Option<&str>` / `out_dir: Option<&str>` パラメータを追加し、`cfg!()` マクロを使ったターゲット分岐を実装（plan.md T2-6）
- [x] すべての `cmd_deploy(` 呼び出し箇所に `None, None` を追加（`driver.rs` 17306 行目付近のテスト内呼び出し含む）: `grep -n "cmd_deploy(" fav/src/driver.rs fav/src/main.rs | grep -v "fn cmd_deploy\|cmd_deploy_"` で全箇所確認
- [x] `v227000_tests::version_is_22_7_0` に `#[ignore]` を追加（plan.md T2-7）
- [x] `v228000_tests` モジュールを追加（5 件、plan.md T2-8 のコードに従う）
- [x] `cargo check --bin fav` でコンパイルエラーが 0 であることを確認
- [x] `cargo test v228000 --bin fav` — 5/5 PASS を確認

---

### T3: `fav/src/main.rs` — CLI フラグ追加

- [x] **事前確認**: `grep -n "let mut trigger_file\|let mut dry_run\|\"--dry-run\"\|cmd_deploy(" fav/src/main.rs | head -15` で挿入位置を確認（`--target` は build コマンド用に 490 行目付近に既存するが deploy ブロックとは別スコープのため衝突しない）
- [x] `let mut target: Option<String> = None;` と `let mut out_dir: Option<String> = None;` を追加（plan.md T3-2）
- [x] `"--target"` / `"--out-dir"` アームを `"--dry-run"` の直後に追加（plan.md T3-1）
- [x] `cmd_deploy` 呼び出しに `target.as_deref(), out_dir.as_deref()` を追加（plan.md T3-3）
- [x] ヘルプ文字列の `deploy` 行を更新（plan.md T3-4）
- [x] `cargo check --bin fav` でコンパイルエラーが 0 であることを確認

---

### T4: Cargo.toml + CHANGELOG + benchmarks + MDX

- [x] **事前確認**: `grep "\[v22.7.0\]" CHANGELOG.md` で先頭エントリを確認
- [x] `tempfile` は既に native-only deps / dev-dependencies に存在するため追加不要（T1 の事前確認で確認済み）
- [x] **注意**: T4-1（Cargo.toml バージョン更新）より前に T2-7（`#[ignore]` 追加）を実施すること（先に version が変わると `version_is_22_7_0` テストが失敗する）
- [x] `fav/Cargo.toml` の `version = "22.7.0"` → `"22.8.0"` に変更
- [x] v22.8.0 エントリを `CHANGELOG.md` の先頭（v22.7.0 の上）に追加（plan.md T4-2）
- [x] `benchmarks/v22.8.0.json` を新規作成（plan.md T4-3）
- [x] `site/content/docs/cli/deploy.mdx` を新規作成（plan.md T4-4）
- [x] `cargo test v228000 --bin fav` — 最終確認 5/5 PASS
- [x] `cargo test --bin fav` — リグレッションなし（1878 件以上合格）を確認

---

## テスト一覧（v228000_tests、5 件）

| テスト名 | 内容 |
|---|---|
| `version_is_22_8_0` | Cargo.toml に `version = "22.8.0"` が含まれる |
| `deploy_ecs_generates_dockerfile` | `cmd_deploy_ecs` が Dockerfile を生成し `FROM debian` を含む |
| `deploy_k8s_generates_cronjob_yaml` | `cmd_deploy_k8s` が `CronJob` YAML を生成する |
| `deploy_fly_generates_fly_toml` | `cmd_deploy_fly` が `fly.toml` を生成し app 名を含む |
| `changelog_has_v22_8_0` | CHANGELOG.md に `[v22.8.0]` が含まれる |

---

## 完了条件チェックリスト

- [x] `DeployConfig` に `cpu`/`cluster`/`namespace`/`schedule`/`app`/`out_dir` が追加される
- [x] `cmd_deploy_ecs` が Dockerfile + `ecs-task-def.json` を `.fav-deploy/` に生成する
- [x] `cmd_deploy_k8s` が Dockerfile + `{name}-cronjob.yaml` を `.fav-deploy/` に生成する
- [x] `cmd_deploy_fly` が Dockerfile + `fly.toml` を `.fav-deploy/` に生成する
- [x] `cmd_deploy` が `--target ecs/k8s/fly` で各関数に分岐する（`cfg!()` マクロ使用）
- [x] `--dry-run` 時はファイルを書かずコンソール出力のみ
- [x] `main.rs` に `--target` / `--out-dir` フラグが追加される
- [x] `fav deploy --help` 出力（ヘルプ文字列）に `--target` / `--out-dir` が含まれる
- [x] すべての新規関数に `#[cfg(not(target_arch = "wasm32"))]` ガードが付く
- [x] 既存の Lambda デプロイが後方互換を維持する
- [x] `cargo test v228000 --bin fav` — 5/5 PASS
- [x] `cargo test --bin fav` — リグレッションなし（1878 件以上合格）
- [x] `CHANGELOG.md` に v22.8.0 エントリ
- [x] `benchmarks/v22.8.0.json` 作成済み
- [x] `site/content/docs/cli/deploy.mdx` 作成済み

---

## 優先度

```
T1（toml.rs）    ← 最初（DeployConfig 拡張）
T2（driver.rs）  ← T1 完了後（ヘルパー + cmd_deploy_* 追加）
T3（main.rs）    ← T2 完了後（CLI フラグ追加）
T4（docs）       ← T3 完了後
```

---

## コードレビュー指摘と対応

| # | ラベル | 内容 | 対応 |
|---|--------|------|------|
| 1 | [HIGH] | `yaml_file`（`project_name` 由来）が `dir.join(filename)` に渡されパストラバーサルの可能性 | `sanitize_name()` ヘルパーを追加し `yaml_file` 生成と YAML 値埋め込みに適用 |
| 2 | [HIGH] | `generate_ecs_task_def` / `generate_k8s_cronjob` / `generate_fly_toml` で `project_name` が未サニタイズのまま JSON/YAML/TOML に埋め込まれる | JSON/TOML は `escape_json_str()` を適用、YAML は `sanitize_name()` を適用 |
| 3 | [MED] | `out_dir` の相対パス検証なし（ユーザー指定引数） | CLI の性質上許容（ドキュメントで相対パスが cwd 依存であることを説明済み）、変更なし |
| 4 | [MED] | `cfg!()` デッドコードブロックが将来のコード誤配置を招くリスク | コメントで意図を明示済み（`#[cfg]` は match 文に付けられないため `cfg!()` 使用）、変更なし |
