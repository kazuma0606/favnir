# v31.6.0 仕様書 — fav test --watch

## 概要

`fav test --watch <dir>` でファイル変更を検知してテストを自動再実行する。
`cmd_watch()` は既に `cmd = "test"` をサポートしているため、
`main.rs` の `fav test` 引数パーサに `--watch` フラグを追加して配線するのみ。

---

## 背景

ロードマップ v31.6 より:

```bash
$ fav test --watch src/
[watch] テストを監視中... (Ctrl+C で終了)
[12:34:01] 変更検知: src/validators.fav
[12:34:01] テスト実行中...
[12:34:02] PASS: validate_row_ok (0.3ms)
[12:34:02] 2/2 テスト通過
```

---

## 既存実装の確認事項

| 項目 | 状態 |
|---|---|
| `cmd_watch(file, "test", dirs, debounce_ms)` | **実装済み** (`driver.rs:5127`) — `"test"` は有効な cmd 値 |
| `fav watch --cmd test <file>` | **動作確認済み** — `main.rs` の `watch` サブコマンドで `--cmd test` を渡せる |
| `fav test --watch` フラグ | **未実装** — `main.rs` の `test` 引数パーサに `--watch` が存在しない |
| `collect_watch_paths_from_dir()` | **実装済み** (`driver.rs:4901`) |

---

## スコープ

### IN SCOPE

- `fav/Cargo.toml` — version `31.5.0` → `31.6.0`
- `fav/src/driver.rs` — `cargo_toml_version_is_31_5_0` をスタブ化
- `fav/src/main.rs` — `Some("test")` パーサに `--watch` フラグを追加
- `fav/src/main.rs` — `--watch` 時に `cmd_watch(file, "test", &dirs, 80)` を呼ぶ
- `fav/src/driver.rs` — `v316000_tests`（3 件）追加（`use super::*` あり）
- `CHANGELOG.md` — `[v31.6.0]` セクション追加
- `benchmarks/v31.6.0.json` 新規作成
- `versions/current.md` — v31.6.0 に更新

### OUT OF SCOPE

- `--watch` フラグへの `--debounce` サポート（デフォルト 80ms を使用）
- `--watch` フラグへの `--dir` 追加オプション（`<file>` 引数のディレクトリを使用）
- `[watch]` 出力フォーマット変更（「テストを監視中...」等の日本語メッセージ）— 既存の `cmd_watch` メッセージをそのまま使用
- site/ MDX 更新（`fav test` の既存 MDX への `--watch` 追記は v32.x ドキュメント整備スプリントで対応）

---

## 実装詳細

### main.rs — `--watch` フラグ追加

`Some("test")` の引数パースループ内に追加:

```rust
"--watch" => {
    watch_mode = true;
    i += 1;
}
```

変数宣言:
```rust
let mut watch_mode = false;
let mut watch_dirs: Vec<String> = Vec::new();
```

`--watch` 時のディレクトリ収集（`file` 引数がディレクトリなら `extra_dirs` に移し `file` は `None` にする）:

```rust
if watch_mode {
    let file_for_watch: Option<&str>;
    if let Some(ref f) = file {
        let path = std::path::Path::new(f);
        if path.is_dir() {
            watch_dirs.push(f.clone());
            file_for_watch = None;   // ディレクトリは extra_dirs に委ねる
        } else {
            file_for_watch = Some(f.as_str());
        }
    } else {
        file_for_watch = None;
    }
    let dir_refs: Vec<&str> = watch_dirs.iter().map(|s| s.as_str()).collect();
    cmd_watch(file_for_watch, "test", &dir_refs, 80);
    return;
}
```

> `fav test --watch src/` では `"src/"` がディレクトリのため `file_for_watch = None`、`extra_dirs = &["src/"]` となり、
> `cmd_watch` 内部の `collect_watch_paths_from_dir("src/")` が `.fav` を再帰収集する。
> `file_for_watch = Some(f)` にしてしまうと `collect_watch_paths(Some("src/"))` が
> `PathBuf::from("src/")` を `.fav` ファイルとして扱い監視ディレクトリ解決が壊れるため NG。
>
> `watch_dirs` は `Vec<String>` を使う（将来の複数 `--watch-dir` 拡張への準備）。
> v31.6.0 では位置引数 1 つのみのため最大 1 件だが、型は `Vec` のままにしておく。
>
> `cmd_test(...)` の呼び出しより前に配置すること。

**`--watch` フラグの引数消費フロー（注意点）:**
`"--watch"` アームは `i += 1` のみ（値なし）。続く `"src/"` は次のループで `other =>` アームに入り `file = Some("src/")` になる。誤って `i += 2` にすると `src/` が読み飛ばされるため注意する。

---

## テスト設計（v316000_tests — 3 件）

| # | テスト名 | 確認内容 |
|---|---------|----------|
| 1 | `cargo_toml_version_is_31_6_0` | `Cargo.toml` に `version = "31.6.0"` |
| 2 | `benchmark_v31_6_0_exists` | `benchmarks/v31.6.0.json` に `"31.6.0"` |
| 3 | `collect_watch_paths_finds_fav_files` | `collect_watch_paths_from_dir` が一時ディレクトリから `.fav` ファイルを収集できること（`cmd_watch` が `"test"` を受け付けることは `driver.rs` 行 5128 の `matches!(cmd, "check" \| "test" \| "run")` で保証済み）|

> テスト#3 は `collect_watch_paths_from_dir()` を直接呼び出し、一時ディレクトリに `.fav` ファイルを配置して収集されることを確認する。
> `cmd_watch` 自体は `process::exit(1)` を含むため直接呼び出しテストは書かない。
> `v316000_tests` は `use super::*` あり。

---

## 完了条件

- `Cargo.toml` version = `"31.6.0"`
- `fav test --watch src/` が `cmd_watch(_, "test", _, 80)` を呼び出す
- `fav test <file>` の既存動作が変わらないこと（`--watch` なし時は `cmd_test` が呼ばれる）
- `cargo test v316000` — 3/3 PASS
- `cargo test` — 全件 PASS（0 failures）
- `CHANGELOG.md` に `[v31.6.0]` セクション
- `benchmarks/v31.6.0.json` 存在かつ `tests_passed` が実測値
- `versions/current.md` を v31.6.0 に更新
- `tasks.md` が COMPLETE
