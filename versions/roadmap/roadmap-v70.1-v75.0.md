# Roadmap v70.1.0 〜 v75.0.0 — Favnir 2.0

Date: 2026-08-08
Status: 計画中（v70.0.0 完了、v70.1.0 から開始）

---

## 前提

- 直前完了: v70.0.0「Intelligent ETL 1.0」（tests = 3559）
- 本文書は v70.1〜v75.0 の**マスターロードマップ**
- 各マイルストーン開始時に対応するサブスプリントロードマップを作成する

| サブスプリント文書 | カバー範囲 | 状態 |
|---|---|---|
| `roadmap-v70.1-v71.0.md` | v70.1〜v70.9 + v71.0 | 未作成 |
| `roadmap-v71.1-v72.0.md` | v71.1〜v71.9 + v72.0 | 未作成 |
| `roadmap-v72.1-v73.0.md` | v72.1〜v72.9 + v73.0 | 未作成 |
| `roadmap-v73.1-v74.0.md` | v73.1〜v73.9 + v74.0 | 未作成 |
| `roadmap-v74.1-v75.0.md` | v74.1〜v74.9 + v75.0 | 未作成 |

---

## ビジョン

> **「幅を広げた。次は深さを掘る。」**

v70.0「Intelligent ETL 1.0」で Favnir は「型安全な AI パイプライン言語」を宣言した。
Math Rune 群・AI Stage Layer・Distributed 実行・Developer Intelligence——
機能の幅は揃った。しかし問いは変わる:

**「それらは本当に動いているか？ 本物のチームが使えるか？」**

2026年末の Favnir が目指すのは、**深さと実証**だ。

- compiler.fav が Favnir 構文を漏れなく処理できているか
- 依存型・refined type が AI パイプラインの正確性を型レベルで保証できているか
- VS Code 拡張と AI アシスタントが開発体験を本物にしているか
- 実際のデータチームが本番で Favnir を動かしているか

その道筋は 4 段階 + 1 宣言だ:

```
Language Complete 1.0   ─ compiler.fav 完全化・積み残し一掃・エラー診断強化
Type System 2.0         ─ 依存型・refined type・AOT 本番品質
Developer Exp 2.0       ─ VS Code・AI アシスタント・REPL・Playground
Production Proven       ─ データコントラクト・Rune 品質・ドッグフーディング
                        ↓
        v75.0 — Favnir 2.0
```

---

## 宣言文（v75.0 目標）

> 「compiler.fav が Favnir を完全に記述し、型システムが次元と制約を保証する。
>  依存型がベクトルの次元を守り、refined type がゼロ除算をコンパイル時に止める。
>  VS Code がパイプラインを補完し、AI がエラーを修正し、
>  実際のデータチームが本番で Favnir を走らせている。
>
>  これが Favnir v75.0 — Favnir 2.0 の姿である。」

---

## Phase 1: v70.1〜v70.9 + v71.0 — Language Complete 1.0

**テーマ**: 「compiler.fav が Favnir を完全に話す」

v65〜v70 のスプリントで積み上げた機能群の土台を固める。
積み残しを解消し、コンパイラの網羅率を高め、エラー診断を実用水準へ引き上げる。
B（言語完成度）を中心に C（開発者体験）を組み合わせるフェーズ。

---

### v70.1.0 — Backlog Blitz（積み残し一掃）

v65〜v70 スプリントで積み上がったまま放置された技術的負債を一括解消する。

**対象負債:**

| 項目 | 症状 | 修正方針 |
|---|---|---|
| `compiler.fav` multi-param ctx 未対応 | `fn f(ctx: AppCtx, data: T)` のパースが失敗（bench CI で表面化） | compiler.fav の `parse_fn_params` を修正 |
| `benchmarks/compare.fav` 旧構文 | `!IO` 構文のまま E0374 で失敗 | ctx: AppCtx 構文に移行済み（v70 で暫定 continue-on-error）→ 恒久修正 |
| `versions/current.md` 旧情報 | "次に切る版" が v66.9.0 のまま | v70.0.0 完了情報に同期 |
| `bench.yml` の `continue-on-error` 暫定対応 | Compare / Regression ステップが無条件スキップ | compiler.fav 修正後に外す |

**完了条件**: Rust テスト 2 件（ベース 3559 + 2 = 3561）
- `backlog_compiler_fav_ctx_multiparams`（compiler.fav が `fn f(ctx: AppCtx, data: T)` をパースできる）
- `backlog_bench_yml_compare_strict`（bench.yml の Compare ステップが `continue-on-error` なしで通る）

---

### v70.2.0 — `fav migrate`（構文自動移行ツール）

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
- `cmd_migrate(from_version, path, in_place)` in driver.rs
- `!IO` / `!HTTP` / `!DB` 等の Effect アノテーション → ctx param 変換
- `IO.args()` / `IO.read_file()` / `IO.write_file()` / `IO.println()` → `ctx.io.*` 変換
- `--dry-run` オプション（変更箇所プレビュー）

**完了条件**: Rust テスト 2 件（3561 + 2 = 3563）
- `migrate_effect_annotation_to_ctx`
- `migrate_io_stdlib_to_ctx_io`

---

### v70.3.0 — `fav bench` サブコマンド実装

`bench.yml` で長期間 `|| exit 1` を書きながら参照されてきた `$FAV bench` を
実際に動作させる。

