# v79.2.0 タスクリスト — Temporal showcase パイプライン

Date: 2026-08-16
Status: COMPLETE

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `79.1.0` であることを確認
- [x] `cargo test` が全 pass（3789 tests = v79.1.0 完了後の実測ベース）であることを確認
- [x] `fav/tmp/hello.fav` が存在することを確認
- [x] `infra/e2e-demo/favnir3-showcase/pipeline.fav` が存在することを確認

---

## T1: pipeline.fav 更新

- [x] `infra/e2e-demo/favnir3-showcase/pipeline.fav` に Temporal ステージコメントと `load_with_freshness` 関数を追加する
  - `// --- Stage 1: Temporal（v75.x）---` コメント
  - `fn load_with_freshness(ctx: AppCtx) -> Result<List<Row>, String>` 関数
  - `bind snapshot <- AsOfQuery { table: "orders", as_of_ts: ctx.run_ts }` を含む
  - `bind _ <- FreshnessPolicy.check(snapshot, max_age: Duration.hours(1))` を含む
  - `bind history <- apply_scd2_update(existing_customers, new_data, ctx.run_ts)` を含む
- [x] `showcase_pipeline` のコメントを「Stage 1: load_with_freshness で実装済み」に更新する

---

## T2: CHANGELOG.md 更新（テスト追加より先）

- [x] `CHANGELOG.md` の先頭に v79.2.0 エントリを追加する（形式: `## [v79.2.0] — 2026-08-16 — Temporal showcase パイプライン`）
- [x] Added セクション（`pipeline.fav` Temporal ステージ追加）を含める
- [x] Tests セクション（2 件）を含める

---

## T3: driver.rs — テストモジュール追加

- [x] `fav/src/driver.rs` の末尾に `// --- v79.2.0: Temporal showcase パイプライン ---` コメントを追加する
- [x] `v792000_tests` モジュールを追加する（`use super::*` は不要）
- [x] `showcase_temporal_freshness_check` テストを実装する
  - `pipeline.fav` に `load_with_freshness` が含まれることを assert
  - `pipeline.fav` に `AsOfQuery` が含まれることを assert
  - `pipeline.fav` に `FreshnessPolicy` が含まれることを assert
- [x] `showcase_temporal_scd2_applied` テストを実装する
  - `pipeline.fav` に `apply_scd2_update` が含まれることを assert
  - `pipeline.fav` に `ctx.run_ts` が含まれることを assert
- [x] `cargo test v792000` で 2 件が pass することを確認する

---

## T4: Cargo.toml バージョン更新

- [x] `fav/Cargo.toml` の `version` を `"79.1.0"` → `"79.2.0"` に変更する
- [x] driver.rs 内の `79.1.0` バージョン文字列アサーションを `79.2.0` に一括更新
- [x] **更新後に** `grep -c "79\.1\.0" /c/Users/yoshi/favnir/fav/src/driver.rs` を実行し **出力が 1** であることを確認する
  - 残るのは `// --- v79.1.0: 統合ショーケース基盤 ---` コメント行の 1 件のみ（アサーション文字列は 0 件であることを確認）

---

## T5: versions/current.md 更新

- [x] `## 進行中バージョン` 欄を `**v79.2.0**（Temporal showcase パイプライン）` に更新する
- [x] `## 次に切る版` 欄を `**v79.3.0**（Provenance showcase パイプライン）` に更新する

---

## T6: 最終確認

- [x] `cargo test` が全 pass であることを確認する（3791 tests）
- [x] `cargo test v792000` で 2 件が pass することを確認する
- [x] `fav/Cargo.toml` のバージョンが `79.2.0` であることを確認する
- [x] `CHANGELOG.md` の先頭が `[v79.2.0]` であることを確認する
- [x] `pipeline.fav` に `load_with_freshness` / `AsOfQuery` / `FreshnessPolicy` / `apply_scd2_update` が含まれることを確認する

---

## 完了チェックリスト

- [x] 全タスク（T0〜T6）が完了している
- [x] `showcase_temporal_freshness_check` が pass
- [x] `showcase_temporal_scd2_applied` が pass
- [x] テスト総数: 3791（+2）
- [x] `fav/Cargo.toml` version = "79.2.0" に更新済み
- [x] `versions/current.md` が v79.2.0 に更新済み
- [x] `changelog_has_v79_2_0` テストの追加: 対象外（x.0.0 宣言バージョンのみに追加する慣例）
