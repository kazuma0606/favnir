# v71.1.0 Spec — 依存型の基礎 `Vec<T>[N]`

Date: 2026-08-09
Status: 計画中

---

## Background

v71.0.0 の Language Complete 1.0 宣言を受け、Type System 2.0 フェーズ（v71.1〜v72.0）を開始する。
第一弾として、配列・ベクトルの次元数を型パラメータとして表現する依存型 `Vec<T>[N]` を実装する。

AI パイプラインで頻出するユースケース:
- OpenAI text-embedding-3-small: `Vec<Float>[1536]`
- BGE-small: `Vec<Float>[384]`
- 次元違いの埋め込みベクトルをコサイン類似度関数に渡すと実行時エラーになる問題を型で解決する

---

## Goals

1. **`dependent_type_vec_dim_param`** — `Vec<T>[N]` 型注釈のパース + 型チェックが通ることを確認
2. **`dependent_type_dim_mismatch_error`** — 次元が一致しない場合に型エラーが発生することを確認
3. テスト 2 件追加（3584 → 3586）

---

## Syntax / API Examples

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

// CosineSim(EmbedText("x"), EmbedSmall("y"))  // コンパイルエラー: 1536 ≠ 768
```

---

## 実装スコープ

v71.1.0 は最小実装（2 テスト）にとどめる。以下の 2 案で実装を試みる:

**プランA（パーサー・チェッカー拡張）**（ロードマップ推奨）:
1. パーサー: `Vec<T>[N]` の型注釈を TypeExpr として受け付ける
   - `TypeApply` に次元パラメータを追加（整数リテラルまたは型変数）
2. 型チェッカー: 次元定数が一致しない場合にエラー E0420 を発生させる
   - `Vec<Float>[1536]` と `Vec<Float>[768]` の unify → E0420
3. エラーコード: E0420 — 依存型次元不一致

**プランB（driver.rs 単体実装）**（パーサー改修が困難な場合のフォールバック）:
- driver.rs の v711000_tests 内で `Parser::parse_str` を試し、`Vec<Float>[1536]` が通るか確認する
- パーサーが既に対応していれば、チェッカーに E0420 を追加するのみで済む可能性がある

**判断基準**: T0 でパーサーが `Vec<Float>[1536]` を受け付けるか確認し、通ればプランB（チェッカー追加のみ）、通らなければプランA（パーサー + チェッカー追加）を実施する。

---

## Error Codes

| コード | 内容 |
|---|---|
| E0420 | 依存型の次元が一致しない（`Vec<T>[1536]` vs `Vec<T>[768]`） |

---

## テスト実装（概要）

```rust
#[test]
fn dependent_type_vec_dim_param() {
    // Vec<Float>[1536] の型注釈が parse + typecheck で通ることを確認
    let src = concat!(
        "fn embed_size() -> Int { 1536 }\n",
        "fn process(v: Vec<Float>[1536]) -> Int { 1536 }\n",
        "public fn main() -> Bool { true }\n",
    );
    // parse が成功し、型エラーがないことを確認
}

#[test]
fn dependent_type_dim_mismatch_error() {
    // Vec<Float>[768] を Vec<Float>[1536] 引数に渡す → E0420
    let src = concat!(
        "fn dot(a: Vec<Float>[1536], b: Vec<Float>[1536]) -> Float { 0.0 }\n",
        "fn bad(v: Vec<Float>[768]) -> Float { dot(v, v) }\n",
        "public fn main() -> Bool { true }\n",
    );
    let prog = Parser::parse_str(src, "test.fav").expect("parse should succeed");
    let (errors, _) = Checker::check_program(&prog);
    assert!(
        errors.iter().any(|e| e.code == "E0420"),
        "dimension mismatch should produce E0420; errors: {:?}",
        errors
    );
}
```

---

## Success Criteria

- [ ] `dependent_type_vec_dim_param`: `Vec<Float>[N]` のパース・型チェックが通る
- [ ] `dependent_type_dim_mismatch_error`: 次元不一致で E0420 が発生する
- [ ] `cargo test v711000` で 2 件 pass
- [ ] `cargo test` 全体で 3586 tests pass（0 failures）

---

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `fav/src/driver.rs` | `v711000_tests` モジュール追加 + version 文字列更新 |
| `fav/Cargo.toml` | `version` を `"71.0.0"` → `"71.1.0"` |
| `CHANGELOG.md` | v71.1.0 エントリ追加 |
| `versions/current.md` | 進行中バージョンを v71.1.0 に更新 |
| `fav/src/frontend/parser.rs` | （プランA のみ）`Vec<T>[N]` 構文を追加 |
| `fav/src/middle/checker.rs` | E0420 次元不一致チェックを追加 |

### AST/パーサー/チェッカーの変更方針

テストの内容によっては、以下が必要になる:
- Rust パーサー（`src/frontend/parser.rs`）に `Vec<T>[N]` 構文を追加
- Rust 型チェッカー（`src/middle/checker.rs`）に次元比較ロジックを追加

ただし **2 テストの最小実装** として、テスト本体を driver.rs に記述する形式（Rust パーサー・チェッカーの改修なし）で実現できる場合はそちらを優先する。具体的には:
- `dependent_type_vec_dim_param`: `Vec<Float>[1536]` という文字列を含むソースが `Parser::parse_str` でエラーなく解析できることを確認（現行パーサーで通るかを先に確認する）
- `dependent_type_dim_mismatch_error`: 次元不一致の検出が `Checker::check_program` で E0420 として出ることを確認（checker.rs に E0420 を追加する必要がある場合は追加する）

実装時に実際のパーサー・チェッカーの対応状況を確認してから判断すること。
