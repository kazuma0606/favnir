# v66.3.0 Spec — Embedding Pipeline Rune（`Rune.embed`）

Version: 66.3.0
Status: 未着手
Base tests: 3479
Target tests: 3481

---

## 概要

ローカルモデル・OpenAI・Cohere 等の埋め込みモデルを統一インターフェースで扱う
Rune `Rune.embed` を実装する。モデルの切り替えは設定変更のみで対応可能。
バッチ処理・キャッシュ付き埋め込みも提供する。

```favnir
// 利用例（用途のイメージ）
// ※ Vec<Float>[N] の次元型パラメータは将来フェーズで型システムに登録する
// 今バージョンは List<Float> をプレースホルダーとして使用

public stage EmbedOpenAI: String -> List<Float> = |text| {
    Rune.embed.openai(text, model: "text-embedding-3-small")
}

public stage EmbedLocal: String -> List<Float> = |text| {
    Rune.embed.local(text, model: "nomic-embed-text")
}
// ※ ロードマップのキーワード引数形式（model: "..."）に合わせた表記。
// 　 スタブの fn シグネチャは位置引数だが、将来のキーワード引数対応を見越して利用例はキーワード形式とする。
```

ロードマップ `roadmap-v66.1-v67.0.md` の v66.3.0 セクションに準拠。

> **スコープ縮小の明示**: ロードマップの利用例では `Vec<Float>[1536]` / `Vec<Float>[1024]` / `Vec<Float>[768]`
> 等の次元型パラメータを使用しているが、型システムへの登録は将来フェーズ。
> 本バージョンでは `List<Float>` をプレースホルダーとして関数シグネチャを確立することに専念する。

---

## 前提確認（T0 で実施）

- `cargo test -j 8 -- --test-threads=8` でベース 3479 tests passed, 0 failed を確認
- `fav/Cargo.toml` の version が `"66.0.0"` であることを確認（sub-version では Cargo.toml は更新しない）
- `runes/embed/` ディレクトリが存在しないことを確認（新規作成対象）
- `driver.rs` に `v66200_tests` が存在することを確認（`v66300_tests` の挿入位置）
- `driver.rs` に `v66300_tests` が存在しないことを確認（新規追加）
- `cargo test --bin fav v66200_tests` で 2 件 PASS することを確認（前バージョンが正常）
  - 前バージョンのテスト関数名: `llm_extract_typed_schema`, `llm_extract_schema_mismatch_error`
- `versions/current.md` の「進行中バージョン」が `v66.2.0` であることを確認

---

## 実装スコープ

### 1. `runes/embed/rune.toml` — Rune メタデータ

```toml
[rune]
name        = "embed"
version     = "0.1.0"
description = "Embedding Pipeline Rune for Favnir — OpenAI / Cohere / local (Ollama) unified embedding interface with batch and cache support"
entry       = "embed.fav"
effects     = []

[dependencies]
```

### 2. `runes/embed/embed.fav` — Rune 実装スタブ

以下の全関数定義を含むスタブファイルを作成する。
今バージョンでは**関数シグネチャの確立**が目的。実際の埋め込み API 呼び出しは将来フェーズ。

```favnir
// Embedding Pipeline Rune — Rune.embed
// OpenAI / Cohere / local (Ollama) unified embedding interface
//
// NOTE: Vec<Float>[N]（次元型パラメータ: 1536 / 1024 / 768）は将来フェーズで型システムに登録する。
//       今バージョンは List<Float> をプレースホルダーとして使用。
//       include_str! テストのみ（型チェックエラーは無視する）。

// --- プロバイダー別埋め込み ---

// OpenAI 埋め込み（text-embedding-3-small: 1536 次元）
public fn openai(text: String, model: String) -> List<Float> {
    []
}

// Cohere 埋め込み（embed-english-v3.0: 1024 次元）
public fn cohere(text: String, model: String) -> List<Float> {
    []
}

// ローカルモデル埋め込み（Ollama 経由: nomic-embed-text: 768 次元）
// EmbedLocalProvider — Ollama を使ったローカル推論
public fn local(text: String, model: String) -> List<Float> {
    []
}

// --- バッチ処理 ---

// バッチ埋め込み: テキストのリスト → 埋め込みベクトルのリスト
public fn embed_batch(texts: List<String>, model: String) -> List<List<Float>> {
    []
}

// --- キャッシュ付き埋め込み ---

// キャッシュ付き埋め込み（同一入力の再計算を防ぐ）
// embed_cached — cache_key でキャッシュを識別する
public fn embed_cached(text: String, model: String, cache_key: String) -> List<Float> {
    []
}
```

