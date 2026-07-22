# Tasks: v51.0.0 — Developer Experience 3.0 宣言 ★クリーンアップ

Status: COMPLETE
Date: 2026-07-19

---

## T0 — 事前確認

- [x] `cargo test` 3109 passed, 0 failed を確認（ベース確認）
- [x] `cargo clippy -- -D warnings` クリーンであることを確認（ベース）
- [x] `MILESTONE.md` に `"Developer Experience 3.0"` が含まれないことを確認（追加対象）
- [x] `README.md` に `"DX 3.0"` / `"Developer Experience 3.0"` が含まれないことを確認（追加対象）
- [x] `v509000_tests::cargo_toml_version_is_50_9_0` が存在することを確認（削除対象）
- [x] `v509000_tests::code_freeze_v50_9_0` が存在することを確認（削除対象）
- [x] `include_str!` パスの確認:
  - [x] `../../CHANGELOG.md` → `favnir/CHANGELOG.md`
  - [x] `../../MILESTONE.md` → `favnir/MILESTONE.md`
  - [x] `../../README.md` → `favnir/README.md`

## T1 — `MILESTONE.md` 更新

- [x] v51.0.0 エントリを先頭（v50.0.0 エントリの直前）に挿入
  - [x] `"Developer Experience 3.0"` を含む
  - [x] 宣言文（「全エラーコードに修正提案が付き...」）を含む
  - [x] `"v51.0"` を含む（`dx3_milestone_declared` テスト用 — AND 条件）
  - [x] `"診断は開発者の思考を止めない"` または `"Developer Experience 3.0"` を含む（OR 条件 — どちらか一方で可）

## T2 — `README.md` 更新

- [x] マイルストーンセクションに `"Developer Experience 3.0"` を含む行を追加
  - [x] `content.contains("DX 3.0") || content.contains("Developer Experience 3.0")` が真になること

## T3 — `driver.rs` — `v51000_tests` 追加・削除

- [x] `v51000_tests` モジュールを `v509000_tests` の直前に追加（6 件）:
  - [x] `cargo_toml_version_is_51_0_0`: version = "51.0.0" を assert
  - [x] `changelog_has_v51_0_0`: CHANGELOG.md に "v51.0.0" を assert
  - [x] `milestone_has_dx3`: MILESTONE.md に "Developer Experience 3.0" を assert
  - [x] `readme_mentions_dx3`: README.md に "DX 3.0" または "Developer Experience 3.0" を assert
  - [x] `dx3_milestone_declared`: MILESTONE.md に "v51.0" かつ宣言文を assert
  - [x] `code_freeze_v51_0_0`: Cargo.toml に "51.0.0" を assert（code_freeze_v50_9_0 の後継）
- [x] `v509000_tests::cargo_toml_version_is_50_9_0` を削除
- [x] `v509000_tests::code_freeze_v50_9_0` を削除
- [x] `v509000_tests::dx3_overview_doc_exists` は保持

## T4 — バージョン更新・テスト確認

- [x] `fav/Cargo.toml` version → `"51.0.0"`
- [x] `cargo test` 3113 passed, 0 failed
- [x] `cargo clippy -- -D warnings` クリーン

## T5 — ★クリーンアップ（`cargo clean`）

- [x] `cargo clean` 実施
- [x] `fav/tmp/hello.fav` が存在することを確認（消えていたら復元）
  - 内容: `fn add(a: Int, b: Int) -> Int { a + b }` + `fn main() -> Bool { add(1, 2) == 3 }`
- [x] `cargo test` 3113 passed, 0 failed（cargo clean 後に再確認）

## T6 — 完了処理

- [x] `CHANGELOG.md` に v51.0.0 エントリ追加
- [x] `versions/current.md` を v51.0.0（3113 tests）に更新
- [x] `versions/roadmap/roadmap-v50.1-v51.0.md` の v51.0.0 実績欄を更新
- [x] tasks.md を COMPLETE に更新（T0〜T6 全 `[x]`）
