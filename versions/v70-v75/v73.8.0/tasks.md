# v73.8.0 タスクリスト — GitHub Actions 公式 Action

Date: 2026-08-13
Status: 完了

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `73.7.0` であることを確認
- [x] `cargo test` が 3661 tests pass（0 failures）であることを確認
- [x] `driver.rs` に `v737000_tests` モジュールが存在することを確認
- [x] `driver.rs` に `v738000_tests` が未存在であることを確認
- [x] `.github/actions/setup-fav/` ディレクトリが未存在であることを確認

---

## T1: `.github/actions/setup-fav/action.yml` を作成

- [x] `.github/actions/setup-fav/` ディレクトリを作成した
- [x] `action.yml` を作成した（composite action 形式）
  - `name: Setup Favnir` を含む
  - `inputs.version`（required, default: latest）を含む
  - `runs.using: composite` を含む
  - OS/ARCH 判別・URL 生成・バイナリダウンロード手順を含む
  - `$GITHUB_PATH` への追加を含む

---

## T2: `.github/actions/setup-fav/README.md` を作成

- [x] `README.md` を作成した
  - basic 使用例（`uses: favnir/setup-fav@v1`）を含む
  - バージョン指定例を含む
  - CI バッジサンプルを含む
  - マトリックスビルドサンプル（ubuntu / macos / windows）を含む
  - `fav check` / `fav test` / `fav quality` / `fav audit` の使用例を含む

---

## T3: `GithubActionConfig` 構造体 + `format_github_action_url` 関数を追加

- [x] `driver.rs` に `// --- v73.8.0: GitHub Actions 公式 Action ---` セクションを追加した
- [x] `#[derive(Debug, Clone)] pub struct GithubActionConfig` を追加した（version / os / arch フィールド）
- [x] `pub fn format_github_action_url(config: &GithubActionConfig) -> String` を実装した
  - URL 形式: `https://github.com/favnir/favnir/releases/download/v{version}/fav-{os}-{arch}`
- [x] `cargo build` でエラーがないことを確認

---

## T4: `v738000_tests` モジュールを追加

- [x] `v737000_tests` の直後に `v738000_tests` モジュールを追加した
- [x] `use super::{GithubActionConfig, format_github_action_url}` を追加した
- [x] `github_action_setup_fav_action_yml_valid` テストを実装した
  - `include_str!("../../.github/actions/setup-fav/action.yml")` でファイルを読み込む
  - `"Setup Favnir"` を含むことを assert
  - `"version"` を含むことを assert
  - `"composite"` を含むことを assert
- [x] `github_action_fav_binary_url_format` テストを実装した
  - `GithubActionConfig { version: "75.0.0", os: "linux", arch: "x86_64" }` を構築
  - `format_github_action_url` の結果が `"v75.0.0"` / `"linux"` / `"x86_64"` を含むことを assert
  - URL が `"https://github.com/favnir/favnir/releases/download/"` で始まることを assert
- [x] `cargo test github_action` で 2 件 pass することを確認

---

## T5: バージョン更新

- [x] `fav/Cargo.toml` の `version = "73.7.0"` → `version = "73.8.0"` に変更した
- [x] `driver.rs` 内の `version = "73.7.0"` 参照を `version = "73.8.0"` に replace_all した
- [x] 残存 `73.7.0` はコメント・セクションヘッダーのみで意図的保持を確認
- [x] `cargo build` でエラーがないことを確認
- [x] `fav/Cargo.lock` が `version = "73.8.0"` を含むことを確認

---

## T5.5: バージョン更新後の部分テスト再確認

- [x] T5 のバージョン更新後も `cargo test github_action` で 2 件 pass することを確認

---

## T6: 全体テスト確認

- [x] `cargo test` 全体で 3663 tests pass（0 failures）であることを確認

---

## T7: `CHANGELOG.md` 更新

- [x] `## [v73.8.0]` エントリを先頭に追加した
  - Added: `action.yml` / `README.md` / `GithubActionConfig` / `format_github_action_url`
  - Tests: 2 件、合計テスト数 3663（+2）

---

## T8: `versions/current.md` 更新

- [x] 「最終更新」を `2026-08-13 (v73.8.0)` に更新した
- [x] 「進行中バージョン」を `v73.8.0` に更新した
- [x] 「次に切る版」を `v73.9.0` に更新した

---

## T9: 最終確認（T7・T8 完了後）

- [x] `cargo test github_action` で 2 件 pass することを確認
- [x] `cargo test` 全体で 3663 tests pass（0 failures）であることを確認
- [x] `fav/Cargo.toml` のバージョンが `73.8.0` であることを確認
- [x] `.github/actions/setup-fav/action.yml` が存在することを確認
- [x] `.github/actions/setup-fav/README.md` が存在することを確認
- [x] `CHANGELOG.md` に `[v73.8.0]` エントリが存在することを確認
- [x] `versions/current.md` の「進行中バージョン」が `v73.8.0` であることを確認

---

## スコープ外（明示的除外）

- 実際の GitHub Releases へのバイナリアップロード
- GitHub Marketplace への公開
- Action のバイナリ実行テスト（ダウンロード先は stub）
