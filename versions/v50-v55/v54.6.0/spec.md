# Spec: v54.6.0 — README / CONTRIBUTING 最終整備

Status: COMPLETE
Date: 2026-07-23

---

## 概要

`README.md` に Production 3.0 への言及・v54.1〜v54.5 機能サマリーを追加する。
`CONTRIBUTING.md` に `fav doctor` 環境診断手順・`fav bench` パフォーマンス確認手順を追記し、
コントリビュート体験を最新状態に更新する。

---

## 実装スコープ

### 1. `README.md` — Production 3.0 言及 + v54.1〜v54.5 機能サマリー追加

ロードマップは「v51〜v55 機能サマリー追加」と記述しているが、README には v51〜v53 の各マイルストーン宣言が既掲載のため、
v54.x サブバージョン整備サマリーに特化する（v51〜v53 の重複記述は不要）。

v54.0 Integration Sprint マイルストーン宣言の直後に追記:

```markdown
v54.1〜v54.5（2026-07-22〜23）で Production 3.0 に向けた最終整備を完了しました。
全エラーコードへの `fav explain --error` 対応（v54.1）・`fav run --watch-diff/--watch-summary`（v54.2）・
パフォーマンスリグレッション CI 統合（v54.3）・`fav dq-report`（v54.4）・`fav doctor`（v54.5）が揃い、
開発者が自信を持って本番へ踏み出せるツールチェーンが完成しました。
```

配置: v54.0 宣言文の直後（v53.0 より前）。
時系列ルール: v54.0 の「マイルストーン宣言」→ v54.1〜v54.5 の「整備完了」の順（v54.0 より下には配置しない）。

### 2. `CONTRIBUTING.md` — `fav doctor` / `fav bench` 手順追加

テスト手順セクションの直前に「環境診断」セクションを追加:

```bash
./target/debug/fav doctor
# [OK]   fav version: X.Y.Z  （実行時の現在バージョンが表示される）
# [OK]   Rust toolchain: stable
# [OK]   fav.toml: valid
# [OK]   .fav-cache: intact
```

テスト手順セクションの直後に「ベンチマーク・パフォーマンス確認」セクションを追加:

```bash
# ベンチマーク実行（全 bench_ テスト）
cargo test bench_ -- --nocapture

# ベースラインとの比較（benchmarks/baseline.json が基準値）
./target/debug/fav bench --compare ../benchmarks/baseline.json --fail-on-regression
```

注意: `--all` フラグは v54.3.0 実装で実質 no-op（`file` 省略と等価）のため、CONTRIBUTING.md には含めない。

### 3. `driver.rs` — `v54600_tests` 追加

`v54500_tests` の直前に追加（2 テスト）:

```rust
mod v54600_tests {
    use super::*;

    #[test]
    fn readme_has_production3_mention() {
        let readme = include_str!("../../README.md");
        assert!(readme.contains("Production 3.0"), "README.md should mention 'Production 3.0'");
        assert!(readme.contains("v54.1"), "README.md should contain v54.1 summary added in v54.6.0");
    }

    #[test]
    fn contributing_has_doctor_step() {
        let contributing = include_str!("../../CONTRIBUTING.md");
        assert!(contributing.contains("fav doctor"), "CONTRIBUTING.md should include 'fav doctor' step");
    }
}
```

`use super::*` は他テストモジュールとの慣習統一のため必須。

---

## テスト仕様

| テスト名 | 検証内容 |
|---|---|
| `readme_has_production3_mention` | `README.md` が `"Production 3.0"` を含む。加えて `"v54.1"` を含む（v54.6.0 追加行の存在確認） |
| `contributing_has_doctor_step` | `CONTRIBUTING.md` が `"fav doctor"` を含む |

`include_str!` パス: `driver.rs` から `../../README.md` / `../../CONTRIBUTING.md`（`favnir/` 直下）。

---

## バージョン更新

- `fav/Cargo.toml`: `"54.5.0"` → `"54.6.0"`

---

## 完了条件

1. `cargo test -j 8 -- --test-threads=8` → 3197 passed, 0 failed（ベース 3195 + 2 件追加）
2. `v54600_tests` 2 件 pass:
   - `readme_has_production3_mention`
   - `contributing_has_doctor_step`
3. `cargo test` 全通過後に `cargo clippy -- -D warnings` → 警告なし確認

---

## 影響範囲

| ファイル | 変更種別 |
|---|---|
| `README.md` | Production 3.0 言及・v54.1〜v54.5 サマリー追加 |
| `CONTRIBUTING.md` | `fav doctor` 環境診断セクション追加・`fav bench` パフォーマンス確認セクション追加 |
| `fav/src/driver.rs` | `v54600_tests` 追加 |
| `fav/Cargo.toml` | version 更新 |
| `fav/Cargo.lock` | version 更新に伴い自動更新 |
| `CHANGELOG.md` | v54.6.0 エントリ追加 |
| `versions/current.md` | v54.6.0 / 3197 tests に更新 |
| `versions/roadmap/roadmap-v54.1-v55.0.md` | v54.6.0 実績欄を COMPLETE に更新 |

---

## 設計上の注意

- `--all` フラグは `fav bench` で no-op のためドキュメントに含めない。
- v54.0 宣言の直後に v54.1〜v54.5 サマリーを配置（README の降順ルールと時系列整合を両立）。
- `use super::*` を `v54600_tests` に明示（他テストモジュールとの慣習統一）。
- `readme_has_production3_mention` は `"Production 3.0"` と `"v54.1"` の 2 アサーションで v54.6.0 の追加行を特定。
