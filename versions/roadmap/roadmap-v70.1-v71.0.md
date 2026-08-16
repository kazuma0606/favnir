# Roadmap v70.1.0 〜 v71.0.0 — Language Complete 1.0 宣言

Date: 2026-08-08
Status: 未着手（v70.0.0 完了後に開始）

マスターロードマップ: [roadmap-v70.1-v75.0.md](roadmap-v70.1-v75.0.md)

---

## 前提

- 直前完了: v70.0.0「Intelligent ETL 1.0」（tests = 3559）
- 本スプリントは Phase 1「Language Complete 1.0」の詳細計画
- 目標: v71.0.0「Language Complete 1.0 宣言」（tests = 3581）

### スプリントの性格

Phase 1 は「積み残し一掃・compiler.fav 完全化」のスプリントである。
v65〜v70 のスプリントで積み上がった技術的負債を解消し、
コンパイラの網羅率を高め、エラー診断を実用水準へ引き上げる。
B（言語完成度）70% + C（開発者体験）30% の構成。

---

## バージョン一覧

| バージョン | 内容 | テスト数 | 状態 |
|---|---|---|---|
| v70.1.0 | Backlog Blitz（積み残し一掃） | 3559 + 2 = 3561 | 未着手 |
| v70.2.0 | `fav migrate` 完成（構文自動移行ツール） | 3561 + 2 = 3563 | 未着手 |
| v70.3.0 | `fav bench` サブコマンド完成 | 3563 + 2 = 3565 | 未着手 |
| v70.4.0 | 構造化エラー診断 | 3565 + 2 = 3567 | 未着手 |
| v70.5.0 | パターンマッチ強化（ネスト・Or パターン・ガード） | 3567 + 2 = 3569 | 未着手 |
| v70.6.0 | `bind` 分割束縛拡張 / Named Destructuring | 3569 + 2 = 3571 | 未着手 |
| v70.7.0 | Self-Hosting Coverage Report | 3571 + 2 = 3573 | 未着手 |
| v70.8.0 | `fav doctor` 強化（プロジェクト健全性チェック） | 3573 + 2 = 3575 | 未着手 |
| v70.9.0 | 安定化・コードフリーズ（Language Complete 前調整） | 3575 + 2 = 3577 | 未着手 |
| v71.0.0 | Language Complete 1.0 宣言 ★クリーンアップ | 3577 + 4 = 3581 | 未着手 |

---

## v70.1.0 — Backlog Blitz（積み残し一掃）

v65〜v70 スプリントで積み上がったまま放置された技術的負債を一括解消する。

**対象負債:**

| 項目 | 症状 | 修正方針 |
|---|---|---|
| `compiler.fav` multi-param ctx 未対応 | `fn f(ctx: AppCtx, data: T)` のパースが失敗（bench CI で表面化） | compiler.fav の `parse_fn_params` を修正 |
| `benchmarks/compare.fav` 旧構文 | `!IO` 構文のまま E0374 で失敗 | ctx: AppCtx 構文に移行済み（v70 で暫定 continue-on-error）→ 恒久修正 |
| `versions/current.md` 旧情報 | "次に切る版" が v66.9.0 のまま | v70.0.0 完了情報に同期 |
| `bench.yml` の `continue-on-error` 暫定対応 | Compare / Regression ステップが無条件スキップ | compiler.fav 修正後に外す |

**実装内容:**
- `compiler.fav` の `parse_fn_params` — 複数パラメータ（`ctx: AppCtx, data: T` 形式）を正しく解析
- `bench.yml` の Compare ステップから `continue-on-error: true` を除去
- `versions/current.md` を v70.0.0 完了・v70.1.0 進行中に更新

**完了条件**: Rust テスト 2 件（ベース 3559 + 2 = 3561）
- `backlog_compiler_fav_ctx_multiparams`（compiler.fav が `fn f(ctx: AppCtx, data: T)` をパースできる）
- `backlog_bench_yml_compare_strict`（bench.yml の Compare ステップが `continue-on-error` なしで通る）

---

## v70.2.0 — `fav migrate` 完成（構文自動移行ツール）

