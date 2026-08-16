# v79.3.0 タスクリスト — Provenance showcase パイプライン

Date: 2026-08-16
Status: COMPLETE

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `79.2.0` であることを確認
- [x] `cargo test` が全 pass（3791 tests = v79.2.0 完了後の実測ベース）であることを確認
- [x] `fav/tmp/hello.fav` が存在することを確認
- [x] `infra/e2e-demo/favnir3-showcase/pipeline.fav` が存在し `load_with_freshness` を含むことを確認

---

## T1: pipeline.fav 更新

- [x] `infra/e2e-demo/favnir3-showcase/pipeline.fav` に `// --- Stage 2: Provenance（v76.x）---` コメントと `load_with_provenance` 関数を追加する
  - `fn load_with_provenance(ctx: AppCtx, rows: List<Row>) -> Result<TracedData, String>` シグネチャ（`rows` を引数として明示）
  - `bind source <- DataSource { ... }` を含む
  - `bind raw <- TracedData.wrap(rows, source)` を含む
  - `bind masked <- raw |> TracedData.map(mask_pii, label: "mask_pii")` を含む
  - `bind facet <- OpenLineage.from_provenance(masked.provenance)` を含む
- [x] `showcase_pipeline` の Stage 2 コメントを「load_with_provenance で実装済み」に更新する

---

## T2: CHANGELOG.md 更新（テスト追加より先）

- [x] `CHANGELOG.md` の先頭に v79.3.0 エントリを追加する（形式: `## [v79.3.0] — 2026-08-16 — Provenance showcase パイプライン`）
- [x] Added セクション（`pipeline.fav` Provenance ステージ追加）を含める
- [x] Tests セクション（2 件）を含める

---

## T3: driver.rs — テストモジュール追加

- [x] `fav/src/driver.rs` の末尾に `// --- v79.3.0: Provenance showcase パイプライン ---` コメントを追加する
- [x] `v793000_tests` モジュールを追加する（`use super::*` 不要）
- [x] モジュール先頭に `const PIPELINE: &str = include_str!("../../infra/e2e-demo/favnir3-showcase/pipeline.fav");` を配置する
- [x] `showcase_provenance_traced` テストを実装する
  - `load_with_provenance` / `TracedData` / `DataSource` / `mask_pii` を assert
- [x] `showcase_provenance_openlineage_generated` テストを実装する
  - `OpenLineage` / `masked.provenance` を assert
- [x] `cargo test v793000` で 2 件が pass することを確認する

---

## T4: Cargo.toml バージョン更新

- [x] `fav/Cargo.toml` の `version` を `"79.2.0"` → `"79.3.0"` に変更する
- [x] driver.rs 内の escaped `\"79.2.0\"` バージョン文字列を `\"79.3.0\"` に一括更新（sed）
- [x] driver.rs 内の unescaped エラーメッセージ `79.2.0` を `79.3.0` に更新する
- [x] **更新後に** `grep -c "79\.2\.0" /c/Users/yoshi/favnir/fav/src/driver.rs` を実行し **出力が 1** であることを確認する
  - 残るのは `// --- v79.2.0: Temporal showcase パイプライン ---` コメント行の 1 件のみ

---

## T5: versions/current.md 更新

- [x] `## 進行中バージョン` 欄を `**v79.3.0**（Provenance showcase パイプライン）` に更新する
- [x] `## 次に切る版` 欄を `**v79.4.0**（Verifiable showcase パイプライン）` に更新する

---

## T6: 最終確認

- [x] `cargo test` が全 pass であることを確認する（3793 tests）
- [x] `cargo test v793000` で 2 件が pass することを確認する
- [x] `fav/Cargo.toml` のバージョンが `79.3.0` であることを確認する
- [x] `fav/Cargo.lock` が cargo test 実行時に自動更新されていることを確認する
- [x] `CHANGELOG.md` の先頭が `[v79.3.0]` であることを確認する
- [x] `pipeline.fav` に `load_with_provenance` / `TracedData` / `OpenLineage` / `masked.provenance` が含まれることを確認する

---

## 完了チェックリスト

- [x] 全タスク（T0〜T6）が完了している
- [x] `showcase_provenance_traced` が pass
- [x] `showcase_provenance_openlineage_generated` が pass
- [x] テスト総数: 3793（+2）
- [x] `CHANGELOG.md` の先頭が `[v79.3.0]` であることを確認済み
- [x] `fav/Cargo.toml` version = "79.3.0" に更新済み
- [x] `versions/current.md` が v79.3.0 に更新済み
- [x] `changelog_has_v79_3_0` テストの追加: 対象外（x.0.0 宣言バージョンのみに追加する慣例）
