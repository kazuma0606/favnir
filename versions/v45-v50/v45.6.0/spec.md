# Spec: v45.6.0 — エラーメッセージ改善 Phase 1（E0101〜E0200）

Date: 2026-07-16
Status: TODO

---

## 概要

`error_catalog.rs` に `suggestion` フィールドを追加し、静的カタログエントリに修正提案テキストを付与する。
あわせて `Expr::Apply` の引数数不一致エラー（line ~4724）に動的 hint を追加し、ユーザーが何が期待されているかを理解しやすくする。

---

## 現状分析

| 機能 | 状態 |
|---|---|
| `levenshtein_candidates` フリー関数 | 実装済み（checker.rs line 837） |
| `TypeError.hints: Vec<String>` | 実装済み |
| `type_error_h` で hints 付き発行 | 実装済み |
| `Expr::Ident` 未定義 → levenshtein hint | **実装済み**（E0102、line 4584） |
| `Expr::Apply` 引数数不一致 → hint | **未実装**（E0101 のみ、hint なし） |
| `ErrorEntry.suggestion` フィールド | **未実装** |

---

## §1 — `ErrorEntry` に `suggestion` フィールド追加

`error_catalog.rs` の `ErrorEntry` struct に追加:

```rust
pub suggestion: Option<&'static str>,
```

**型について**: ロードマップでは `Option<String>` と記載されているが、`ErrorEntry` の他フィールドはすべて `&'static str` であり、`ERROR_CATALOG` は `const` スライス。ヒープ割り当てが不要かつコンパイル時に確定するため `&'static str` が適切。

**全エントリへの追加**: `ErrorEntry` は `Default` を実装していないため `..Default::default()` は使えない。全 88 エントリに明示的に `suggestion: None,` を追加する必要がある。

### 追加する suggestion テキスト（代表例）

| コード | suggestion |
|---|---|
| E0101 | `"Check stage names for typos, or verify the return type matches the declared type."` |
| E0102 | `"Use `bind x <- expr` to introduce a variable, or check for typos in the name."` |
| E0103 | `"Add a transformation stage between them, or change one stage's type to match."` |

---

## §2 — `Expr::Apply` 引数数不一致に hint 追加

`checker.rs` の `Expr::Apply` → `Type::Fn(params, ret)` 分岐（line ~4724）で引数数不一致時に hint を追加:

```rust
if inst_params.len() != arg_tys.len() {
    let hint = if let Expr::Ident(fn_name, _) = func.as_ref() {
        format!(
            "function `{}` expects {} argument(s), but {} were provided",
            fn_name, inst_params.len(), arg_tys.len()
        )
    } else {
        format!(
            "this function expects {} argument(s), but {} were provided",
            inst_params.len(), arg_tys.len()
        )
    };
    self.type_error_h("E0101", format!("expected {} argument(s), got {}", ...), span, vec![hint]);
    return Type::Error;
}
```

---

## §3 — 既存 levenshtein hint の動作確認

`Expr::Ident` 未定義時の levenshtein hint は実装済み（line 4565-4588）。
テストでその動作を確認するだけでよい。

---

## テスト

| テスト名 | 検証内容 | 期待エラーコード | 期待 hint |
|---|---|---|---|
| `e0102_suggestion_similar_name` | `ordr()` 呼び出し（`order` の typo） | E0102 | hints に "order" が含まれる |
| `e0101_suggestion_arg_count` | `add(1)`（2引数関数に1引数） | E0101 | hints に "2" が含まれる |

テスト数: 2980 → **2982**

---

## 変更ファイル

| ファイル | 変更内容 |
|---|---|
| `fav/src/error_catalog.rs` | `ErrorEntry` に `suggestion: Option<&'static str>` 追加、全エントリに `suggestion: None,`、主要エントリに suggestion テキスト |
| `fav/src/middle/checker.rs` | `Expr::Apply` 引数数不一致箇所に hint 追加 |
| `fav/Cargo.toml` | version `45.5.0` → `45.6.0` |
| `fav/src/driver.rs` | `v456000_tests` モジュール追加（2件） |
| `CHANGELOG.md` | v45.6.0 エントリ追加 |
| `versions/current.md` | v45.6.0（2982 tests）に更新 |
| `versions/roadmap/roadmap-v45.1-v46.0.md` | テスト名・テスト数・`suggestion` 型を実態に合わせて修正 |

---

## 変更しないファイル

- `ast.rs` — 変更不要
- `lint.rs` — 変更不要
- `compiler.fav` / `checker.fav` — 変更不要
- `site/` — 変更不要（ドキュメント更新は v45.9.0 で実施）
