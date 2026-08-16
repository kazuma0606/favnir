# v80.0.0 タスクリスト — Favnir 3.0 宣言 ★クリーンアップ

Date: 2026-08-16
Status: COMPLETE

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `79.9.0` であることを確認
- [x] `cargo test` が全 pass（3805 tests = v79.9.0 完了後の実測ベース）であることを確認
- [x] `fav/tmp/hello.fav` が存在することを確認

---

## T1: cargo clean

- [x] `cd /c/Users/yoshi/favnir/fav && cargo clean` を実行する（29.2GiB 削除）
- [x] `fav/tmp/hello.fav` の存在を確認する（消えていなかった）

---

## T2: CHANGELOG.md 更新（テスト追加より先）

- [x] `CHANGELOG.md` の先頭に v80.0.0 エントリを追加する（形式: `## [v80.0.0] — 2026-08-16 — Favnir 3.0 宣言 ★クリーンアップ`）
- [x] Declaration セクション（全スプリント完了宣言）を含める
- [x] Cleanup セクション（cargo clean / MILESTONE / README）を含める
- [x] Tests セクション（4 件）を含める

---

## T3: MILESTONE.md 更新

- [x] `MILESTONE.md` を読む
- [x] `MILESTONE.md` に Favnir 3.0 宣言セクションを追記する
- [x] 「Favnir 3.0」という文字列が含まれていることを確認する

---

## T4: README.md 更新

- [x] `README.md` を読む
- [x] `README.md` に v80.0.0 達成（Favnir 3.0）を追記する
- [x] 「Favnir 3.0」という文字列が含まれていることを確認する

---

## T5: driver.rs — テストモジュール追加

- [x] `fav/src/driver.rs` の末尾に `// --- v80.0.0: Favnir 3.0 宣言 ★クリーンアップ ---` コメントを追加する
- [x] `v80000_tests` モジュールを追加する（`use super::*` 不要）
- [x] `const CARGO_TOML` / `const CHANGELOG` / `const MILESTONE` / `const README` を配置する
  - `CARGO_TOML`: `include_str!("../Cargo.toml")`（`../../` は fav/ の上の階層で存在しないため `../` を使用）
  - `CHANGELOG`: `include_str!("../../CHANGELOG.md")`
  - `MILESTONE`: `include_str!("../../MILESTONE.md")`
  - `README`: `include_str!("../../README.md")`
- [x] `cargo_toml_version_is_80_0_0` テストを実装する
- [x] `changelog_has_v80_0_0` テストを実装する
- [x] `milestone_has_favnir_3` テストを実装する
- [x] `readme_mentions_favnir_3` テストを実装する
- [x] `cargo test v80000` で 4 件が pass することを確認する

---

## T6: Cargo.toml バージョン更新

- [x] `fav/Cargo.toml` の `version` を `"79.9.0"` → `"80.0.0"` に変更する
- [x] driver.rs 内の escaped `\"79.9.0\"` を `\"80.0.0\"` に一括更新（sed）
- [x] driver.rs 内の unescaped エラーメッセージ `79.9.0` を `80.0.0` に更新する
- [x] **更新後に** `grep -c "79\.9\.0" /c/Users/yoshi/favnir/fav/src/driver.rs` を実行し **出力が 1** であることを確認する
  - 残るのは `// --- v79.9.0: 安定化・コードフリーズ ---` コメント行の 1 件のみ

---

## T7: versions/current.md 更新

- [x] `## 進行中バージョン` 欄を `**v80.0.0**（Favnir 3.0 宣言 ★クリーンアップ — 完了）` に更新する
- [x] `## 次に切る版` 欄を `**v80.1.0**（次フェーズ）` に更新する
- [x] `## 最新安定版` 欄を `**v80.0.0** — Favnir 3.0 宣言 ★クリーンアップ — 3809 tests` に更新する

---

## T8: ロードマップ更新

- [x] `versions/roadmap/roadmap-v79.1-v80.0.md` の v80.0.0 スプリントに完了コメントを追記する

---

## T9: 最終確認

- [x] `cargo test` が全 pass であることを確認する（3809 tests）
- [x] `cargo test v80000` で 4 件が pass することを確認する
- [x] `fav/Cargo.toml` のバージョンが `80.0.0` であることを確認する
- [x] `fav/Cargo.lock` が cargo test 実行時に自動更新されていることを確認する
- [x] `CHANGELOG.md` の先頭が `[v80.0.0]` であることを確認する

---

## 完了チェックリスト

- [x] 全タスク（T0〜T9）が完了している
- [x] `cargo_toml_version_is_80_0_0` が pass
- [x] `changelog_has_v80_0_0` が pass
- [x] `milestone_has_favnir_3` が pass
- [x] `readme_mentions_favnir_3` が pass
- [x] テスト総数: 3809（+4）
- [x] `CHANGELOG.md` の先頭が `[v80.0.0]` であることを確認済み
- [x] `fav/Cargo.toml` version = "80.0.0" に更新済み
- [x] `versions/current.md` が v80.0.0 に更新済み
- [x] `v80000_tests` に 4 件すべてのテストが追加されていることを確認した

## 実装メモ

- `CARGO_TOML` の include_str! パス: `"../../Cargo.toml"` は fav/ の2階層上（存在しない） → `"../Cargo.toml"` が正しい（fav/src/ → fav/Cargo.toml）
- spec.md / plan.md も同様に修正済み