```bash
# 全ベンチマークを実行して JSON 出力
$ fav bench --all
{
  "version": "71.0.0",
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
- `cmd_bench(all, compare, fail_on_regression, target)` in driver.rs
- 組み込みベンチマーク: コンパイル時間・型チェック時間・VM 実行速度
- baseline.json との diff 計算・regression 判定

**完了条件**: Rust テスト 2 件（3563 + 2 = 3565）
- `bench_subcommand_all_outputs_json`
- `bench_subcommand_regression_fail`

---

### v70.4.0 — 構造化エラー診断

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

### v70.5.0 — パターンマッチ強化

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

**完了条件**: Rust テスト 2 件（3567 + 2 = 3569）
- `pattern_match_nested_record`
- `pattern_match_or_pattern`

---

### v70.6.0 — `bind` 分割束縛拡張 / Named Destructuring

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

### v70.7.0 — Self-Hosting Coverage Report

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

### v70.8.0 — `fav doctor`（プロジェクト健全性チェック）

プロジェクト全体を静的に診断し、問題・改善点を一覧表示するコマンド。

```bash
$ fav doctor
Favnir v71.0.0 — project health check

✓ fav.toml           valid
✓ Cargo.toml         version = "71.0.0"
✓ self/compiler.fav  coverage 95.1%
⚠ runes/linalg/      rune.toml あり、実装 .fav が空（paper rune）
⚠ runes/autodiff/    同上
✗ CHANGELOG.md       v70.9.0 エントリが存在しない
✗ benchmarks/        baseline.json が 90日以上古い

2 errors, 2 warnings
Hint: fav doctor --fix で自動修正を試みます
```

**実装内容:**
- `cmd_doctor` — fav.toml・rune 整合性・self-hosting coverage・CHANGELOG 整合性チェック
- `--fix` フラグ — 自動修正可能な項目を修正
- Paper Rune 検出（rune.toml あり・実装ファイル空）

**完了条件**: Rust テスト 2 件（3573 + 2 = 3575）
- `doctor_detects_paper_rune`
- `doctor_detects_missing_changelog_entry`

---

### v70.9.0 — 安定化・コードフリーズ（Language Complete 前調整）

v70.1〜v70.8 の全機能が正常動作することを確認する安定化バージョン。
bench.yml の `continue-on-error` を外し、CI が全グリーンであることを確認する。

**完了条件**: Rust テスト 2 件（3575 + 2 = 3577）
- `language_complete_all_stable`（v70.1〜v70.8 の代表テストが全 pass）
- `bench_ci_no_continue_on_error`（bench.yml の Compare ステップが strict mode で通る）

---

### v71.0.0 — Language Complete 1.0 宣言 ★クリーンアップ

**宣言文**:

> 「compiler.fav が Favnir の全構文を処理し、
>  積み残しのない CI が毎回グリーンで終わる。
>  エラーメッセージは修正方法を即座に示し、
>  fav migrate が旧コードを自動で現代に変換する。
>
>  これが Favnir v71.0 — Language Complete 1.0 の姿である。」

**完了条件**: `v71000_tests` 4 件（3577 + 4 = 3581）
- `cargo_toml_version_is_71_0_0`
- `changelog_has_v71_0_0`
- `milestone_has_language_complete`
- `readme_mentions_language_complete`

---

## Phase 2: v71.1〜v71.9 + v72.0 — Type System 2.0

**テーマ**: 「型が次元・制約・精緻さを表現する」

v70 の「Intelligent ETL 1.0」で AI パイプラインの型安全を宣言したが、
次元数・値域制約・コンパイル時評価は未整備だった。
このフェーズでそれらを型システムに統合し、「型で証明する」言語へと進化させる。
B（型システム）を中心に、AOT・WebAssembly などランタイム側も強化する。

---

### v71.1.0 — 依存型の基礎 `Vec<T>[N]`

配列・ベクトルの次元数を型パラメータとして表現する。
AI パイプラインにおける埋め込み次元の型安全が主要ユースケース。

```favnir
// N を型変数として伝播
fn dot_product[N: Int](a: Vec<Float>[N], b: Vec<Float>[N]) -> Float {
    Rune.linalg.dot(a, b)
}

// 次元違いはコンパイルエラー
stage EmbedText: String -> Vec<Float>[1536] = |text| {
    OpenAI.embed(text)
}

stage CosineSim: (Vec<Float>[1536], Vec<Float>[1536]) -> Float = |(a, b)| {
    dot_product(a, b)  // 型一致 → OK
}

// stage EmbedSmall: String -> Vec<Float>[768]
// CosineSim(EmbedText("x"), EmbedSmall("y"))  // コンパイルエラー: 1536 ≠ 768
```

**完了条件**: Rust テスト 2 件（3581 + 2 = 3583）
- `dependent_type_vec_dim_param`
- `dependent_type_dim_mismatch_error`

---

### v71.2.0 — Refined Types（型レベル制約 `where self`）

値域制約を型に組み込み、実行時エラーをコンパイル時エラーに変換する。

```favnir
// 型レベル制約
type PositiveFloat = Float where self > 0.0
type NonEmptyStr   = String where String.length(self) > 0
type BatchSize     = Int where self >= 1 && self <= 10000

// 型違反はコンパイルエラー
fn safe_log(x: PositiveFloat) -> Float {
    Float.log(x)  // x が 0 以下になれないことが型で保証される
}

