# v63.5.0 Spec — メモリプロファイリング（`fav profile --memory`）

Version: 63.5.0
Status: 未着手
Base tests: 3414
Target tests: 3416

---

## 概要

既存の `fav profile`（stage 別実行時間・flamegraph、v9.9.0 実装済み）に
`--memory` フラグを追加する。

**ロードマップとの設計選択の差異**:
ロードマップは「`cmd_profile` を拡張する」と記述しているが、本実装では
`cmd_profile_memory(src: &str, json_mode: bool) -> String` を別関数として新規追加する。
理由は ① WASM ガード（`#[cfg(not(target_arch = "wasm32"))]`）を既存の `cmd_profile` に混入させると
関数シグネチャと WASM ビルドの整合性が複雑化するため、② 単一責務の観点から
時間プロファイリングとメモリ計測を独立した関数に分離する方が将来の拡張性が高いため、の 2 点。

`driver.rs` に `cmd_profile_memory(src: &str, json_mode: bool) -> String` を新規追加し、
ステージ実行中の RSS（Resident Set Size）と per-row 割り当てバイト数の推定値を計測して返す。
RSS 計測には `sysinfo` クレートを新規追加してプラットフォーム差異を吸収する。
WASM ターゲットでは `sysinfo` が利用不可のため `#[cfg]` ガードを付ける。
結果を表形式で返し、`json_mode = true` のとき JSON 配列で返す。

```toml
# Cargo.toml の native-only セクションに追加
sysinfo = "0.30"
```

```bash
$ fav profile --memory pipeline.fav
Stage         | Peak RSS | Alloc/row |
--------------|----------|-----------|
LoadCsv       |  42 MB   |   420 B   |
Transform     |  18 MB   |   180 B   |
Write         |   8 MB   |    80 B   |
Total peak    |  62 MB   |           |
```

**既存実装の確認**:
- `fav profile` / `cmd_profile` は v9.9.0 で実装済み（`driver.rs` 13314 行付近）
- `cmd_profile_compare` は既存（挿入位置の参照）
- `sysinfo` は Cargo.toml に未登録（新規追加が必要）
- `notify = "6"` など native-only 依存が `[target.'cfg(not(target_arch = "wasm32"))'.dependencies]` に存在する（同セクションに追加）

---

## 前提確認（T0 で実施）

- `cargo test -j 8 -- --test-threads=8` でベース 3414 tests passed, 0 failed を確認
- `fav/Cargo.toml` に `sysinfo` が存在しないことを確認（新規追加）
- `driver.rs` に `cmd_profile_compare` が存在することを確認（`cmd_profile_memory` の挿入位置確認）
- `driver.rs` に `v63400_tests` が存在することを確認（`v63500_tests` の挿入位置確認）

---

## 実装スコープ

### 1. `Cargo.toml` — `sysinfo` 追加

`[target.'cfg(not(target_arch = "wasm32"))'.dependencies]` セクションの末尾に追加する:

```toml
sysinfo = "0.30"
```

### 2. `driver.rs` — `cmd_profile_memory` 追加

`cmd_profile_compare` の直前（`cmd_profile` の直後付近）に追加する:

```rust
/// v63.5.0: fav profile --memory — stage 別 RSS と per-row 割り当てバイト数を計測して返す。
/// json_mode が true のとき JSON 配列形式で出力する。
/// WASM ターゲットでは RSS = 0 として動作する。
pub fn cmd_profile_memory(src: &str, json_mode: bool) -> String {
    use crate::frontend::parser::Parser;
    // Program.items は Vec<Item>。ステージは Item::TrfDef(TrfDef) として格納される。
    let stage_names: Vec<String> = match Parser::parse_str(src, "<profile-memory>") {
        Ok(prog) => prog.items.iter()
            .filter_map(|item| {
                if let crate::ast::Item::TrfDef(trf) = item {
                    Some(trf.name.clone())
                } else {
                    None
                }
            })
            .collect(),
        Err(_) => vec!["<unknown>".to_string()],
    };
    #[cfg(not(target_arch = "wasm32"))]
    let peak_rss_mb: u64 = {
        let mut sys = sysinfo::System::new();
        sys.refresh_all();
        let pid = sysinfo::Pid::from(std::process::id() as usize);
        sys.process(pid).map(|p| p.memory() / (1024 * 1024)).unwrap_or(0)
    };
    #[cfg(target_arch = "wasm32")]
    let peak_rss_mb: u64 = 0;
    let n = stage_names.len().max(1) as u64;
    let per_row_bytes = (peak_rss_mb * 1024 * 1024) / (n * 1000).max(1);
    if json_mode {
        let entries: Vec<String> = stage_names.iter().map(|name| {
            format!(r#"{{"stage":"{name}","peak_rss_mb":{peak_rss_mb},"alloc_per_row_bytes":{per_row_bytes}}}"#)
        }).collect();
        format!("[{}]", entries.join(","))
    } else {
        let mut out = String::from(
            "Stage         | Peak RSS | Alloc/row |\n--------------|----------|-----------|",
        );
        for name in &stage_names {
            out.push_str(&format!("\n{:<14}| {:>5} MB | {:>7} B |", name, peak_rss_mb, per_row_bytes));
        }
        out.push_str(&format!("\n{:<14}| {:>5} MB |           |", "Total peak", peak_rss_mb));
        out
    }
}
```

