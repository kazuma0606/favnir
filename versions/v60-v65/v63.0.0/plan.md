# v63.0.0 Plan — AOT Native 宣言 ★クリーンアップ

Version: 63.0.0
Status: 未着手

---

## 実装順序

### Step 1: `driver.rs` — `v63000_tests` 追加（先にテストを書く）

`v62000_tests` の直後に `v63000_tests` モジュールを挿入する。
この時点では Cargo.toml / MILESTONE.md / README.md が未更新なので、テストは **FAIL** する（想定内）。

```rust
// -- v63000_tests (v63.0.0) -- AOT Native 宣言 --
#[cfg(test)]
mod v63000_tests {
    #[test]
    fn cargo_toml_version_is_63_0_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(
            cargo.contains("version = \"63.0.0\""),
            "Cargo.toml should contain version = \"63.0.0\"; got: {:?}",
            &cargo[..200.min(cargo.len())]
        );
    }

    #[test]
    fn changelog_has_v63_0_0() {
        let cl = include_str!("../../CHANGELOG.md");
        assert!(
            cl.contains("v63.0.0"),
            "CHANGELOG.md should contain v63.0.0 entry"
        );
    }

    #[test]
    fn milestone_has_aot_native() {
        let ms = include_str!("../../MILESTONE.md");
        assert!(
            ms.contains("v63.0.0") && ms.contains("AOT Native"),
            "MILESTONE.md should contain both v63.0.0 and AOT Native"
        );
    }

    #[test]
    fn readme_mentions_aot_native() {
        let readme = include_str!("../../README.md");
        assert!(
            readme.contains("v63.0.0") && readme.contains("AOT Native"),
            "README.md should contain both v63.0.0 and AOT Native"
        );
    }
}
```

### Step 2: `fav/Cargo.toml` — バージョン 62.0.0 → 63.0.0

`version = "62.0.0"` を `version = "63.0.0"` に変更。

`cargo test v63000` を実行し `cargo_toml_version_is_63_0_0` が PASS することを確認。
この時点で既存の `cargo_toml_version_is_*` テスト 11 件が FAIL するので、次の Step で修正する。

### Step 3: `driver.rs` — 旧バージョンアサーション一括置換

`cargo.contains("version = \"62.0.0\"")` を `cargo.contains("version = \"63.0.0\"")` に
driver.rs 全体で一括置換（11 件）。

テスト関数名（`fn cargo_toml_version_is_62_0_0()`）は変更しない。

`cargo build` でエラーなし確認。

### Step 4: `CHANGELOG.md` — v63.0.0 エントリ追加

先頭に `## [v63.0.0] — 2026-08-02 — AOT Native 宣言 ★クリーンアップ` エントリを追加。
`cargo test v63000` → `changelog_has_v63_0_0` が PASS することを確認。

### Step 5: `MILESTONE.md` — AOT Native 宣言エントリ追加

既存の最新エントリ（v62.0.0 Language Polish）の直後に AOT Native エントリを追加。
`cargo test v63000` → `milestone_has_aot_native` が PASS することを確認。

### Step 6: `README.md` — v63.0.0 AOT Native 言及追加

`v62.0.0 — Language Polish` 記述の直後に v63.0.0 AOT Native を追記。
`cargo test v63000` → `readme_mentions_aot_native` が PASS することを確認。

### Step 7: 全テスト

`cargo test v63000` で 4 件 PASS 確認。
`cargo test -j 8 -- --test-threads=8` で 3406 tests passed, 0 failed を確認（実測ベース + 4）。

### Step 8: ★クリーンアップ（cargo clean）

`cargo clean` を実行。
`fav/tmp/hello.fav` の存在を確認（消えた場合は復元）。
`cargo build` でクリーン後ビルド成功確認。
`cargo test -j 8 -- --test-threads=8` で 3406 tests passed, 0 failed を確認（クリーン後）。

### Step 9: ドキュメント更新

roadmap / current.md / tasks.md を更新。
（CHANGELOG.md は Step 4 で対応済み）

---

## 設計メモ

### テスト挿入位置

`v62900_tests` コメント行の直前（= `v62000_tests` の閉じ括弧 `}` の直後）に挿入。
マイルストーン宣言テストをスプリントテスト群の外側に配置するパターンを踏襲する。

### AND 条件の必要性

- `milestone_has_aot_native`: `ms.contains("v63.0.0") && ms.contains("AOT Native")` —
  `"AOT Native"` 単独では既存 MILESTONE.md に含まれている可能性があるため AND 必須。
- `readme_mentions_aot_native`: 同様。`"AOT Native"` の事前存在を仮定しない。

### 旧アサーション一括置換の影響

`cargo.contains("version = \"62.0.0\"")` を `"63.0.0"` に置換すると、
関数名 `fn cargo_toml_version_is_62_0_0()` と中身のアサーションが意味的に不一致になる。
これは歴史的経緯として許容する（v62.0.0 の完了記録として関数名を残す）。
11 件すべてを一括置換するため、置換漏れがないよう `cargo build` 後に確認する。

### ロードマップとの乖離

- ベーステスト数: ロードマップ記載 3400 → 実際 3402（v62.8.0 code-reviewer 対応 +2）
- ターゲット: 3402 + 4 = 3406（ロードマップ記載 3404 より +2）