// 型の絞り込み（narrowing）
fn process(n: Int) -> Float {
    if n > 0 {
        safe_log(n)     // ここでは n: PositiveFloat として扱える
    } else {
        0.0
    }
}
```

**完了条件**: Rust テスト 2 件（3583 + 2 = 3585）
- `refined_type_positive_float`
- `refined_type_violation_compile_error`

---

### v71.3.0 — Phantom Types（型タグによる誤使用防止）

異なる意味を持つ同型値の混用をコンパイル時に防ぐ。

```favnir
// UserId と OrderId は String だが混用不可
type UserId  = phantom String
type OrderId = phantom String

fn get_user(id: UserId) -> User { ... }
fn get_order(id: OrderId) -> Order { ... }

bind uid = UserId("u-123")
bind oid = OrderId("o-456")
get_user(uid)   // OK
get_user(oid)   // コンパイルエラー: OrderId ≠ UserId
```

**完了条件**: Rust テスト 2 件（3585 + 2 = 3587）
- `phantom_type_prevents_id_confusion`
- `phantom_type_explicit_cast`

---

### v71.4.0 — Const / Compile-Time Evaluation

定数式をコンパイル時に評価する。依存型の次元数指定に必須。

```favnir
const MAX_BATCH_SIZE: Int  = 1024
const EMBED_DIM:      Int  = 1536
const API_BASE_URL:   String = "https://api.favnir.dev"

// 依存型で定数を使用
stage EmbedText: String -> Vec<Float>[EMBED_DIM] = |text| {
    OpenAI.embed(text, dim: EMBED_DIM)
}
```

**完了条件**: Rust テスト 2 件（3587 + 2 = 3589）
- `const_eval_int_expr`
- `const_used_in_dependent_type`

---

### v71.5.0 — Generic Constraints（`impl Trait`風の境界）

```favnir
// 複数制約を & で結合
fn serialize_all[T: Serializable & Comparable](items: List<T>) -> String {
    items
    |> List.sort
    |> List.map(T.serialize)
    |> String.join(",")
}

// インターフェース実装要求
fn store[T: impl DbRecord](ctx: AppCtx, item: T) -> Result<Int, String> {
    ctx.db.insert(T.table_name(), T.to_row(item))
}
```

**完了条件**: Rust テスト 2 件（3589 + 2 = 3591）
- `generic_constraint_multi_interface`
- `generic_constraint_impl_trait`

---

### v71.6.0 — AOT Native Compilation 本番品質化

cranelift バックエンドを強化し、単体で配布可能なネイティブバイナリを生成する。
Rust ランタイム不要・Docker イメージサイズ 1/10。

```bash
# ELF バイナリ生成（Linux x86_64）
$ fav build --target native pipeline.fav -o pipeline_bin
Compiling pipeline.fav → native (linux/amd64)
Binary: ./pipeline_bin (4.2 MB)

# 実行
$ ./pipeline_bin --input data.csv --output results.parquet

# ARM64 クロスコンパイル
$ fav build --target native --arch arm64 pipeline.fav -o pipeline_arm
```

**完了条件**: Rust テスト 2 件（3591 + 2 = 3593）
- `aot_native_binary_compiles`
- `aot_native_binary_runs_hello`

---

### v71.7.0 — WebAssembly ターゲット

Favnir パイプラインを WASM バイナリとして出力する。
Playground でのブラウザ内実行・エッジコンピューティング対応。

```bash
# WASM 出力
$ fav build --target wasm pipeline.fav -o pipeline.wasm
$ wasm-run pipeline.wasm --input data.json

# Playground での利用（ブラウザ内完結）
# @favnir/wasm パッケージ更新により自動対応
```

**完了条件**: Rust テスト 2 件（3593 + 2 = 3595）
- `wasm_target_compiles`
- `wasm_target_runs_simple_pipeline`

---

### v71.8.0 — 型推論強化（型注釈省略可能範囲の拡大）

ローカル変数・クロージャ引数での型注釈を省略できる範囲を広げる。

```favnir
// Before（型注釈が必要だった箇所）
bind items: List<Order>  <- load_orders(ctx)
bind total: Float        <- List.fold(items, 0.0, |acc: Float, o: Order| acc + o.amount)

