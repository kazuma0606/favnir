# v66.7.0 Spec — Feature Store Rune（`Rune.featurestore`）

Version: 66.7.0
Status: 未着手
Base tests: 3487
Target tests: 3489

---

## 概要

型安全なフィーチャーエンジニアリングを提供する Rune。
フィーチャーの定義・バージョン管理・取得・共有を型で保証する。
Point-in-time lookup によりトレーニングデータのリークを防ぐ。

ロードマップ `roadmap-v66.1-v67.0.md` の v66.7.0 セクションに準拠。

> **スコープ縮小の明示**: ロードマップでは `schema UserFeatures { ... }` 等の
> スキーマ型定義・`compute_fn` の実際の計算実行・Point-in-time lookup の実DB参照を示しているが、
> これらは将来フェーズ。本バージョンでは `String` をプレースホルダーとして
> 関数シグネチャを確立することに専念する。

---

## 前提確認（T0 で実施）

- `cargo test -j 8 -- --test-threads=8` でベース 3487 tests passed, 0 failed を確認
- `fav/Cargo.toml` の version が `"66.0.0"` であることを確認（v66.0.0 宣言時に設定済み。v66.x sub-version では更新しない。v67.0.0 宣言時に `"67.0.0"` に更新する）
- `runes/featurestore/` ディレクトリが存在しないことを確認（新規作成対象）
- `driver.rs` に `v66600_tests` が存在することを確認（`v66700_tests` の挿入位置）
- `driver.rs` に `v66700_tests` が存在しないことを確認（新規追加）
- `cargo test --bin fav v66600_tests` で 2 件 PASS することを確認（前バージョンが正常）
  - 前バージョンのテスト関数名: `model_serve_endpoint_type`, `model_serve_schema_validation`
- `versions/current.md` の「進行中バージョン」が `v66.6.0` であることを確認（確認失敗時は前バージョンの tasks.md T4 が完了していることを確認してから current.md を手動修正すること）

---

## 実装スコープ

### 1. `runes/featurestore/rune.toml` — 新規作成

```toml
[rune]
name        = "featurestore"
version     = "0.1.0"
description = "Feature Store Rune for Favnir — type-safe feature engineering: define, get, get_batch, get_version, get_at with point-in-time lookup"
entry       = "featurestore.fav"
effects     = []

[dependencies]
```

### 2. `runes/featurestore/featurestore.fav` — 新規作成スタブ

```favnir
// featurestore Rune — 型安全フィーチャーストア
// define_feature, get, get_batch, get_version, get_at
//
// NOTE: スキーマ型定義・compute_fn 実行は将来フェーズ。
//       FeatureStoreInterface — フィーチャーストア統一インターフェース（将来フェーズ）
//       include_str! テストのみ（型チェックエラーは無視する）。

// フィーチャーを定義・登録する
public fn define_feature(name: String, version: String, schema: String, compute_fn: String) -> String {
    ""
}

// フィーチャーをオンライン推論向けに取得する（低レイテンシ）
public fn get(feature_name: String, entity_key: String) -> String {
    ""
}

// フィーチャーをバッチ取得する（訓練データ生成向け）
public fn get_batch(feature_name: String, keys: List<String>) -> List<String> {
    []
}

// バージョン指定でフィーチャーを取得する（再現性保証）
public fn get_version(feature_name: String, version: String) -> String {
    ""
}

// Point-in-time フィーチャー取得（訓練データリークを防ぐ）
public fn get_at(feature_name: String, key: String, timestamp: String) -> String {
    ""
}
```

### 3. `driver.rs` — `v66700_tests` 追加

挿入位置: `// -- v66600_tests (v66.6.0)` コメントの直前

