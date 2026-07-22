# Tasks: v53.0.0 — Data Quality & Observability 2.0 宣言

Status: COMPLETE
Date: 2026-07-22

---

## T0 — 事前確認

- [x] `cargo test` 3156 passed, 0 failed を確認（ベース確認）
- [x] `cargo clippy -- -D warnings` クリーンであることを確認
- [x] `MILESTONE.md` に `"Data Quality & Observability 2.0"` が**存在しない**ことを確認:
  - [x] `rg "Data Quality & Observability 2.0" MILESTONE.md` → 0 件
- [x] `README.md` に `"Data Quality"` が**存在しない**ことを確認:
  - [x] 既に "Data Quality First"（v37.0）が存在 → テストは既に充足可能。v53.0 の明示的言及を追加する方針で継続
- [x] `CHANGELOG.md` に `"v53.0.0"` が**存在しない**ことを確認:
  - [x] `rg "v53.0.0" CHANGELOG.md` → 0 件
- [x] `driver.rs` に `v53000_tests` が**存在しない**ことを確認:
  - [x] `rg -n "v53000_tests" fav/src/driver.rs` → 0 件
- [x] `driver.rs` に `v52900_tests` が存在することを確認（挿入位置の確認）:
  - [x] `rg -n "v52900_tests" fav/src/driver.rs` → 行 47565 を特定
- [x] `include_str!` パスの整合性確認:
  - [x] `"../Cargo.toml"` → `fav/Cargo.toml` ✓
  - [x] `"../../CHANGELOG.md"` → `favnir/CHANGELOG.md` ✓
  - [x] `"../../MILESTONE.md"` → `favnir/MILESTONE.md` ✓
  - [x] `"../../README.md"` → `favnir/README.md` ✓
- [x] `Cargo.toml` の現在バージョンが `52.9.0` であることを確認
- [x] `fav/tmp/hello.fav` が存在することを確認（cargo clean 前のバックアップ確認）:
  - [x] 内容: `fn add(a: Int, b: Int) -> Int { a + b }` + `fn main() -> Bool { add(1, 2) == 3 }`

---

## T1 — `MILESTONE.md` 更新

- [x] `MILESTONE.md` 先頭に v53.0.0 エントリを追加:
  - [x] `"Data Quality & Observability 2.0"` キーワードを含む（テスト要件）
  - [x] 宣言文（4行）を含む
  - [x] 日付 2026-07-22 を含む

---

## T2 — `README.md` 更新

- [x] `README.md` に v53.0.0 / "Data Quality" の言及を追加:
  - [x] `"Data Quality"` キーワードを含む（テスト要件: `src.contains("Data Quality")`）
  - [x] v53.0 マイルストーン宣言文を v52.0 の直前に追記

---

## T3 — `driver.rs` — `v53000_tests` 追加

- [x] `rg -n "v52900_tests" fav/src/driver.rs` で挿入位置（行 47565）を確認
- [x] `v52900_tests` モジュールの直前に `v53000_tests` を追加:
  - [x] `cargo_toml_version_is_53_0_0` テスト:
    - [x] `include_str!("../Cargo.toml")` を使用
    - [x] `src.contains("version = \"53.0.0\"")` を assert
  - [x] `changelog_has_v53_0_0` テスト:
    - [x] `include_str!("../../CHANGELOG.md")` を使用
    - [x] `content.contains("v53.0.0")` を assert
  - [x] `milestone_has_data_quality` テスト:
    - [x] `include_str!("../../MILESTONE.md")` を使用
    - [x] `content.contains("Data Quality & Observability 2.0")` を assert
  - [x] `readme_mentions_data_quality` テスト:
    - [x] `include_str!("../../README.md")` を使用
    - [x] `content.contains("Data Quality")` を assert
- [x] `cargo build` → コンパイルエラーなし確認

---

## T4 — `fav/Cargo.toml` 更新 + `CHANGELOG.md` 更新 + テスト実行

- [x] `version = "52.9.0"` → `version = "53.0.0"` に変更
- [x] `CHANGELOG.md` に v53.0.0 エントリを追加（`"v53.0.0"` キーワードを含む）
- [x] `v52900_tests::cargo_toml_version_is_52_9_0` のアサートを削除（バージョンバンプ対応）:
  - [x] コメント「Version bump is tested in v53000_tests::cargo_toml_version_is_53_0_0.」に置換
- [x] `cargo test -j 8 -- --test-threads=8` 実行 → 3160 passed, 0 failed を確認（≥ 3157 ✓）
- [x] `cargo clippy -- -D warnings` クリーンを確認

---

## T5 — ★クリーンアップ

- [x] `cargo clean` 実行（33.5GiB 削除）
- [x] `fav/tmp/hello.fav` の状態確認:
  - [x] `ls fav/tmp/hello.fav` → 存在する（`cargo clean` は `target/` のみ削除のため影響なし）
- [x] `cargo test -j 8 -- --test-threads=8` 再実行 → 3160 passed, 0 failed を確認:
  - [x] `bootstrap_c2_artifact_roundtrip` が pass することを確認

---

## T6 — 後処理

- [x] `versions/current.md` を v53.0.0（3160 tests）に更新
- [x] `roadmap-v52.1-v53.0.md` の v53.0.0 実績欄を更新（未実施 → COMPLETE）:
  - [x] 実績テスト数を記録（3160）
  - [x] ≥ 3157 の条件を満たすことを確認（3160 ≥ 3157 ✓）
- [x] tasks.md を COMPLETE に更新（T0〜T6 全 `[x]`）
- [x] spec.md の全完了条件（v52.1〜v52.9 動作 / テスト ≥ 3157 / v53000_tests 4件 / MILESTONE.md / cargo clean）を充足していることを最終確認