// After（推論で省略可能）
bind items <- load_orders(ctx)      // List<Order> を推論
bind total <- List.fold(items, 0.0, |acc, o| acc + o.amount)  // Float を推論
```

**完了条件**: Rust テスト 2 件（3595 + 2 = 3597）
- `type_infer_local_var_omit_annotation`
- `type_infer_closure_arg_omit`

---

### v71.9.0 — 安定化・コードフリーズ（Type System 2.0 前調整）

**完了条件**: Rust テスト 2 件（3597 + 2 = 3599）
- `type_system_2_all_stable`
- `dependent_refined_phantom_e2e`

---

### v72.0.0 — Type System 2.0 宣言 ★クリーンアップ

**宣言文**:

> 「依存型がベクトルの次元を守り、refined type がゼロ除算を型で止める。
>  Phantom type が ID の混用を防ぎ、定数がコンパイル時に評価される。
>  AOT バイナリが Docker 不要で動き、WASM がパイプラインをブラウザへ運ぶ。
>
>  これが Favnir v72.0 — Type System 2.0 の姿である。」

**完了条件**: `v72000_tests` 4 件（3599 + 4 = 3603）
- `cargo_toml_version_is_72_0_0`
- `changelog_has_v72_0_0`
- `milestone_has_type_system_2`
- `readme_mentions_type_system_2`

---

## Phase 3: v72.1〜v72.9 + v73.0 — Developer Experience 2.0

**テーマ**: 「データエンジニアが Favnir を選ぶ開発体験」

型システムが強力でも、開発体験が貧しければ選ばれない。
VS Code 拡張・AI アシスタント・REPL・Playground——
実際に手を動かすすべての場面で Favnir が寄り添う開発環境を整える。
C（実証・UX）を中心に、AI コード生成などの先端機能も取り込む。

---

### v72.1.0 — VS Code 拡張（本格実装）

既存 LSP を VS Code Extension として完全統合する。
マーケットプレイス公開を視野に入れた品質で実装する。

```
機能一覧:
✓ シンタックスハイライト（.fav ファイル）
✓ 型ホバー（変数・関数にカーソルを当てると型を表示）
✓ 定義ジャンプ（F12）・参照検索（Shift+F12）
✓ インライン型ヒント（引数名・戻り値型）
✓ エラーアンダーライン + 修正ヒント（Quick Fix）
✓ Rune メソッド補完（ctx.io. → argv / println / read_file_raw ...）
✓ コードフォーマット（保存時 fav fmt 自動実行）
✓ fav run / fav check をエディタから実行（Run Task）
```

**完了条件**: Rust テスト 2 件（3603 + 2 = 3605）
- `vscode_extension_package_json_valid`
- `vscode_extension_lsp_integration`

---

### v72.2.0 — AI エラーアシスタント（`fav ai explain` / `fav ai fix`）

コンパイルエラーを AI に渡し、自然言語での説明と修正案を得る。

```bash
$ fav check pipeline.fav --ai-explain
E0374 detected at line 43.

[AI Explanation]
このエラーは `!IO` というエフェクトアノテーション構文が使われているために
発生しています。v35.4.0 でこの構文は廃止され、代わりに `ctx: AppCtx` を
関数の第1引数として渡す方式に変わりました。

[Suggested Fix]
Before: fn write_results_md(data: JsonValue) -> Result<Unit, String> !IO
After:  fn write_results_md(ctx: AppCtx, data: JsonValue) -> Result<Unit, String>

さらに `IO.write_file(...)` → `ctx.io.write_file_raw(...)` への変更も必要です。

Apply this fix? [y/N]: y
✓ Applied. Run `fav check pipeline.fav` to verify.

# または自動修正のみ
$ fav ai fix pipeline.fav
```

**完了条件**: Rust テスト 2 件（3605 + 2 = 3607）
- `ai_explain_e0374_returns_hint`
- `ai_fix_applies_ctx_migration`

---

### v72.3.0 — `fav ai generate`（自然言語 → Favnir パイプライン）

自然言語の要求仕様から Favnir パイプラインの雛形を生成する。

```bash
$ fav ai generate "S3のCSVを読んでスキーマ検証しPostgresに挿入するパイプライン"
Generating pipeline...

# Generated: pipeline.fav
import rune "csv"
import rune "postgres"

schema OrderRow {
    order_id: String
    amount:   Float
    status:   String
}

fn main(ctx: AppCtx) -> Result<Unit, String> {
    bind raw   <- ctx.io.read_file_raw("s3://bucket/data.csv")
    bind rows  <- Csv.parse_typed(raw, OrderRow)
    bind valid <- Schema.validate_all(rows)
    bind _     <- Postgres.execute_raw("INSERT INTO orders ...", valid)
    ctx.io.println("Done.")
}

Open in editor? [y/N]: y
```

**完了条件**: Rust テスト 2 件（3607 + 2 = 3609）
- `ai_generate_returns_valid_fav_code`
- `ai_generate_schema_inferred_from_description`

---

### v72.4.0 — REPL 2.0

既存 `fav repl` を大幅強化する。

```bash
$ fav repl
Favnir v73.0.0 REPL — :help でヘルプ

fav> :import rune "json"
rune "json" loaded.

fav> bind data <- Json.parse("[1,2,3]")
data: JsonValue = [1, 2, 3]

fav> List.length(data)
3: Int

fav> :timing on
fav> List.map([1..100], |x| x * x)
[1, 4, 9, ...] : List<Int>  (0.3ms)

fav> :history          # 入力履歴表示
fav> :save session.fav # セッションをファイルに保存
fav> :load session.fav # セッションを再現
```

**新機能:**
- 入力履歴（↑↓キー）+ 永続化（`~/.fav_history`）
- TAB 補完（変数名・関数名・Rune メソッド）
- 複数行入力（`{` で開始、`}` で確定）
- `:timing` モード・`:save` / `:load`

**完了条件**: Rust テスト 2 件（3609 + 2 = 3611）
- `repl2_multiline_input`
- `repl2_tab_completion`

---

### v72.5.0 — Playground 2.0

ブラウザ内の Favnir Playground を全面強化する。

```
新機能:
- AI 補完（GitHub Copilot 風のインライン提案）
- 共有リンク（実行結果 + コード を永続 URL で共有）
- テンプレートギャラリー（AI ETL / 分散 / データ品質 / 時系列）
- 実行結果の可視化（List<Record> → テーブル表示、List<Float> → グラフ）
- WASM ビルド対応（ブラウザ内で完全実行）
```

**完了条件**: Rust テスト 2 件（3611 + 2 = 3613）
- `playground2_template_gallery_has_5_entries`
- `playground2_share_url_format`

---

### v72.6.0 — `fav init` テンプレート拡充

```bash
$ fav init --template ai-etl          # LLM 抽出 → VectorDB
$ fav init --template streaming       # Kafka + ML スコアリング
$ fav init --template enterprise      # マルチテナント + 監査ログ
$ fav init --template data-quality    # データ品質検証パイプライン
$ fav init --template distributed     # マルチノード par
```

各テンプレートに `README.md`・動作確認コマンド・`fav.toml` を同梱。

**完了条件**: Rust テスト 2 件（3613 + 2 = 3615）
- `init_template_ai_etl_valid`
- `init_template_data_quality_valid`

---

### v72.7.0 — Hot Reload 改善（`fav watch` 2.0）

ファイル変更を検知して型チェック + 差分ステージのみ再実行する。

```bash
$ fav watch pipeline.fav --on-change "fav check && fav run --dry-run"
Watching pipeline.fav... (Ctrl+C to stop)
[10:32:01] Change detected: pipeline.fav
[10:32:01] Type check: OK (0.8s)
[10:32:01] Dry run: 3 stages would run (LoadCsv, Transform, Validate)
[10:32:01] Ready.
```

**完了条件**: Rust テスト 2 件（3615 + 2 = 3617）
- `watch2_triggers_on_file_change`
- `watch2_runs_custom_command`

---

### v72.8.0 — インタラクティブチュートリアル（`fav learn`）

```bash
$ fav learn
Favnir インタラクティブチュートリアル v1.0