```rust
// -- v66700_tests (v66.7.0) -- Feature Store Rune --
#[cfg(test)]
mod v66700_tests {
    #[test]
    fn feature_store_define_feature() {
        let featurestore = include_str!("../../runes/featurestore/featurestore.fav");
        assert!(
            featurestore.contains("fn define_feature("),
            "featurestore.fav should define define_feature"
        );
        assert!(
            featurestore.contains("fn get("),
            "featurestore.fav should define get"
        );
        assert!(
            featurestore.contains("fn get_batch("),
            "featurestore.fav should define get_batch"
        );
        assert!(
            featurestore.contains("FeatureStoreInterface"),
            "featurestore.fav should reference FeatureStoreInterface"
        );
    }

    #[test]
    fn feature_store_versioned_retrieval() {
        let featurestore = include_str!("../../runes/featurestore/featurestore.fav");
        assert!(
            featurestore.contains("fn get_version("),
            "featurestore.fav should define get_version"
        );
        assert!(
            featurestore.contains("fn get_at("),
            "featurestore.fav should define get_at"
        );
    }
}
```

挿入後、`cargo build` でエラーなしを確認。

---

## 完了条件

- `runes/featurestore/rune.toml` が存在する
- `runes/featurestore/featurestore.fav` が存在し以下を含む:
  - `fn define_feature(` — フィーチャー定義・登録
  - `fn get(` — オンライン推論向け取得
  - `fn get_batch(` — バッチ取得
  - `fn get_version(` — バージョン指定取得
  - `fn get_at(` — Point-in-time 取得
  - ヘッダーコメントに `FeatureStoreInterface` を含む（**この文字列はコメント行に固定配置。削除・変更した場合は `feature_store_define_feature` テストのアサーションも連動更新すること**）
- `cargo test --bin fav v66700_tests` で 2 件 PASS
  - `feature_store_define_feature` PASS
  - `feature_store_versioned_retrieval` PASS
- `cargo test -j 8 -- --test-threads=8` で 3489 tests passed, 0 failed
- CHANGELOG.md 更新・site/ MDX 作成は意図的に省略（非スコープセクション参照）

---

## 非スコープ

- `schema` キーワードによるスキーマ型定義 — 将来フェーズ
- `compute_fn` の実際の計算実行 — 将来フェーズ（文字列プレースホルダー）
- Point-in-time lookup の実 DB 参照 — 将来フェーズ（スタブのみ）
- オンライン推論向け低レイテンシ最適化 — 将来フェーズ
- `rune.toml` の `effects` 更新 — 本番 API 呼び出し実装時に追加（将来フェーズ）
- `fav check` での型チェック通過 — 今バージョンは `include_str!` テストのみ
- CHANGELOG.md 更新 — v67.0.0 宣言時に一括追記
- site/ MDX ドキュメント作成 — v66.9.0 安定化時に一括作成するため今バージョンは省略

---

## 技術ノート

### `include_str!` パス（`fav/src/driver.rs` 起点）

- `"../../runes/featurestore/featurestore.fav"` → 新規ファイル

### `contains` 判定の設計方針

- `featurestore.contains("fn define_feature(")` — `public fn define_feature(` にマッチ
- `featurestore.contains("fn get(")` — `public fn get(` にマッチ
- `featurestore.contains("fn get_batch(")` — `public fn get_batch(` にマッチ
- `featurestore.contains("FeatureStoreInterface")` — ヘッダーコメントでマッチ。**注意**: コメントを変更・削除した場合は `feature_store_define_feature` テストのアサーションも連動して更新すること
- `featurestore.contains("fn get_version(")` — `public fn get_version(` にマッチ
- `featurestore.contains("fn get_at(")` — `public fn get_at(` にマッチ

### Favnir 構文ルール（v66.x 共通）

- `bind x <- expr` は Result/Option を返す式にのみ使用する（スタブでは不要）
- `let` は使わない
- `Math.sqrt` を使う（`Float.sqrt` は VM に存在しない）
- `Float.from_int` は VM に存在しない

### 新規 Rune の rune.toml フォーマット

- `entry = "ファイル名.fav"`（`main` ではなく `entry`）
- `effects = []` を明示
- `[dependencies]` セクションを含める（依存なしの場合も空セクションとして明示。`runes/embed/rune.toml` と同一フォーマット）
