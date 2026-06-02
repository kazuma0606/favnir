# Favnir v9.9.0 Spec

Date: 2026-06-02
Theme: `fav profile` + `fav watch` — パイプライン実行時間計測 + ファイル監視

---

## 概要

v9.9.0 は開発体験（DX）改善に特化した 2 機能を追加する。

1. **`fav profile`** — stage 呼び出しに計測コードを自動挿入し、実行時間テーブルを表示
2. **`fav watch`** — ファイルタイムスタンプをポーリングして変更検出・自動再実行

どちらも **Rust 変更なし**。`compiler.fav` の AST 変換と `cli.fav` のコマンドループで実装する。

---

## 1. fav profile

### ユーザーインターフェース

```
fav profile pipeline.fav
```

オプション:

```
fav profile --out table pipeline.fav     # テーブル形式（デフォルト）
fav profile --out json pipeline.fav      # JSON 形式
```

### 出力例（テーブル）

```
Stage              Time (ms)    %      Effects
─────────────────────────────────────────────
FetchOrders          1,203      58%    [!Llm]
ValidateSchema          87       4%    []
EnrichWithCRM          801      38%    [!Http]
─────────────────────────────────────────────
Total                2,091     100%
```

### 実装方針

`compiler.fav` に `--profile` フラグ対応の計測コード挿入を追加する。

**AST 変換**: `instrument_stage_call(name: String, expr: Expr) -> Expr`

stage 呼び出し `stage_name(input)` を以下の形に変換する:

```
let t0 = Env.now_ms()
let result = stage_name(input)
let t1 = Env.now_ms()
Env.profile_record(stage_name_str, t1 - t0)
result
```

- `--profile` フラグが立っているときのみ変換（通常ビルドへのゼロオーバーヘッド）
- `Env.now_ms()` は既存 primitive (`IO.now_ms_raw` をラップ)
- `Env.profile_record(name, ms)` は新規 primitive — in-memory テーブルに追記
- 実行後 `Env.profile_dump()` で集計結果を取得

### 新規 Rust primitive（vm.rs）

| primitive | 型 | 説明 |
|---|---|---|
| `Env.profile_record_raw` | `(String, Int) -> Unit` | ステージ名・ms を記録 |
| `Env.profile_dump_raw` | `() -> String` | JSON 文字列で返す |

VM スレッドローカルな Vec に追記。実行ごとに初期化。

### compiler.fav 変更

- `compile_stage_calls(prog: Program, profile: Bool) -> Program` を追加
  — `profile = true` のとき stage 呼び出し Expr を `instrument_stage_call` で変換
- `compile_source_profiled(src: String) -> Result<List<Int>, String>` を追加
  — `compile_source` と同じだが変換ステップを追加
- `public fn doc_source` のすぐ後に実装

### Rust driver 変更（driver.rs）

- `cmd_profile(path: &str, out_fmt: &str)` を追加
  — `compile_profiled_str` で計測バイトコードをコンパイル
  — 実行後 `Env.profile_dump_raw` で結果取得 → テーブル / JSON 表示

### cli.fav 変更

```favnir
| CmdProfile(String, String)   // (path, out_fmt)
```

- `parse_profile_cmd` / `run_profile` を追加
- `run_help` に profile 説明を追加

---

## 2. fav watch

### ユーザーインターフェース

```
fav watch pipeline.fav                # fav run（デフォルト）
fav watch --check pipeline.fav        # fav check のみ
fav watch --test pipeline.fav         # fav test（テストファイル）
```

### 動作

1. 起動時に対象ファイルの mtime を記録
2. 500ms ごとにポーリング
3. mtime が変わったら自動再実行
4. エラーでも停止せず次のポーリングを続ける
5. Ctrl+C で終了（`IO.exit_raw`）

### 出力例

```
Watching pipeline.fav (Ctrl+C to stop)
[12:34:01] Running...
ok: pipeline.fav
[12:34:17] File changed, re-running...
error: type mismatch at line 42
[12:34:45] File changed, re-running...
ok: pipeline.fav
```

### 実装方針

**Rust 変更なし**。以下の既存 / 新規 primitive のみ使用:

| primitive | 型 | 備考 |
|---|---|---|
| `IO.file_mtime_raw` | `String -> Result<Int, String>` | ファイル mtime (ms) を返す |
| `IO.sleep_ms_raw` | `Int -> Unit` | ミリ秒スリープ |
| `IO.now_ms_raw` | `() -> Int` | 現在時刻 ms（既存） |
| `IO.read_file_raw` | 既存 | — |

`IO.file_mtime_raw` と `IO.sleep_ms_raw` が未実装なら vm.rs に追加する（Rust 2 行程度）。

### cli.fav 変更

```favnir
| CmdWatch(String, String)   // (path, mode)  mode = "run" | "check" | "test"
```

```favnir
fn watch_loop(path: String, mode: String, last_mtime: Int) -> Unit !IO {
    bind _ <- IO.sleep_ms_raw(500)
    match IO.file_mtime_raw(path) {
        Err(_)  => watch_loop(path, mode, last_mtime)
        Ok(mtime) => {
            if mtime != last_mtime {
                bind ts <- format_timestamp()
                bind _ <- IO.println(String.concat(ts, " File changed, re-running..."))
                bind _ <- run_watch_action(path, mode)
                watch_loop(path, mode, mtime)
            } else {
                watch_loop(path, mode, last_mtime)
            }
        }
    }
}
```

---

## 完了条件

| 条件 | 確認 |
|---|---|
| `fav profile file.fav` で stage 別実行時間テーブルを表示 | |
| `--profile` なし（通常ビルド）にパフォーマンス影響なし | |
| `fav watch file.fav` がファイル変更を検出して自動再実行 | |
| エラー発生後も watch が停止しない | |
| `cargo test v990` — 3 件以上通過 | |
| `cargo test checker_fav_wire_self_check` 通過 | |
| `cargo test bootstrap` 維持 | |

---

## スコープ外（v10.0.0 以降）

- `fav watch` の inotify / FSEvents 対応（ポーリング → イベント駆動）
- `fav profile --flamegraph` フレームグラフ出力
- プロファイル結果の永続化（`.fav-profile.json`）
- `fav bench`（繰り返し実行・統計）
