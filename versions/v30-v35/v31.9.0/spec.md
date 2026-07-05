# v31.9.0 — Spec: ドッグフード修正 vol.2

## 概要

v31.1〜v31.8 の実装中に発見した 2 件のバグを修正する。
- REPL が空行を履歴に追加してしまう（edge case）
- `fav check --all` でファイルがゼロ件のとき無言で終了する

どちらも小さな修正だが、実際に使う際に混乱を招くため修正対象とする。

---

## 修正 1: REPL 空行の履歴スキップ

### 現状

`ReplSession::add_history`（driver.rs:12020）は無条件に `line` を `history` に追加する。

```rust
// driver.rs:12020
fn add_history(&mut self, line: &str) {
    self.history.push(line.to_string());
    if self.history.len() > 100 {
        self.history.remove(0);
    }
}
```

ユーザーが Enter キーを空行で押したとき（あるいは空白のみの行を入力したとき）、
履歴に空エントリが追加される。`:history` コマンドで空行が並ぶのは見づらい。

### 修正方針

`add_history` の先頭で `line.trim().is_empty()` を確認し、空または空白のみの行はスキップする。

```rust
fn add_history(&mut self, line: &str) {
    if line.trim().is_empty() {
        return;
    }
    self.history.push(line.to_string());
    if self.history.len() > 100 {
        self.history.remove(0);
    }
}
```

---

## 修正 2: `fav check --all` — ファイルゼロ件メッセージ

### 現状

`check_all_files`（driver.rs:4148〜）は `collect_fav_files_recursive(dir)` の結果がゼロ件のとき
ループに入らず `0` を返すだけで、非 JSON モードでは何もメッセージを出力しない。

```rust
// driver.rs:4148〜4154（抜粋）
pub(crate) fn check_all_files(dir: &std::path::Path, json: bool) -> usize {
    // ... (root / resolver 構築)
    let files = collect_fav_files_recursive(dir);  // 4153 行
    if json {   // 4154 行 — ゼロ件でも JSON モードは空配列を出力する（正常）
        // ...  ← 変更なし
    } else {
        // ... ← ゼロ件のとき何も出力せず 0 を返す（バグ）
    }
}
```

`fav check --all` を fav.toml のない場所や src/ が空のプロジェクトで実行すると、
コマンドが成功終了 (`exit 0`) するが何も表示されないため、ユーザーは何が起きたか分からない。

### 修正方針

`files.is_empty()` チェックを追加する。

- **非 JSON モード**: `eprintln!("no .fav files found in `{}`", dir.display())` を出力して `0` を返す。
- **JSON モード**: 空配列 `[]` を出力して `0` を返す（変更なし、既に正常）。

```rust
let files = collect_fav_files_recursive(dir);  // driver.rs:4153
if files.is_empty() && !json {
    eprintln!("no .fav files found in `{}`", dir.display());
    return 0;
}
if json {   // ← 元の 4154 行、位置変更なし
```

---

## テスト方針

### テスト 1（バージョン確認）

```rust
fn cargo_toml_version_is_31_9_0() {
    let src = include_str!("../../Cargo.toml");
    assert!(src.contains("31.9.0"), "Cargo.toml must contain '31.9.0'");
}
```

### テスト 2（ベンチマーク存在確認）

```rust
fn benchmark_v31_9_0_exists() {
    let src = include_str!("../../benchmarks/v31.9.0.json");
    assert!(src.contains("31.9.0"), "benchmarks/v31.9.0.json must contain '31.9.0'");
}
```

### テスト 3（REPL 空行スキップ確認）

`ReplSession` は driver.rs 内で定義された struct。`history` フィールドはプライベートだが、
`mod v319000_tests { use super::*; }` は driver.rs の子モジュールになるため
Rust のプライバシー規則（子モジュールは親の private にアクセス可能）により直接参照できる。

```rust
fn repl_add_history_skips_blank_lines() {
    let mut state = ReplSession::new();
    state.add_history("");
    state.add_history("   ");
    state.add_history("\t");
    assert!(state.history.is_empty(), "blank lines should not be added to history");
    state.add_history("List.length([1,2,3])");
    assert_eq!(state.history.len(), 1);
}
```

---

## 完了条件

- `Cargo.toml` version = `"31.9.0"`
- `cargo_toml_version_is_31_8_0` が空スタブになっていること
- `ReplSession::add_history` が空行・空白行をスキップする
- `check_all_files` が非 JSON モードでファイルゼロ件のとき警告を出力する
- `cargo test --bin fav v319000` — 3/3 PASS
- `cargo test` — 全件 PASS
- `CHANGELOG.md` に `[v31.9.0]` セクション
- `benchmarks/v31.9.0.json` 存在かつ `tests_passed` が実測値
- `versions/current.md` を v31.9.0 に更新