### 3. `driver.rs` — `v66300_tests` 追加

挿入位置: `// -- v66200_tests (v66.2.0)` コメントの直前

```rust
// -- v66300_tests (v66.3.0) -- Embedding Pipeline Rune --
#[cfg(test)]
mod v66300_tests {
    #[test]
    fn embed_rune_openai() {
        let content = include_str!("../../runes/embed/embed.fav");
        assert!(!content.is_empty(), "embed.fav should not be empty");
        assert!(content.contains("fn openai("), "embed.fav should define openai");
        assert!(content.contains("fn cohere("), "embed.fav should define cohere");
        assert!(
            content.contains("fn embed_batch("),
            "embed.fav should define embed_batch"
        );
    }

    #[test]
    fn embed_rune_local_model() {
        let content = include_str!("../../runes/embed/embed.fav");
        assert!(content.contains("fn local("), "embed.fav should define local");
        assert!(
            content.contains("fn embed_cached("),
            "embed.fav should define embed_cached"
        );
        assert!(
            content.contains("EmbedLocalProvider"),
            "embed.fav should reference EmbedLocalProvider"
        );
    }
}
```

挿入後、`cargo build` でエラーなしを確認。

---

## 完了条件

- `runes/embed/embed.fav` が存在し空でない
- `runes/embed/rune.toml` が存在する
- `embed.fav` に全 5 関数が定義されている:
  - `openai`, `cohere`, `local`（プロバイダー別埋め込み）
  - `embed_batch`（バッチ処理）
  - `embed_cached`（キャッシュ付き）
- `cargo test --bin fav v66300_tests` で 2 件 PASS
  - `embed_rune_openai` PASS
  - `embed_rune_local_model` PASS
- `cargo test -j 8 -- --test-threads=8` で 3481 tests passed, 0 failed

---

## 非スコープ

- `Vec<Float>[N]` 次元型パラメータの型システム登録 — 将来フェーズ
- 実際の OpenAI / Cohere / Ollama API 呼び出し実装 — 将来フェーズ
- `embed_cached` のキャッシュストア実装 — 将来フェーズ
- `fav check` での型チェック通過 — 今バージョンは `include_str!` テストのみ
- CHANGELOG.md 更新 — v67.0.0 宣言時に一括追記
- site/ MDX ドキュメント作成 — v66.9.0 安定化時に一括作成するため今バージョンは省略
- `rune.toml` の `effects` 更新 — 実際の API 呼び出し実装時には `effects = ["!Http"]` 等の追加が必要（将来フェーズ）

---

## 技術ノート

### `include_str!` パス（`fav/src/driver.rs` 起点）

- `"../../runes/embed/embed.fav"` → `favnir/runes/embed/embed.fav`

### `contains` 判定の設計方針

- `contains("fn openai(")` — `public fn openai(` にマッチ
- `contains("fn cohere(")` — `public fn cohere(` にマッチ
- `contains("fn embed_batch(")` — `public fn embed_batch(` にマッチ。`fn embed_cached(` とは区別可能（偽陽性なし）
- `contains("fn local(")` — `public fn local(` にマッチ
- `contains("fn embed_cached(")` — `public fn embed_cached(` にマッチ
- `contains("EmbedLocalProvider")` — コメント `// EmbedLocalProvider — Ollama を使ったローカル推論` でマッチ。**注意**: コメントを変更・削除した場合は当該テストアサーションも連動して更新すること

### Favnir 構文ルール（v66.x 共通）

- `bind x <- expr` は Result/Option を返す式にのみ使用する（スタブでは不要）
- tasks.md での `bind.*=` grep パターンは「`<-` を含む bind」を誤検知しない（`<-` に `=` は含まれないため偽陽性なし）
- `let` は使わない
- `Math.sqrt` を使う（`Float.sqrt` は VM に存在しない）
- `Float.from_int` は VM に存在しない

### rune.toml フォーマット

- `entry = "embed.fav"`（`main` ではなく `entry`）
- `effects = []` を明示
- `[dependencies]` セクションを含める
