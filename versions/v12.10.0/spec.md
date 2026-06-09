# Favnir v12.10.0 仕様書

Date: 2026-06-09
Theme: 全エラーに `help:` + `fav check --strict` + `fav lint --deny-warnings`

---

## 概要

v12.9.0 で CI の構造的ギャップ（`fav test self/*.fav` 未実行・Postgres 統合テスト未整備）を埋めた。
v12.10.0 は「コンパイラの出力品質」と「CI の品質ゲート」を Rust と同水準に揃えるフェーズ。

### 問題

1. **エラーメッセージに `help:` がなく次の行動が不明**
   Rust コンパイラが AI に強い理由は `help:` / `note:` で次の行動が明示されているから。
   現状 Favnir のエラーは E0018・W006 以外に `help:` がない。
   AI がコンパイラの出力だけで自己修正できない。

2. **`fav check --strict` がない**
   `fav check` はエラーで exit 1 するが、警告（W006 等）は exit 0 のまま通過する。
   `--strict` フラグで警告もエラーとして扱い exit 1 できるようにする必要がある。

3. **`fav lint --deny-warnings` がない**
   `fav lint` のデフォルトは警告で exit 1 するが、CI スクリプトに明示的フラグがなく意図が不明。
   `--deny-warnings` フラグを追加して CI スクリプトを明示的にする。

4. **`fav.toml [lint]` セクションで細粒度制御ができない**
   プロジェクトごとに特定警告をエラー扱い・または抑制する手段がない。

---

## 機能 1: 全エラー・警告への `help:` 追加

### 実装方針

`TypeError` / `TypeWarning` 構造体に `help: Vec<&'static str>` フィールドを追加するのは
変更箇所が多く リスクが高い。代わりに**静的マップ**アプローチをとる:

```rust
// driver.rs に追加
fn get_help_text(code: &str) -> &'static [&'static str] {
    match code {
        "E0001" => &["変数名のスペルを確認してください",
                     "use `bind x <-` to introduce a new binding"],
        "E0007" => &["Primitive 関数の一覧は `fav doc --builtins` で確認できます"],
        "E0018" => &["別の名前を使ってください: `bind x2 <- ...`",
                     "値を捨てる場合は `bind _ <- ...`"],
        "W006"  => &["`chain _ <- ...` を使うとエラーを自動伝播できます",
                     "明示的に処理する: `match expr { Ok(_) => ... Err(e) => ... }`"],
        _ => &[],
    }
}
```

`cmd_check` の出力ループに `get_help_text(code)` を差し込み、
help テキストがある場合は各行を `  = help: ...` 形式で追記する。

同様に `cmd_lint` 出力にも `get_help_text(lint.code)` を追加。

### 対象エラーコード（help 追加）

| コード | help 内容 |
|---|---|
| E0001 | 変数名のスペル確認 + `bind` で導入する方法 |
| E0007 | `fav doc --builtins` 参照 |
| E0008 | 引数の個数確認 |
| E0009 | 戻り型と本体型の確認 + 型注釈の修正方法 |
| E0013 | `where` バリデータの修正方法 |
| E0014 | `interface` に `fn` を追加する方法 |
| E0015 | `impl` に実装が必要な `fn` の確認 |
| E0018 | 別名を使う + `bind _` で捨てる |
| W001 | 未使用変数は `_` プレフィックスで抑制 |
| W004 | `chain` で代替する方法 |
| W006 | `chain _ <-` または `match` で明示処理 |
| W007 | Result 戻り値を必ず処理する |

---

## 機能 2: `fav check --strict`

### 動作

```bash
fav check --strict pipeline.fav
```

- 通常の型エラー（exit 1）に加えて、W006 等の警告も exit 1 に昇格
- `--strict` 時は警告を `warning[W006]` ではなく `error[W006]` として表示
- `--json` と組み合わせ可能: `fav check --strict --json` で errors 配列に警告も含む

### 実装

`cmd_check(file, no_warn, legacy_check, json, show_types)` に `strict: bool` を追加:

```rust
pub fn cmd_check(file, no_warn, legacy_check, json, show_types, strict: bool)
```

W006 は現状 `--show-types` でのみ検出。`--strict` 時は `--show-types` 相当の警告検出も行い
exit code に反映する。

---

## 機能 3: `fav lint --deny-warnings`

### 動作

```bash
fav lint --deny-warnings self/compiler.fav
```

現状 `fav lint` のデフォルトは警告で exit 1（`warn_only=false`）。
`--warn-only` を指定すると exit 0。

`--deny-warnings` は `warn_only=false` と同義だが CI スクリプトでの意図を明示する専用フラグ。

### CI 更新

`.github/workflows/ci.yml` の `Self-lint` ステップに `--deny-warnings` を追加:

```yaml
- name: Self-lint (fav lint)
  working-directory: fav
  run: |
    ./target/debug/fav lint --deny-warnings self/compiler.fav
    ./target/debug/fav lint --deny-warnings self/checker.fav
```

---

## 機能 4: `fav.toml [lint]` セクション

### 設定例

```toml
[lint]
warn_as_error = ["W006", "W007"]   # 特定警告をエラー化
allow         = ["W004"]            # 特定警告を抑制
```

### 動作

- `warn_as_error` に含まれるコードが発生した場合、`fav lint` が exit 1
- `allow` に含まれるコードは lint 出力から除外
- `fav.toml` が見つからない場合はデフォルト動作（現状と同じ）

### 実装

`FavToml` に `lint: Option<LintTomlConfig>` フィールドを追加:

```rust
#[derive(serde::Deserialize, Default)]
pub struct LintTomlConfig {
    pub warn_as_error: Option<Vec<String>>,
    pub allow:         Option<Vec<String>>,
}
```

`cmd_lint` で `fav.toml` を読み、`allow` リストに含まれるコードをフィルタし、
`warn_as_error` リストに含まれる警告は exit 1 に昇格。

---

## テストケース

| テスト名 | 内容 |
|---|---|
| `check_strict_w006_exits_1` | `--strict` + W006 あるコードで exit 1 になること |
| `check_strict_no_warning_exits_0` | `--strict` + 警告なしで exit 0 |
| `lint_deny_warnings_exits_1` | `--deny-warnings` + 警告あるコードで exit 1 |
| `lint_allow_suppresses_code` | `[lint] allow = ["W001"]` で W001 が表示されないこと |
| `lint_warn_as_error_exits_1` | `[lint] warn_as_error = ["W006"]` で W006 が exit 1 |
| `help_text_e0001` | E0001 エラーに `= help:` 行が含まれること |
| `help_text_w006` | W006 警告に `= help:` 行が含まれること |
| `version_is_12_10_0` | `CARGO_PKG_VERSION == "12.10.0"` |

---

## 完了条件

- [ ] 全対象エラー・警告の出力に `= help:` 行が追加される
- [ ] `fav check --strict` が W006 を exit 1 に昇格する
- [ ] `fav lint --deny-warnings` が明示的に exit 1 フラグとして機能する
- [ ] `fav.toml [lint]` の `warn_as_error` / `allow` が動作する
- [ ] CI `Self-lint` に `--deny-warnings` が追加される
- [ ] 全テストケース通過
- [ ] `cargo test` 全通過

---

## 非目標

- checker.fav / compiler.fav のソースコードを `--strict` 対応に修正
  （self/ は現状 lint ok なので変更不要）
- `fav check --strict` を CI に追加（v13.0.0 以降の判断）
- E0002〜E0006 / E0010〜E0012 / E0016〜E0017 への help 追加
  （利用頻度が低いため、主要なエラーに絞って追加する）
