# v66.7.0 実装計画 — Feature Store Rune（`Rune.featurestore`）

Version: 66.7.0
Status: 未着手
Base tests: 3487
Target tests: 3489

---

## 実装ステップ

> **前提**: spec.md の T0 前提確認を完了してから開始する。

### Step 1: 新規 Rune ファイル作成

1. `runes/featurestore/rune.toml`（entry / effects = [] / [dependencies] 形式）
2. `runes/featurestore/featurestore.fav`（define_feature / get / get_batch / get_version / get_at、FeatureStoreInterface コメント付き）

### Step 2: `driver.rs` テスト追加

- `// -- v66600_tests (v66.6.0)` コメントの直前に `v66700_tests` を挿入
- 2 テスト関数:
  - `feature_store_define_feature`（featurestore.fav の define_feature / get / get_batch / FeatureStoreInterface 検証）
  - `feature_store_versioned_retrieval`（get_version / get_at 検証）

### Step 3: ビルド・テスト確認

```bash
# 以下は順番に実行すること（前コマンドが PASS してから次へ進む）
cargo build
cargo test --bin fav v66700_tests
cargo test -j 8 -- --test-threads=8
```

---

## 関数一覧

| Rune | 関数 | 戻り値 | 備考 |
|---|---|---|---|
| featurestore | `define_feature(name, version, schema, compute_fn)` | `""` | フィーチャー定義スタブ |
| featurestore | `get(feature_name, entity_key)` | `""` | オンライン取得スタブ |
| featurestore | `get_batch(feature_name, keys)` | `[]` | バッチ取得スタブ |
| featurestore | `get_version(feature_name, version)` | `""` | バージョン指定取得スタブ |
| featurestore | `get_at(feature_name, key, timestamp)` | `""` | Point-in-time 取得スタブ |

---

## `driver.rs` 挿入コード

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

---

## リスク・注意点

- `FeatureStoreInterface` はコメント行にのみ存在するため、featurestore.fav のヘッダーコメントを変更・削除してはならない。やむを得ず変更する場合は `driver.rs` の `feature_store_define_feature` 内の `assert!(featurestore.contains("FeatureStoreInterface"), ...)` も同時に更新すること
- `featurestore.contains("fn get(")` の誤マッチリスクについては spec.md 技術ノート参照（`fn get_batch(` 等は `fn get(` を含まないため誤マッチは発生しない）
- 新規 Rune は `public fn` 形式でスタブを統一（pinecone.fav の `fn Namespace.method` 形式とは異なる）

## 非スコープ

- `schema` キーワードによるスキーマ型定義 — 将来フェーズ
- `compute_fn` の実際の計算実行 — 将来フェーズ（文字列プレースホルダー）
- Point-in-time lookup の実 DB 参照 — 将来フェーズ（スタブのみ）
- オンライン推論向け低レイテンシ最適化 — 将来フェーズ
- `rune.toml` の `effects` 更新 — 本番 API 呼び出し実装時（将来フェーズ）
