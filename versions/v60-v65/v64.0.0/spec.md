# v64.0.0 Spec — Incremental & Scale 宣言 ★クリーンアップ

Version: 64.0.0
Status: 未着手

---

## 概要

v63.1〜v63.9 で実装した「Incremental & Scale」機能群を統合し、マイルストーン宣言を行う。

宣言文（MILESTONE.md に記載）:

> 「変更されたステージだけが再コンパイルされ、未使用のステージは除去される。
>  スレッドはコアの数だけ走り、キューはバックプレッシャーで制御される。
>  ベンチマークは数字で真実を語る。
>
>  Favnir は大規模 ETL を安心して任せられるエンジンになった。
>
>  これが Favnir v64.0 — Incremental & Scale の姿である。」

---

## 前提確認（T0 で実施）

- `cargo test -j 8 -- --test-threads=8` でベース 3427 tests passed, 0 failed を確認
- `fav/Cargo.toml` の version が `"63.0.0"` であることを確認（`"64.0.0"` への更新対象）
- `MILESTONE.md` に `"Incremental & Scale"` が含まれないことを確認（新規追加対象）
- `README.md` に `"Incremental & Scale"` が含まれないことを確認（追記対象）
- `driver.rs` に `v63900_tests` が存在することを確認（`v64000_tests` の挿入位置確認）
- `driver.rs` に `v64000_tests` が存在しないことを確認（新規追加）

**ロードマップとの差異（重要）**:
- ロードマップ完了条件（行 264）は `ベース 3424 + 4 = 3428` と記載しているが、実際の base は 3427。3427 + 4 = **3431** が正しい目標値（推移表行 290 の 3431 と一致）。
- `★クリーンアップ`（`cargo clean`）はテスト全通過後、ビルド再確認を兼ねて実施する。

---

## 実装スコープ

### 1. `fav/Cargo.toml` — バージョン更新

```toml
version = "64.0.0"
```

### 2. `MILESTONE.md` — 宣言エントリ追加

ファイル先頭の `## v63.0.0` エントリの前に挿入:

```markdown
## v64.0.0（2026-08-02）— Incremental & Scale

> 「変更されたステージだけが再コンパイルされ、未使用のステージは除去される。
>  スレッドはコアの数だけ走り、キューはバックプレッシャーで制御される。
>  ベンチマークは数字で真実を語る。
>
>  Favnir は大規模 ETL を安心して任せられるエンジンになった。
>
>  これが Favnir v64.0 — Incremental & Scale の姿である。」

**Incremental & Scale** の宣言バージョン。v63.1〜v63.9 で実装した全機能を統合し、
差分コンパイル・DAG 最適化・並列実行・バックプレッシャー制御・ETL ベンチマークの完成を宣言した。

**v63.1〜v63.9 達成内容:**
- v63.1（差分キャッシュ）: `cmd_run_with_cache` / `IncrementalCache` / E0428
- v63.2（fav watch 改善）: ポーリング最適化・変更ファイルのみ再コンパイル
- v63.3（E0428）: キャッシュ無効化エラー
- v63.4（par 動的スレッドプール）: `cmd_parallel_stats` / `[parallel]` 設定
- v63.5（メモリプロファイリング）: `cmd_profile_memory`
- v63.6（バックプレッシャー・W041）: `BackpressureConfig` / `[backpressure]` / W041 lint
- v63.7（DAG 最適化）: `cmd_opt_stats` / dead stage elimination / pure stage fusion
- v63.8（ETL ベンチスイート）: `cmd_bench_suite` / "etl-standard" スイート
- v63.9（安定化）: `scale_e2e_incremental_par` / `scale_dag_opt_dead_and_fused`

**テスト数**: 3431

---
```

### 3. `README.md` — v64.0.0 宣言追記

`v63.0.0` エントリの前に追加:

```markdown
**v64.0.0 — Incremental & Scale を宣言しました（2026-08-02）。**
v63.1〜v63.9 で実装した差分コンパイル・DAG 最適化・並列実行・バックプレッシャー制御・ETL ベンチマークを統合し、
大規模 ETL を安心して任せられるエンジンとしての完成を宣言した。
```

### 4. `CHANGELOG.md` — v64.0.0 エントリ追加

### 5. `driver.rs` — `v64000_tests` 追加

`v63900_tests` の直前に挿入:

```rust
// -- v64000_tests (v64.0.0) -- Incremental & Scale 宣言 --
#[cfg(test)]
mod v64000_tests {
    #[test]
    fn cargo_toml_version_is_64_0_0() {
        let toml = include_str!("../Cargo.toml");
        assert!(
            toml.contains("version = \"64.0.0\""),
            "Cargo.toml should have version 64.0.0: {}",
            &toml[..200.min(toml.len())]
        );
    }

    #[test]
    fn changelog_has_v64_0_0() {
        let cl = include_str!("../../CHANGELOG.md");
        assert!(cl.contains("v64.0.0"), "CHANGELOG.md should mention v64.0.0");
    }

    #[test]
    fn milestone_has_incremental_scale() {
        let ms = include_str!("../../MILESTONE.md");
        assert!(
            ms.contains("Incremental & Scale"),
            "MILESTONE.md should contain 'Incremental & Scale'"
        );
    }

    #[test]
    fn readme_mentions_incremental_scale() {
        let readme = include_str!("../../README.md");
        assert!(
            readme.contains("Incremental & Scale") || readme.contains("v64.0"),
            "README.md should mention Incremental & Scale or v64.0: (truncated)"
        );
    }
}
```

---

## 完了条件

- `cargo build` エラーなし
- `cargo test --bin fav v64000_tests` で 4 件 PASS
  - `cargo_toml_version_is_64_0_0` PASS
  - `changelog_has_v64_0_0` PASS
  - `milestone_has_incremental_scale` PASS
  - `readme_mentions_incremental_scale` PASS
- `cargo test -j 8 -- --test-threads=8` で 3431 tests passed, 0 failed
- `cargo clean` 実行後 `cargo build` でエラーなし（★クリーンアップ）

---

## 非スコープ

- v63.x の機能追加・修正（全スプリント完了済み）
- E0428 の表示・消去の詳細テスト（後送り）
- `fav bench --suite` CLI フラグ（後送り）
- `fav/benchmarks/` ディレクトリへの `.fav` ファイル追加（後送り）

---

## 技術ノート

### include_str! パス

`driver.rs` は `fav/src/driver.rs` に位置する:
- `../Cargo.toml` → `fav/Cargo.toml`
- `../../CHANGELOG.md` → `CHANGELOG.md`（リポジトリルート）
- `../../MILESTONE.md` → `MILESTONE.md`（リポジトリルート）
- `../../README.md` → `README.md`（リポジトリルート）

### cargo clean について

`cargo clean` で `fav/tmp/hello.fav` が削除されることがある（past issue）。
クリーンアップ後に `bootstrap_c2_artifact_roundtrip` テストが通ることを確認すること。
`fav/tmp/hello.fav` の正しい内容:
```
fn add(a: Int, b: Int) -> Int { a + b }
fn main() -> Bool { add(1, 2) == 3 }
```

### テスト数の差異

ロードマップ記載: base=3424、target=3428。
実際: base=3427（v63.6.0 code-reviewer 対応等の影響）、target=3431（推移表と一致）。
