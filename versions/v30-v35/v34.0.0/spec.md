# v34.0.0 — Spec

## 概要

**テーマ**: Performance & Tooling マイルストーン宣言

**方針**: マイルストーン宣言パターン。v33.1〜v33.9 の確認・記録シリーズを経て、「Performance & Tooling」マイルストーンを正式宣言する。`cargo clean` 必須。

---

## 背景

v33.1〜v33.9（確認・記録）で以下が確認された:

| バージョン | 確認内容 |
|---|---|
| v33.1 | AOT ネイティブバイナリ（Cranelift / `fav build --target native`） |
| v33.2 | インクリメンタルコンパイル（`~/.fav/cache/` / SHA256 キャッシュ） |
| v33.3 | ストリーミング評価（`#[streaming(chunk_size)]` / 定常メモリ） |
| v33.4 | Arrow 列指向統合（`ArrowBatch` 型 / Parquet ゼロコピー） |
| v33.5 | precompiled 起動（`fav run --precompiled` / `.favc`） |
| v33.6 | WASM 最適化（DCE / `WasmBuildConfig` / wasm-opt 統合） |
| v33.7 | エフェクトシステム移行準備（`migrate_effects_in_source` / `resolve_use_effects`） |
| v33.8 | プロファイリング強化（`parse_profile_json` / `to_folded_stacks`） |
| v33.9 | 並列コンパイル（`compile_parallel` / `topo_layers` 循環依存検出） |

---

## 実装スコープ

### 変更ファイル
1. `fav/Cargo.toml` — version `33.9.0` → `34.0.0`
2. `fav/src/driver.rs` — `cargo_toml_version_is_33_9_0` をスタブ化、`v340000_tests` 4 件追加
3. `benchmarks/v34.0.0.json` — 新規作成
4. `CHANGELOG.md` — `[v34.0.0]` セクション先頭追記
5. `MILESTONE.md` — `v34.0.0 — Performance & Tooling` セクション先頭追加
6. `README.md` — v34.0 マイルストーン行を追記
7. `versions/current.md` — 最新安定版を v34.0.0 に更新

### 新規ファイル
- `versions/v30-v35/v34.0.0/` — spec.md / plan.md / tasks.md

### cargo clean（必須）
```bash
cd /c/Users/yoshi/favnir/fav && cargo clean
cargo build 2>&1 | tail -3
cargo test -j 8 -- --test-threads=8 2>&1 | grep "test result"
```

> ⚠️ `cargo clean` 後は `fav/tmp/hello.fav` が消えない（target/ のみ削除）ため復元不要。
> ただし `cargo build` が通ることを必ず確認すること。

---

## テスト仕様（v340000_tests）

```rust
#[cfg(test)]
mod v340000_tests {
    #[test]
    fn cargo_toml_version_is_34_0_0() {
        let src = include_str!("../Cargo.toml");
        assert!(src.contains("34.0.0"), "Cargo.toml must contain '34.0.0'");
    }

    #[test]
    fn benchmark_v34_0_0_exists() {
        let src = include_str!("../../benchmarks/v34.0.0.json");
        assert!(src.contains("34.0.0"), "benchmarks/v34.0.0.json must contain '34.0.0'");
    }

    #[test]
    fn milestone_performance_tooling_declared() {
        let src = include_str!("../../MILESTONE.md");
        assert!(
            src.contains("Performance & Tooling"),
            "MILESTONE.md must contain 'Performance & Tooling'"
        );
    }

    #[test]
    fn readme_mentions_v34() {
        let src = include_str!("../../README.md");
        assert!(src.contains("v34"), "README.md must mention 'v34'");
    }
}
```

### 設計注記
- `use super::*` なし（`include_str!` のみ使用）
- WASM ゲートなし（ファイル読み込みのみ）
- v340000_tests は v339000_tests 直後・`// ── v31.7.0 tests` の前に挿入

---

## MILESTONE.md 追加内容

```markdown
## v34.0.0 — Performance & Tooling（2026-07-04）

> 「`fav build --target native` でネイティブバイナリが生成でき、
>  10GB CSV を定常メモリで処理でき、
>  Lambda コールドスタートが 100ms 以下になること」
> = Performance & Tooling の完成を象徴する定義

v34.0.0 をもって、Favnir の **Performance & Tooling** を正式に宣言する。

AOT ネイティブバイナリ（Cranelift）/ インクリメンタルコンパイル / ストリーミング評価 /
Arrow 列指向統合 / precompiled 起動 / WASM 最適化 / エフェクトシステム移行準備 /
プロファイリング強化 / 並列コンパイルが v33.x シリーズで確認・記録された。

### 達成コンポーネント（v33.1〜v33.9）

| コンポーネント | バージョン | 内容 |
|---|---|---|
| AOT ネイティブバイナリ | v33.1 | `fav build --target native` / Cranelift バックエンド |
| インクリメンタルコンパイル | v33.2 | `~/.fav/cache/` / SHA256 ハッシュキャッシュ |
| ストリーミング評価 | v33.3 | `#[streaming(chunk_size)]` / 定常メモリ処理 |
| Arrow 列指向統合 | v33.4 | `ArrowBatch` 型 / Parquet ゼロコピー書き込み |
| precompiled 起動 | v33.5 | `fav run --precompiled` / `.favc` アーティファクト |
| WASM 最適化 | v33.6 | DCE / wasm-opt 統合 / `WasmBuildConfig` |
| エフェクトシステム移行準備 | v33.7 | `migrate_effects_in_source` / `resolve_use_effects` |
| プロファイリング強化 | v33.8 | `parse_profile_json` / `to_folded_stacks` |
| 並列コンパイル | v33.9 | `compile_parallel` / `topo_layers` 循環依存検出 |

**宣言日**: 2026-07-04
**宣言バージョン**: v34.0.0
```

---

## 完了条件

- [ ] `cargo clean` + `cargo build` + `cargo test` 全 PASS
- [ ] `Cargo.toml` version = `"34.0.0"`
- [ ] `cargo_toml_version_is_33_9_0` が空スタブ（他3テストは残存・スタブ化しない）
- [ ] `cargo test --bin fav v340000` — 4/4 PASS
- [ ] `cargo test` — 全件 PASS（2536 件、0 failures）
- [ ] `CHANGELOG.md` に `[v34.0.0]` セクション
- [ ] `MILESTONE.md` に `v34.0.0 — Performance & Tooling` セクション
- [ ] `README.md` に v34.0 マイルストーン行
- [ ] `benchmarks/v34.0.0.json` 存在かつ `tests_passed` が実測値
- [ ] `benchmarks/v34.0.0.json` の `milestone` フィールドが `"Performance & Tooling"`（そのバージョン自身のマイルストーン名）
- [ ] `versions/current.md` を v34.0.0 に更新
