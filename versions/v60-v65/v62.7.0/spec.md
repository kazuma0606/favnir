# v62.7.0 Spec — `fav.toml` `[build]` セクション（AOT 設定）

Version: 62.7.0
Status: 未着手
Base tests: 3394
Target tests: 3396

---

## 概要

`fav.toml` に `[build]` セクションを追加し、AOT ビルドのデフォルト設定を記述できるようにする。
`fav build` コマンドが `fav.toml` の `[build]` 設定をデフォルト値として読み込み、
CLI フラグが `fav.toml` 設定を上書きする優先順位（CLI > fav.toml > デフォルト）を実装する。

```toml
[build]
target = "x86_64-unknown-linux-gnu"
opt_level = 2
inline_pure_stages = true
output_dir = "dist/"
```

---

## 前提確認（T0 で実施）

- `toml.rs` に `BuildConfig` が **存在しない** ことを確認
- `toml.rs` の `FavToml` に `build` フィールドが **存在しない** ことを確認
- `parse_fav_toml` に `"[build]"` セクション処理が **存在しない** ことを確認
- `driver.rs` に `resolve_build_config` が **存在しない** ことを確認
- `driver.rs` に `v62600_tests` が存在することを確認（挿入位置確認）
- `cargo test -j 8 -- --test-threads=8` でベース 3394 tests passed, 0 failed を確認

---

## 実装スコープ

### 1. `toml.rs` — `BuildConfig` 構造体 + `FavToml.build` フィールド + パース追加

**`BuildConfig` 構造体**（新規追加、`// ── Build config (v62.7.0) ──` コメント付き）:

```rust
#[derive(Debug, Clone)]
pub struct BuildConfig {
    /// ターゲットトリプル。デフォルト: "x86_64-unknown-linux-gnu"。
    pub target: String,
    /// 最適化レベル (0–3)。デフォルト: 2。
    pub opt_level: u8,
    /// `!Pure` ステージをインライン展開するか。デフォルト: true。
    pub inline_pure_stages: bool,
    /// 生成物の出力ディレクトリ。デフォルト: "dist/"。
    pub output_dir: String,
}

impl Default for BuildConfig {
    fn default() -> Self {
        BuildConfig {
            target: "x86_64-unknown-linux-gnu".to_string(),
            opt_level: 2,
            inline_pure_stages: true,
            output_dir: "dist/".to_string(),
        }
    }
}
```

**`FavToml` 構造体** に追加（ロードマップの `FavConfig` 表記は `FavToml` の別名—実体は同一）:
```rust
/// Optional build configuration (v62.7.0).
pub build: Option<BuildConfig>,
```

**`parse_fav_toml`** に追加:
- ローカル変数: `let mut build_cfg: Option<BuildConfig> = None;`
- セクション検出: `if trimmed == "[build]" { section = "build"; continue; }`
- パースアーム:
  ```rust
  "build" => {
      let mut current = build_cfg.take().unwrap_or_default();
      if let Some((key, val)) = parse_kv(trimmed) {
          match key {
              "target"             => current.target             = val.to_string(),
              "opt_level"          => current.opt_level          = val.parse().unwrap_or(2),
              "inline_pure_stages" => current.inline_pure_stages = val == "true",
              "output_dir"         => current.output_dir         = val.to_string(),
              _ => {}
          }
      }
      build_cfg = Some(current);
  }
  ```
- `FavToml { ... }` の戻り値に `build: build_cfg,` を追加

**注意**: `FavToml` への `build` フィールド追加に伴い `parse_fav_toml` の最終 `FavToml { ... }` 構造体リテラルにも追加が必要。

### 2. `driver.rs` — `ResolvedBuildConfig` + `resolve_build_config` 追加

`cmd_build_docker` の後（または `cmd_build_aot_stats` の後）に配置。

**`ResolvedBuildConfig` 構造体**:
```rust
/// v62.7.0: CLI / fav.toml / default を統合したビルド設定。
pub struct ResolvedBuildConfig {
    pub target: String,
    pub opt_level: u8,
    pub inline_pure_stages: bool,
    pub output_dir: String,
}
```

