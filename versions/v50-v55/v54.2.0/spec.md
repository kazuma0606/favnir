# Spec: v54.2.0 — fav run --watch 高度化（差分表示・サマリー）

Status: COMPLETE
Date: 2026-07-22

---

## 概要

`fav run --watch` に差分表示（`--watch-diff`）とサマリー出力（`--watch-summary`）フラグを追加する。
`WatchEvent` 構造体・`format_watch_diff`・`format_watch_summary` を `driver.rs` に実装し、
テストで動作を検証する。

---

## 実装スコープ

### 1. `driver.rs` — `WatchEvent` 構造体 + フォーマット関数

`cmd_run` の直前に追加:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WatchEvent {
    pub field: String,
    pub stage: String,
    pub before: String,
    pub after: String,
}

pub fn format_watch_diff(event: &WatchEvent) -> String {
    // before/after を f64 にパース; 差分 d を計算
    // d == 0.0 → delta 省略（変化なし扱い）
    // d > 0.0  → "Δ+{:.1}" / d < 0.0 → "Δ{:.1}"
    // 非数値   → delta 省略
    // delta あり: "[watch] field:  before  → after   Δ±X.X  (stage: S)"
    // delta なし: "[watch] field:  before  → after  (stage: S)"
}

pub fn format_watch_summary(events: &[WatchEvent]) -> String {
    // 空スライス → "[watch-summary] no changes recorded"
    // それ以外  → "[watch-summary]\n  field (stage): before → after\n..."
}
```

設計上の注意:
- `d == 0.0` のとき `"Δ+0.0"` を出力せず delta 省略（変化なしイベント対応）
- f64 フォーマットは `:.1`（小数点 1 桁固定）で `Δ+99.0` 形式を保証
- `PartialEq, Eq` を derive することで `assert_eq!` 利用可能

### 2. `main.rs` — `fav run` に `--watch-diff` / `--watch-summary` フラグ追加

```
--watch-diff    数値フィールドの before/after 差分（Δ）を表示
--watch-summary 複数 stage にまたがった全ウォッチ変化のサマリーを出力
```

未実装機能としてフラグを受け付け `eprintln!` 警告を出力（サイレント無視を防ぐ）:

```rust
if watch_diff {
    eprintln!("warning: --watch-diff is not yet fully implemented; ...");
}
if watch_summary {
    eprintln!("warning: --watch-summary is not yet fully implemented; ...");
}
```

### 3. `driver.rs` — `v54200_tests` 追加

`v54100_tests` の直前に追加（2 テスト）:

```rust
// -- v54200_tests (v54.2.0) -- fav run --watch 高度化（差分表示・サマリー） --
mod v54200_tests {
    #[test] fn run_watch_diff_numeric()    { ... }
    #[test] fn run_watch_summary_output() { ... }
}
```

---

## テスト仕様

| テスト名 | 検証内容 |
|---|---|
| `run_watch_diff_numeric` | `before=0.0, after=99.0` → `[watch]`・`order.amount`・`0.0`・`99.0`・`Δ+99.0`・`Parse` が含まれること |
| `run_watch_summary_output` | 2 件のイベント → `[watch-summary]`・`order.amount`・`order.status`・`Parse`・`Validate` が含まれること; 空スライス → `no changes` が含まれること |

---

## バージョン更新

- `fav/Cargo.toml`: `"54.1.0"` → `"54.2.0"`

---

## 完了条件

- `cargo test` 3189 passed, 0 failed（ベース 3187 + 2 件追加）
- `v54200_tests` 2 件 pass:
  - `run_watch_diff_numeric`
  - `run_watch_summary_output`
- `cargo clippy -- -D warnings` クリーン

---

## 影響範囲

| ファイル | 変更種別 |
|---|---|
| `fav/src/driver.rs` | `WatchEvent`・`format_watch_diff`・`format_watch_summary` 追加 / `v54200_tests` 追加 |
| `fav/src/main.rs` | `--watch-diff` / `--watch-summary` フラグ解析追加 |
| `fav/Cargo.toml` | version 更新 |
| `fav/Cargo.lock` | version 更新に伴い自動更新 |
| `CHANGELOG.md` | v54.2.0 エントリ追加 |
| `versions/current.md` | v54.2.0 / 3189 tests に更新 |
| `versions/roadmap/roadmap-v54.1-v55.0.md` | v54.2.0 実績欄を COMPLETE に更新 |

---

## 設計上の注意

- 非数値文字列（`"abc"` 等）を `before`/`after` に渡した場合は delta を省略（非数値ケースのテストは将来バージョンで追加予定）。
- フル実装（runtime VM フック経由のフィールドレベル差分追跡）は将来バージョンで行う。
  本バージョンはフォーマット関数・フラグ解析・テストを先行実装し、
  未実装機能に対してはユーザーへ `eprintln!` 警告を出すことでサイレント無視を防ぐ。
- `v54200_tests` は `v54100_tests` の直前に挿入（逆時系列順の標準パターン）。
- `v54100_tests` には `cargo_toml_version_is_X` テストがないため空化対象なし。