Chapter 1: 最初のパイプライン
[1/5] fn main(ctx: AppCtx) -> Result<Unit, String> を書いてみましょう
>>> _
（正解するまでヒントを出しながら次へ進む）

Chapter 2: 型システムの力
Chapter 3: Rune を使ったデータ処理
Chapter 4: AI パイプライン
Chapter 5: 分散実行
```

**完了条件**: Rust テスト 2 件（3617 + 2 = 3619）
- `learn_chapter1_exists`
- `learn_chapter5_exists`

---

### v72.9.0 — 安定化・コードフリーズ（Developer Experience 2.0 前調整）

**完了条件**: Rust テスト 2 件（3619 + 2 = 3621）
- `dev_exp2_all_stable`
- `vscode_repl2_playground2_e2e`

---

### v73.0.0 — Developer Experience 2.0 宣言 ★クリーンアップ

**宣言文**:

> 「VS Code がパイプラインを補完し、AI がエラーを修正し、
>  REPL が型を即座に返し、Playground がコードを世界と共有する。
>  自然言語一文が、型安全なパイプラインの雛形になる。
>
>  これが Favnir v73.0 — Developer Experience 2.0 の姿である。」

**完了条件**: `v73000_tests` 4 件（3621 + 4 = 3625）
- `cargo_toml_version_is_73_0_0`
- `changelog_has_v73_0_0`
- `milestone_has_dev_exp2`
- `readme_mentions_dev_exp2`

---

## Phase 4: v73.1〜v73.9 + v74.0 — Production Proven

**テーマ**: 「実際のチームが本番で Favnir を動かしている」

開発体験が整ったら、実証だ。
データコントラクト・品質スコアリング・PII 保護・SLA 監視——
企業のデータパイプラインが要求する非機能要件を整備する。
さらに paper Rune を実装に昇格させ、ドッグフーディングで実証する。
C（実証・品質）を中心に構成する。

---

### v73.1.0 — データコントラクト

パイプラインのステージ境界にスキーマ・SLA・品質条件を宣言する。
違反はコンパイル時（スキーマ不一致）または実行時（SLA 超過）に検出される。

```favnir
// データコントラクトの宣言
contract OrderPipelineContract {
    input: {
        order_id: String where String.length(self) > 0
        amount:   PositiveFloat
        status:   "pending" | "paid" | "cancelled"
    }
    output: {
        inserted: Int where self >= 0
        skipped:  Int where self >= 0
    }
    sla: {
        max_latency_ms:  5000
        min_throughput:  1000  // rows/sec
        max_error_rate:  0.01  // 1%
    }
    quality: {
        min_completeness: 0.99
        max_null_ratio:   0.01
    }
}

// コントラクトをステージに適用
stage ProcessOrders: OrderPipelineContract.Input -> OrderPipelineContract.Output = |rows| {
    // 違反は実行前にコンパイルエラーまたは実行時例外
    ...
}
```

**完了条件**: Rust テスト 2 件（3625 + 2 = 3627）
- `data_contract_schema_mismatch_error`
- `data_contract_sla_monitoring`

---

### v73.2.0 — データ品質スコアリング（`fav quality`）

```bash
$ fav quality report pipeline.fav --input data.csv
Favnir Data Quality Report
==========================
Overall Score: 87/100

Dimension        Score   Detail
──────────────── ─────── ────────────────────────────────
Completeness      94%    58/1000 rows have null fields
Validity          89%    112 schema violations (amount < 0)
Consistency       78%    220 potential duplicates
Freshness         92%    8% of records older than 24h
Referential       95%    52 orphaned foreign keys

Recommendations:
  1. Add `where self > 0.0` constraint to `amount` field
  2. Enable dedup stage: fav add rune dedup
  3. Add freshness filter: Filter.by_age(max_hours: 24)
```

**完了条件**: Rust テスト 2 件（3627 + 2 = 3629）
- `quality_report_completeness_score`
- `quality_report_recommendations`

---

### v73.3.0 — PII 検出・マスキング Rune（`Rune.privacy`）

```favnir
import rune "privacy"

