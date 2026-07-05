# v30.9.0 仕様書 — ドッグフード発見修正

## 概要

v30.3〜v30.6 のドッグフードで発見した 4 つの問題を修正する。

---

## 背景

ロードマップ v30.9 より:

> v30.5 のサンプル実装・v30.3 の E2E 検証で見つかった問題の修正版。

ドッグフードで判明した具体的な問題:

### 問題 1: `[project]` セクションが `toml.rs` に未対応

`fav.toml` のすべてのユーザー向けテンプレート（`script` / `pipeline` / `postgres-etl` など）は
`[project]` セクションを使う。しかし `parse_fav_toml` は `[rune]` セクションのみを認識する。

結果: `src = "src"` が無視され `src_dir = "."` （プロジェクトルート）になる。

影響:
- `fav check`（引数なし）: プロジェクトルート以下の **すべての** `.fav` ファイルをスキャンする（`tests/` 含む）
- `fav test`（引数なし）: `src_dir = "."` を起点にスキャンするため `tests/` との重複が発生 → v30.6 の canonical dedup で回避済みだが根本原因は未修正

### 問題 2: 非 rune import が `src_dir` 相対で解決される

`load_all_items` 内の `ImportDecl { is_rune: false }` の解決:

```rust
// 現状（src_dir 相対）
src_dir.join(import_name).with_extension("fav")
```

テンプレートは `import src/types`（プロジェクトルート相対）と書く。
- `src_dir = "."` のとき: `./src/types.fav` = `src/types.fav` ✓（偶然正しい）
- `src_dir = "src"` のとき（問題 1 修正後）: `src/src/types.fav` ✗（二重パス）

問題 1 の修正と同時に本修正が必要。

### 問題 3: `fav test` の `false` 返却時メッセージが不明瞭

```
FAIL  validate_amount (tests/pipeline_test.fav)  (0ms)
      test returned false
```

`test returned false` では何が失敗したか分からない。`assert_eq!` / `assert!` を使えば
詳細なエラーメッセージが出ることをユーザーに伝えるヒントがない。

### 問題 4: `fav new`（引数なし）に `fav new --list` へのヒントがない

```
$ fav new
error: new requires a project name
```

v30.8.0 で `fav new --list` を追加したが、ユーザーが `fav new` だけ叩いたときに
`--list` の存在を知るすべがない。

---

## スコープ

### IN SCOPE

- `fav/src/toml.rs` — `[project]` セクション認識（`section = "project"` 設定）
- `fav/src/driver.rs` — `load_all_items` 非 rune `ImportDecl` 解決を `root` ベースに変更
- `fav/src/driver.rs` — `fav test` の `Ok(Bool(false))` 時にヒント文字列を追加
- `fav/src/main.rs` — `fav new` 引数なし時のエラーに `fav new --list` ヒントを追加
- Rust テスト（`v309000_tests` — 3 件）

### OUT OF SCOPE

- `fav check` / `fav test` / `fav run` の統合 E2E テスト（Rust テストでは実行しない）
- `use` 宣言（`program.uses`）の解決ロジック変更（`ImportDecl` のみ対象）
- rune import（`is_rune: true`）の解決ロジック変更（変更なし）
- site/ MDX 更新

---

## 実装仕様

### Fix 1: `toml.rs` — `[project]` セクション認識

`parse_fav_toml` 関数内の catch-all `if trimmed.starts_with('[')` の **直前** に追加:

```rust
if trimmed == "[project]" {
    section = "project";
    continue;
}
```

`match section` ブロックに `"project"` アームを追加（`"rune"` と同一内容）:

```rust
"project" => {
    if let Some((key, val)) = parse_kv(trimmed) {
        match key {
            "name" => name = val.to_string(),
            "version" => version = val.to_string(),
            "description" => description = Some(val.to_string()),
            "license" => license = Some(val.to_string()),
            "authors" => {
                authors = val.split(',').map(|s| s.trim().to_string()).collect()
            }
            "src" => src = val.to_string(),
            _ => {}
        }
    }
}
```

> `edition` フィールドは `FavToml` 構造体に存在しないため `_ => {}` で無視する。

### Fix 2: `driver.rs` — 非 rune import の `root` ベース解決

`load_all_items` 内の `load_rec` ヘルパー、`ImportDecl { is_rune: false }` のアーム:

```rust
// 変更前（src_dir 相対）
src_dir.join(import_name).with_extension("fav")

// 変更後（root ベース）
root.join(import_name).with_extension("fav")
```

> `import runes/postgres`（`is_rune: true`）のアームは変更しない。
>
> Fix 1 + Fix 2 の組み合わせにより、`src_dir` の値に関わらず `root.join(import_name)` でプロジェクトルート基準の解決が得られる。
> `import src/types` → `<root>/src/types.fav` と一貫して解決される。

### Fix 3: `driver.rs` — `fav test` false 返却時ヒント

`cmd_test` 内の `Ok(crate::value::Value::Bool(false))` アーム:

```rust
// 変更前
error_msg: Some("test returned false".into()),

// 変更後
error_msg: Some("test returned false\n  hint: use assert_eq! or assert! for descriptive error messages".into()),
```

### Fix 4: `main.rs` — `fav new` 引数なし時ヒント

`Some("new")` ハンドラ内の `args.get(2).unwrap_or_else` クロージャ:

```rust
// 変更前
let name = args.get(2).unwrap_or_else(|| {
    eprintln!("error: new requires a project name");
    process::exit(1);
});

// 変更後
let name = args.get(2).unwrap_or_else(|| {
    eprintln!("error: new requires a project name");
    eprintln!("  hint: run 'fav new --list' to see available templates");
    process::exit(1);
});
```

---

## テスト設計（v309000_tests — 3 件）

| # | テスト名 | 確認内容 |
|---|---------|----------|
| 1 | `cargo_toml_version_is_30_9_0` | `Cargo.toml` に `version = "30.9.0"` |
| 2 | `project_section_sets_src_dir` | `parse_fav_toml_pub("[project]\nname=\"x\"\nsrc=\"src\"\n")` → `.src == "src"` |
| 3 | `benchmark_v30_9_0_exists` | `benchmarks/v30.9.0.json` に `"30.9.0"` |

> テスト 2 は `crate::toml::parse_fav_toml_pub(...)` をフルパスで参照する（`pub fn` として `toml.rs` 行 330 に既存）。
> `use super::*` は不要 — フルパス参照のみで動作する。`v309000_tests` に `use super::*` は追加しない。
>
> 後方互換性（Fix 1 + Fix 2 の組み合わせ）:
> Fix 1 により `src_dir = "src"` になっても、Fix 2 で非 rune import を `root.join(import_name)` で解決するため
> `import src/types` は常に `<root>/src/types.fav` と解決される。`src_dir` の値に依存しない。

---

## 完了条件

- `Cargo.toml` version = `"30.9.0"`
- `toml.rs` が `[project]` セクションを認識し `src` フィールドを正しくパースする
- `load_all_items` の非 rune import が `root` ベースで解決される
- `fav test` の `false` 返却時に hint が表示される
- `fav new`（引数なし）に `fav new --list` ヒントが表示される
- `cargo test v309000` — 3/3 PASS
- `cargo test` — 全件 PASS（0 failures）
- `CHANGELOG.md` に `[v30.9.0]` セクション
- `benchmarks/v30.9.0.json` 存在
- `versions/current.md` を v30.9.0 に更新
- `tasks.md` が COMPLETE
