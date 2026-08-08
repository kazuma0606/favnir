# v61.8.0 — `fav check --strict` モード（追加 lint の有効化）

## 概要

`lint.rs` に `LintConfig { strict: bool, perf: bool }` を追加し、`fav check --strict` / `fav lint --strict` フラグおよび `fav.toml` の `[lint] strict = true` で strict モードを有効化する。

strict モード有効時は W040 等の型ヒント系警告に `[strict]` タグを付与してレポートする。
将来の strict 専用ルール追加のための基盤インフラを提供する。

**W040 の動作について**: W040 は v61.7.0 時点で通常の `fav lint` に既に含まれている。
v61.8.0 では W040 を新規有効化するのではなく、strict モード時に `[strict]` タグを付与する変更のみ行う。

---

## 動機

v61.7.0 で W040（`type_hole_inferred`）を通常 lint に追加した。しかし大規模プロジェクトでは
型プレースホルダーの警告を CI ではなくオプトインで受けたい場合がある。
`--strict` フラグと `[lint] strict = true` を設けることで、厳密な型注釈ポリシーを
プロジェクト単位またはコマンドライン単位で選択できるようにする。

```bash
$ fav check --strict pipeline.fav
W040: type hole `_` in return type of `f` — consider making explicit [strict]
```

```toml
# fav.toml
[lint]
strict = true
```

---

## スコープ

### 変更ファイル一覧

| ファイル | 変更内容 |
|---|---|
| `fav/src/lint.rs` | `LintConfig { strict, perf }` 追加、`lint_program_with_config` 追加、W040 に `[strict]` タグ付与 |
| `fav/src/toml.rs` | `LintTomlConfig` に `strict: Option<bool>` フィールド追加、パース処理追加（`FavConfig` 側の変更は不要 — `FavConfig.lint: Option<LintTomlConfig>` は既存） |
| `fav/src/driver.rs` | `cmd_lint` に `strict: bool` 引数追加、lint 呼び出しを `lint_program_with_config` に切り替え |
| `fav/src/main.rs` | `fav lint --strict` フラグ追加（`fav check --strict` は既存）、`cmd_lint` 呼び出しに strict 引数追加 |
| `fav/src/driver.rs` | `v61800_tests` モジュール追加 |

---

## 実装詳細

### 1. lint.rs — `LintConfig` 追加

```rust
/// v61.8.0: 実行時 lint 設定。
#[derive(Debug, Clone, Default)]
pub struct LintConfig {
    /// `--strict` / `[lint] strict = true`: 型ヒント系警告に [strict] タグを付与する。
    pub strict: bool,
    /// 将来用: パフォーマンス系警告の有効化。
    pub perf: bool,
}
```

`lint_program_with_config` を追加:

```rust
/// v61.8.0: LintConfig を受け取る版。
pub fn lint_program_with_config(program: &Program, config: &LintConfig) -> Vec<LintError> {
    let mut errors = lint_program(program);
    if config.strict {
        // W040 に [strict] タグを付与
        for e in &mut errors {
            if e.code == "W040" {
                e.message = format!("{} [strict]", e.message);
            }
        }
    }
    errors
}
```

既存の `lint_program` はそのまま保持（後方互換）。

### 2. toml.rs — `LintTomlConfig.strict` 追加

既存の `LintTomlConfig` に `strict` フィールドを追加:

```rust
/// `[lint]` section of fav.toml (v12.10.0, updated v61.8.0).
#[derive(Debug, Clone)]
pub struct LintTomlConfig {
    pub warn_as_error: Option<Vec<String>>,
    pub allow: Option<Vec<String>>,
    /// v61.8.0: `strict = true` で LintConfig::strict を有効化。
    pub strict: Option<bool>,
}
```

`parse_lint_config` / `parse_fav_toml` でキー `"strict"` を `bool` としてパース。

### 3. driver.rs — `cmd_lint` に `strict` 引数追加

```rust
pub fn cmd_lint(file: Option<&str>, warn_only: bool, deny: bool, allow: Vec<String>, strict: bool) {
    // ...
    let config = LintConfig { strict, perf: false };
    let lint_errors = lint_program_with_config(&program, &config);
    // ...
}
```

### 4. main.rs — `fav lint --strict` フラグ追加

`fav lint` コマンドのフラグ解析に `"--strict"` を追加し、`cmd_lint` へ `strict` 引数を渡す。

`fav check --strict` は既存（L788）だが、lint 呼び出しも `lint_program_with_config` に切り替える。

---

## 完了条件

- **Rust テスト 2 件**（ベース 3374 + 2 = 3376 tests passed, 0 failed）
  - `check_strict_mode_w040_tagged` — strict モードで W040 のメッセージに `[strict]` が付くことを確認
  - `fav_toml_lint_strict` — `strict = true` が `LintTomlConfig.strict = Some(true)` としてパースされることを確認

---

## 注意事項

- `lint_program`（既存）は変更しない。`lint_program_with_config` を新規追加して呼び出し側を切り替える
- `fav check --strict` の既存動作（W006 → error 扱い、L4634-4641）は変更しない
- `fav check` 内には現時点で `lint_program` の直接呼び出しが存在しない。v61.8.0 では `cmd_check` に `lint_program_with_config` を**新規追加**し、W040 タグ付けを行う（既存呼び出しの切り替えではない）
- `fav lint` コマンドにも同様に `--strict` フラグを追加（main.rs の `fav lint` フラグ解析ループに `"--strict"` アームを追加）。現状 `--strict` なしでは unknown フラグがファイル名として扱われてしまう
- `LintTomlConfig.strict` は `Option<bool>`（未指定時は None → false 扱い）
- `toml.rs` の `parse_lint_config` という独立関数が存在しない場合は `parse_fav_toml` 内の `"lint"` match アームで `strict` キーをパースする
- `perf` フィールドは v61.8.0 では常に `false`（将来拡張用）
- テスト名はロードマップ記載の `check_strict_mode_enables_w040` から `check_strict_mode_w40_tagged` に変更（実際の動作をより正確に表現）。ロードマップも更新済み
