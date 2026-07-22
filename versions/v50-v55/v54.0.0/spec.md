# Spec: v54.0.0 — Integration Sprint 宣言

Status: COMPLETE
Date: 2026-07-22

---

## 概要

v51.0（DX 3.0）・v52.0（Performance & Scale）・v53.0（Data Quality 2.0）の 3 マイルストーンを統合し、
**Integration Sprint** として宣言する。

v53.1〜v53.9 の統合サブバージョン（lineage × LSP・par bench・assert_schema 詳細診断・E2E デモ・
cookbook・用語集・CHANGELOG/MILESTONE 整理・integration-overview ドキュメント・コードフリーズ）を完了した上で、
`MILESTONE.md` に v54.0.0 宣言セクションを追加し、`README.md` に Integration Sprint 宣言を追記する。

`★クリーンアップ`（`cargo clean`）を v54.0.0 完了の一環として実施する。

---

## 実装スコープ

### 1. `MILESTONE.md` — v54.0.0 宣言セクション追加

ファイル先頭（既存の「v51.0〜v53.0 Integration Sprint サマリー」セクションの直前）に追加:

```md
## v54.0.0（2026-07-22）— Integration Sprint

> 「エディタはデータの来歴を示し、並列パイプラインの性能は
>  計測可能で、スキーマ違反は即座に修正できる。
>  Favnir の 3 つの柱が一体となった。
>
>  これが Favnir v54.0 — Integration の姿である。」

**Integration Sprint** の宣言バージョン。v53.1〜v53.8 の統合作業（lineage × LSP・par bench・assert_schema 詳細診断・
E2E デモ・cookbook・用語集・CHANGELOG/MILESTONE 整理・integration-overview ドキュメント）および
v53.9 のコードフリーズを経て、v51.0〜v53.0 の 3 マイルストーンを一体として機能させた。
```

必須要件:
- `Integration Sprint` という文字列を含む

---

### 2. `README.md` — v54.0 Integration Sprint 宣言追記

v53.0 宣言エントリの直前（ファイル先頭側）に追加:

```md
**v54.0（2026-07-22）で、[Integration Sprint](./MILESTONE.md) マイルストーンを宣言しました。**
エディタはデータの来歴を示し、並列パイプラインの性能は計測可能で、スキーマ違反は即座に修正できる。
Favnir の 3 つの柱（DX 3.0 / Performance & Scale / Data Quality 2.0）が一体となった — これが **Integration Sprint** の宣言です。
```

必須要件:
- `Integration Sprint` という文字列を含む

---

### 3. `CHANGELOG.md` — v54.0.0 エントリ追加

```md
## [v54.0.0] — 2026-07-22 — Integration Sprint 宣言

### Added
- `MILESTONE.md`: v54.0.0 Integration Sprint 宣言セクション追加（宣言文・v53.1〜v53.9 完了サマリー）
- `README.md`: v54.0 Integration Sprint マイルストーン宣言を追加
- `v54000_tests` 追加 — 3185 tests
```

必須要件:
- `v54.0.0` という文字列を含む

---

### 4. テスト仕様

`v54000_tests` モジュールを `driver.rs` に追加（`v53900_tests` の直前）:

```rust
// -- v54000_tests (v54.0.0) -- Integration Sprint 宣言 --
#[cfg(test)]
mod v54000_tests {
    #[test]
    fn cargo_toml_version_is_54_0_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("\"54.0.0\""), "Cargo.toml must have version 54.0.0");
    }

    #[test]
    fn changelog_has_v54_0_0() {
        let content = include_str!("../../CHANGELOG.md");
        assert!(content.contains("v54.0.0"), "CHANGELOG.md must contain v54.0.0 entry");
    }

    #[test]
    fn milestone_has_integration_sprint() {
        let content = include_str!("../../MILESTONE.md");
        assert!(
            content.contains("Integration Sprint"),
            "MILESTONE.md must contain Integration Sprint declaration"
        );
    }

    #[test]
    fn readme_mentions_integration_sprint() {
        let content = include_str!("../../README.md");
        assert!(
            content.contains("Integration Sprint"),
            "README.md must mention Integration Sprint"
        );
    }
}
```

