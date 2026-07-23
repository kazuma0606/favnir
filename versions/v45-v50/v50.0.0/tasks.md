# Tasks: v50.0.0 — Production 2.0 宣言 ★クリーンアップ

Status: COMPLETE
Date: 2026-07-18

---

## T0 — 事前確認

- [x] `cargo test` 3087 passed, 0 failed を確認（ベース確認、v49.9.0 COMPLETE 後）
- [x] `README.md` に `"Language Maturity"` が含まれていないことを確認（追記対象）
- [x] `MILESTONE.md` に `"Language Maturity"` が含まれることを確認（v49.8.0 で追加済み）
- [x] `CHANGELOG.md` に `"v50.0.0"` が含まれていないことを確認（追記対象）
- [x] `v499000_tests` モジュールが `driver.rs` に存在することを確認（挿入位置の前提）

## T1 — `README.md` 更新

- [x] `README.md` に `"Language Maturity"` という文字列を追加
- [x] `README.md` に `"v50"` および Production 2.0 への言及を追加

## T2 — `v50000_tests` 追加

- [x] `v50000_tests` モジュールを `v499000_tests` の直前に追加（4 テスト）
- [x] 挿入後 `grep -n v50000_tests src/driver.rs` で存在確認
  - [x] `cargo_toml_version_is_50_0_0`: `version = "50.0.0"` を含むことを確認
  - [x] `changelog_has_v50_0_0`: `"v50.0.0"` を含むことを確認
  - [x] `milestone_has_language_maturity`: `"Language Maturity"` と `"v50.0.0"` を含むことを確認
  - [x] `readme_mentions_language_maturity`: `"Language Maturity"` を含むことを確認

## T3 — バージョン更新・CHANGELOG 追加

- [x] `fav/Cargo.toml` version → `"50.0.0"`（先に更新）
- [x] `CHANGELOG.md` に v50.0.0 エントリ追加（Production 2.0 宣言文を含める）
- [x] `cargo test` 3091 passed, 0 failed
- [x] `v499000_tests::cargo_toml_version_is_49_9_0` の陳腐化アサーションを `name = "fav"` チェックに修正

## T4 — ★クリーンアップ

- [x] `cargo clean` 実施（29.5 GiB 削除）
- [x] `fav/tmp/hello.fav` の存在確認 — 存在していた（復元不要）
- [x] `cargo test` 再実行 — 3091 passed, 0 failed（クリーンビルド後）

## T5 — 最終更新・完了

- [x] `cargo clippy -- -D warnings` クリーン（最終確認）
- [x] `versions/current.md` を v50.0.0（3091 tests）に更新
- [x] `versions/roadmap/roadmap-v49.1-v50.0.md` の v50.0.0 実績を 3091 に記入
- [x] tasks.md を COMPLETE に更新（T0〜T5 全 `[x]`）

---

> **注記**: `cargo_toml_version_is_49_9_0`（v499000_tests）は v50.0.0 への version bump により陳腐化。
> アサーションを `name = "fav"` チェックに変更して全通過を維持した。
>
> **code-reviewer 対応**:
> - [HIGH] README.md バージョン順序修正（v44→v50→v49→...→v45 の正順）
> - [MED] MILESTONE.md `（予定）` → `（2026-07-18）` に修正
> - [MED] `changelog_has_v50_0_0` アサーションを `"## [v50.0.0]"` に強化（偽陽性防止）
> - [LOW] `cargo_toml_version_is_49_9_0` を `name = "fav"` チェックに変更（上記注記と同一）
