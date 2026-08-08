# v64.6.0 Spec — `fav lint --perf`（パフォーマンス lint 一括実行）

Version: 64.6.0
Status: 未着手
Base tests: 3441
Target tests: 3443

---

## 概要

`fav.toml` の `[lint]` セクションに `perf = true` オプションを追加し、
`toml.rs` の `LintTomlConfig` に `perf: Option<bool>` フィールドを追加してパースする。
`v64600_tests` 2 件を `driver.rs` に追加する。

ロードマップ `roadmap-v64.1-v65.0.md` の v64.6.0 セクションに準拠。

---

## 背景

### 既存実装

- `lint.rs`: `LintConfig { strict: bool, perf: bool }` はすでに存在（v63.6.0 追加）
- `lint.rs`: W041（large collect without filter）は `config.perf || config.strict` でゲート済み（v63.6.0）
- `lint.rs`: W039（as-name shadows）・W040（type hole `_`）は常時有効
- `driver.rs`: `cmd_lint` に `LintConfig { strict: strict_mode, perf: false }` がハードコード（line ~11185）
- `toml.rs`: `LintTomlConfig` は `warn_as_error / allow / strict` を持つが `perf` はなし

### スコープ縮小

`--perf` CLI フラグの `main.rs` 追加と `cmd_lint` シグネチャ変更は後送り（v64.7 以降）。
ロードマップ v64.6.0 実績欄に「`--perf` CLI フラグは後送り」と明記する（T5 にて対応）。
本バージョンでは以下のみ実装する:
1. `LintTomlConfig` に `perf: Option<bool>` を追加し `[lint] perf = true` をパース
2. `cmd_lint` の `perf` を toml 設定から読み取るよう更新（`perf: false` → `toml.perf.unwrap_or(false)`）
3. `v64600_tests` 2 件を追加

**スコープ外（v64.8 以降）**

`site/content/docs/tools/lint.mdx` 等への `perf` オプション説明追記は Performance 1.0 総括記事（v64.8.0）でまとめて対応する。

---

## 実装内容

### 1. `toml.rs` — `LintTomlConfig` に `perf` フィールド追加

```rust
pub struct LintTomlConfig {
    pub warn_as_error: Option<Vec<String>>,
    pub allow: Option<Vec<String>>,
    pub strict: Option<bool>,
    /// v64.6.0: `perf = true` で W041 等のパフォーマンス lint を有効化。
    pub perf: Option<bool>,
}
```

`parse_fav_toml` の `[lint]` セクション処理:
- struct literal に `perf: None` を追加
- `key == "perf"` アームを追加: `current.perf = Some(val.trim() == "true");`（`strict` と同じパターン）

### 2. `driver.rs` — `cmd_lint` の `perf` を toml 設定から読み取るよう更新

```rust
// 変更前（line ~11185）:
let run_config = LintConfig { strict: strict_mode, perf: false };

// 変更後:
let perf_mode = lint_config.as_ref().and_then(|c| c.perf).unwrap_or(false);
let run_config = LintConfig { strict: strict_mode, perf: perf_mode };
```

### 3. `v64600_tests` モジュール追加（`driver.rs`）

`v64500_tests` の直前に挿入。

```rust
mod v64600_tests {
    use super::*;
    use crate::lint::{lint_program_with_config, LintConfig};

    #[test]
    fn lint_perf_flag_enables_w041() {
        // W041 は perf モードでのみ有効（large collect without filter）
        // v63600_tests と同じ `collect { ... }` ブロック構文で W041 を確実に発火させる
        let src = concat!(
            "public stage Collect: Int -> Int = |x| {\n",
            "    collect { yield 1; yield 2; 0 }\n",
            "}\n",
            "pipeline P { step \"run\" = seq Collect }\n"
        );
        let program = crate::frontend::parser::Parser::parse_str(src, "<test>")
            .expect("parse should succeed");
        // lint_program_with_config のシグネチャ: (program: &Program, config: &LintConfig) -> Vec<LintError>
        let errors = lint_program_with_config(&program, &LintConfig { strict: false, perf: true });
        assert!(
            errors.iter().any(|e| e.code == "W041"),
            "W041 should fire in perf mode for large collect; errors: {:?}", errors
        );
    }

    #[test]
    fn lint_toml_perf_setting() {
        let toml_src = "[project]\nname = \"myproj\"\n\n[lint]\nperf = true\n";
        let toml = crate::toml::parse_fav_toml_pub(toml_src);
        let lint = toml.lint.expect("lint section should be parsed");
        assert_eq!(lint.perf, Some(true), "perf should be Some(true) when perf = true in [lint]");
    }
}
```

**注意**: テスト名は `lint_perf_flag_enables_w041`（W039/W040 は常時有効のため perf ゲートなし、W041 のみ perf ゲート対象）。ロードマップのテスト名 `lint_perf_flag_enables_w039_w040` と異なるが実態に即した名称とする。

---

## 完了条件

- `cargo test --bin fav v64600_tests` で 2 件 PASS:
  - `lint_perf_flag_enables_w041`（ロードマップ名 `lint_perf_flag_enables_w039_w040` を実態に即して変更）
  - `lint_toml_perf_setting`
- `cargo test -j 8 -- --test-threads=8` で **3443 tests passed, 0 failed**

---

## 参照

- ロードマップ: `versions/roadmap/roadmap-v64.1-v65.0.md`（v64.6.0 セクション）
- 前バージョン: `versions/v60-v65/v64.5.0/`
- 関連: `fav/src/lint.rs`（`LintConfig` / W041）、`fav/src/toml.rs`（`LintTomlConfig`）
