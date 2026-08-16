# Roadmap v71.1.0 〜 v72.0.0 — Type System 2.0 宣言

Date: 2026-08-08
Status: 未着手（v71.0.0 完了後に開始）

マスターロードマップ: [roadmap-v70.1-v75.0.md](roadmap-v70.1-v75.0.md)

---

## 前提

- 直前完了: v71.0.0「Language Complete 1.0」（tests = 3584）
- 本スプリントは Phase 2「Type System 2.0」の詳細計画
- 目標: v72.0.0「Type System 2.0 宣言」（tests = 3606）

### スプリントの性格

Phase 2 は「型が次元・制約・精緻さを表現する」スプリントである。
v70「Intelligent ETL 1.0」で AI パイプラインの型安全を宣言したが、
次元数・値域制約・コンパイル時評価は未整備だった。
このフェーズでそれらを型システムに統合し、「型で証明する」言語へと進化させる。
B（型システム）85% + C（ランタイム）15% の構成。

---

## バージョン一覧

| バージョン | 内容 | テスト数 | 状態 |
|---|---|---|---|
| v71.1.0 | 依存型の基礎 `Vec<T>[N]` | 3584 + 2 = 3586 | 未着手 |
| v71.2.0 | Refined Types（型レベル制約 `where self`） | 3586 + 2 = 3588 | 未着手 |
| v71.3.0 | Phantom Types（型タグによる誤使用防止） | 3588 + 2 = 3590 | 未着手 |
| v71.4.0 | Const / Compile-Time Evaluation | 3590 + 2 = 3592 | 未着手 |
| v71.5.0 | Generic Constraints（`impl Trait` 風の境界） | 3592 + 2 = 3594 | 未着手 |
| v71.6.0 | AOT Native Compilation 本番品質化 | 3594 + 2 = 3596 | 未着手 |
| v71.7.0 | WebAssembly ターゲット | 3596 + 2 = 3598 | 未着手 |
| v71.8.0 | 型推論強化（型注釈省略可能範囲の拡大） | 3598 + 2 = 3600 | 未着手 |
| v71.9.0 | 安定化・コードフリーズ（Type System 2.0 前調整） | 3600 + 2 = 3602 | 未着手 |
| v72.0.0 | Type System 2.0 宣言 ★クリーンアップ | 3602 + 4 = 3606 | 未着手 |

---

## v71.1.0 — 依存型の基礎 `Vec<T>[N]`

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

**実装内容:**
- AST: `TypeApply` に整数リテラル次元パラメータを追加（`Vec<Float>[N]`）
- checker: 次元変数の伝播・ユニフィケーション
- error: 次元不一致エラー（E0420）

**完了条件**: Rust テスト 2 件（3584 + 2 = 3586）
- `dependent_type_vec_dim_param`
- `dependent_type_dim_mismatch_error`

---

## v71.2.0 — Refined Types（型レベル制約 `where self`）

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

**実装内容:**
- AST: `TypeDef` に `where self <expr>` 節を追加
- checker: refinement 制約の SMT ライクな静的検査（軽量版）
- error: 制約違反エラー（E0421）

**完了条件**: Rust テスト 2 件（3583 + 2 = 3585）
- `refined_type_positive_float`
- `refined_type_violation_compile_error`

---

## v71.3.0 — Phantom Types（型タグによる誤使用防止）

異なる意味を持つ同型値の混用をコンパイル時に防ぐ。

```favnir
// UserId と OrderId は String だが混用不可
type UserId  = phantom String
type OrderId = phantom String

fn get_user(id: UserId) -> User { ... }
fn get_order(id: OrderId) -> Order { ... }

bind uid <- UserId("u-123")
bind oid <- OrderId("o-456")
get_user(uid)   // OK
get_user(oid)   // コンパイルエラー: OrderId ≠ UserId
```

**実装内容:**
- `type X = phantom T` 宣言のパース・型チェック
- phantom 型のコンストラクタ（`UserId("...")`）とアンラップ（`UserId.unwrap(uid)`）
- error: phantom 型不一致エラー（E0422）

**完了条件**: Rust テスト 2 件（3585 + 2 = 3587）
- `phantom_type_prevents_id_confusion`
- `phantom_type_explicit_cast`

---

## v71.4.0 — Const / Compile-Time Evaluation

定数式をコンパイル時に評価する。依存型の次元数指定に必須。

```favnir
const MAX_BATCH_SIZE: Int    = 1024
const EMBED_DIM:      Int    = 1536
const API_BASE_URL:   String = "https://api.favnir.dev"

// 依存型で定数を使用
stage EmbedText: String -> Vec<Float>[EMBED_DIM] = |text| {
    OpenAI.embed(text, dim: EMBED_DIM)
}

// 算術定数式
const HALF_DIM: Int = EMBED_DIM / 2   // 768（コンパイル時評価）
```

**実装内容:**
- `const` 宣言のパース・型チェック
- `ConstEval` — 整数・文字列・算術式のコンパイル時評価
- 依存型の次元パラメータで定数を参照できるよう checker を拡張

**完了条件**: Rust テスト 2 件（3587 + 2 = 3589）
- `const_eval_int_expr`
- `const_used_in_dependent_type`

---

## v71.5.0 — Generic Constraints（`impl Trait` 風の境界）

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