// PII フィールドを自動検出してマスク
stage MaskPII: CustomerRecord -> CustomerRecord = |r| {
    Rune.privacy.mask(r, strategy: Hash, fields: ["email", "phone", "ssn"])
}

// 正規表現ベースの PII スキャン
stage ScanPII: String -> PiiReport = |text| {
    Rune.privacy.scan(text, rules: [EmailPattern, PhonePattern, CreditCardPattern])
}

// GDPR 削除要求対応
fn handle_erasure_request(ctx: AppCtx, user_id: UserId) -> Result<Unit, String> {
    Rune.privacy.gdpr_erase(ctx, user_id, tables: ["orders", "sessions", "events"])
}
```

**完了条件**: Rust テスト 2 件（3629 + 2 = 3631）
- `privacy_rune_mask_pii_fields`
- `privacy_rune_gdpr_erase`

---

### v73.4.0 — 監査ログ + OpenLineage エクスポート

```bash
# すべてのパイプライン実行を追跡
$ fav run pipeline.fav --audit-log audit.jsonl

# OpenLineage 形式でエクスポート
$ fav lineage --export openlineage --output lineage.json
# → Marquez / DataHub / OpenMetadata に送信可能
```

**実装内容:**
- `--audit-log` フラグ: 実行開始・完了・エラーを JSONL に記録
- `fav lineage --export openlineage`: 静的リネージ解析 → OpenLineage JSON
- 実行ごとの runId・parentRunId でパイプライン系譜を追跡

**完了条件**: Rust テスト 2 件（3631 + 2 = 3633）
- `audit_log_records_run_start_end`
- `lineage_export_openlineage_format`

---

### v73.5.0 — SLA 監視 + アラート統合

```toml
# fav.toml
[sla]
max_latency_ms   = 5000
min_throughput   = 1000
max_error_rate   = 0.01

[sla.alerts]
slack   = "https://hooks.slack.com/..."
pagerduty = "${PAGERDUTY_KEY}"
```

```bash
$ fav run pipeline.fav --enforce-sla
[SLA] Latency: 4823ms (< 5000ms OK)
[SLA] Throughput: 1243 rows/sec (> 1000 OK)
[SLA] Error rate: 0.3% (< 1% OK)
All SLA conditions met.
```

**完了条件**: Rust テスト 2 件（3633 + 2 = 3635）
- `sla_violation_triggers_alert`
- `sla_toml_config_parsed`

---

### v73.6.0 — Rune 品質パス（Paper Rune → 実装昇格）

`runes/` ディレクトリに存在するが実装が空の "paper Rune" を実装に昇格させる。

**対象 Paper Rune（優先順）:**

| Rune | 実装内容 |
|---|---|
| `runes/linalg/` | `dot`, `matmul`, `transpose`, `svd` の VM primitive 追加 |
| `runes/autodiff/` | `grad`, `jacobian` の VM primitive 追加 |
| `runes/stats/` | `mean`, `std`, `median`, `t_test` の VM primitive 追加 |
| `runes/timeseries/` | `rolling_mean`, `ewm`, `decompose` の VM primitive 追加 |
| `runes/ml/` | `knn_predict`, `random_forest_fit` の VM primitive 追加 |

各 Rune に `.fav` 実装ファイル・`rune.toml`・統合テストを追加。

**完了条件**: Rust テスト 2 件（3635 + 2 = 3637）
- `rune_linalg_matmul_runs`
- `rune_stats_mean_std_runs`

---

### v73.7.0 — ドッグフーディング Sprint（Favnir で Favnir を運用）

Favnir 自身の開発ワークフローに Favnir パイプラインを使う実証スプリント。

**実装するパイプライン:**

| パイプライン | 内容 |
|---|---|
| `pipelines/benchmark_analytics.fav` | bench JSON を集計してトレンド可視化 |
| `pipelines/coverage_report.fav` | テストカバレッジ → Slack 通知 |
| `pipelines/changelog_lint.fav` | CHANGELOG.md の形式を検証 |
| `pipelines/rune_catalog_sync.fav` | `runes/` ディレクトリ → catalog.mdx 自動更新 |
| `pipelines/doc_link_check.fav` | MDX ファイルの broken link を検出 |

全パイプラインが `fav run` で完走し、実際の CI に組み込まれることを確認。

**完了条件**: Rust テスト 2 件（3637 + 2 = 3639）
- `dogfooding_benchmark_pipeline_runs`
- `dogfooding_doc_link_check_runs`

---

### v73.8.0 — GitHub Actions 公式 Action

```yaml
# .github/workflows/favnir-ci.yml
steps:
  - uses: favnir/setup-fav@v1
    with:
      version: "75.0.0"

  - name: Type Check
    run: fav check pipeline.fav

  - name: Test
    run: fav test pipeline.fav

  - name: Quality Gate
    run: fav quality report pipeline.fav --min-score 80 --fail-below

  - name: Audit
    run: fav audit --deny-high
