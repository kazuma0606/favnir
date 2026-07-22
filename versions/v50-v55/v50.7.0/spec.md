# Spec: v50.7.0 — `fav run --trace` / `fav run --watch` 強化

## 概要

`fav run --trace` の出力を stage 単位の構造化ログに統一し、
`--watch <var.field>` フラグを新規追加して VM の変数束縛フックに照合ロジックを挿入する。

既存の `fav run --debug`（DAP）・`fav dap` とは独立した軽量導線として位置づける。

---

## 背景・既存実装

### 既存の trace 実装（v50.7.0 前）

- `SeqStageEnter` opcode: `[TRACE] stage X: enter` を emit（stage 開始前）
- `SeqStageCheck` opcode: `[TRACE] stage X: exit Ok(...)` または `[TRACE] stage X: exit Err(...)` を emit（stage 終了後）
- `VM.trace_lines: Vec<String>` に全 trace を蓄積
- `set_verbose_level` / `VERBOSE_LEVEL` スレッドローカルで有効/無効切替
- `VM::run_with_trace(artifact, fn_idx, args, db_path, source_file)` → `Ok((Value, Vec<EmitVal>, Vec<String>))`（第3要素が trace_lines）

### 問題点

- `[TRACE] stage X: exit Ok(...)` は非構造化テキスト → ログ集約ツールでのパースが困難
- `in=` フィールド（stage 入力値）がない → どの値が何に変換されたか不明
- `--watch` フラグが存在しない → 特定フィールドの変化追跡ができない

---

## 変更仕様

### 1. 構造化 trace ログ (`SeqStageCheck` 強化)

**出力形式（変更後）:**

```
[trace] stage=Parse    out=Order{id:1,amount:99.0}
[trace] stage=Validate out=Ok(Order{id:1})
```

**仕様詳細:**
- `[TRACE]` → `[trace]` に小文字統一
- `stage=NAME` フォーマット（`=` で接続）
- `out=VALUE` フォーマット（`truncate_for_trace` 既存関数を使用）
- Ok 経路のみ構造化（Err 経路は既存 `[TRACE] stage X: exit Err(...)` 維持）
- `in=` フィールドは本バージョンでは **未実装**（`SeqStageEnter` 時点でスタック上に値がない制約による）

**変更箇所:** `fav/src/backend/vm.rs` — `SeqStageCheck` ハンドラの Ok 分岐（`if vlevel > 0` 内の trace_emit 呼び出し行）

### 2. `--watch <var.field>` フラグ

**出力形式:**

```
[watch] order.amount: — → 99.0   (stage: Parse)
[watch] order.status: — → "ok"   (stage: Validate)
```

**仕様詳細:**
- `--watch order.amount` → `order.amount` を監視対象として登録
- stage の出力値がレコード型（`VMValue::Record`）の場合、最後のドット以降のフィールド名（`amount`）が存在すれば `[watch]` ログを emit
- 前回値追跡は将来対応、初回は `—`（U+2014 EM DASH）固定
- 複数 `--watch` フラグをサポート
- スレッドローカル `WATCH_FIELDS: RefCell<Vec<String>>` で watch 対象を伝達（`VERBOSE_LEVEL: Cell<u8>` と同パターン。`Vec<String>` は `Copy` 非対応のため `Cell` ではなく `RefCell` を使用）
- **`--watch` は `verbose_level` とは独立して動作する**: `set_verbose_level(0)` の状態でも watch フックは有効（`[trace]` ログは混入しない）
- CLI レベルの `--watch` 解析は本バージョン対象外（テスト用 API のみ実装）

**変更箇所:**
- `fav/src/backend/vm.rs`:
  - `WATCH_FIELDS: RefCell<Vec<String>>` スレッドローカル追加
  - `pub fn set_watch_fields(fields: Vec<String>)` 追加
  - `fn watch_fields() -> Vec<String>` 追加
  - `SeqStageCheck` Ok 分岐: `uvm` の生成条件に `|| !watch_fields().is_empty()` を追加
  - `SeqStageCheck` Ok 分岐: watch フック挿入
- `fav/src/driver.rs`: `run_with_watch(source, &[targets]) -> Vec<String>` テストヘルパー

---

## テスト仕様

### `run_trace_structured_output`

```favnir
pipeline P {
  stage Double = |n: Int| -> Int { n * 2 }
  stage Triple = |n: Int| -> Int { n * 3 }
}
fn main() -> Int { P.run(1) }
```

- `run_verbose(source, 1)` で実行（既存ヘルパー使用）
- `traces` に `[trace] stage=Double  out=` を含む行があることを assert
- `traces` に `[trace] stage=Triple  out=` を含む行があることを assert

### `run_watch_tracks_variable`

```favnir
pipeline Q {
  stage Parse = |n: Int| -> { amount: Int } { { amount: n * 10 } }
}
fn main() -> { amount: Int } { Q.run(5) }
```

- `run_with_watch(source, &["amount"])` で実行
- traces に `[watch] amount:` を含む行が存在することを assert

---

## バージョン要件

- `fav/Cargo.toml` version: `50.7.0`
- テスト数: 3103 → **3105**
  - `v507000_tests` 3 件追加、`v506000_tests::cargo_toml_version_is_50_6_0` 1 件削除 = 純増 +2

---

## 完了条件

- `cargo test` 3105 tests passed, 0 failed
- `cargo clippy -- -D warnings` クリーン
- `run_trace_structured_output`: `[trace] stage=Double  out=` を含む行が存在する
- `run_watch_tracks_variable`: `[watch] amount:` を含む行が存在する
- `v507000_tests` 3 件:
  - `cargo_toml_version_is_50_7_0`
  - `run_trace_structured_output`
  - `run_watch_tracks_variable`

---

## ロードマップ対応

roadmap-v50.1-v51.0.md v50.7.0 より:

> `fav run --trace` の出力を stage 単位の構造化ログに統一（既存実装を拡張）。
> `--watch <var.field>` フラグを新規追加し、VM の変数束縛フックに照合ロジックを挿入。
> 既存の `fav run --debug`（DAP）・`fav dap` とは独立した軽量導線として位置づける。

**差異・制約の明記:**
- `in=` フィールドは未実装（`SeqStageEnter` 時点でスタック上に入力値がない制約 — 将来バージョンで対応可能）
- CLI `--watch` 解析は未実装（テスト API 経由のみ。CLI 連携は v50.8.0 以降）
- `--watch` は Record フィールドに限定（スカラー値 watch はスコープ外）
