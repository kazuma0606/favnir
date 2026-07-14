# v44.2.0 Spec — CEP x Refinement type

## 概要

CEP パターン（`cep pattern`）のイベント節に Refinement type 名が参照されていることを **AST レベルで検出・解析できる**ようにする。

checker.fav への完全統合（`seq`/`any`/`not` パターンの型チェックに refinement 条件を統合）は将来版のスコープ。本バージョンは **パーサー受容 + AST レベル MVP** とする。

ロードマップ（`roadmap-v44.1-v45.0.md`）の `Purchase<HighValue>` 構文は現 AST（`CepExpr::Event(String)` — 型パラメータなし）では非対応。本バージョンでは「CEP イベント名が refinement type 名と一致する」パターンを検出する MVP とする。

---

## AST 確認事項

- `CepPatternDef { name: String, body: Vec<CepClause>, span: Span }` — `Item::CepPatternDef`
- `CepClause { expr: CepExpr, within_secs: Option<i64>, span: Span }` — 各節
- `CepExpr::Event(String)` — 単純イベント名（`Login`、`HighValue` 等）
- `CepExpr::Seq(Vec<CepExpr>)` / `Any(Vec<CepExpr>)` / `Not(Box<CepExpr>)` — 複合パターン
- `TypeDef.invariants: Vec<Expr>` — 非空なら refinement type

---

## 機能詳細

### 1. `collect_cep_refinement_event_refs` ヘルパー追加

`driver.rs` に以下の関数を追加:

```rust
pub fn collect_cep_refinement_event_refs(src: &str, filename: &str) -> Vec<String>
```

- ソースを parse して `TypeDef` の refinement（`invariants` 非空）名を収集
- `CepPatternDef` の各 `CepClause.expr` を再帰走査し、`CepExpr::Event(name)` の `name` が refinement type 名に一致するものを検出
- 返り値: `"<filename>:<line>: <pattern_name>: <event_name>"` 形式の文字列リスト

### 2. CepExpr 再帰走査

`seq`/`any` は子リストを再帰、`not` は子を再帰、`Event` は葉ノードでマッチ確認。

---

## テスト

`v44200_tests` 3 件:

| テスト名 | 内容 |
|---|---|
| `cargo_toml_version_is_44_2_0` | `Cargo.toml` に `"44.2.0"` が含まれる |
| `cep_simple_event_matches_refinement_type` | `type HighValue = Float where \|v\| v > 1000.0` + `cep pattern HV { HighValue within 300 }` → 検出 |
| `cep_seq_pattern_refinement_event_detected` | `seq(Login, HighValue)` の中の `HighValue` → 検出（再帰走査確認） |

---

## 完了条件

- `cargo test -j 8 -- --test-threads=8` で **2947 passed; 0 failed**（2944 + 3）
- `v44200_tests` 3 件 pass

---

## 注意事項

- `CepExpr::Event(String)` は型パラメータを持たない（`Purchase<HighValue>` は将来版）
- `collect_cep_expr_refinement_refs` は `pub` なし（内部ヘルパー）
- `CepPatternDef.span` の行番号を `CepClause.span` で補う（clause レベルで行を報告）
- refinement 型名と同名の通常イベントが存在する場合は誤検出となる（MVP 制限、将来版で型パラメータ構文により解決予定）
- ロードマップ推定（2937）は旧見積もり。実績 2944 を基準とする
- `v44100_tests::cargo_toml_version_is_44_1_0` をスタブ化すること
- ロードマップ（`roadmap-v44.1-v45.0.md`）の v44.2.0 節は「型チェック統合」を謳っているが、本バージョンは AST レベル MVP。T5 でロードマップにスコープ制限注記を追記すること
