# v62.9.0 タスクリスト

Status: COMPLETE
Version: 62.9.0
Base tests: 3400
Target tests: 3402

---

## T0: 事前確認

- [x] `cargo test -j 8 -- --test-threads=8` でベース tests passed, 0 failed ���確認
  （期待値: 3400。v62.8.0 code-reviewer 対応で `aot_no_emit_passes` が追加されたため
   ロードマップ記載 3398 より +2 増加している）
- [x] `infra/e2e-demo/aot/` が **存在しない** ことを確認
- [x] `site/content/docs/runtime/aot.mdx` が **存在しない** ことを確認
- [x] `driver.rs` に `v62800_tests` が存在することを確認（挿入位置確認）
- [x] `site/content/docs/runtime/` ディレクトリが存在することを確認（新規作成不要）

---

## T1: `infra/e2e-demo/aot/` — デモ環境ファイル��成

- [x] `infra/e2e-demo/aot/src/pipeline.fav` を新規作成:
  ```favnir
  // AOT E2E Demo — Pure Transformation Pipeline (v62.9.0)
  // Compatible with `fav build --link -o dist/pipeline`

  type OrderRow = {
    id: Int
    amount: Float
    region: String
  }

  type SummaryRow = {
    region: String
    total: Float
    count: Int
  }

  fn parse_order(raw: String) -> OrderRow {
    { id: 1, amount: 99.9, region: raw }
  }

  fn summarize(orders: List<OrderRow>) -> SummaryRow {
    { region: "ALL", total: 999.0, count: List.length(orders) }
  }

  fn main() -> Bool {
    let order = parse_order("us-east")
    let summary = summarize([order])
    summary.count == 1
  }
  ```
- [x] `infra/e2e-demo/aot/scripts/build-aot.sh` を新規作成
- [x] `infra/e2e-demo/aot/README.md` を新規作成
- [x] `cargo build` でエラーなし

---

## T2: `site/content/docs/runtime/aot.mdx` — ドキュメント作成

- [x] `site/content/docs/runtime/aot.mdx` を新規作成
  （`fav build` コマンド一覧・E0427 解説・コード例）
- [x] `cargo build` でエラーなし

---

## T3: `driver.rs` — `v62900_tests` 追加

- [x] `v62800_tests` の直前（ファイル先頭方向）に以下を挿入:
  （注意: `include_str!` はクレートスコープマク��なので `use super::*;` は不要。前後モジュールに合わせて追加しないこと）
  （注意: `include_str!` パスは `"../../"` — 他テストと同じ。`"../../../"` ���誤り）
  ```rust
  // -- v62900_tests (v62.9.0) -- AOT E2E デモ構造 + docs/runtime/aot.mdx --
  #[cfg(test)]
  mod v62900_tests {
      #[test]
      fn aot_e2e_demo_structure() {
          let src = include_str!("../../infra/e2e-demo/aot/src/pipeline.fav");
          assert!(
              src.contains("pipeline") || src.contains("OrderRow"),
              "aot e2e demo pipeline.fav should define pipeline types"
          );
          assert!(
              src.contains("SummaryRow"),
              "aot e2e demo pipeline.fav should define SummaryRow"
          );
      }

      #[test]
      fn docs_aot_mdx_exists() {
          let mdx = include_str!("../../site/content/docs/runtime/aot.mdx");
          assert!(
              mdx.contains("AOT Compilation"),
              "aot.mdx should contain 'AOT Compilation'"
          );
          assert!(
              mdx.contains("fav build"),
              "aot.mdx should mention 'fav build'"
          );
          assert!(
              mdx.contains("E0427"),
              "aot.mdx should reference E0427"
          );
      }
  }
  ```
- [x] `cargo test v62900` で 2 件 PASS

---

## T4: ビルド・テスト

- [x] `cargo build` でコンパイルエラー 0
- [x] `cargo test v62900` で 2 件 PASS
- [x] `cargo test -j 8 -- --test-threads=8` で 3402 tests passed, 0 failed を確認（実測ベース + 2）

---

## T5: ドキュメント更新

- [x] `versions/roadmap/roadmap-v62.1-v63.0.md` v62.9.0 セクションに実績を追記
- [x] `versions/current.md` の「進行中」を v62.9.0（3402 tests）に更新、「次」を v63.0.0 に
- [x] `CHANGELOG.md` に v62.9.0 エントリを追加
- [x] tasks.md を COMPLETE に更新（本ファイル）

---

## コードレビュー指摘対応

spec-reviewer 指摘（実装前修正）:
- [HIGH] `docs_aot_mdx_exists` の `include_str!` パスが `"../../../site/..."` と記述されてい�� → `"../../site/..."` に修正（既存テストのパターンと同じ）
- [HIGH] ベーステ��ト数を実測前に断言 → 「T0 で実測して���認する（期待値: 3400）」形��に緩和
- [MED] `v62900_tests` に `use super::*;` 不要の注記追加
- [MED] T0 に `site/content/docs/runtime/` ディレクトリ存在確認を��加
- [MED] spec.md にロードマップ `[3/3]` との意図的乖離を明記

実装時判明（追加修正）:
- spec-reviewer の「`"../../../infra/"`が正しい」という指摘は誤りだった。実際の既存テストは `"../../infra/"` を使用（`fav/src/` から `../../` = `favnir/` root）。tasks.md・spec.md・plan.md のパスを `"../../"` に修正。

code-reviewer 指摘（実装後修正）:
- [HIGH] `build-aot.sh` の `|| true` が全エラーを握り潰し `set -euo pipefail` が無意味 → `if/else` 構造に変更（コマンドが使えない環境でも明示的にスキップ理由を出力）、`mkdir -p` 追加
- [MED] `build-aot.sh` の出力ディレクトリ `/tmp/aot-demo/` が未作成 → `mkdir -p "${AOT_OUT_DIR}"` 追加（Fix 1 で対応）
- [MED] `v62900_tests` の OR 条件 `src.contains("pipeline") || src.contains("OrderRow")` が事実上常に true → `src.contains("OrderRow")` 単独に修正
- [MED] `pipeline.fav` の `let` 構文指摘 → `let` は Favnir の有効な純粋バインディング構文（`examples/lambda-deploy/src/main.fav` 等で使用済み）のため修正不要と判断

---

## 完了サマリー

- Status: COMPLETE
- Tests: 3402 passed, 0 failed（base 3400 + 2）
- 主要実装: `infra/e2e-demo/aot/`（3ファイル）/ `site/content/docs/runtime/aot.mdx` / `v62900_tests`（`driver.rs`）
- 完了日: 2026-08-01
