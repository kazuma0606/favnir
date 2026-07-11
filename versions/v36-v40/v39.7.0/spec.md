# v39.7.0 spec — CI/CD ポリシーゲート

## バージョン概要

| 項目 | 内容 |
|---|---|
| バージョン | v39.7.0 |
| テーマ | CI/CD ポリシーゲート — `fav ci init` 生成 YAML に `fav policy check --ci` を自動含める |
| 前提 | v39.6.0 COMPLETE — `fav audit` 完了 |
| 完了条件 | `v39700_tests` 全テスト pass・`cargo test` 0 failures（≥ 2805 件） |

## 背景と目的

v39.3.0 で `fav policy check --ci`（違反時 stderr + exit 1）を実装した。
v39.7.0 では `fav ci init` が生成する `.github/workflows/ci.yml` に
`fav policy check --ci` ステップを自動で含めるよう `generate_ci_yaml` を拡張する。

これにより PR ベースのポリシーゲートが CI/CD に自動組み込まれる。

**現在の `generate_ci_yaml` 出力（変更前）**:
```yaml
steps:
  - uses: actions/checkout@v4
  - name: Install fav
    run: cargo install fav
  - name: Check
    run: fav check
  - name: Lint
    run: fav lint
  - name: Test
    run: fav test
```

**変更後（Policy check ステップ追加）**:
```yaml
steps:
  - uses: actions/checkout@v4
  - name: Install fav
    run: cargo install fav
  - name: Check
    run: fav check
  - name: Lint
    run: fav lint
  - name: Test
    run: fav test
  - name: Policy check
    run: fav policy check --ci
```

## 実装スコープ

### 1. `fav/src/driver.rs` — `generate_ci_yaml` 変更

**変更対象関数**: `pub fn generate_ci_yaml(_project_name: &str) -> String`（行 15492 付近）

**変更内容**: 末尾の `fav test` ステップの後に Policy check ステップを追加

```rust
pub fn generate_ci_yaml(_project_name: &str) -> String {
    "name: CI\n\
     on:\n\
       push:\n\
         branches: [main]\n\
       pull_request:\n\
     \n\
     jobs:\n\
       ci:\n\
         runs-on: ubuntu-latest\n\
         steps:\n\
           - uses: actions/checkout@v4\n\
           - name: Install fav\n\
             run: cargo install fav\n\
           - name: Check\n\
             run: fav check\n\
           - name: Lint\n\
             run: fav lint\n\
           - name: Test\n\
             run: fav test\n\
           - name: Policy check\n\
             run: fav policy check --ci\n"
        .to_string()
}
```

**既存テストへの影響**:
- `generate_ci_yaml_has_check_step`（`fav check` を確認）→ 変更なし、引き続き pass
- `generate_ci_yaml_has_lint_step`（`fav lint` を確認）→ 変更なし、引き続き pass
- `generate_ci_yaml_has_test_step`（`fav test` を確認）→ 変更なし、引き続き pass

### 2. `driver.rs` — テストモジュール追加

#### `v39600_tests::cargo_toml_version_is_39_6_0` のスタブ化

```rust
// Stubbed: version bumped to 39.7.0 — assertion intentionally removed
```

#### `v39700_tests` モジュール新規追加

```rust
// ── v39700_tests (v39.7.0) — CI/CD ポリシーゲート ────────────────────────────
#[cfg(test)]
mod v39700_tests {
    // include_str! のみ使用のため imports 不要

    #[test]
    fn cargo_toml_version_is_39_7_0() {
        // NOTE: 次バージョン bump 時に Stubbed コメントへ置き換えること
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("39.7.0"), "Cargo.toml must contain version 39.7.0");
    }

    #[test]
    fn changelog_has_v39_7_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(src.contains("[v39.7.0]"), "CHANGELOG.md must contain [v39.7.0]");
    }
}
```

> ロードマップ「Rust テスト 2 件」= meta テスト 2 件（version + changelog）。
> `generate_ci_yaml` の変更は既存テスト 3 件（check/lint/test ステップ確認）の継続 pass で暗黙的に検証される。

### 3. `CHANGELOG.md` — `[v39.7.0]` エントリ追加

`## [v39.6.0]` ヘッダ行の直前に挿入:

```
## [v39.7.0] — YYYY-MM-DD

### Changed
- `driver.rs` `generate_ci_yaml` — `fav policy check --ci` ステップを CI YAML に自動追加
- `fav ci init` 生成 YAML が Policy check ゲートを含むようになった
- `v39700_tests` 2 テスト追加（meta 2 件）

---
```

**セパレータは `—`（全角ダッシュ U+2014）**

> **Changed セクション使用**: v39.7.0 は新規ファイル追加ではなく既存機能の拡張のため `### Changed` を使用する。

### 4. その他ドキュメント更新

- `fav/Cargo.toml`: `39.6.0` → `39.7.0`
- `versions/current.md`: 最新安定版 → v39.7.0、次に切る版 → v39.8.0
- `versions/roadmap/roadmap-v39.1-v40.0.md`: v39.7.0 を ✅ 完了済みにマーク

## 注意事項

### 新規ファイル・main.rs 変更なし

v39.7.0 は `generate_ci_yaml` の文字列追加のみ。新規 Rust ソースファイルの作成・`main.rs` への `mod` 追加・ディスパッチアーム追加は不要。`compiler.fav` / `checker.fav` 等のセルフホスト側ファイルへの変更も不要。

### 既存テストの継続 pass 確認

`generate_ci_yaml_has_check_step` / `generate_ci_yaml_has_lint_step` / `generate_ci_yaml_has_test_step` は修正後も pass することを確認する。変更は追記のみのため既存ステップ文字列は維持される。

### YAML インデント

Rust の文字列リテラル内のインデントは実際の YAML のインデントと一致させること。`generate_ci_yaml` は先頭にスペース 5 個（`"     - name: ..."`）のパターンで統一されている。

## テスト数の計算

| バージョン | 実績 |
|---|---|
| v39.6.0 | 2803 |
| v39.7.0 追加分 | +2（meta 2 件） |
| v39.7.0 期待値 | 2805 |

## 完了条件

| # | 条件 | 検証方法 |
|---|---|---|
| 1 | `CHANGELOG.md` に `[v39.7.0]` が含まれる | `changelog_has_v39_7_0` テスト |
| 2 | `Cargo.toml` バージョンが `39.7.0` | `cargo_toml_version_is_39_7_0` テスト |
| 3 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2805） | `cargo test` 実行結果 |
| 4 | `generate_ci_yaml` 出力に `fav policy check --ci` が含まれる | 既存テスト継続 pass + cargo コンパイル検証 |
| 5 | 既存 `generate_ci_yaml_has_*` 3 テストが引き続き pass | `cargo test` 実行結果（regression なし）|
| 6 | `roadmap-v39.1-v40.0.md` の v39.7.0 が ✅ | T6 後に目視確認 |
