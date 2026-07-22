# Spec: v53.0.0 — Data Quality & Observability 2.0 宣言

Status: 計画中
Date: 2026-07-22

---

## 宣言文

> 「スキーマはランタイムで検証され、データの来歴はグラフで見え、
>  SLA 違反は即座に検知され、アクセスはすべて記録される。
>  Favnir のパイプラインは信頼できるデータを届ける。
>
>  これが Favnir v53.0 — Data Quality & Observability 2.0 の姿である。」

---

## 概要

v52.1〜v52.9 で実装した Data Quality & Observability 2.0 機能群（assert_schema・
リネージ強化・SLA 監視・audit-log・OTel 強化・ドキュメント整備）を統合し、
マイルストーン宣言する。`★クリーンアップ`（`cargo clean`）を実施する。

---

## 実装スコープ

### 1. `MILESTONE.md` 更新

先頭に v53.0.0 エントリを追加。`"Data Quality & Observability 2.0"` を含む。

### 2. `README.md` 更新

v53.0.0 / "Data Quality" に関する言及を追加（`milestone_has_data_quality` の兄弟テストに対応）。

### 3. `CHANGELOG.md` 更新

v53.0.0 エントリを追加。`v53.0.0` キーワードを含む。

### 4. `fav/Cargo.toml` バージョン更新

`"52.9.0"` → `"53.0.0"`

### 5. `★クリーンアップ`（`cargo clean`）

**重要**: `cargo clean` 実行後は `fav/tmp/hello.fav` が消えるため必ず復元する。
`hello.fav` の正しい内容:

```
fn add(a: Int, b: Int) -> Int { a + b }
fn main() -> Bool { add(1, 2) == 3 }
```

復元後に `cargo test` を再実行して `bootstrap_c2_artifact_roundtrip` が pass することを確認する。

---

## テスト仕様

`v53000_tests` モジュールを `driver.rs` に追加（`v52900_tests` の直前）:

```rust
#[cfg(test)]
mod v53000_tests {
    #[test]
    fn cargo_toml_version_is_53_0_0() {
        let src = include_str!("../Cargo.toml");
        assert!(src.contains("version = \"53.0.0\""), "Cargo.toml must be version 53.0.0");
    }

    #[test]
    fn changelog_has_v53_0_0() {
        let content = include_str!("../../CHANGELOG.md");
        assert!(content.contains("v53.0.0"), "CHANGELOG.md must contain v53.0.0 entry");
    }

    #[test]
    fn milestone_has_data_quality() {
        let content = include_str!("../../MILESTONE.md");
        assert!(
            content.contains("Data Quality & Observability 2.0"),
            "MILESTONE.md must contain 'Data Quality & Observability 2.0'"
        );
    }

    #[test]
    fn readme_mentions_data_quality() {
        let content = include_str!("../../README.md");
        assert!(
            content.contains("Data Quality"),
            "README.md must mention 'Data Quality'"
        );
    }
}
```

`include_str!` パス（`fav/src/driver.rs` 起点）:
- `"../Cargo.toml"` → `fav/Cargo.toml` ✓
- `"../../CHANGELOG.md"` → `favnir/CHANGELOG.md` ✓
- `"../../MILESTONE.md"` → `favnir/MILESTONE.md` ✓
- `"../../README.md"` → `favnir/README.md` ✓

---

## 完了条件

- v52.1〜v52.9 の全機能が動作する
- `cargo test` 全通過（failures=0 かつテスト数 ≥ **3157**、実績推定 3160）
- `v53000_tests` 4 件 pass:
  - `cargo_toml_version_is_53_0_0`
  - `changelog_has_v53_0_0`
  - `milestone_has_data_quality`
  - `readme_mentions_data_quality`
- `MILESTONE.md` に `"Data Quality & Observability 2.0"` が含まれる
- `cargo clean` 完了 + `fav/tmp/hello.fav` 復元 + 再テスト全通過

---

## 影響範囲

| ファイル | 変更種別 |
|---|---|
| `MILESTONE.md` | v53.0.0 エントリ追加 |
| `README.md` | "Data Quality" 言及追加 |
| `CHANGELOG.md` | v53.0.0 エントリ追加 |
| `fav/src/driver.rs` | `v53000_tests` 追加 |
| `fav/Cargo.toml` | version 更新 |
| `fav/tmp/hello.fav` | `cargo clean` 後に復元（内容不変） |
| `versions/current.md` | v53.0.0 に更新 |
| `versions/roadmap/roadmap-v52.1-v53.0.md` | v53.0.0 実績欄を更新 |

Rust ソースコードの実装変更なし（テスト追加のみ）。
