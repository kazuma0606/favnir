# v79.5.0 タスクリスト — Execution Effects showcase パイプライン

Date: 2026-08-16
Status: COMPLETE

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `79.4.0` であることを確認
- [x] `cargo test` が全 pass（3795 tests = v79.4.0 完了後の実測ベース）であることを確認
- [x] `fav/tmp/hello.fav` が存在することを確認
- [x] `infra/e2e-demo/favnir3-showcase/pipeline.fav` が存在し `load_with_provenance` を含むことを確認
- [x] `infra/e2e-demo/favnir3-showcase/fav.toml` に `[effects.cached]` / `[effects.adaptive]` セクションが存在することを確認（v79.1.0 追加済み）
- [x] `pipeline.fav` に `join_stage` がまだ含まれていないことを確認（重複追加防止）
- [x] `pipeline.fav` に `!Adaptive` / `!Cached` がまだ含まれていないことを確認（重複追加防止）

---

## T1: pipeline.fav 更新

- [x] `infra/e2e-demo/favnir3-showcase/pipeline.fav` の `load_with_provenance` と `showcase_pipeline` の間に `// --- Stage 3: Execution Effects（v78.x）---` コメントと `join_stage` 関数を追加する
  - `fn join_stage(ctx: AppCtx, customers: List<Row>, orders: List<Row>) -> Result<List<Row>, String> !Adaptive !Cached` シグネチャ
  - `bind joined <- customers |> join(orders, on: "id")` を含む
  - `!Adaptive` / `!Cached` コメントを含む
  - `Result.ok(joined)` を含む
- [x] `showcase_pipeline` のコメント行を更新（Stage 3 = Verifiable、Stage 4 = Execution Effects に修正）

---

## T2: CHANGELOG.md 更新（テスト追加より先）

- [x] `CHANGELOG.md` の先頭に v79.5.0 エントリを追加する（形式: `## [v79.5.0] — 2026-08-16 — Execution Effects showcase パイプライン`）
- [x] Added セクション（pipeline.fav join_stage 追加）を含める
- [x] Tests セクション（2 件）を含める

---

## T3: driver.rs — テストモジュール追加

- [x] `fav/src/driver.rs` の末尾に `// --- v79.5.0: Execution Effects showcase パイプライン ---` コメントを追加する
- [x] `v795000_tests` モジュールを追加する（`use super::*` 不要）
- [x] モジュール先頭に `const PIPELINE: &str = include_str!("../../infra/e2e-demo/favnir3-showcase/pipeline.fav");` を配置する
- [x] `showcase_execution_cached_effect` テストを実装する
  - `join_stage` / `!Cached` を assert
- [x] `showcase_execution_adaptive_effect` テストを実装する
  - `!Adaptive` / `join(orders, on:` を assert
- [x] `cargo test v795000` で 2 件が pass することを確認する

---

## T4: Cargo.toml バージョン更新

- [x] `fav/Cargo.toml` の `version` を `"79.4.0"` → `"79.5.0"` に変更する
- [x] driver.rs 内の escaped `\"79.4.0\"` を `\"79.5.0\"` に一括更新（sed）
- [x] driver.rs 内の unescaped エラーメッセージ `79.4.0` を `79.5.0` に更新する
- [x] **更新後に** `grep -c "79\.4\.0" /c/Users/yoshi/favnir/fav/src/driver.rs` を実行し **出力が 1** であることを確認する
  - 残るのは `// --- v79.4.0: Verifiable showcase パイプライン ---` コメント行の 1 件のみ

---

## T5: versions/current.md 更新

- [x] `## 進行中バージョン` 欄を `**v79.5.0**（Execution Effects showcase パイプライン）` に更新する
- [x] `## 次に切る版` 欄を `**v79.6.0**（ドッグフーディング強化）` に更新する

---

## T6: 最終確認

- [x] `cargo test` が全 pass であることを確認する（3797 tests）
- [x] `cargo test v795000` で 2 件が pass することを確認する
- [x] `fav/Cargo.toml` のバージョンが `79.5.0` であることを確認する
- [x] `fav/Cargo.lock` が cargo test 実行時に自動更新されていることを確認する
- [x] `CHANGELOG.md` の先頭が `[v79.5.0]` であることを確認する
- [x] `pipeline.fav` に `join_stage` / `!Adaptive` / `!Cached` が含まれることを確認する

---

## 完了チェックリスト

- [x] 全タスク（T0〜T6）が完了している
- [x] `showcase_execution_cached_effect` が pass
- [x] `showcase_execution_adaptive_effect` が pass
- [x] テスト総数: 3797（+2）
- [x] `CHANGELOG.md` の先頭が `[v79.5.0]` であることを確認済み
- [x] `fav/Cargo.toml` version = "79.5.0" に更新済み
- [x] `versions/current.md` が v79.5.0 に更新済み
- [x] `changelog_has_v79_5_0` テストの追加: 対象外（x.0.0 宣言バージョンのみに追加する慣例）
- [x] `site/content/docs/v3/execution-effects.mdx` 追加: 対象外（v79.8.0 で実施）
