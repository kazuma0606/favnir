# Tasks: v48.4.0 — `fav install` コマンド（`[runes]` 対応）

Status: COMPLETE
Date: 2026-07-18

---

## T0 — 事前確認

- [x] `cargo test` 3051 passed, 0 failed を確認（ベース確認）
- [x] `driver.rs` に `install_rune_stubs` 関数が存在しないことを確認
- [x] `driver.rs` に `cmd_install_runes` 関数が存在しないことを確認
- [x] 既存の `cmd_install` が `[dependencies]` を読んでいることを確認（`[runes]` は読まない）

## T1 — `driver.rs` 関数追加

- [x] `install_rune_stubs(pkg_name: Option<&str>, root: &Path, runes: &HashMap<String, String>) -> Vec<String>` を `cmd_install` の直後に追加
  - [x] `runes/<name>/` ディレクトリを `create_dir_all` で作成
  - [x] `rune.toml` スタブを書き込み（存在しない場合のみ）
  - [x] インストールしたパッケージ名を Vec で返す
  - [x] `pub` 修飾子をつけること（テストから参照するため）
- [x] `cmd_install_runes(pkg_name: Option<&str>)` を `install_rune_stubs` の直後に追加
  - [x] `FavToml::find_root` → `FavToml::load` → `toml.runes` を参照
  - [x] `toml.runes.is_empty()` の場合は「No runes declared」を出力して return
  - [x] `install_rune_stubs` を呼び出して結果をコンソール出力

## T2 — `main.rs` 更新

- [x] import 行に `cmd_install_runes` を追加（`cmd_install` の直後）
- [x] `Some("install-rune")` アームを `Some("install")` の直後に追加

## T3 — `driver.rs` テスト追加・バージョン更新・完了

- [x] `v484000_tests` モジュールを `v483000_tests` の直前に追加（2テスト）
  - [x] `fav_install_creates_rune_dir`: `install_rune_stubs(Some("kafka"), ...)` → `runes/kafka/` と `rune.toml` が作成される
  - [x] `fav_install_all_from_toml`: `parse_fav_toml_pub` で `[runes] kafka/postgres` → `install_rune_stubs(None, ...)` → 両ディレクトリ存在・len == 2
- [x] `fav/Cargo.toml` version → `"48.4.0"`
- [x] `CHANGELOG.md` に v48.4.0 エントリ追加
- [x] `cargo test` 3053 passed, 0 failed（3051 + 2 件）
- [x] `cargo clippy -- -D warnings` クリーン
- [x] `versions/current.md` を v48.4.0（3053 tests）に更新、進行中バージョンを `v48.5.0` に更新
- [x] `versions/roadmap/roadmap-v48.1-v49.0.md` の v48.4.0 テスト数を実績値 3053 に更新（`roadmap-v45.1-v50.0.md` への反映は v49.0.0 時・変更不要）
- [x] tasks.md を COMPLETE に更新（T0〜T3 全 `[x]`）

---

> **注記**: 実際のダウンロード・バージョン解決は v48.4.0 のスコープ外（MVP スタブのみ）
> **注記**: マスターロードマップ（`roadmap-v45.1-v50.0.md`）への反映は v49.0.0 マイルストーン宣言時に実施
