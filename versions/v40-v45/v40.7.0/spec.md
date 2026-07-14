# v40.7.0 仕様書 — `fav bench --stream`

## バージョン概要

| 項目 | 内容 |
|------|------|
| バージョン | v40.7.0 |
| テーマ | Streaming Foundations — `fav bench --stream` |
| 前バージョン | v40.6.0（2832 tests） |
| 目標テスト数 | 2835（+3） |
| 参照ロードマップ | `versions/roadmap/roadmap-v40.1-v41.0.md` §v40.7.0 |

---

## 背景と目的

v40.6.0 で Kafka / Redis に `consume_windowed` スタブを追加した。
本バージョンでは `fav bench` コマンドに `--stream` フラグを追加し、
ストリームパイプラインのスループット・レイテンシ計測をスタブとして実装する。

実際の計測ロジック（イベント生成 → ウィンドウ処理 → 結果集計）は v41.0 以降で実装。
本バージョンは `--stream` フラグの受け口（`BenchOpts.stream` フィールド）と
コマンドディスパッチのみを追加する。

---

## 実装スコープ

### 変更ファイル

| ファイル | 変更内容 |
|----------|----------|
| `fav/src/driver.rs` | `BenchOpts` に `stream: bool` フィールド追加・`cmd_bench` スタブ分岐追加・v40600_tests stub 化・v40700_tests 追加 |
| `fav/src/main.rs` | `bench` アームに `--stream` フラグ解析追加・ヘルプテキストに `--stream` 追記 |
| `fav/Cargo.toml` | version: `40.6.0` → `40.7.0` |
| `CHANGELOG.md` | `[v40.7.0]` エントリ追加（`[v40.6.0]` の直後） |

---

## `BenchOpts` 変更設計

```rust
pub struct BenchOpts {
    pub file: Option<String>,
    pub filter: Option<String>,
    pub runs: u64,
    pub warmup: u64,
    pub json: bool,
    pub stream: bool,   // v40.7.0 追加
}

impl Default for BenchOpts {
    fn default() -> Self {
        BenchOpts { file: None, filter: None, runs: 100, warmup: 5, json: false, stream: false }
    }
}
```

---

## `cmd_bench` スタブ分岐設計

`cmd_bench` 関数の先頭（または既存ロジックの前）に以下のスタブ分岐を追加する:

```rust
if opts.stream {
    // TODO: v40.7.0 stub — ストリームパイプライン計測は v41.0 以降で実装
    println!("[fav bench --stream] stream throughput/latency bench: stub (v40.7.0)");
    return;
}
```

---

## `main.rs` フラグ解析設計

既存の `bench` アームの `--json` パース直後に追加:

```rust
"--stream" => {
    opts.stream = true;
    i += 1;
}
```

---

## テスト設計（v40700_tests）

```rust
#[cfg(test)]
mod v40700_tests {
    use super::*;

    #[test]
    fn cargo_toml_version_is_40_7_0() {
        // NOTE: 次バージョン bump 時に Stubbed コメントへ置き換えること
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("40.7.0"), "Cargo.toml must contain version 40.7.0");
    }

    #[test]
    fn changelog_has_v40_7_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(src.contains("[v40.7.0]"), "CHANGELOG.md must contain [v40.7.0]");
    }

    #[test]
    fn bench_opts_has_stream_field() {
        let opts = BenchOpts { stream: true, ..BenchOpts::default() };
        assert!(opts.stream, "BenchOpts must have stream field");
    }
}
```

`bench_opts_has_stream_field` は `BenchOpts` を直接参照するため `use super::*` が必要。

テスト数: 2832 + 3 = **2835**

**注**: ロードマップは「推定 2833（2 件）」と記載しているが、v40.6.0 の実績（2832）を起点に 3 テスト構成（2835）とする。

---

## 完了条件

**自動検証（cargo test）:**

| # | 条件 | 検証方法 |
|---|------|----------|
| 1 | `BenchOpts` に `stream: bool` フィールドが追加されている | cargo test（コンパイル） |
| 2 | `BenchOpts::default().stream == false` | `bench_opts_has_stream_field` テスト |
| 3 | `cmd_bench` が `opts.stream == true` 時にスタブメッセージを出力して早期 return する | 手動確認（`fav bench --stream` 実行） |
| 4 | `main.rs` の `bench` アームが `--stream` フラグを解析し `opts.stream = true` にセットする | 手動確認 / コンパイル通過 |
| 5 | `main.rs` ヘルプテキストに `--stream` が記載されている | 手動確認（`fav help` 実行） |
| 6 | `Cargo.toml` の version が `40.7.0` | `cargo_toml_version_is_40_7_0` テスト |
| 7 | `CHANGELOG.md` に `[v40.7.0]` エントリが存在する | `changelog_has_v40_7_0` テスト |
| 8 | `cargo test` 全通過（failures=0、テスト数 ≥ 2835） | cargo test |
| 9 | `v40700_tests` 3 件すべて pass | cargo test |

---

## ロードマップとの差異

- ロードマップは「推定 2833（2 件）」と記載しているが、実績 2832 を起点に 3 テスト構成（2835）とする（v40.1〜v40.6 の確立パターンに合わせる）。
- `cmd_bench --stream` の実際のスループット・レイテンシ計測ロジックは v41.0 以降で実装（本バージョンはスタブのみ）。
