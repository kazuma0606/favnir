# v63.0.0 Spec — AOT Native 宣言 ★クリーンアップ

Version: 63.0.0
Status: 未着手
Base tests: 3402
Target tests: 3406

---

## 概要

v62.1〜v62.9 の AOT スプリント全成果を統合し、**AOT Native** マイルストーンを宣言する。
`fav build` によるネイティブバイナリ生成・Docker イメージ化・AOT 互換性チェックが揃った
Favnir の新段階を記録する。

あわせて `cargo clean`（★クリーンアップ）を実施し、ビルドキャッシュをリセットする。

---

## 宣言文

> 「パイプラインはネイティブバイナリにコンパイルされ、VM オーバーヘッドを超える速度で動く。
>  クロスコンパイルで ARM にも届き、Docker イメージは最小限のサイズに収まる。
>
>  Favnir は型安全なコンパイル言語として新たな段階に達した。
>
>  これが Favnir v63.0 — AOT Native の姿である。」

---

## 前提確認（T0 で実施）

- `cargo test -j 8 -- --test-threads=8` でベース 3402 tests passed, 0 failed を確認
  （ロードマップ記載 3400 より +2 — v62.8.0 code-reviewer 対応で `aot_no_emit_passes` が追加されたため）
- `fav/Cargo.toml` の現行バージョンが `62.0.0` であることを確認
- `MILESTONE.md` に `"v63.0.0"` かつ `"AOT Native"` の組み合わせが **存在しない** ことを確認
- `MILESTONE.md` に `"v63.0.0"` が **存在しない** ことを確認
- `README.md` に `"v63.0.0"` が **存在しない** ことを確認
- `driver.rs` に `v62000_tests` が存在することを確認（挿入位置確認）

---

## 実装スコープ

### 1. `fav/Cargo.toml` — バージョン更新 + 既存アサーション一括置換

`version = "62.0.0"` → `version = "63.0.0"` に変更。

**重要**: `driver.rs` 内に `cargo.contains("version = \"62.0.0\"")` を assertする既存テストが 11 件ある。
Cargo.toml を 63.0.0 に更新すると、これら 11 件が全て FAIL する。
一括置換で `"62.0.0"` → `"63.0.0"` に更新することが必須（全バージョン宣言時の標準手順）。

### 2. `MILESTONE.md` — AOT Native 宣言エントリ追加

既存の最新エントリ（v62.0.0 Language Polish）の直後に `## v63.0.0 — AOT Native（2026-08-02）` セクションを追加。

```markdown
## v63.0.0（2026-08-02）— AOT Native

> 「パイプラインはネイティブバイナリにコンパイルされ、VM オーバーヘッドを超える速度で動く。
>  クロスコンパイルで ARM にも届き、Docker イメージは最小限のサイズに収まる。
>
>  Favnir は型安全なコンパイル言語として新たな段階に達した。
>
>  これが Favnir v63.0 — AOT Native の姿である。」

**AOT Native** の宣言バージョン。v62.1〜v62.9 で実装した AOT 全機能を統合し、
ネイティブバイナリ生成・クロスコンパイル・Docker イメージ化・AOT 互換性チェックの完成を宣言した。

**v62.1〜v62.9 達成内容:**
- v62.1（`fav build` 基盤）: `cmd_build_basic` / `cmd_build_link` / `cmd_build_docker` API 基盤
- v62.2（native binary 生成）: Cranelift AOT コンパイル → `.o` ファイル生成
- v62.3（クロスコンパイル）: x86_64 / aarch64 クロスコンパイル対応
- v62.4（Pure stage インライン化）: `analyze_for_inlining` / `is_aot_pure` による最適化
- v62.5（`fav bench`）: ステージ別ベンチマーク計測
- v62.6（Docker 出力）: `fav build --docker` OCI イメージ生成
- v62.7（`fav.toml [build]`）: `BuildConfig` / `ResolvedBuildConfig` AOT 設定
- v62.8（E0427）: AOT 未サポート機能検出バリデーター・エラーカタログ登録
- v62.9（E2E デモ）: `infra/e2e-demo/aot/` + `site/content/docs/runtime/aot.mdx`

**テスト数**: 3406
```

### 3. `README.md` — v63.0 AOT Native 言及追加

既存の `v62.0.0 — Language Polish` 記述の直後に追記：

```
v63.0.0（2026-08-02）で、**AOT Native** マイルストーンを宣言しました。
`fav build --link` でネイティブバイナリを生成し、`--docker` で OCI イメージを出力し、
`--validate` で AOT 互換性（E0427）を事前チェックできます。
Favnir は VM 実行に加え、型安全なコンパイル言語としての段階に達しました。
```

### 4. `CHANGELOG.md` — v63.0.0 エントリ追加

```markdown
## [v63.0.0] — 2026-08-02 — AOT Native 宣言 ★クリーンアップ

### Added
- `MILESTONE.md` に AOT Native 宣言エントリを追加（v62.1〜v62.9 の全 AOT 機能集約）
- Rust テスト 4 件追加（`v63000_tests`）
  - `cargo_toml_version_is_63_0_0`
  - `changelog_has_v63_0_0`
  - `milestone_has_aot_native`
  - `readme_mentions_aot_native`

### Changed
- `fav/Cargo.toml` バージョンを `62.0.0` → `63.0.0` に更新
- `driver.rs` 内の `cargo.contains("version = \"62.0.0\"")` アサーション 11 件を `63.0.0` に一括更新

### Notes
- ★クリーンアップ（`cargo clean`）実施済み
```

### 5. `driver.rs` — `v63000_tests` 追加

`v62000_tests` の閉じ括弧の**直後**（v62900_tests の直前）に挿入。

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

---

## 完了条件

- `cargo build` エラーなし
- `cargo test v63000` で 4 件 PASS
- `cargo test -j 8 -- --test-threads=8` で 3406 tests passed, 0 failed
- `cargo clean` + クリーン後ビルド成功 + テスト全通過
- `fav/tmp/hello.fav` 復元確認（cargo clean で消える可能性あり）

---

## 非スコープ

- 新機能の追加（v63.x 以降）
- v62.x スプリント機能の追加実装
- サイト MDX の更新（v62.9.0 で aot.mdx 追加済み）

---

## 技術ノート

### ベーステスト数の変更について

ロードマップ記載のベースは 3400 だが、v62.8.0 code-reviewer 対応で `aot_no_emit_passes` が追加されたため
実際のベースは **3402**。完了条件のターゲットは 3402 + 4 = **3406**。

### 既存 `cargo_toml_version_is_X` アサーション更新について

バージョン宣言時の標準手順：Cargo.toml バージョン更新後、`driver.rs` 内の
`cargo.contains("version = \"62.0.0\"")` アサーション（11 件）を
`cargo.contains("version = \"63.0.0\"")` に一括置換する。
テスト関数名（`fn cargo_toml_version_is_62_0_0()`）は変更しない（歴史的記録として残す）。

### `★クリーンアップ` 後の `fav/tmp/hello.fav` 復元

`cargo clean` で `fav/tmp/` が消える可能性がある。
`hello.fav` の正しい内容：
```
fn add(a: Int, b: Int) -> Int { a + b }
fn main() -> Bool { add(1, 2) == 3 }
```
T7 で明示的に確認・復元する。

### `v63000_tests` 挿入位置

`v62000_tests` モジュールの閉じ括弧（`}`）の直後、`v62900_tests` の直前に挿入する。
（v62.0.0 と同じ「マイルストーン宣言テストをスプリントテストの前に配置」パターン）