```

**実装内容:**
- `favnir/setup-fav` Action（GitHub Releases から fav バイナリをダウンロード）
- `action.yml`・`README.md`・使用例

**完了条件**: Rust テスト 2 件（3639 + 2 = 3641）
- `github_action_setup_fav_action_yml_valid`
- `github_action_fav_binary_url_format`

---

### v73.9.0 — 安定化・コードフリーズ（Production Proven 前調整）

**完了条件**: Rust テスト 2 件（3641 + 2 = 3643）
- `production_proven_all_stable`
- `dogfooding_all_5_pipelines_pass`

---

### v74.0.0 — Production Proven 宣言 ★クリーンアップ

**宣言文**:

> 「データコントラクトがスキーマ境界を守り、品質スコアが劣化を警告する。
>  PII が型で保護され、監査ログが法的要件を満たす。
>  Favnir が Favnir 自身を運用し、GitHub Action が CI に溶け込む。
>
>  これが Favnir v74.0 — Production Proven の姿である。」

**完了条件**: `v74000_tests` 4 件（3643 + 4 = 3647）
- `cargo_toml_version_is_74_0_0`
- `changelog_has_v74_0_0`
- `milestone_has_production_proven`
- `readme_mentions_production_proven`

---

## Phase 5: v74.1〜v74.9 + v75.0 — Favnir 2.0 宣言

**テーマ**: 「言語・型・開発体験・実証——4つが揃った」

4つのフェーズで積み上げたものを統合・磨き上げ、Favnir 2.0 として宣言する。
コミュニティ・エコシステム・ドキュメント整備が中心。

---

### v74.1.0 — Rune マーケットプレイス（バージョン管理・依存解決）

```bash
# 公式マーケットプレイスへの公開
$ fav publish rune ./runes/mycompany-crm
Published: mycompany/crm@1.0.0

# インストール
$ fav add rune mycompany/crm@^1.0
# fav.toml に [rune.deps] として記録

# 依存関係一覧
$ fav rune list
  mycompany/crm  1.0.2  (latest: 1.0.2)
  favnir/json    9.0.0  (latest: 9.0.0)
  favnir/postgres 5.1.0 (latest: 5.2.0) ← update available
```

**完了条件**: Rust テスト 2 件（3647 + 2 = 3649）
- `rune_marketplace_publish_format`
- `rune_marketplace_add_updates_toml`

---

### v74.2.0 — Multi-tenant Runtime

```toml
# fav.toml
[tenant]
isolation = "strict"        # ステージ間でリソースを分離
quota.max_memory_mb = 512
quota.max_cpu_pct   = 80
quota.max_rows      = 1_000_000

[tenant.team_a]
db_url     = "${TEAM_A_DB_URL}"
s3_bucket  = "team-a-data"

[tenant.team_b]
db_url     = "${TEAM_B_DB_URL}"
s3_bucket  = "team-b-data"
```

**完了条件**: Rust テスト 2 件（3649 + 2 = 3651）
- `multitenant_config_parsed`
- `multitenant_resource_quota_enforced`

---

### v74.3.0 — Documentation Site 2.0

```
新規・大幅拡充:
- Getting Started (5分チュートリアル)
- Language Reference (全構文・全エラーコード)
- Rune Catalog (実装済み全 Rune のドキュメント)
- Cookbook (10+ レシピ: AI ETL / 分散 / データ品質...)
- Migration Guide (v35 → v75 の移行手順)
- API Reference (fav CLI の全フラグ)
- Video Transcripts (将来の動画対応を見越した構造)
```

**完了条件**: Rust テスト 2 件（3651 + 2 = 3653）
- `docs_site2_getting_started_exists`
- `docs_site2_migration_guide_v35_to_v75`

---

### v74.4.0 — OSS Hardening

GitHub 上での公開 OSS として機能するための整備。

```
- CONTRIBUTING.md（コントリビュートガイド・PR テンプレート）
- SECURITY.md（脆弱性報告手順）
- .github/ISSUE_TEMPLATE/（バグ報告・機能要望テンプレート）
- CODE_OF_CONDUCT.md
- 依存ライブラリのライセンス確認（cargo-deny）
- SBOM（Software Bill of Materials）生成
```

**完了条件**: Rust テスト 2 件（3653 + 2 = 3655）
- `oss_contributing_md_exists`
- `oss_security_md_exists`

---

### v74.5.0 — Pipeline Scheduling（`fav schedule`）

```bash
# cron ベースのパイプライン定期実行
$ fav schedule add daily-report \
    --cron "0 9 * * *" \
    --pipeline pipelines/daily_report.fav \
    --notify slack://my-channel

$ fav schedule list
NAME            CRON          LAST RUN              STATUS
daily-report    0 9 * * *     2026-08-08 09:00:02   OK
hourly-sync     0 * * * *     2026-08-08 10:00:01   OK

$ fav schedule run daily-report  # 即時実行
```

**完了条件**: Rust テスト 2 件（3655 + 2 = 3657）
- `schedule_add_parses_cron`
- `schedule_list_returns_entries`

---

### v74.6.0 — `fav audit`（依存関係セキュリティ）

```bash
$ fav audit
Auditing 47 dependencies...

CRITICAL  libduckdb-sys 1.2.2  CVE-2026-XXXX  Update to 1.3.0
HIGH      tokio 1.38.0         CVE-2026-YYYY  Update to 1.38.1
OK        45 dependencies clean

