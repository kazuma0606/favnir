# v67.0.0 Spec — AI-Native Stage Layer 宣言 ★クリーンアップ

Version: 67.0.0
Status: 未着手
Base tests: 3493
Target tests: 3497

---

## 概要

v66.1〜v66.9 で実装した AI-Native Stage Layer（9 AI Rune 群 + AI Lint Rules）を正式宣言するマイルストーンバージョン。
`cargo clean` による成果物クリーンアップを行い、フルビルド・全テスト通過を確認する。

**宣言文**:

> 「LLM の出力にスキーマが付き、ベクトルの次元が型で保証される。
>  埋め込みモデルの選択が型エラーを生まず、
>  リアルタイム推論パイプラインがバックプレッシャー制御下で動く。
>
>  これが Favnir v67.0 — AI-Native Stage Layer の姿である。」

ロードマップ `roadmap-v66.1-v67.0.md` の v67.0.0 セクションに準拠。

---

## 前提確認（T0 で実施）

- `cargo test -j 8 -- --test-threads=8` でベース 3493 tests passed, 0 failed を確認
- `fav/Cargo.toml` の version が `"66.0.0"` であることを確認（本バージョンで `"67.0.0"` に更新する）
- `driver.rs` に `v66900_tests` が存在することを確認（`v67000_tests` の挿入位置）
- `driver.rs` に `v67000_tests` が存在しないことを確認（新規追加）
- `cargo test --bin fav v66900_tests` で 2 件 PASS することを確認（前バージョンが正常）
  - 前バージョンのテスト関数名: `ai_stage_layer_all_stable`, `ai_rune_docs_complete`
- `versions/current.md` の「進行中バージョン」が `v66.9.0` であることを確認

---

## 実装スコープ

### 1. `fav/Cargo.toml` — バージョン更新

```toml
version = "67.0.0"
```

（`"66.0.0"` → `"67.0.0"` に変更）

### 2. `MILESTONE.md` — v67.0.0 エントリを先頭に追加

既存の `## v66.0.0` エントリの直前に挿入:

```markdown
## v67.0.0（2026-08-06）— AI-Native Stage Layer

> 「LLM の出力にスキーマが付き、ベクトルの次元が型で保証される。
>  埋め込みモデルの選択が型エラーを生まず、
>  リアルタイム推論パイプラインがバックプレッシャー制御下で動く。
>
>  これが Favnir v67.0 — AI-Native Stage Layer の姿である。」

**AI-Native Stage Layer** の宣言バージョン。v66.1〜v66.9 で実装した
9 AI Rune 群と AI Pipeline Lint Rules W055〜W059 の統合を宣言した。

**v66.1〜v66.9 達成内容:**
- v66.1（Rune.vec）: ベクトル演算（normalize / dot / cosine_similarity / euclidean_distance）
- v66.2（LLM Extraction）: 型安全 JSON 抽出ステージ
- v66.3（Rune.embed）: 統一埋め込みインターフェース（OpenAI / Cohere / ローカル）
- v66.4（Vector DB Runes）: Pinecone / pgvector / Weaviate / Qdrant
- v66.5（Rune.inference）: ストリーミング ML 推論（backpressure / SLA / stateful）
- v66.6（Rune.serve）: モデルサービングエンドポイント（rate limit / OpenAPI）
- v66.7（Rune.featurestore）: 型安全フィーチャーストア
- v66.8（AI Lint Rules）: W055〜W059 AI パイプラインアンチパターン検出スタブ
- v66.9（安定化）: ai-runes-overview.mdx / 全 AI Rune 存在確認

**テスト数**: 3497

---
```

### 3. `README.md` — v67.0.0 宣言を追加

既存の v66.0.0 / Math & Science Foundation の記述の直前に v67.0.0 の言及を追加。
`"AI-Native"` または `"v67.0"` を含む必要がある（`readme_mentions_ai_native` テストで検証）。

### 4. `CHANGELOG.md` — v67.0.0 エントリを先頭に追加

v66.1〜v66.9 で CHANGELOG 更新を保留していたため、v67.0.0 エントリに一括追記する。
既存の `## [v66.0.0]` エントリの直前に挿入:

```markdown
## [v67.0.0] — 2026-08-06 — AI-Native Stage Layer 宣言 ★クリーンアップ

### Added
- `MILESTONE.md` に v67.0.0「AI-Native Stage Layer」宣言文エントリを追加
- `v67000_tests`: 4 件追加（3493 → 3497 tests）
  - `cargo_toml_version_is_67_0_0`
  - `changelog_has_v67_0_0`
  - `milestone_has_ai_native_stage`
  - `readme_mentions_ai_native`
- `site/content/docs/runes/ai-runes-overview.mdx` 新規作成（v66.9.0）
- AI-Native Stage Layer Rune 群（v66.1〜v66.9）の成果を統合:
  - `Rune.vec`（v66.1）: ベクトル演算（normalize / dot / cosine_similarity / euclidean_distance）
  - LLM Extraction Stage（v66.2）: 型安全 JSON 抽出
  - `Rune.embed`（v66.3）: 統一埋め込みインターフェース
  - Vector DB Runes（v66.4）: Pinecone / pgvector / Weaviate / Qdrant
  - `Rune.inference`（v66.5）: ストリーミング ML 推論
  - `Rune.serve`（v66.6）: モデルサービングエンドポイント
  - `Rune.featurestore`（v66.7）: 型安全フィーチャーストア
  - AI Lint Rules W055〜W059（v66.8）: AI パイプライン特有アンチパターン検出スタブ
  - 安定化・`ai-runes-overview.mdx`（v66.9）

### Changed
- `fav/Cargo.toml` version `"66.0.0"` → `"67.0.0"`
- `README.md` に AI-Native Stage Layer 宣言を追記

### Note
- ★クリーンアップ（`cargo clean`）完了
- `cargo clean` 後は `fav/tmp/hello.fav` を復元すること（bootstrap テスト要件）

---
```

