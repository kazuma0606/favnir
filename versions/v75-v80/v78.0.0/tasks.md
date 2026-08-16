# v78.0.0 タスクリスト — Verifiable Pipelines 宣言 ★クリーンアップ

Date: 2026-08-16
Status: COMPLETE

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `77.9.0` であることを確認
- [x] `fav/tmp/hello.fav` が存在することを確認（cargo clean 前に必須確認）
- [x] `cargo test` が全 pass（3756 tests）であることを確認

---

## T1: cargo clean

- [x] `fav/tmp/hello.fav` の内容を確認する（必須: `fn add(a: Int, b: Int) -> Int { a + b }` + `fn main() -> Bool { add(1, 2) == 3 }`）
- [x] `cargo clean` を実施する
- [x] `fav/tmp/hello.fav` が残っていることを確認する（過去に cargo clean で消えた実績あり）
- [x] 消えていた場合は内容を復元する: `fn add(a: Int, b: Int) -> Int { a + b }` + 改行 + `fn main() -> Bool { add(1, 2) == 3 }`

---

## T2: MILESTONE.md 更新

- [x] `MILESTONE.md` の先頭（`## v77.0.0` エントリの前）に v78.0.0 宣言エントリを追加する
- [x] エントリに宣言文・v77.1〜v77.9 の達成内容一覧を含める（spec.md 参照）

---

## T3: README.md 更新

- [x] `README.md` の `## v77.0` セクションの前に `## v78.0 — Verifiable Pipelines 宣言（2026-08-16）` セクションを追加する
- [x] `Verifiable Pipelines` / `v78.0` のキーワードが含まれていることを確認する

---

## T4: CHANGELOG.md 更新（テスト追加より先）

- [x] `CHANGELOG.md` の先頭に v78.0.0 エントリを追加する（形式: `## [v78.0.0] — 2026-08-16 — Verifiable Pipelines 宣言 ★クリーンアップ`）
- [x] Milestone セクション（宣言文）と Tests セクション（4 件）を含める

---

## T5: driver.rs — テストモジュール追加

- [x] `fav/src/driver.rs` の末尾に `// --- v78.0.0: Verifiable Pipelines 宣言 ---` コメントを追加する
- [x] `v78000_tests` モジュールを追加する（`use super::*` 不要）
- [x] `cargo_toml_version_is_78_0_0` テストを実装する（`include_str!("../Cargo.toml")` で `"version = \"78.0.0\""` を確認）
- [x] `changelog_has_v78_0_0` テストを実装する（`include_str!("../../CHANGELOG.md")` で `"[v78.0.0]"` を確認）
- [x] `milestone_has_verifiable_pipelines` テストを実装する（`include_str!("../../MILESTONE.md")` で `"Verifiable Pipelines"` を確認）
- [x] `readme_mentions_verifiable_pipelines` テストを実装する（`include_str!("../../README.md")` で `"Verifiable Pipelines"` と `"v78.0"` を確認）
- [x] **この時点では `cargo test v78000` を実行しない** — `cargo_toml_version_is_78_0_0` が Cargo.toml バージョン更新前（77.9.0）のため必ず失敗する。T6 完了後に確認する。

---

## T6: Cargo.toml バージョン更新

- [x] `fav/Cargo.toml` の `version` を `"77.9.0"` → `"78.0.0"` に変更する
- [x] `driver.rs` 内の `77.9.0` バージョン文字列アサーションを `78.0.0` に一括更新（`replace_all: true` で全件置換）
- [x] **replace_all 後に** `grep "v77.9.0" fav/src/driver.rs` を実行し、`// --- v77.9.0: 安定化・コードフリーズ ---` が残っていることを確認する（`v78.0.0` に書き換わっていた場合は手動で `v77.9.0` に戻す）
- [x] `cargo test v78000` で 4 件が pass することを確認する（T6 完了後に初めて全 4 件 pass 可能）

---

## T7: versions/current.md 更新

- [x] `## 進行中バージョン` 欄を `**v78.0.0**（Verifiable Pipelines 宣言 ★クリーンアップ）` に更新する
- [x] `## 次に切る版` 欄を `**v78.1.0**（次スプリント開始）` に更新する

---

## T8: 最終確認

- [x] `cargo test` が全 pass であることを確認する（3760 tests）
- [x] `cargo test v78000` で 4 件が pass することを確認する
- [x] `fav/Cargo.toml` のバージョンが `78.0.0` であることを確認する
- [x] `CHANGELOG.md` の先頭が `[v78.0.0]` であることを確認する
- [x] `MILESTONE.md` に `Verifiable Pipelines` が含まれていることを確認する
- [x] `README.md` に `Verifiable Pipelines` と `v78.0` が含まれていることを確認する
- [x] `versions/current.md` の「進行中バージョン」が `v78.0.0` であることを確認する

---

## 完了チェックリスト

- [x] 全タスク（T0〜T8）が完了している
- [x] `cargo_toml_version_is_78_0_0` が pass
- [x] `changelog_has_v78_0_0` が pass
- [x] `milestone_has_verifiable_pipelines` が pass
- [x] `readme_mentions_verifiable_pipelines` が pass
- [x] テスト総数: 3760（+4）
- [x] `cargo clean` 実施済み
- [x] `fav/tmp/hello.fav` 存在確認（または復元）済み
- [x] `versions/current.md` 更新済み
- [x] site/ MDX 追加: 本バージョンでは対象外（宣言バージョン）