パス確認:
- `include_str!("../Cargo.toml")`: `fav/src/` → `../` = `fav/Cargo.toml` ✓
- `include_str!("../../CHANGELOG.md")`: `fav/src/` → `../../` = `favnir/CHANGELOG.md` ✓
- `include_str!("../../MILESTONE.md")`: `fav/src/` → `../../` = `favnir/MILESTONE.md` ✓
- `include_str!("../../README.md")`: `fav/src/` → `../../` = `favnir/README.md` ✓

### 5. `v53900_tests::cargo_toml_version_is_53_9_0` の空化

バージョンが 54.0.0 に進んだため、`v53900_tests` のバージョンピンテストを空化:

```rust
fn cargo_toml_version_is_53_9_0() {
    // v54.0.0 にバンプしたためアサートを空化。
}
```

---

### 6. ★クリーンアップ（`cargo clean`）

v54.0.0 の完了条件として `cargo clean` を実施し、clean 後も全テストが通過することを確認する。

注意: `cargo clean` 前に `fav/tmp/hello.fav` の内容を確認すること。
正しい内容:
```
fn add(a: Int, b: Int) -> Int { a + b }
fn main() -> Bool { add(1, 2) == 3 }
```
`cargo clean` は `target/` ディレクトリのみ削除するため `fav/tmp/hello.fav` は影響を受けないが、
念のため clean 後も存在することを確認する。

---

## バージョン更新

- `fav/Cargo.toml`: `"53.9.0"` → `"54.0.0"`

---

## 完了条件

- `cargo test` 3185 passed, 0 failed（ベース 3181 + 4 件追加）
  - テスト数 ≥ 3179（ロードマップ要件）を満たす
- `v54000_tests` 4 件 pass:
  - `cargo_toml_version_is_54_0_0`
  - `changelog_has_v54_0_0`
  - `milestone_has_integration_sprint`
  - `readme_mentions_integration_sprint`
- `MILESTONE.md` に `"Integration Sprint"` が含まれる
- `cargo clean` 完了（★クリーンアップ）
- `cargo clean` 後に `cargo test` 3185 passed, 0 failed を再確認
- `cargo clippy -- -D warnings` クリーン

---

## 影響範囲

| ファイル | 変更種別 |
|---|---|
| `MILESTONE.md` | v54.0.0 Integration Sprint 宣言セクション追加 |
| `README.md` | v54.0 マイルストーン宣言追加 |
| `CHANGELOG.md` | v54.0.0 エントリ追加 |
| `fav/src/driver.rs` | `v54000_tests` 追加 / `cargo_toml_version_is_53_9_0` 空化 |
| `fav/Cargo.toml` | version 更新 |
| `fav/Cargo.lock` | version 更新に伴い自動更新 |
| `versions/current.md` | v54.0.0 / 3185 tests に更新 |
| `versions/roadmap/roadmap-v53.1-v54.0.md` | v54.0.0 実績欄を COMPLETE に更新 |

---

## 設計上の注意

- ロードマップの宣言引用文（4 行構成・空行あり）を MILESTONE.md に正確に転記する。
  v53.x の宣言文と同じ blockquote スタイル（`>` 行間空行あり）で統一する。
- `v54000_tests` は `use super::*` 不要（`include_str!` マクロのみ使用）。
- `readme_mentions_integration_sprint` のアサート対象: README.md の v54.0 宣言追記箇所に
  "Integration Sprint" が含まれること。README.md は v53.0 の直前に v54.0 を挿入（降順リスト）。
- コードレビュー対応:
  - [LOW] MILESTONE.md の v54.0 説明文で「v53.1〜v53.9 の統合作業」ではなく
    「v53.1〜v53.8 の統合作業・v53.9 のコードフリーズ」と分離して記述する
  - [LOW] Integration Sprint サマリーの "v53.1〜v53.8 で...準備を整えた" を
    "v53.1〜v53.8 統合作業・v53.9 コードフリーズ完了・v54.0 宣言達成" に更新する
