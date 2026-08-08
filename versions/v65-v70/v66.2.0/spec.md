# v66.2.0 Spec — LLM Extraction Stage（型安全 JSON 抽出）

Version: 66.2.0
Status: 未着手
Base tests: 3477
Target tests: 3479

---

## 概要

LLM の出力を型安全なスキーマに変換するステージを提供する `Rune.llm.extract` を実装する。
非構造テキスト → 型付きレコードの変換を保証し、スキーマ違反は型エラーとする。
既存の `runes/llm/` に `llm_extract.fav` を追加する形で拡張する。

```favnir
// 利用例（用途のイメージ）
// ※ ジェネリック型 T は将来フェーズで型システムに登録する
// 今バージョンは String をプレースホルダーとして使用

public stage ExtractInvoice: String -> String = |raw_text| {
    Rune.llm.extract(raw_text, "InvoiceData", "claude-sonnet-4-6")
}
```

ロードマップ `roadmap-v66.1-v67.0.md` の v66.2.0 セクションに準拠。

> **スコープ縮小の明示**: ロードマップの利用例では `T`（ジェネリック型パラメータ）と `Option<T>` を使用しているが、
> 型システムへの登録は将来フェーズ。本バージョンでは `String` をプレースホルダーとして
> 関数シグネチャを確立することに専念する。

---

## 前提確認（T0 で実施）

- `cargo test -j 8 -- --test-threads=8` でベース 3477 tests passed, 0 failed を確認
- `fav/Cargo.toml` の version が `"66.0.0"` であることを確認（sub-version では Cargo.toml は更新しない）
- `runes/llm/llm_extract.fav` が存在しないことを確認（新規作成対象）
- `driver.rs` に `v66100_tests` が存在することを確認（`v66200_tests` の挿入位置）
- `driver.rs` に `v66200_tests` が存在しないことを確認（新規追加）
- `cargo test --bin fav v66100_tests` で 2 件 PASS することを確認（前バージョンが正常）
- `versions/current.md` の「進行中バージョン」が `v66.1.0` であることを確認

---

## 実装スコープ

### 1. `runes/llm/llm_extract.fav` — Rune 実装スタブ（新規作成）

以下の全関数定義を含むスタブファイルを作成する。
今バージョンでは**関数シグネチャの確立**が目的。実際の LLM 抽出は将来フェーズ。

```favnir
// LLM Extraction Stage — Rune.llm.extract
// Schema-typed extraction from LLM output
//
// NOTE: ジェネリック型 T / Option<T> は将来フェーズで型システムに登録する。
//       今バージョンは String をプレースホルダーとして使用。
//       include_str! テストのみ（型チェックエラーは無視する）。

// スキーマ付き単一レコード抽出
// text: LLM への入力テキスト, schema: スキーマ名（文字列）, model: モデル名
public fn extract(text: String, schema: String, model: String) -> String {
    ""
}

// スキーマ付き複数レコード抽出（リスト返し）
public fn extract_list(text: String, schema: String, model: String) -> List<String> {
    []
}

// デフォルト値付き抽出（抽出失敗時は default_val を返す）
// LLMExtractionFallback — extract_or_default が失敗した場合のフォールバック
public fn extract_or_default(text: String, schema: String, model: String, default_val: String) -> String {
    default_val
}

// Option 型抽出（抽出失敗時は "" を返すスタブ）
// extract_maybe — 失敗時に Option<T> を返す（スタブでは "" で代替）
public fn extract_maybe(text: String, schema: String, model: String) -> String {
    ""
}
```

### 2. `driver.rs` — `v66200_tests` 追加

挿入位置: `// -- v66100_tests (v66.1.0)` コメントの直前

