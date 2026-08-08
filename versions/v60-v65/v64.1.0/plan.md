# v64.1.0 Plan — AOT ビルドの CI 統合（`fav build --ci`）

Version: 64.1.0
Status: 未着手

---

## 変更ファイル一覧

| ファイル | 変更内容 |
|---|---|
| `fav/src/driver.rs` | `cmd_build_ci` 追加 + `create_ci_workflow_project` + `try_cmd_new` アーム追加 + `v64100_tests` 追加 |

---

## 実装ステップ

### Step 1: `cmd_build_ci` 追加

`cmd_build_link_target` の直後に追加。
- parse → compile → AOT lower の結果を `"ci: ok — ..."` / `"ci: error: ..."` 形式で返す
- ANSI コードなし、機械可読出力

### Step 2: `create_ci_workflow_project` + `try_cmd_new` 更新

- `create_rag_pipeline_project` の直後に `create_ci_workflow_project` を追加
  - `pipeline.fav` / `fav.toml` / `.github/workflows/build.yml` を生成
  - ワークフロー内に `fav build pipeline.fav --link --ci` ステップを含める
- テスト用 `pub(crate)` ラッパー `create_ci_workflow_project_pub` を追加
- `try_cmd_new` の `"rag-pipeline"` アームの直後に `"ci-workflow"` アームを追加
- エラーメッセージの末尾に `ci-workflow` を追記

### Step 3: `v64100_tests` 追加

`v64000_tests` の直前に挿入:
- `build_ci_flag_output_format`（`cmd_build_ci` の出力形式確認）
- `new_template_has_ci_workflow`（`.github/workflows/build.yml` 生成確認）

### Step 4: ビルド・テスト全件確認

- `cargo build` エラーなし
- `cargo test --bin fav v64100_tests` で 2 件 PASS
- `cargo test -j 8 -- --test-threads=8` で 3433 tests passed, 0 failed

---

## テスト計画

| テスト名 | 確認内容 |
|---|---|
| `build_ci_flag_output_format` | `"ci:"` プレフィックス + ANSI なし + ok/error を含む |
| `new_template_has_ci_workflow` | `.github/workflows/build.yml` 生成 + `fav build` + `--ci` |

ベース: 3431 → 目標: 3433（+2）