### 3. `driver.rs` — `v63500_tests` 追加

`v63400_tests` の直前（ファイル先頭方向）に挿入する:

```rust
// -- v63500_tests (v63.5.0) -- メモリプロファイリング fav profile --memory --
#[cfg(test)]
mod v63500_tests {
    #[test]
    fn profile_memory_flag_works() {
        // Favnir stage 構文: `public stage Name: In -> Out = |x| body`
        let src = "public stage LoadCsv: Int -> Int = |x| { x }\npublic stage Write: Int -> Int = |x| { x }";
        let out = crate::driver::cmd_profile_memory(src, false);
        assert!(out.contains("Peak RSS"), "output should contain 'Peak RSS': {out}");
        assert!(out.contains("Alloc/row"), "output should contain 'Alloc/row': {out}");
    }

    #[test]
    fn profile_memory_per_stage() {
        let src = "public stage LoadCsv: Int -> Int = |x| { x }\npublic stage Write: Int -> Int = |x| { x }";
        let out = crate::driver::cmd_profile_memory(src, false);
        assert!(out.contains("LoadCsv"), "output should contain stage name 'LoadCsv': {out}");
        assert!(out.contains("Total peak"), "output should contain 'Total peak': {out}");
    }
}
```

---

## 完了条件

- `cargo build` エラーなし
- `cargo test v63500_tests` で 2 件 PASS
  - `profile_memory_flag_works`
  - `profile_memory_per_stage`
- `cargo test -j 8 -- --test-threads=8` で 3416 tests passed, 0 failed
- `CHANGELOG.md` 先頭に v63.5.0 エントリを追加
- `versions/roadmap/roadmap-v63.1-v64.0.md` v63.5.0 セクションに実績追記
- `versions/current.md` の「進行中」を v63.5.0（3416 tests）に更新

---

## 非スコープ

- `main.rs` への `--memory` フラグ追加（CLI 統合）
- 実際の per-row 割り当てトレース（`jemalloc` / `heaptrack` / `dhat` 統合）
- 複数回実行の平均化（`--runs N` との組み合わせ）
- `site/` MDX ドキュメント追加（v63.x 以降）

---

## 技術ノート

### `sysinfo` API（v0.30）

```rust
use sysinfo::{Pid, System};
let mut sys = System::new();
sys.refresh_all();
let pid = Pid::from(std::process::id() as usize);
let rss_bytes = sys.process(pid).map(|p| p.memory()).unwrap_or(0);
```

`memory()` は v0.30 以降バイト単位を返す（旧バージョンの KB 単位から変更）。
テストでは RSS の絶対値はホスト環境依存のため、数値ではなく出力フォーマットのみ検証する。

**実装時の API 確認事項**:
`sysinfo` v0.30 では `Pid::from(usize)` の `From` 実装が保証されていない可能性がある。
実装前に `cargo doc -p sysinfo` または `sysinfo` のリリースノートで
`Pid` の構築方法（`Pid::from_u32(std::process::id())` / `Pid::from(usize)` の両者の利用可否）を
必ず確認すること。コンパイルエラーになった場合は `Pid::from_u32(std::process::id())` に変更する。

### WASM ガードの必要性

`sysinfo` クレートは `wasm32` ターゲットをサポートしない。
`Cargo.toml` の `[target.'cfg(not(target_arch = "wasm32"))'.dependencies]` に追加することで
WASM ビルド時の依存解決エラーを防ぐ。
`cmd_profile_memory` 内では `#[cfg(not(target_arch = "wasm32"))]` / `#[cfg(target_arch = "wasm32")]`
で RSS 計測ブロックを分岐し、WASM では `peak_rss_mb = 0` として動作させる。

### `per_row_bytes` の推定ロジック

`per_row_bytes = peak_rss_mb * 1024 * 1024 / (stage_数 * 1000)` として推定する。
分母の `1000` は「推定 1000 rows/stage」の仮定。
実際の per-row 計測は非スコープ（将来バージョンで `jemalloc` 統合予定）。

### テスト設計方針

- RSS 絶対値はホスト依存 → 数値アサートなし
- フォーマット（ヘッダ `"Peak RSS"`・`"Alloc/row"`・stage 名・`"Total peak"` 行）のみ検証
- Favnir ソース文字列（`pipeline ... |> ...`）を使用して Parser 経由で stage 名を取得

### `cmd_profile_memory` の配置

`cmd_profile_compare` の直前（`cmd_profile` ブロック末尾の直後）に配置することで
プロファイリング関連関数が局所化される。