### 5. `driver.rs` — `v67000_tests` 追加

挿入位置: `// -- v66900_tests (v66.9.0)` コメントの直前

```rust
// -- v67000_tests (v67.0.0) -- AI-Native Stage Layer 宣言 --
#[cfg(test)]
mod v67000_tests {
    #[test]
    fn cargo_toml_version_is_67_0_0() {
        let toml = include_str!("../Cargo.toml");
        assert!(
            toml.contains("version = \"67.0.0\""),
            "Cargo.toml should have version 67.0.0: {}",
            &toml[..200.min(toml.len())]
        );
    }

    #[test]
    fn changelog_has_v67_0_0() {
        let cl = include_str!("../../CHANGELOG.md");
        assert!(cl.contains("v67.0.0"), "CHANGELOG.md should mention v67.0.0");
    }

    #[test]
    fn milestone_has_ai_native_stage() {
        let ms = include_str!("../../MILESTONE.md");
        assert!(
            ms.contains("AI-Native Stage Layer"),
            "MILESTONE.md should contain 'AI-Native Stage Layer'"
        );
    }

    #[test]
    fn readme_mentions_ai_native() {
        let readme = include_str!("../../README.md");
        assert!(
            readme.contains("AI-Native") || readme.contains("v67.0"),
            "README.md should mention AI-Native Stage Layer or v67.0"
        );
    }
}
```

### 6. `cargo clean` + `fav/tmp/hello.fav` 復元

★クリーンアップとして `cargo clean` を実行。
**実行後、`fav/tmp/hello.fav` を必ず復元すること**（削除されると `bootstrap_c2_artifact_roundtrip` テストが FAIL する）。

`fav/tmp/hello.fav` の正しい内容:
```favnir
fn add(a: Int, b: Int) -> Int { a + b }
fn main() -> Bool { add(1, 2) == 3 }
```

---

## 完了条件

- `fav/Cargo.toml` に `version = "67.0.0"` が含まれる
- `CHANGELOG.md` に `"v67.0.0"` が含まれる
- `MILESTONE.md` に `"AI-Native Stage Layer"` が含まれる
- `README.md` に `"AI-Native"` または `"v67.0"` が含まれる
- `cargo build` でエラーなし
- `cargo test --bin fav v67000_tests` で 4 件 PASS
  - `cargo_toml_version_is_67_0_0` PASS
  - `changelog_has_v67_0_0` PASS
  - `milestone_has_ai_native_stage` PASS
  - `readme_mentions_ai_native` PASS
- `cargo clean` 実行済み
- `fav/tmp/hello.fav` 復元済み
- `cargo test -j 8 -- --test-threads=8` で 3497 tests passed, 0 failed

---

## 非スコープ

- W055〜W059 の実際の検出ロジック — 将来フェーズ（スタブのまま）
- v68.x 以降のスプリント計画 — 別途策定

---

## 技術ノート

### `include_str!` パス（`fav/src/driver.rs` 起点）

- `"../Cargo.toml"` → `fav/Cargo.toml`（1つ上の `fav/` ディレクトリ）
- `"../../CHANGELOG.md"` → `favnir/CHANGELOG.md`（リポジトリルート）
- `"../../MILESTONE.md"` → `favnir/MILESTONE.md`（リポジトリルート）
- `"../../README.md"` → `favnir/README.md`（リポジトリルート）

### `cargo clean` 後の `hello.fav` 復元理由

`cargo clean` 自体は `target/` ディレクトリのみを削除し、`fav/tmp/` を直接消去することはない。
ただし過去実績として `cargo clean` 後に `fav/tmp/hello.fav` が消えるケースが報告されているため、念のため必ず復元する。
`bootstrap_c2_artifact_roundtrip` テストが `fav/tmp/hello.fav` を参照するため、存在しないと FAIL する。

### テスト数の変化（+4）

マイルストーン宣言バージョン（x.0.0 リリース）では通常 +4 テスト。
サブバージョン（x.y.0）の +2 とは異なる。

### `readme_mentions_ai_native` の OR 条件

`contains("AI-Native") || contains("v67.0")` を使用。
README には将来 v67.0 以外の記述でもマッチできる柔軟性を持たせる。
