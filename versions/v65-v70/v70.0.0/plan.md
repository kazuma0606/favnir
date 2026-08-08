# v70.0.0 実装計画

## 前提

- ベース: 3555 tests（v69.9.0 完了後）
- 目標: 3559 tests（+4）
- ★クリーンアップあり（cargo clean 後の hello.fav 復元が必須）

---

## 実装ステップ

### Step 1: `fav/Cargo.toml` — バージョン更新

```toml
version = "70.0.0"
```

`"69.0.0"` → `"70.0.0"` に変更。

### Step 2: `MILESTONE.md` — 先頭にエントリ追加

現在の先頭行 `## v69.0.0（2026-08-07）— Distributed Favnir` の直前に挿入:

```markdown
## v70.0.0（2026-08-08）— Intelligent ETL 1.0

> 「型チェックが、LLM の出力を安全にする。
>  ベクトルの次元は型で保証され、スキーマ違反は推論の前に止まる。
>  自動微分は数値安定性を型レベルで保ち、
>  デバッガがパイプラインを時間遡行し、AI が次の最適化を提案する。
>  型安全な並列処理が、AI パイプラインをクラスタ規模で動かす。
>
>  Favnir は「AI データエンジニアリングのための型安全言語」になった。
>
>  これが Favnir v70.0 — Intelligent ETL 1.0 の姿である。」

**Intelligent ETL 1.0** の宣言バージョン。v65.1〜v69.9 で実装した
Math Rune 群（linalg/stats/autodiff/optim/numeric/timeseries/ml）・
AI Rune 群（embed/llm/VectorDB/serve/featurestore）・
Playground 拡張・E2E AI ETL デモ・パフォーマンスベースラインの統合を宣言した。

**v65.1〜v69.9 達成内容:**
- v65.1〜v65.8（Math Rune 群）: 型付き行列・ベクトル演算・統計・自動微分・最適化
- v66.1〜v66.8（AI-Native Stage Layer）: LLM 型安全抽出・埋め込み・VectorDB・モデルサービング
- v67.1〜v67.9（Developer Intelligence）: デバッガ・タイムトラベル・DAG 可視化・AI アドバイザー
- v68.1〜v68.9（Distributed Favnir）: マルチノード par・チェックポイント・K8s・コスト見積もり
- v69.1〜v69.9（Intelligent ETL 統合）: E2E デモ・Playground・ドキュメント整備・コードフリーズ

---

```

### Step 3: `README.md` — v70.0.0 宣言追記

README.md の冒頭付近（`---` 区切りの直後）に v70.0.0 宣言セクションを追加:

```markdown
## v70.0 — Intelligent ETL 1.0 宣言（2026-08-08）

Favnir v70.0 で「Intelligent ETL 1.0」を宣言しました。
型安全な AI パイプライン言語として、Math Rune・AI Rune・分散実行・開発者ツールが揃いました。
```

### Step 4: `CHANGELOG.md` — 先頭にエントリ追加

現在の先頭エントリ `## [v69.0.0]` の直前に挿入:

```markdown
## [v70.0.0] — 2026-08-08 — Intelligent ETL 1.0 宣言 ★クリーンアップ

### Added
- `MILESTONE.md` に v70.0.0「Intelligent ETL 1.0」宣言文エントリを追加
- `v70000_tests`: 4 件追加（3555 → 3559 tests）
  - `cargo_toml_version_is_70_0_0`
  - `changelog_has_v70_0_0`
  - `milestone_has_intelligent_etl`
  - `readme_mentions_intelligent_etl`
- Intelligent ETL 機能群（v65.1〜v69.9）の成果を統合

### Changed
- `fav/Cargo.toml` version `"69.0.0"` → `"70.0.0"`
- `README.md` に Intelligent ETL 1.0 宣言を追記

### Note
- ★クリーンアップ（`cargo clean`）完了
- `cargo clean` 後は `fav/tmp/hello.fav` を復元すること（bootstrap テスト要件）

---

```

### Step 5: `driver.rs` — `v70000_tests` モジュール追加

`v69900_tests` の直前に挿入（降順ルール: v70000 → v69900 → v69800 → ...）:

```rust
// -- v70000_tests (v70.0.0) -- Intelligent ETL 1.0 宣言 ★クリーンアップ --
#[cfg(test)]
mod v70000_tests {
    #[test]
    fn cargo_toml_version_is_70_0_0() {
        let src = include_str!("../Cargo.toml");
        assert!(
            src.contains("version = \"70.0.0\""),
            "Cargo.toml should declare version 70.0.0"
        );
    }

    #[test]
    fn changelog_has_v70_0_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(
            src.contains("v70.0.0"),
            "CHANGELOG.md should contain v70.0.0 entry"
        );
    }

    #[test]
    fn milestone_has_intelligent_etl() {
        let src = include_str!("../../MILESTONE.md");
        assert!(
            src.contains("Intelligent ETL"),
            "MILESTONE.md should contain Intelligent ETL declaration"
        );
    }

    #[test]
    fn readme_mentions_intelligent_etl() {
        let src = include_str!("../../README.md");
        assert!(
            src.contains("Intelligent ETL") || src.contains("v70.0"),
            "README.md should mention Intelligent ETL or v70.0"
        );
    }
}
```

### Step 6: ビルド・テスト確認

```bash
cargo build 2>&1 | grep "^error"  # エラーゼロ確認
cargo test --bin fav -- --test-threads=8  # 3559 tests passed 確認
```

### Step 7: ★クリーンアップ

```bash
cargo clean
```

**直後に `fav/tmp/hello.fav` を復元**（bootstrap_c2_artifact_roundtrip テストが依存）:

```
fn add(a: Int, b: Int) -> Int { a + b }
fn main() -> Bool { add(1, 2) == 3 }
```

```bash
cargo test --bin fav -- --test-threads=8  # クリーンアップ後も 3559 確認
```

### Step 8: ドキュメント・ステータス更新

- `versions/roadmap/roadmap-v69.1-v70.0.md` の v70.0.0 行を「完了 ✓」に更新
- `versions/current.md` の最新安定版を v70.0.0 に更新

---

## 依存関係

Step 1〜4 は並行実施可能（ファイルが異なるため）。
Step 5（driver.rs）は Step 1〜4 が完了してから（テストが参照するファイルが整ってから）。
Step 6 は Step 5 の後。
Step 7（cargo clean）は Step 6 の成功後。
Step 8 は Step 7 の後。
