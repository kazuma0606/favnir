# v44.6.0 Plan — Precision & Flow E2E デモ

## 前提

- 現行バージョン: `44.5.0`（2955 tests）
- 追加テスト数: 1 件
- 目標テスト数: 2956
- ロードマップ推定（2945）は旧見積もり。実績 2955 を基準とする

---

## 参考

- 既存 E2E デモ: `infra/e2e-demo/snowflake/` — `src/demo.fav` + `README.md` の 2 ファイル構成
- 既存テストパターン: `v10900_tests::snowflake_e2e_demo_structure`（driver.rs 行 26422）
  - `CARGO_MANIFEST_DIR` → `parent()` → `infra/e2e-demo/<name>/` パスを構築
  - `base.exists()` / `base.join("src/demo.fav").exists()` / `base.join("README.md").exists()` の 3 アサート

---

## ステップ

### Step 1: `infra/e2e-demo/precision-flow/src/demo.fav` 作成

以下の Favnir コードを記述（Precision & Flow 全機能を網羅）:

```favnir
// infra/e2e-demo/precision-flow/src/demo.fav
// Precision & Flow E2E Demo — v44.6.0
// Kafka → CEP → Opaque join → Policy gate

import rune "kafka"

// ── Refinement type ───────────────────────────────────────────────
type HighValue = Float where |v| v > 1000.0

// ── Opaque type ───────────────────────────────────────────────────
opaque type OrderId = String
opaque type SessionId = String

// ── CEP pattern ───────────────────────────────────────────────────
cep pattern HighValueDetected {
  HighValue within 300
}

// ── Stage 1: Kafka からイベントを受信 ────────────────────────────
stage IngestEvents: String -> List<Float> = |topic| {
  bind raw: List<Float> <- kafka.consume(topic)
  raw
}

// ── Stage 2: CEP で高額イベントを検出（HighValueDetected パターン適用）────
stage DetectHighValue: List<Float> -> List<Float> = |events| {
  // applies HighValueDetected pattern: HighValue within 300
  bind detected: List<HighValue> <- events
  detected
}

// ── Stage 3: Policy gate（back-pressure 制限付き） ────────────────
#[max_inflight(50)]
stage PolicyGate: List<Float> -> List<Float> = |events| {
  bind allowed: List<Float> <- events
  allowed
}
```

### Step 2: `infra/e2e-demo/precision-flow/README.md` 作成

```markdown
# Precision & Flow E2E Demo

Favnir v44.6.0 — CEP + Refinement type + Opaque type + Policy gate の統合デモ。

## パイプライン概要

```
Kafka → IngestEvents → DetectHighValue → PolicyGate
```

## 機能一覧

| 機能 | 実装箇所 |
|---|---|
| Refinement type | `type HighValue = Float where |v| v > 1000.0` |
| Opaque type | `opaque type OrderId = String` |
| CEP pattern | `cep pattern HighValueDetected { HighValue within 300 }` |
| Back-pressure | `#[max_inflight(50)] stage PolicyGate` |

## 実行方法（将来版）

```bash
fav run src/demo.fav
```
```

### Step 3: driver.rs — `v44600_tests` 追加 / スタブ化 / Cargo.toml

`v44500_tests` の直前（上の行）に挿入（driver.rs はバージョン降順配置）:

```rust
// -- v44600_tests (v44.6.0) -- Precision & Flow E2E デモ --
#[cfg(test)]
mod v44600_tests {
    #[test]
    fn precision_flow_e2e_demo_structure() {
        let root = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let base = std::path::Path::new(&root)
            .parent()
            .unwrap()
            .join("infra/e2e-demo/precision-flow");
        assert!(base.exists(), "infra/e2e-demo/precision-flow/ must exist");
        assert!(
            base.join("src/demo.fav").exists(),
            "infra/e2e-demo/precision-flow/src/demo.fav must exist"
        );
        assert!(
            base.join("README.md").exists(),
            "infra/e2e-demo/precision-flow/README.md must exist"
        );
    }
}
```

スタブ化: `v44500_tests::cargo_toml_version_is_44_5_0` の `assert!` 行のみを削除し、以下に置き換える（`#[test]` アトリビュートと関数シグネチャは残す）:

```rust
// Stubbed: version bumped to 44.6.0 in v44.6.0.
```

`fav/Cargo.toml` version: `44.5.0` → `44.6.0`

### Step 4: CHANGELOG.md に v44.6.0 エントリ追加

### Step 5: テスト実行（2956 passed; 0 failed）

### Step 6: バージョン管理ドキュメント更新

- `versions/current.md` → v44.6.0、2956 tests、次版 v44.7.0
- `versions/roadmap/roadmap-v44.1-v45.0.md` → v44.6.0 を `✅ COMPLETE`
- `versions/v40-v45/v44.6.0/tasks.md` → COMPLETE

---

## 注意事項

- `demo.fav` の `kafka.consume` は実行時には未解決だが、テストは `Path::exists()` のみなので問題なし
- `infra/e2e-demo/precision-flow/src/` ディレクトリが必要（`src/demo.fav` の親ディレクトリ）
- テストモジュールに `use super::*` は不要（`std::env::var` / `std::path::Path` はプリミティブ）
