# v76.9.0 実装計画 — 安定化・コードフリーズ

Date: 2026-08-15

---

## Step 1: cargo test（事前確認）

既存の 3730 テストが全 pass であることを確認する（v769000_tests 追加前の状態）。

---

## Step 2: CHANGELOG.md 更新

`CHANGELOG.md` の先頭に v76.9.0 エントリを追加する（テストモジュール追加より先）。

---

## Step 3: driver.rs — v769000_tests モジュール追加

```rust
#[cfg(test)]
mod v769000_tests {
    use super::*;  // スプリント全型・関数を参照するため必須

    #[test]
    fn provenance_full_sprint_all_stable() { ... }

    #[test]
    fn provenance_e2e_pipeline_valid() { ... }
}
```

新型・関数の追加はなし。既存型の統合テストのみ。

---

## Step 4: Cargo.toml バージョン更新

`76.8.0` → `76.9.0`

また、driver.rs 内に存在する `76.8.0` バージョン文字列アサーションを `76.9.0` へ一括置換する（`replace_all: true`）。

---

## Step 5: versions/current.md 更新

進行中バージョンを v76.9.0 に、次に切る版を v77.0.0 に更新する。

---

## Step 6: 最終確認

`cargo test` が 3732 tests all pass であることを確認する。