$ fav audit --fix
Updated: libduckdb-sys 1.2.2 → 1.3.0
Updated: tokio 1.38.0 → 1.38.1
```

**完了条件**: Rust テスト 2 件（3657 + 2 = 3659）
- `audit_detects_vulnerable_dep`
- `audit_fix_updates_cargo_toml`

---

### v74.7.0 — コミュニティ Rune 品質基準

コミュニティが公開した Rune の品質を担保するための基準と検証ツール。

```bash
# Rune 公開前のチェック
$ fav rune validate ./runes/my-rune
✓ rune.toml: valid
✓ implementation: my-rune.fav (247 lines)
✓ tests: 3 test cases found
✓ documentation: README.md exists
⚠ No example .fav file found
Score: 85/100 (Publish requires ≥ 80)
```

**完了条件**: Rust テスト 2 件（3659 + 2 = 3661）
- `rune_validate_scoring`
- `rune_validate_min_score_enforced`

---

### v74.8.0 — 統合デモ（v70〜v74 の全機能を使ったショーケース）

すべてのフェーズで実装した機能を一本のデモパイプラインで示す。

```
infra/e2e-demo/favnir2-showcase/
├── pipeline.fav          # AI ETL + 依存型 + データコントラクト + 分散実行
├── fav.toml              # マルチテナント + SLA + スケジュール
├── rune.toml             # カスタム Rune 依存
├── contract.fav          # データコントラクト定義
├── quality.fav           # 品質スコアリングパイプライン
└── README.md             # 実行手順
```

**完了条件**: Rust テスト 2 件（3661 + 2 = 3663）
- `showcase_demo_structure_complete`
- `showcase_pipeline_fav_valid`

---

### v74.9.0 — 安定化・コードフリーズ（Favnir 2.0 前最終調整）

v70.1〜v74.8 の全機能を通しで確認する最終安定化スプリント。

**完了条件**: Rust テスト 2 件（3663 + 2 = 3665）
- `favnir2_full_sprint_all_stable`
- `favnir2_e2e_showcase_runs`

---

### v75.0.0 — Favnir 2.0 宣言 ★クリーンアップ

**宣言文**:

> 「compiler.fav が Favnir を完全に記述し、型システムが次元と制約を保証する。
>  依存型がベクトルの次元を守り、refined type がゼロ除算をコンパイル時に止める。
>  VS Code がパイプラインを補完し、AI がエラーを修正し、
>  実際のデータチームが本番で Favnir を走らせている。
>
>  データコントラクトがスキーマ境界を守り、品質スコアが劣化を警告する。
>  Favnir が Favnir 自身を運用し、Rune マーケットプレイスが
>  コミュニティの知恵を型安全なピースとして流通させる。
>
>  これが Favnir v75.0 — Favnir 2.0 の姿である。」

**完了条件**: `v75000_tests` 4 件（3665 + 4 = 3669）
- `cargo_toml_version_is_75_0_0`
- `changelog_has_v75_0_0`
- `milestone_has_favnir_2`
- `readme_mentions_favnir_2`

---

## テスト数推移（計画値）

| バージョン | テスト数 | 増加 | 備考 |
|---|---|---|---|
| v70.0.0（ベース） | 3,559 | — | Intelligent ETL 1.0 宣言 |
| v70.1〜v70.9 | 3,559 + 18 = 3,577 | +18 | 各 +2 |
| v71.0.0 | 3,577 + 4 = 3,581 | +4 | Language Complete 宣言 |
| v71.1〜v71.9 | 3,581 + 18 = 3,599 | +18 | 各 +2 |
| v72.0.0 | 3,599 + 4 = 3,603 | +4 | Type System 2.0 宣言 |
| v72.1〜v72.9 | 3,603 + 18 = 3,621 | +18 | 各 +2 |
| v73.0.0 | 3,621 + 4 = 3,625 | +4 | Developer Exp 2.0 宣言 |
| v73.1〜v73.9 | 3,625 + 18 = 3,643 | +18 | 各 +2 |
| v74.0.0 | 3,643 + 4 = 3,647 | +4 | Production Proven 宣言 |
| v74.1〜v74.9 | 3,647 + 18 = 3,665 | +18 | 各 +2 |
| v75.0.0 | 3,665 + 4 = 3,669 | +4 | Favnir 2.0 宣言 |

**合計増加**: +110 tests（3,559 → 3,669）

---

## 積み残し解消マップ

| 積み残し項目 | 対応バージョン | 対応内容 |
|---|---|---|
| compiler.fav multi-param ctx 未対応 | **v70.1** | `parse_fn_params` 修正 |
| benchmarks/compare.fav 旧構文 | **v70.1** | strict mode 復元 |
| versions/current.md 旧情報 | **v70.1** | 同期 |
| bench.yml の continue-on-error 暫定 | **v70.1** | v70.1 完了後に外す |
| fav migrate 未実装 | **v70.2** | 新規実装 |
| fav bench 未実装 | **v70.3** | 新規実装 |
| Paper Rune 多数 | **v73.6** | linalg / autodiff / stats / timeseries / ml を実装昇格 |
| fav doctor 未実装 | **v70.8** | 新規実装 |

---

## B / C 分類マップ

| フェーズ | B 比率 | C 比率 | 主要特徴 |
|---|---|---|---|
| v71.0 Language Complete | 70% | 30% | compiler.fav 完全化・エラー診断 |
| v72.0 Type System 2.0 | 85% | 15% | 依存型・refined type・AOT |
| v73.0 Developer Exp 2.0 | 20% | 80% | VS Code・AI アシスタント・REPL |
| v74.0 Production Proven | 15% | 85% | データ品質・ドッグフーディング |
| v75.0 Favnir 2.0 | 40% | 60% | マーケットプレイス・宣言 |

---

## 参考リンク

- 前フェーズ: `versions/roadmap/roadmap-v65.1-v70.0.md`
- 現行マスター: `versions/roadmap/roadmap-v70.1-v75.0.md`（本文書）
- 達成宣言: `MILESTONE.md`
- 進行状況: `versions/current.md`
