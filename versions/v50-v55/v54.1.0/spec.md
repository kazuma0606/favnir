# Spec: v54.1.0 — 全エラーコード fav explain --error 対応完備

Status: COMPLETE
Date: 2026-07-22

---

## 概要

`error_catalog.rs` に登録された全 92 エラーコードについて、`cmd_explain_error_collect` が
`Some` かつ非空のテキストを返すことをテストで強制する。
将来追加されるコードも `list_all()` の動的走査により自動カバーされる。

既存の `fav explain-error`（`main.rs` `Some("explain-error")`）は実装済み。
本バージョンはカバレッジ網羅の検証テストと E0419（assert_schema 型不一致）の個別テストを追加する。

---

## 実装スコープ

### 1. `driver.rs` — `v54100_tests` 追加

`v54000_tests` の直前に `v54100_tests` モジュールを追加:

```rust
// -- v54100_tests (v54.1.0) -- 全エラーコード fav explain --error 対応完備 --
#[cfg(test)]
mod v54100_tests {
    use super::*;

    #[test]
    fn explain_error_all_codes_have_collect_text() {
        let all = crate::error_catalog::list_all();
        assert!(!all.is_empty(), "ERROR_CATALOG must not be empty");
        for entry in all {
            let result = cmd_explain_error_collect(entry.code);
            assert!(
                result.is_some(),
                "cmd_explain_error_collect({}) returned None",
                entry.code
            );
            let text = result.unwrap();
            assert!(
                !text.is_empty(),
                "explain text for {} must not be empty",
                entry.code
            );
        }
    }

    #[test]
    fn explain_error_e0419_exists() {
        let result = cmd_explain_error_collect("E0419");
        assert!(result.is_some(), "E0419 must have an explain entry");
        let text = result.unwrap();
        assert!(
            text.contains("E0419"),
            "explain text for E0419 must contain the code"
        );
        assert!(
            text.contains("assert_schema"),
            "explain text for E0419 must reference assert_schema"
        );
    }
}
```

### 2. `v54000_tests::cargo_toml_version_is_54_0_0` 空化

バージョンが 54.1.0 に進んだため空化:

```rust
fn cargo_toml_version_is_54_0_0() {
    // v54.1.0 にバンプしたためアサートを空化。
}
```

### 3. `fav/Cargo.toml` バージョン更新

`"54.0.0"` → `"54.1.0"`

---

## テスト仕様

| テスト名 | 検証内容 |
|---|---|
| `explain_error_all_codes_have_collect_text` | `list_all()` で全 92 コードを走査し `cmd_explain_error_collect` が `Some` かつ非空を返すこと |
| `explain_error_e0419_exists` | E0419 のテキストが `"E0419"` と `"assert_schema"` を含むこと |

パス確認:
- `cmd_explain_error_collect`: `driver.rs` 内に実装済み（`error_catalog::lookup` → フォーマット）
- `crate::error_catalog::list_all()`: `error_catalog.rs` に実装済み（92 エントリ）
- E0419 エントリ: `error_catalog.rs` に title `"assert_schema type mismatch"` で登録済み

---

## バージョン更新

- `fav/Cargo.toml`: `"54.0.0"` → `"54.1.0"`

---

## 完了条件

- `cargo test` 3187 passed, 0 failed（ベース 3185 + 2 件追加）
- `v54100_tests` 2 件 pass:
  - `explain_error_all_codes_have_collect_text`
  - `explain_error_e0419_exists`
- `cargo clippy -- -D warnings` クリーン

---

## 影響範囲

| ファイル | 変更種別 |
|---|---|
| `fav/src/driver.rs` | `v54100_tests` 追加 / `cargo_toml_version_is_54_0_0` 空化 |
| `fav/Cargo.toml` | version 更新 |
| `fav/Cargo.lock` | version 更新に伴い自動更新 |
| `CHANGELOG.md` | v54.1.0 エントリ追加 |
| `versions/current.md` | v54.1.0 / 3187 tests に更新 |
| `versions/roadmap/roadmap-v54.1-v55.0.md` | v54.1.0 実績欄を COMPLETE に更新 |

---

## 設計上の注意

- `explain_error_all_codes_have_collect_text` はモジュール `v503000_tests` に同名テスト
  `explain_error_all_codes_have_text` が存在するため、名前を `_collect_text` サフィックス付きで区別する。
- `v54100_tests` は `v54000_tests` の直前に挿入（逆時系列順）。
- `cargo_toml_version_is_54_0_0` の空化は直前バージョンバンプの標準パターンに従う。