> `cmd_migrate` は driver.rs に既に存在するが、`!Effect` → `ctx: AppCtx` への
> 完全な変換ロジックが未実装。本バージョンで実用水準に完成させる。

旧構文（`!Effect` アノテーション）を新構文（`ctx: AppCtx`）へ自動変換する。
v35.4.0 の E0374 導入以降、手動移行が必要だったユーザーコードを一括救済する。

```bash
# 旧構文ファイルを自動変換
$ fav migrate --from v35 pipeline.fav
Migrating benchmarks/compare.fav...
  line 43: !IO → ctx: AppCtx (fn signature updated)
  line 54: IO.args() → ctx.io.argv()
  line 61: IO.read_file() → ctx.io.read_file_raw()
  line 74: IO.println() → ctx.io.println()
✓ Written: pipeline.fav.migrated

$ fav migrate --from v35 --in-place pipeline.fav
```

**実装内容:**
- `cmd_migrate(from_version, path, in_place)` in driver.rs — 変換ロジック本体を実装
- `!IO` / `!HTTP` / `!DB` 等の Effect アノテーション → ctx param 変換
- `IO.args()` / `IO.read_file()` / `IO.write_file()` / `IO.println()` → `ctx.io.*` 変換
- `--dry-run` オプション（変更箇所プレビュー）

**完了条件**: Rust テスト 2 件（3561 + 2 = 3563）
- `migrate_effect_annotation_to_ctx`
- `migrate_io_stdlib_to_ctx_io`

---

## v70.3.0 — `fav bench` サブコマンド完成

> `cmd_bench` は driver.rs に既に存在し、`Some("bench")` も main.rs に登録済み。
> しかし `--all` / `--compare` / `--fail-on-regression` の実装が不完全で
> bench.yml が `continue-on-error` に頼っている。本バージョンで完全動作させる。

`bench.yml` で長期間 `|| exit 1` を書きながら参照されてきた `$FAV bench` を
実際に動作させる。

```bash
# 全ベンチマークを実行して JSON 出力
$ fav bench --all
{
  "version": "70.3.0",
  "timestamp": "2026-08-08T10:00:00Z",
  "metrics": {
    "compile_hello_fav_ms": 12,
    "run_csv_1k_rows_ms": 45,
    "type_check_checker_fav_ms": 230
  }
}

# ベースラインと比較して regression があれば非ゼロ終了
$ fav bench --all --compare benchmarks/baseline.json --fail-on-regression
OK: all metrics within 5% of baseline.

# 特定ベンチマークのみ実行
$ fav bench compile_hello_fav
```

**実装内容:**
- `cmd_bench(all, compare, fail_on_regression, target)` — `--all` / `--compare` / `--fail-on-regression` フラグ実装
- 組み込みベンチマーク: コンパイル時間・型チェック時間・VM 実行速度
- baseline.json との diff 計算・regression 判定

**完了条件**: Rust テスト 2 件（3563 + 2 = 3565）
- `bench_subcommand_all_outputs_json`
- `bench_subcommand_regression_fail`

---

## v70.4.0 — 構造化エラー診断

エラーメッセージを「人間が読んで即解決できる」レベルに引き上げる。
「Did you mean?」提案・修正ヒント・ドキュメントリンクを統合。

```
error[E0374] benchmarks/compare.fav:43:62
  |
43| fn write_results_md(data: JsonValue) -> Result<Unit, String> !IO {
  |                                                              ^^^^
  | `!Effect` アノテーション構文は v35.4.0 で廃止されました
  |
  = ヒント: `ctx: AppCtx` を第1引数として追加し、`!IO` を削除してください
  |
  | 修正後:
43| fn write_results_md(ctx: AppCtx, data: JsonValue) -> Result<Unit, String> {
  |
  = 参照: https://favnir.dev/docs/language/ctx-migration
  = 自動移行: fav migrate --from v35 --in-place benchmarks/compare.fav

error[E0001] pipeline.fav:12
  |
12|     bind result <- process(ordr)
  |                            ^^^^
  | 未定義変数 `ordr`
  |
  = ヒント: `order` のことですか？（3文字以内の編集距離）
```

