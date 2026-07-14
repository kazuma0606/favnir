# v44.7.0 Plan — ドキュメントサイト Precision & Flow 概要ページ

## 前提

- 現行バージョン: `44.6.0`（2956 tests）
- 追加テスト数: 2 件（`cargo_toml_version_is_44_7_0` + `precision_and_flow_doc_exists`）
- 目標テスト数: 2958
- ロードマップ推定（2946）は旧見積もり。実績 2956 を基準とする

---

## 参考

- 既存 MDX テストパターン: `v41900_tests` モジュール内の `type_precision_doc_exists`
  - `include_str!("../../site/content/docs/type-precision.mdx")` で読み込み
  - `.contains("Type Precision")` でアサート
- MDX ファイルの配置先: `site/content/docs/`（fav/ と同じ親ディレクトリ下）

---

## ステップ

### Step 1: `site/content/docs/precision-and-flow.mdx` 作成

```mdx
---
title: Precision & Flow
description: 型安全なリアルタイムパイプラインを最小限の注釈で記述するための Favnir 機能群
---

# Precision & Flow

Favnir v44.x スプリント「Precision & Flow」では、型推論・Refinement type・CEP・Opaque type・Back-pressure を統合し、**最小限の型注釈で安全なリアルタイムパイプラインを記述できる**状態を実現します。

## Refinement type

値の制約を型として表現します。

```favnir
type HighValue = Float where |v| v > 1000.0

stage Validate: List<Float> -> List<HighValue> = |events| {
  bind valid: List<HighValue> <- events
  valid
}
```

## CEP（Complex Event Processing）

イベントパターンを宣言的に記述します。

```favnir
cep pattern HighValueDetected {
  HighValue within 300
}
```

## Opaque type

内部表現を隠蔽し、型レベルで誤 join を防止します。

```favnir
opaque type OrderId = String
opaque type SessionId = String
// OrderId と SessionId は型システム上で区別される
```

## 型注釈 lineage

型注釈付き bind 束縛をリネージ追跡の起点として活用します。

```favnir
stage Process: List<Float> -> List<Float> = |events| {
  bind filtered: List<Float> <- events
  filtered
}
```

## Back-pressure（`#[max_inflight]`）

ステージの同時処理上限を宣言します。

```favnir
#[max_inflight(50)]
stage PolicyGate: List<Float> -> List<Float> = |events| {
  bind allowed: List<Float> <- events
  allowed
}
```

## E2E デモ

[Precision & Flow E2E デモ](../../../infra/e2e-demo/precision-flow/) では、これらすべての機能を統合したパイプラインを確認できます。

Kafka → CEP → Opaque join → Policy gate の完全パイプラインが `infra/e2e-demo/precision-flow/src/demo.fav` に実装されています。
```

### Step 2: driver.rs — `v44700_tests` 追加 / スタブ化 / Cargo.toml

`v44600_tests` の直前（上の行）に挿入（driver.rs はバージョン降順配置）:

```rust
// -- v44700_tests (v44.7.0) -- ドキュメントサイト Precision & Flow 概要ページ --
#[cfg(test)]
mod v44700_tests {
    #[test]
    fn cargo_toml_version_is_44_7_0() {
        let toml = include_str!("../Cargo.toml");
        assert!(toml.contains("version = \"44.7.0\""), "Cargo.toml version mismatch");
    }
    #[test]
    fn precision_and_flow_doc_exists() {
        let src = include_str!("../../site/content/docs/precision-and-flow.mdx");
        assert!(
            src.contains("Precision & Flow"),
            "site/content/docs/precision-and-flow.mdx must exist and mention Precision & Flow"
        );
    }
}
```

スタブ化: `v44600_tests::cargo_toml_version_is_44_6_0` の `assert!` 行のみを削除し、以下に置き換える（`#[test]` アトリビュートと関数シグネチャは残す）:

```rust
// Stubbed: version bumped to 44.7.0 in v44.7.0.
```

`fav/Cargo.toml` version: `44.6.0` → `44.7.0`

### Step 3: CHANGELOG.md に v44.7.0 エントリ追加

### Step 4: テスト実行（2958 passed; 0 failed）

### Step 5: バージョン管理ドキュメント更新

- `versions/current.md` → v44.7.0、2957 tests、次版 v44.8.0
- `versions/roadmap/roadmap-v44.1-v45.0.md` → v44.7.0 を `✅ COMPLETE`
- `versions/v40-v45/v44.7.0/tasks.md` → COMPLETE

---

## 注意事項

- `include_str!` のパスは `fav/src/driver.rs` から見て `../../site/content/docs/precision-and-flow.mdx`
- MDX 内に `"Precision & Flow"` が必ず含まれること（アサート条件）
- `v44600_tests` には `cargo_toml_version_is_44_6_0` テストが存在する — スタブ化対象はこのテストのみ
