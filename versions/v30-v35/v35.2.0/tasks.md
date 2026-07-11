# v35.2.0 タスクリスト — `fav deploy --target docker`

## ステータス: COMPLETE

## T0: 事前確認

- [x] 現在のテスト数が 2633（0 failures）であることを確認
- [x] Cargo.toml バージョンが `35.1.0` であることを確認
- [x] 既存 `generate_dockerfile` の場所を確認（`#[cfg(not(target_arch = "wasm32"))]` ガードあり）
- [x] `v35200_tests` の `cargo_toml_version_is_35_2_0` テストが現在スタブであることを確認

## T1: toml.rs — `tag` フィールド追加

- [x] `DeployConfig` 構造体に `tag: Option<String>` フィールドを追加
- [x] `Default` impl に `tag: None` を追加

## T2: driver.rs — `generate_dockerfile_native` 追加

- [x] 既存 `generate_dockerfile` の直後に `fn generate_dockerfile_native(project_name: &str) -> String` を追加
- [x] テンプレート: `debian:bookworm-slim` ベース + `target/native/<name>` COPY + ENTRYPOINT `/app/pipeline`
- [x] `#[cfg(not(target_arch = "wasm32"))]` ガードを付ける

## T3: driver.rs — `build_docker_image` 追加

- [x] `generate_dockerfile_native` の直後に `fn build_docker_image(out_dir: &str, tag: &str) -> Result<bool, String>` を追加
- [x] Docker CLI 存在確認（`docker --version` の exit code）
- [x] CLI 不在時は警告を出して `Ok(false)` を返す（フォールバック）
- [x] `docker build -t <tag> <out_dir>` を実行
- [x] exit code チェック、非 0 は `Err` 返却

## T4: driver.rs — `cmd_deploy_docker` 追加

- [x] `build_docker_image` の直後に `pub fn cmd_deploy_docker` を追加
- [x] タグ解決順: CLI `--tag` > `fav.toml [deploy] tag` > `<project-name>:latest`（String 所有型使用、Box::leak 回避）
- [x] `generate_dockerfile_native` で Dockerfile 生成 → `write_deploy_file` で書き出し
- [x] `--package-only` または `--dry-run` 時は `build_docker_image` をスキップ
- [x] `build_docker_image` 戻り値で "built" / "skipped" を分岐して表示

## T5: driver.rs — `cmd_deploy` に `tag` 引数追加 + `"docker"` アーム追加

- [x] `cmd_deploy` シグネチャに `tag: Option<&str>` を追加
- [x] `"lambda"` アームの直後（`other if other != "aws-lambda"` の直前）に `"docker"` アームを追加
- [x] 既存の内部テスト呼び出しの引数を更新:
  - [x] `driver.rs` 内テスト（`cmd_deploy(..., false, None)`）に末尾 `None` を追加

## T6: main.rs — `--tag` フラグ追加 + `cmd_deploy` 呼び出し更新

- [x] `Some("deploy")` アームに `let mut tag_arg: Option<String> = None;` を追加
- [x] `"--tag"` match アームを追加（`--output` の直後）
- [x] `--target` エラーメッセージに `docker` を追加（`lambda|docker|ecs|k8s|fly|aws-lambda`）
- [x] `cmd_deploy` 呼び出しに `tag_arg.as_deref()` を追加

## T7: driver.rs — v35200_docker_tests 追加

- [x] `v35200_tests` の `cargo_toml_version_is_35_2_0` テストを確認（既にスタブ済み）
- [x] `v35200_docker_tests` モジュールを追加（5 件）
  - [x] `cargo_toml_version_is_35_2_0`
  - [x] `dockerfile_native_uses_bookworm_slim`
  - [x] `dockerfile_native_copies_project_binary`
  - [x] `dockerfile_native_entrypoint_is_pipeline`
  - [x] `changelog_has_v35_2_0`

## T8: テスト実行

- [x] `cargo test` 全通過（0 failures）— 2638 passed; 0 failed
- [x] v35200_docker_tests の 5 テストが pass

## T9: バージョン管理と CHANGELOG（T8 完了後に実施）

- [x] `fav/Cargo.toml` バージョンを `35.2.0` に更新
- [x] `CHANGELOG.md` に `## [35.2.0]` エントリを追加

## T10: ドキュメント更新

- [x] `versions/v30-v35/v35.2.0/tasks.md` を COMPLETE ステータスに更新
- [x] （`versions/current.md` はマイナー版のため更新しない）

---

## 完了条件チェックリスト（spec.md 対応）

| # | spec.md 完了条件 | 確認方法 |
|---|---|---|
| 1 | `--package-only` で `<out-dir>/Dockerfile` が生成される | 手動確認（`write_deploy_file` の動作 + `cmd_deploy_docker --package-only` 実行）✅ |
| 2 | Dockerfile が `debian:bookworm-slim` ベースでネイティブバイナリを COPY する | `dockerfile_native_uses_bookworm_slim` + `dockerfile_native_copies_project_binary` テスト ✅ |
| 3 | `fav.toml [deploy] tag` が `cmd_deploy_docker` に正しく渡される | `deploy_config_parse_from_toml`（手動確認）✅ |
| 4 | Docker CLI 不在時に警告を出してスキップし exit 1 しない | `build_docker_image` 実装確認 ✅ |
| 5 | `cargo test` が 0 failures（v35200_docker_tests 全 pass）| T8 実行結果 ✅ |
| 6 | `CHANGELOG.md` に `[35.2.0]` エントリが存在する | `changelog_has_v35_2_0` テスト ✅ |

---

## コードレビュー事前チェックリスト

- [x] `generate_dockerfile_native` が `#[cfg(not(target_arch = "wasm32"))]` ガードを持つ
- [x] `build_docker_image` が Docker CLI 不在時に `panic` せず `Ok(false)` を返す
- [x] `cmd_deploy_docker` が `--package-only` / `--dry-run` 時に `docker build` をスキップする
- [x] `cmd_deploy` の `"docker"` アームが `if cfg!(not(target_arch = "wasm32"))` ブロック内にある
- [x] `cmd_deploy` シグネチャ変更後、すべての呼び出しサイトが更新されている（main.rs + driver.rs テスト）
- [x] タグのデフォルト値が `<project-name>:latest` になっている（String 所有型）

## コードレビュー対応（実施後に記録）

| 指摘 | 優先度 | 対応 |
|---|---|---|
| （実施後に記録） | — | — |
