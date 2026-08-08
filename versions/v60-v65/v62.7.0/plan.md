# v62.7.0 Plan — `fav.toml` `[build]` セクション（AOT 設定）

Version: 62.7.0
Status: 未着手

---

## 実装順序

### Step 1: `toml.rs` — `BuildConfig` + `FavToml.build` + パース追加

1. `TenancyConfig` 定義の直後（`impl TenancyConfig` ブロックの後）に
   `// ── Build config (v62.7.0)` セクションと `BuildConfig` 構造体を追加。
2. `FavToml` 構造体の末尾フィールド（`tenancy: Option<TenancyConfig>` の後）に
   `pub build: Option<BuildConfig>,` を追加。
3. `parse_fav_toml` に:
   - ローカル変数 `let mut build_cfg: Option<BuildConfig> = None;` を追加（`tenancy_cfg` の後あたり）。
   - `[build]` セクション検出ブロックを追加（`[lint]` ブロックの近くに配置）。
   - `"build"` アームを `parse_fav_toml` の `match section {}` に追加。
   - 最終 `FavToml { ... }` に `build: build_cfg,` を追加。
4. `cargo build` でエラーなし確認。

### Step 2: `driver.rs` — `ResolvedBuildConfig` + `resolve_build_config` 追加

`cmd_build_docker` の直後に配置。
`cargo build` でエラーなし確認。

### Step 3: `main.rs` — `Some("build")` アームで `fav.toml [build]` を読み込む

`if docker { ... }` ブランチの直前に `fav.toml` 読み込み + `resolve_build_config` 呼び出しを追加。
`let _ = resolved.target.as_str();` でコンパイラ警告を抑制。
`cargo build` でエラーなし確認。

### Step 4: `driver.rs` — `v62700_tests` 追加

`v62600_tests` の直前（ファイル先頭方向）に挿入。
`cargo test v62700` で 2 件 PASS 確認。

### Step 5: 全テスト

`cargo test -j 8 -- --test-threads=8` で 3396 tests passed, 0 failed を確認。

### Step 6: ドキュメント更新

roadmap / current.md / CHANGELOG.md / tasks.md を更新。

---

## 設計メモ

### `FavToml.build` フィールド追加時の影響範囲

`FavToml` 構造体リテラルを直接構築している箇所が `parse_fav_toml` の末尾 1 箇所のみ（テスト含む）。
他コードは `FavToml::load` 経由で取得するため影響なし。
`parse_fav_toml` の戻り値リテラルに `build: build_cfg,` を追加するだけでコンパイルが通る。

### `resolve_build_config` の `Default` 利用

`BuildConfig::default()` で:
```
target = "x86_64-unknown-linux-gnu"
opt_level = 2
inline_pure_stages = true
output_dir = "dist/"
```
`toml` が `None` の場合（`fav.toml` なし、または `[build]` セクション未記載）は
`Default::default()` を使用。`toml` が `Some` の場合はその値をベースに CLI で上書き。

### `main.rs` の最小実装方針

既存の `Some("build")` アームはすでに多数のフラグを扱っている。
v62.7.0 では「`fav.toml` を読んで `ResolvedBuildConfig` を生成する」コードを追加するが、
生成結果を既存の `target` / `out_file` 変数に反映するのは非スコープ。
コンパイラ警告 (`dead_code`/`unused`) を避けるため `let _ = resolved;` または `let _ = resolved.target.as_str();` で抑制する。

### テスト競合なし

`build_toml_config_parsed` と `build_cli_overrides_toml` はいずれもファイル I/O 不要。
並列実行しても問題なし。

### ロードマップとの乖離

- `fav.toml [build]` 設定を `cmd_build_basic` / `cmd_build_link` の実際の動作（`--target` 引数等）に
  反映する処理は非スコープ。「読み込んで解決する」ところまでが v62.7.0 の対象。
- `--opt-level` / `--inline-pure-stages` / `--output-dir` CLI フラグは v62.9.0 以降で追加。
