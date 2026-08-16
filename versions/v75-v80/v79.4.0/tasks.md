# v79.4.0 タスクリスト — Verifiable showcase パイプライン

Date: 2026-08-16
Status: COMPLETE

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `79.3.0` であることを確認
- [x] `cargo test` が全 pass（3793 tests = v79.3.0 完了後の実測ベース）であることを確認
- [x] `fav/tmp/hello.fav` が存在することを確認
- [x] `infra/e2e-demo/favnir3-showcase/contract.fav` が存在し `ShowcaseContract3` を含むことを確認
- [x] `contract.fav` に `Favnir3ShowcaseContract` がまだ含まれていないことを確認（重複追加防止）

---

## T1: contract.fav 更新

- [x] `infra/e2e-demo/favnir3-showcase/contract.fav` の末尾に `// --- Verifiable セクション（v77.x）---` コメントと `Favnir3ShowcaseContract` 不変条件ブロックを追加する
  - `contract Favnir3ShowcaseContract { ... }` ブロック
  - `invariant: output.row_count <= input.row_count` を含む
  - `invariant: SUM(output.amount) >= 0.0` を含む
  - `probabilistic_invariant score_dist:` ブロックを含む
  - `confidence: 0.95, sample_size: 1000` を含む

---

## T2: CHANGELOG.md 更新（テスト追加より先）

- [x] `CHANGELOG.md` の先頭に v79.4.0 エントリを追加する（形式: `## [v79.4.0] — 2026-08-16 — Verifiable showcase パイプライン`）
- [x] Added セクション（contract.fav 不変条件追加）を含める
- [x] Tests セクション（2 件）を含める

---

## T3: driver.rs — テストモジュール追加

- [x] `fav/src/driver.rs` の末尾に `// --- v79.4.0: Verifiable showcase パイプライン ---` コメントを追加する
- [x] `v794000_tests` モジュールを追加する（`use super::*` 不要）
- [x] モジュール先頭に `const CONTRACT: &str = include_str!("../../infra/e2e-demo/favnir3-showcase/contract.fav");` を配置する
- [x] `showcase_verifiable_invariants_declared` テストを実装する
  - `Favnir3ShowcaseContract` / `invariant` / `row_count` を assert
- [x] `showcase_verifiable_probabilistic_contract` テストを実装する
  - `probabilistic_invariant` / `confidence` / `sample_size` を assert
- [x] `cargo test v794000` で 2 件が pass することを確認する

---

## T4: Cargo.toml バージョン更新

- [x] `fav/Cargo.toml` の `version` を `"79.3.0"` → `"79.4.0"` に変更する
- [x] driver.rs 内の escaped `\"79.3.0\"` を `\"79.4.0\"` に一括更新（sed）
- [x] driver.rs 内の unescaped エラーメッセージ `79.3.0` を `79.4.0` に更新する
- [x] **更新後に** `grep -c "79\.3\.0" /c/Users/yoshi/favnir/fav/src/driver.rs` を実行し **出力が 1** であることを確認する
  - 残るのは `// --- v79.3.0: Provenance showcase パイプライン ---` コメント行の 1 件のみ

---

## T5: versions/current.md 更新

- [x] `## 進行中バージョン` 欄を `**v79.4.0**（Verifiable showcase パイプライン）` に更新する
- [x] `## 次に切る版` 欄を `**v79.5.0**（Execution Effects showcase パイプライン）` に更新する

---

## T6: 最終確認

- [x] `cargo test` が全 pass であることを確認する（3795 tests）
- [x] `cargo test v794000` で 2 件が pass することを確認する
- [x] `fav/Cargo.toml` のバージョンが `79.4.0` であることを確認する
- [x] `fav/Cargo.lock` が cargo test 実行時に自動更新されていることを確認する
- [x] `CHANGELOG.md` の先頭が `[v79.4.0]` であることを確認する
- [x] `contract.fav` に `Favnir3ShowcaseContract` / `invariant` / `probabilistic_invariant` が含まれることを確認する

---

## 完了チェックリスト

- [x] 全タスク（T0〜T6）が完了している
- [x] `showcase_verifiable_invariants_declared` が pass
- [x] `showcase_verifiable_probabilistic_contract` が pass
- [x] テスト総数: 3795（+2）
- [x] `CHANGELOG.md` の先頭が `[v79.4.0]` であることを確認済み
- [x] `fav/Cargo.toml` version = "79.4.0" に更新済み
- [x] `versions/current.md` が v79.4.0 に更新済み
- [x] `changelog_has_v79_4_0` テストの追加: 対象外（x.0.0 宣言バージョンのみに追加する慣例）
