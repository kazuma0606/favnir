# v79.1.0 タスクリスト — 統合ショーケース基盤

Date: 2026-08-16
Status: COMPLETE

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `79.0.0` であることを確認
- [x] `cargo test` が全 pass（3787 tests = v79.0.0 完了後の実測ベース）であることを確認
- [x] `fav/tmp/hello.fav` が存在することを確認

---

## T1: ショーケースディレクトリ・ファイル作成

- [x] `infra/e2e-demo/favnir3-showcase/` ディレクトリを作成する
- [x] `infra/e2e-demo/favnir3-showcase/pipeline.fav` を作成する（`showcase_pipeline` 関数・骨格 4 ステージのコメント含む）
- [x] `infra/e2e-demo/favnir3-showcase/fav.toml` を作成する（`[project]` / `[schedule]` / `[effects.cached]` / `[effects.adaptive]` セクション含む）
- [x] `infra/e2e-demo/favnir3-showcase/contract.fav` を作成する（`ShowcaseContract3` 型 / `validate_showcase_contract` 関数含む）
- [x] `infra/e2e-demo/favnir3-showcase/README.md` を作成する（「Favnir 3.0」表記・実行手順含む）

---

## T2: CHANGELOG.md 更新（テスト追加より先）

- [x] `CHANGELOG.md` の先頭に v79.1.0 エントリを追加する（形式: `## [v79.1.0] — 2026-08-16 — 統合ショーケース基盤`）
- [x] Added セクション（4 ファイル）を含める
- [x] Tests セクション（2 件）を含める

---

## T3: driver.rs — テストモジュール追加

- [x] `fav/src/driver.rs` の末尾に `// --- v79.1.0: 統合ショーケース基盤 ---` コメントを追加する
- [x] `v791000_tests` モジュールを追加する（`use super::*` 必須）
- [x] `favnir3_showcase_structure_exists` テストを実装する
  - `pipeline.fav` に `showcase_pipeline` が含まれることを assert
  - `fav.toml` に `favnir3-showcase` が含まれることを assert
  - `contract.fav` に `ShowcaseContract3` が含まれることを assert
  - `README.md` に `Favnir 3.0` が含まれることを assert
- [x] `favnir3_showcase_contract_valid` テストを実装する
  - `contract.fav` に `ShowcaseContract3` が含まれることを assert
  - `contract.fav` に `validate_showcase_contract` が含まれることを assert
  - `contract.fav` に `temporal_enabled` が含まれることを assert
  - `contract.fav` に `execution_effects_enabled` が含まれることを assert
- [x] `cargo test v791000` で 2 件が pass することを確認する

---

## T4: Cargo.toml バージョン更新

- [x] `fav/Cargo.toml` の `version` を `"79.0.0"` → `"79.1.0"` に変更する
- [x] driver.rs 内の `79.0.0` バージョン文字列アサーションを `79.1.0` に一括更新（`replace_all: true`）
- [x] **replace_all 後に** `grep -c "79.0.0" /c/Users/yoshi/favnir/fav/src/driver.rs` を実行し **出力が 1** であることを確認する（Git Bash で実行）
  - 残るのは `// --- v79.0.0: Execution Effects 1.0 宣言 ★クリーンアップ ---` の 1 件のみ

---

## T5: versions/current.md 更新

- [x] `## 進行中バージョン` 欄を `**v79.1.0**（統合ショーケース基盤）` に更新する
- [x] `## 次に切る版` 欄を `**v79.2.0**（Temporal showcase パイプライン）` に更新する

---

## T6: 最終確認

- [x] `cargo test` が全 pass であることを確認する（3789 tests）
- [x] `cargo test v791000` で 2 件が pass することを確認する
- [x] `fav/Cargo.toml` のバージョンが `79.1.0` であることを確認する
- [x] `CHANGELOG.md` の先頭が `[v79.1.0]` であることを確認する
- [x] `versions/current.md` の「進行中バージョン」が v79.1.0 であることを確認する
- [x] `infra/e2e-demo/favnir3-showcase/` に 4 ファイルが存在することを確認する

---

## 完了チェックリスト

- [x] 全タスク（T0〜T6）が完了している
- [x] `favnir3_showcase_structure_exists` が pass
- [x] `favnir3_showcase_contract_valid` が pass
- [x] テスト総数: 3789（+2）
- [x] 新機能追加なし（インフラファイル + テストのみ）
- [x] site/ MDX 追加: 対象外
- [x] `changelog_has_v79_1_0` テストの追加: 対象外（x.0.0 宣言バージョンのみに追加する慣例）