**`resolve_build_config` 関数**:
```rust
/// CLI > fav.toml > default の優先順位でビルド設定を解決する（v62.7.0）。
pub fn resolve_build_config(
    cli_target: Option<&str>,
    cli_opt_level: Option<u8>,
    cli_inline_pure: Option<bool>,
    cli_output_dir: Option<&str>,
    toml: Option<&crate::toml::BuildConfig>,
) -> ResolvedBuildConfig {
    let base = toml.cloned().unwrap_or_default();
    ResolvedBuildConfig {
        target:             cli_target.map(|s| s.to_string()).unwrap_or(base.target),
        opt_level:          cli_opt_level.unwrap_or(base.opt_level),
        inline_pure_stages: cli_inline_pure.unwrap_or(base.inline_pure_stages),
        output_dir:         cli_output_dir.map(|s| s.to_string()).unwrap_or(base.output_dir),
    }
}
```

### 3. `main.rs` — `Some("build")` アームで `fav.toml [build]` を読み込む

`fav build` の実行パス（`if docker { ... } else if aot_stats { ... } else if link { ... }` の前）で
プロジェクトルートの `fav.toml` を読み込み、`resolve_build_config` を呼ぶスタブを追加。

実際の接続はフラグ数が多いため「ロードして resolve するが既存フラグを優先する」最小実装に留める。
（`--target` は既存変数、`--output`/`-o` は既存変数として残し、`resolve_build_config` への接続は v62.8.0 以降でも可能）

**最小実装**: `fav build` でカレントディレクトリの `fav.toml` を `FavToml::load` で読み込み、
`resolve_build_config` に渡して `ResolvedBuildConfig` を生成する処理を `if docker` ブランチ前に追加する。
生成された `resolved` 変数は現時点では `--aot-stats` / `--link` ブランチで未使用だが、警告を避けるため `let _ = resolved.target.as_str();` を置く。

### 4. `driver.rs` — `v62700_tests` 追加

`v62600_tests` の直前（ファイル先頭方向）に挿入。

**`build_toml_config_parsed`**:
- `parse_fav_toml_pub` を呼んでフル `fav.toml` 文字列をパース
- toml 内容:
  ```toml
  [rune]
  name = "mypipeline"
  version = "1.0.0"

  [build]
  target = "aarch64-unknown-linux-gnu"
  opt_level = 3
  inline_pure_stages = false
  output_dir = "out/"
  ```
- `t.build.as_ref().unwrap().target == "aarch64-unknown-linux-gnu"` を確認
- `t.build.as_ref().unwrap().opt_level == 3` を確認
- `t.build.as_ref().unwrap().inline_pure_stages == false` を確認
- `t.build.as_ref().unwrap().output_dir == "out/"` を確認

**`build_cli_overrides_toml`**:
- `BuildConfig { target: "x86_64-...", opt_level: 1, inline_pure_stages: false, output_dir: "build/" }` を作成
- CLI: `cli_target = Some("aarch64-unknown-linux-gnu")`, `cli_opt_level = Some(3u8)`
- `resolve_build_config(cli_target, cli_opt_level, None, None, Some(&toml_cfg))` を呼ぶ
- `resolved.target == "aarch64-unknown-linux-gnu"` — CLI が toml を上書き
- `resolved.opt_level == 3` — CLI が toml を上書き
- `resolved.inline_pure_stages == false` — toml の値がそのまま（CLI None → toml 値）
- `resolved.output_dir == "build/"` — toml の値がそのまま（CLI None → toml 値）

---

## 完了条件

- `cargo build` エラーなし
- `cargo test v62700` で 2 件 PASS
- `cargo test -j 8 -- --test-threads=8` で 3396 tests passed, 0 failed

---

## 非スコープ

- `--opt-level` / `--inline-pure-stages` / `--output-dir` 等の CLI フラグ追加（v62.9.0 以降）
- `fav.toml` の `[build]` 設定を `cmd_build_basic` / `cmd_build_link` の実際の動作に反映
- `site/content/docs/` の MDX ドキュメント — v62.9.0 でまとめて作成
- ビルド設定の検証（`opt_level > 3` 等のエラー処理）
