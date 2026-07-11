# v35.3.0 タスクリスト — `fav ci init`

## ステータス: COMPLETE

## T0: 事前確認

- [x] 現在のテスト数が 2638（0 failures）であることを確認
- [x] Cargo.toml バージョンが `35.2.0` であることを確認
- [x] `Some("ci")` が main.rs に存在しないことを確認
- [x] `v35300_tests` の `cargo_toml_version_is_35_3_0` テストがスタブであることを確認（コメント: "stubbed: version bumped to 35.4.0"）
- [x] `write_text_file` の挙動確認（`create_dir_all` で親ディレクトリを自動作成する）

## T1: driver.rs — `generate_ci_yaml` 追加

- [x] deploy 関連関数群の末尾に `pub fn generate_ci_yaml(_project_name: &str) -> String` を追加
- [x] YAML に `fav check` / `fav lint` / `fav test` の 3 ステップを含める
- [x] `on.push.branches: [main]` と `pull_request` トリガーを含める
- [x] `#[cfg(not(target_arch = "wasm32"))]` ガードなし（純粋な文字列生成）

## T2: driver.rs — `cmd_ci_init` 追加

- [x] `generate_ci_yaml` の直後に `pub fn cmd_ci_init(out_dir: Option<&str>, dry_run: bool)` を追加
- [x] `fav.toml` が存在すればプロジェクト名を取得（省略時 `"fav-project"`）
- [x] `--dry-run` 時は標準出力にプレビューを表示して return
- [x] 出力先: `<out-dir?>/.github/workflows/ci.yml`（デフォルト: cwd）
- [x] `write_text_file` でファイル書き出し、失敗時は `eprintln!` + exit 1

## T3: main.rs — `Some("ci")` アーム追加 + ヘルプ更新

- [x] `Some("deploy")` アームの直後に `Some("ci")` アームを追加
- [x] サブコマンド: `"init"` → `cmd_ci_init` 呼び出し
- [x] `"init"` の引数: `--out-dir` / `--dry-run`（未知フラグは error + exit 1）
- [x] サブコマンドなし → usage を stderr に出力して exit 1
- [x] 未知サブコマンド → `"error: unknown ci subcommand"` + exit 1
- [x] ヘルプテキストに `ci init` を追記
- [x] `use crate::driver::{..., cmd_ci_init, generate_ci_yaml}` を追加

## T4: driver.rs — v35300_ci_tests 追加

- [x] `v35300_tests` の `cargo_toml_version_is_35_2_0`（v35200_docker_tests 内）をスタブ化
- [x] `v35300_tests` の直後に `v35300_ci_tests` モジュールを追加（6 件）
  - [x] `cargo_toml_version_is_35_3_0`
  - [x] `ci_command_exists_in_main`
  - [x] `generate_ci_yaml_has_check_step`
  - [x] `generate_ci_yaml_has_lint_step`
  - [x] `generate_ci_yaml_has_test_step`
  - [x] `changelog_has_v35_3_0`

## T5: バージョン更新（T4 完了後、テスト前に実施）

- [x] `fav/Cargo.toml` バージョンを `35.3.0` に更新

## T6: テスト実行

- [x] `cargo test` 全通過（0 failures）— 2644 passed; 0 failed
- [x] v35300_ci_tests の 6 テストが pass

## T6b: CHANGELOG 更新（T6 完了後に実施）

- [x] `CHANGELOG.md` に `## [35.3.0]` エントリを追加

## T7: ドキュメント更新

- [x] `versions/v30-v35/v35.3.0/tasks.md` を COMPLETE ステータスに更新
- [x] （`versions/current.md` はマイナー版のため更新しない）
- [x] （`site/content/cookbook/github-actions-ci.mdx` などサイト MDX は v35.8.0 で追加予定のため本バージョンでは追加しない）

---

## 完了条件チェックリスト（spec.md 対応）

| # | spec.md 完了条件 | 確認方法 |
|---|---|---|
| 1 | `fav ci init` で `.github/workflows/ci.yml` が生成される | 手動確認（`cmd_ci_init` 実装確認）✅ |
| 2 | 生成 YAML に `check` / `lint` / `test` の 3 ステップが含まれる | `generate_ci_yaml_has_*` テスト（3 件）✅ |
| 3 | `--dry-run` でファイルを書かずにプレビューが表示される | `dry_run` 分岐実装確認 ✅ |
| 4 | `cargo test` が 0 failures（v35300_ci_tests 全 pass） | T6 実行結果 ✅ |
| 5 | `CHANGELOG.md` に `[35.3.0]` エントリが存在する | `changelog_has_v35_3_0` テスト ✅ |

---

## コードレビュー事前チェックリスト

- [x] `generate_ci_yaml` の引数 `_project_name` が意図通り未使用（`_` プレフィックスで Clippy 抑制済み）
- [x] `cmd_ci_init` の `--dry-run` 時にファイルを書かない
- [x] `write_text_file` の失敗時に exit 1 する
- [x] `Some("ci")` アームで未知フラグが `eprintln!` + exit 1 になっている
- [x] `cmd_ci_init` は `pub fn` として `driver.rs` に追加し、`use crate::driver::cmd_ci_init` でインポート済み
- [x] `fav/src/ci/` ディレクトリが存在しないことを確認（driver.rs に直接追加）

## コードレビュー対応（実施後に記録）

| 指摘 | 優先度 | 対応 |
|---|---|---|
| `generate_ci_yaml` が main.rs で未使用 import → Clippy `unused_imports` | HIGH | main.rs の use リストから `generate_ci_yaml` を削除 |
| `--out-dir` にパストラバーサルチェックなし | MED | `val.contains("..")` チェックを追加（eprintln + exit 1） |
| `_project_name` の将来利用意図が不明 | MED | `// TODO(v35.x): _project_name を使って workflow 名カスタマイズ` コメントを追加 |