```rust
// -- v66200_tests (v66.2.0) -- LLM Extraction Stage --
#[cfg(test)]
mod v66200_tests {
    #[test]
    fn llm_extract_typed_schema() {
        let content = include_str!("../../runes/llm/llm_extract.fav");
        assert!(!content.is_empty(), "llm_extract.fav should not be empty");
        assert!(content.contains("fn extract("), "llm_extract.fav should define extract");
        assert!(
            content.contains("fn extract_list("),
            "llm_extract.fav should define extract_list"
        );
        assert!(
            content.contains("schema"),
            "llm_extract.fav should reference schema parameter"
        );
    }

    #[test]
    fn llm_extract_schema_mismatch_error() {
        let content = include_str!("../../runes/llm/llm_extract.fav");
        assert!(
            content.contains("fn extract_or_default("),
            "llm_extract.fav should define extract_or_default"
        );
        assert!(
            content.contains("fn extract_maybe("),
            "llm_extract.fav should define extract_maybe"
        );
        assert!(
            content.contains("LLMExtractionFallback"),
            "llm_extract.fav should reference LLMExtractionFallback"
        );
    }
}
```

挿入後、`cargo build` でエラーなしを確認。

---

## 完了条件

- `runes/llm/llm_extract.fav` が存在し空でない
- `llm_extract.fav` に全 4 関数が定義されている:
  - `extract`, `extract_list`（基本抽出）
  - `extract_or_default`, `extract_maybe`（フォールバック）
- `cargo test --bin fav v66200_tests` で 2 件 PASS
  - `llm_extract_typed_schema` PASS
  - `llm_extract_schema_mismatch_error` PASS
- `cargo test -j 8 -- --test-threads=8` で 3479 tests passed, 0 failed

---

## 非スコープ

- ジェネリック型 `T` / `Option<T>` の型システム登録 — 将来フェーズ
- 実際の LLM API 呼び出し実装 — 将来フェーズ
- JSON スキーマ自動生成（`schema` 定義 → JSON Schema 変換） — 将来フェーズ
- バリデーション（型チェック + 必須フィールド確認） — 将来フェーズ
- `runes/llm/rune.toml` の `exports` フィールド更新 — 既存 rune.toml が非標準形式のため今バージョンは省略
- `fav check` での型チェック通過 — 今バージョンは `include_str!` テストのみ
- CHANGELOG.md 更新 — v67.0.0 宣言時に一括追記
- site/ MDX ドキュメント作成 — v66.9.0 安定化時に一括作成するため今バージョンは省略

---

## 技術ノート

### `include_str!` パス（`fav/src/driver.rs` 起点）

- `"../../runes/llm/llm_extract.fav"` → `favnir/runes/llm/llm_extract.fav`

### `contains` 判定の設計方針

- `contains("fn extract(")` — `public fn extract(` にマッチ。`fn extract_list(` / `fn extract_or_default(` / `fn extract_maybe(` とは文字列として区別可能（偽陽性なし）
- `contains("fn extract_list(")` — `public fn extract_list(` にマッチ
- `contains("schema")` — 引数名 `schema:` にマッチ（全 4 関数に共通）
- `contains("fn extract_or_default(")` — `public fn extract_or_default(` にマッチ
- `contains("fn extract_maybe(")` — `public fn extract_maybe(` にマッチ
- `contains("LLMExtractionFallback")` — コメント `// LLMExtractionFallback — extract_or_default が失敗した場合のフォールバック` でマッチ。**注意**: コメントを変更・削除した場合は当該テストアサーションも連動して更新すること

### Favnir 構文ルール（v66.x 共通）

- `bind x <- expr` は Result/Option を返す式にのみ使用する（スタブでは不要）
- `let` は使わない
- `Math.sqrt` を使う（`Float.sqrt` は VM に存在しない）
- `Float.from_int` は VM に存在しない

### 既存 llm Rune との関係

- `runes/llm/llm.fav` — 既存ファイル（変更しない）
- `runes/llm/client.fav` — 既存ファイル（変更しない）
- `runes/llm/llm_extract.fav` — 今バージョンで新規追加
- `runes/llm/rune.toml` — 変更しない（既存の非標準形式を保持）