**実装内容:**
- 型パラメータ境界 `[T: A & B]` のパース・checker 統合
- `impl Trait` 記法のサポート（既存 `interface` / `impl` との連携）
- error: 境界を満たさない型の使用（E0423）

**完了条件**: Rust テスト 2 件（3589 + 2 = 3591）
- `generic_constraint_multi_interface`
- `generic_constraint_impl_trait`

---

## v71.6.0 — AOT Native Compilation 本番品質化

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

**実装内容:**
- `cranelift_aot.rs` 強化 — 全 VM opcode を Cranelift IR に変換
- クロスコンパイルターゲット（`--arch arm64`）
- バイナリサイズ最適化（LTO / strip）

**完了条件**: Rust テスト 2 件（3591 + 2 = 3593）
- `aot_native_binary_compiles`
- `aot_native_binary_runs_hello`

---

## v71.7.0 — WebAssembly ターゲット

Favnir パイプラインを WASM バイナリとして出力する。
Playground でのブラウザ内実行・エッジコンピューティング対応。

```bash
# WASM 出力
$ fav build --target wasm pipeline.fav -o pipeline.wasm
$ wasm-run pipeline.wasm --input data.json

# Playground での利用（ブラウザ内完結）
# @favnir/wasm パッケージ更新により自動対応
```

**実装内容:**
- `fav build --target wasm` — wasm32 バックエンド（v51.7.0 以降実装済み）のテストカバレッジ確立
- WASM 標準入出力ブリッジ（実装済み）の動作確認テスト追加
- `@favnir/wasm` npm パッケージ更新は CI/CD スコープ外（後続バージョンで対応）

**完了条件**: Rust テスト 2 件（3602 + 2 = 3604）
- `wasm_target_compiles`
- `wasm_target_runs_simple_pipeline`

---

## v71.8.0 — 型推論強化（型注釈省略可能範囲の拡大）

ローカル変数・クロージャ引数での型注釈を省略できる範囲を広げる。

```favnir
// Before（型注釈が必要だった箇所）
bind items: List<Order>  <- load_orders(ctx)
bind total: Float        <- List.fold(items, 0.0, |acc: Float, o: Order| acc + o.amount)

// After（推論で省略可能）
bind items <- load_orders(ctx)      // List<Order> を推論
bind total <- List.fold(items, 0.0, |acc, o| acc + o.amount)  // Float を推論
```

**実装内容:**
- `bind` 束縛・クロージャ引数の型注釈省略が既存実装で動作することを確認するテストを追加
- `fresh_var` / `unify` の改善は実施しない（既存挙動で動作するため）
- `fav check --show-types` は v12.5.0 実装済み（本バージョンでの変更なし）

**完了条件**: Rust テスト 2 件（3604 + 2 = 3606）
- `type_infer_local_var_omit_annotation`
- `type_infer_closure_arg_omit`

---

## v71.9.0 — 安定化・コードフリーズ（Type System 2.0 前調整）

v71.1〜v71.8 の全機能が正常動作することを確認する安定化バージョン。
依存型・refined type・phantom type の E2E テストを実施する。

**完了条件**: Rust テスト 2 件（3606 + 2 = 3608）
- `type_system_2_all_stable`
- `dependent_refined_phantom_e2e`

---

## v72.0.0 — Type System 2.0 宣言 ★クリーンアップ

**宣言文**:

> 「依存型がベクトルの次元を守り、refined type がゼロ除算を型で止める。
>  Phantom type が ID の混用を防ぎ、定数がコンパイル時に評価される。
>  AOT バイナリが Docker 不要で動き、WASM がパイプラインをブラウザへ運ぶ。
>
>  これが Favnir v72.0 — Type System 2.0 の姿である。」

**クリーンアップ作業:**
- `cargo clean` 実施
- `Cargo.toml` バージョンを `72.0.0` に更新
- `CHANGELOG.md` に v72.0.0 エントリを追加
- `MILESTONE.md` に「Type System 2.0」を追記
- `README.md` に v72.0 達成を追記
- `versions/current.md` を更新（進行中 → v72.1.0）

**完了条件**: `v72000_tests` 4 件（3608 + 4 = 3612）
- `cargo_toml_version_is_72_0_0`
- `changelog_has_v72_0_0`
- `milestone_has_type_system_2`
- `readme_mentions_type_system_2`

---

## テスト数推移（本スプリント）

| バージョン | テスト数 | 増加 |
|---|---|---|
| v71.0.0（ベース） | 3,581 | — |
| v71.1.0 | 3,583 | +2 |
| v71.2.0 | 3,585 | +2 |
| v71.3.0 | 3,587 | +2 |
| v71.4.0 | 3,589 | +2 |
| v71.5.0 | 3,591 | +2 |
| v71.6.0 | 3,593 | +2 |
| v71.7.0 | 3,595 | +2 |
| v71.8.0 | 3,597 | +2 |
| v71.9.0 | 3,599 | +2 |
| v72.0.0（宣言） | 3,603 | +4 |

**本スプリント合計**: +22 tests（3,581 → 3,603）

---

## 参考リンク

- マスターロードマップ: `versions/roadmap/roadmap-v70.1-v75.0.md`
- 前スプリント（完了予定）: `versions/roadmap/roadmap-v70.1-v71.0.md`
- 次スプリント: `versions/roadmap/roadmap-v72.1-v73.0.md`
- 進行状況: `versions/current.md`