**実装内容:**
- `ErrorReport` 構造体（code / span / message / hint / suggestion / doc_url）
- `suggest_similar_name` — Levenshtein 距離によるタイポ候補
- `format_diagnostic` — カラー付きターミナル出力 + LSP JSON 出力

**完了条件**: Rust テスト 2 件（3565 + 2 = 3567）
- `diagnostic_e0374_shows_migration_hint`
- `diagnostic_e0001_suggests_similar_name`

---

## v70.5.0 — パターンマッチ強化

複雑なデータ構造のパターンマッチを、より表現力豊かに記述できるよう拡張する。

```favnir
// ネストパターン（レコードフィールドを直接分解）
match api_response {
    Ok({status: 200, body}) => process(body)
    Ok({status: 404, _})    => Result.err("not found")
    Err({code, message})    => Result.err(message)
}

// Or-パターン（複数ケースをまとめる）
match event.kind {
    Created | Updated => handle_write(event)
    Deleted | Expired => handle_delete(event)
}

// ガード付きパターン
match row.amount {
    x if x > 10000.0 => classify(Large)
    x if x > 1000.0  => classify(Medium)
    _                => classify(Small)
}
```

**実装内容:**
- parser: ネストレコードパターン・Or パターン（`A | B`）・ガード（`if cond`）を追加
- checker: 各パターンに型を伝播
- compiler.fav: `parse_match_arm` に新パターン分岐を追加

**完了条件**: Rust テスト 2 件（3567 + 2 = 3569）
- `pattern_match_nested_record`
- `pattern_match_or_pattern`

---

## v70.6.0 — `bind` 分割束縛拡張 / Named Destructuring

既存の `bind x <- expr` 構文を拡張し、レコード・リストの分解束縛を
`bind` で直接書けるようにする。`let` は Favnir では使用しない。

```favnir
// レコード分割束縛（bind の左辺にパターン）
bind {order_id, amount, status} <- row
// 等価: bind order_id <- row.order_id
//        bind amount   <- row.amount
//        bind status   <- row.status

// リスト分割束縛
bind [head, second, ...tail] <- items
// head: items[0], second: items[1], tail: items[2..]

// ネスト分割束縛
bind {customer: {name, email}, total} <- order

// 既存の bind と同一式内で混在可能
fn process_order(ctx: AppCtx, row: OrderRow) -> Result<Unit, String> {
    bind {order_id, amount} <- row
    bind result             <- Postgres.insert(ctx, order_id, amount)
    ctx.io.println(f"Inserted {order_id}: {result} rows")
}
```

**実装内容:**
- parser: `bind` の左辺にレコードパターン `{field, ...}` / リストパターン `[h, ...t]` を受け付ける
- checker: 分割束縛の各フィールドに型を伝播
- compiler.fav: `parse_bind_lhs` にパターン分岐を追加

**完了条件**: Rust テスト 2 件（3569 + 2 = 3571）
- `bind_destructure_record`
- `bind_destructure_list_spread`

---

## v70.7.0 — Self-Hosting Coverage Report

compiler.fav / checker.fav が処理できる Favnir 構文の網羅率を定量化し、
カバーできていない構文を発見・修正する。

```bash
$ fav self-coverage
compiler.fav coverage: 94.2% (48/51 syntax forms)
  Missing: bind-destructure, or-pattern, dependent-type-annotation
checker.fav coverage: 91.3% (42/46 error codes)
  Missing: E0411, E0412, E0413, E0414

$ fav self-coverage --fix-missing
# 未対応構文のスタブを compiler.fav / checker.fav に生成
```

**実装内容:**
- `cmd_self_coverage` — AST の全ノード種別をリストアップし、compiler.fav の処理対象と照合
- Missing 構文を一覧表示・優先度付け
- v70.5・v70.6 で追加した bind-destructure / or-pattern を compiler.fav に反映

**完了条件**: Rust テスト 2 件（3571 + 2 = 3573）
- `self_coverage_compiler_fav_above_95pct`
- `self_coverage_checker_fav_above_90pct`

---

## v70.8.0 — `fav doctor` 強化（プロジェクト健全性チェック）

> `cmd_doctor_run` は driver.rs に既に存在し、`Some("doctor")` も main.rs に登録済み。
> 本バージョンでは Paper Rune 検出・CHANGELOG 整合性・self-hosting coverage
> など v70 スプリントで追加された検査項目を実装として追加する。

プロジェクト全体を静的に診断し、問題・改善点を一覧表示するコマンドを強化する。

```bash
$ fav doctor
Favnir v71.0.0 — project health check

✓ fav.toml           valid
✓ Cargo.toml         version = "71.0.0"
✓ self/compiler.fav  coverage 95.1%
⚠ runes/linalg/      VM primitive 未接続（スタブ実装）
⚠ runes/autodiff/    VM primitive 未接続（スタブ実装）
✗ CHANGELOG.md       v70.9.0 エントリが存在しない
✗ benchmarks/        baseline.json が 90日以上古い

2 errors, 2 warnings
Hint: fav doctor --fix で自動修正を試みます
```

**実装内容:**
- `cmd_doctor` 強化 — Paper Rune 検出・CHANGELOG 整合性・self-hosting coverage チェックを追加
- `--fix` フラグ — 自動修正可能な項目を修正
- Paper Rune 検出（rune.toml あり・.fav 実装は存在するが VM primitive 未接続）

**完了条件**: Rust テスト 2 件（3573 + 2 = 3575）
- `doctor_detects_paper_rune`
- `doctor_detects_missing_changelog_entry`

---

## v70.9.0 — 安定化・コードフリーズ（Language Complete 前調整）

v70.1〜v70.8 の全機能が正常動作することを確認する安定化バージョン。
bench.yml の `continue-on-error` を外し、CI が全グリーンであることを確認する。

**完了条件**: Rust テスト 2 件（3575 + 2 = 3577）
- `language_complete_all_stable`（v70.1〜v70.8 の代表テストが全 pass）
- `bench_ci_no_continue_on_error`（bench.yml の Compare ステップが strict mode で通る）

---

## v71.0.0 — Language Complete 1.0 宣言 ★クリーンアップ

**宣言文**:

> 「compiler.fav が Favnir の全構文を処理し、
>  積み残しのない CI が毎回グリーンで終わる。
>  エラーメッセージは修正方法を即座に示し、
>  fav migrate が旧コードを自動で現代に変換する。
>
>  これが Favnir v71.0 — Language Complete 1.0 の姿である。」

**クリーンアップ作業:**
- `cargo clean` 実施（ビルド生成物を削除）
- `Cargo.toml` バージョンを `71.0.0` に更新
- `CHANGELOG.md` に v71.0.0 エントリを追加
- `MILESTONE.md` に「Language Complete 1.0」を追記
- `README.md` に v71.0 達成を追記
- `versions/current.md` を更新（進行中 → v71.1.0）

**完了条件**: `v71000_tests` 4 件（3577 + 4 = 3581）
- `cargo_toml_version_is_71_0_0`
- `changelog_has_v71_0_0`
- `milestone_has_language_complete`
- `readme_mentions_language_complete`

---

## テスト数推移（本スプリント）

| バージョン | テスト数 | 増加 |
|---|---|---|
| v70.0.0（ベース） | 3,559 | — |
| v70.1.0 | 3,561 | +2 |
| v70.2.0 | 3,563 | +2 |
| v70.3.0 | 3,565 | +2 |
| v70.4.0 | 3,567 | +2 |
| v70.5.0 | 3,569 | +2 |
| v70.6.0 | 3,571 | +2 |
| v70.7.0 | 3,573 | +2 |
| v70.8.0 | 3,575 | +2 |
| v70.9.0 | 3,577 | +2 |
| v71.0.0（宣言） | 3,581 | +4 |

**本スプリント合計**: +22 tests（3,559 → 3,581）

---

## 参考リンク

- マスターロードマップ: `versions/roadmap/roadmap-v70.1-v75.0.md`
- 前スプリント（完了）: `versions/roadmap/roadmap-v69.1-v70.0.md`
- 次スプリント: `versions/roadmap/roadmap-v71.1-v72.0.md`
- 進行状況: `versions/current.md`
