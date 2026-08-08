# v63.4.0 Spec — `par` 動的スレッドプール（`[parallel]` fav.toml 設定）

Version: 63.4.0
Status: 未着手
Base tests: 3412
Target tests: 3414

---

## 概要

`toml.rs` に `ParallelConfig { max_threads: usize, queue_depth: usize }` 構造体を追加し、
`FavToml` に `parallel: Option<ParallelConfig>` フィールドを追加してパースする。
`parse_fav_toml` が `[parallel]` セクションを処理できるよう更新する。
`driver.rs` に `cmd_parallel_stats(toml_content: &str) -> String` を追加し、
設定値と有効スレッド数（`available_parallelism` デフォルト）を表示する。

```toml
[parallel]
max_threads = 8
queue_depth = 1000
```

**既存実装の確認**:
- `par [A, B]` Tokio 並列は v52.0 で `vm.rs` に実装済み（`ParStages` opcode + `std::thread::spawn`）
- スレッド数は現在ハードコードなし（stage 数分 `spawn` する設計）
- v63.4.0 では TOML パース + `cmd_parallel_stats` を実装する
- VM への `ParallelConfig` 注入（スレッドプール制御）は v63.x 以降のスコープとして後送り
  ※ ロードマップ「`vm.rs` の `par` 実行エンジンが `ParallelConfig` を読み込む」は
    将来バージョンの実装方針であり、`roadmap-v63.1-v64.0.md` の `**既存機能の扱い**` 注記と整合

**型名の注記**:
ロードマップでは `FavConfig` と記述されているが、実際の型名は `fav/src/toml.rs` の `FavToml` である。
`FavConfig` はロードマップ上の略称であり、コード上の正式名は `FavToml`。

---

## 前提確認（T0 で実施）

- `cargo test -j 8 -- --test-threads=8` でベース 3412 tests passed, 0 failed を確認
- `fav/src/toml.rs` に `BuildConfig` 構造体と `build: Option<BuildConfig>` フィールドが
  存在することを確認（追加パターンの参照）
- `driver.rs` に `v63300_tests` が存在することを確認（挿入位置確認）

---

## 実装スコープ

### 1. `toml.rs` — `ParallelConfig` 構造体追加

`BuildConfig` 構造体の直前に追加する:

```rust
/// `[parallel]` section of fav.toml (v63.4.0).
#[derive(Debug, Clone)]
pub struct ParallelConfig {
    /// 最大並列スレッド数。0 = CPU コア数（`available_parallelism()`）。デフォルト: 0。
    pub max_threads: usize,
    /// ステージ間キューの最大深度。デフォルト: 256。
    pub queue_depth: usize,
}

impl Default for ParallelConfig {
    fn default() -> Self {
        ParallelConfig {
            max_threads: 0,
            queue_depth: 256,
        }
    }
}
```

### 2. `toml.rs` — `FavToml` にフィールド追加

`build: Option<BuildConfig>` フィールドの直後に追加する:

```rust
/// Optional parallel execution configuration (v63.4.0).
pub parallel: Option<ParallelConfig>,
```

### 3. `toml.rs` — `parse_fav_toml` を更新

① `let mut build_cfg: Option<BuildConfig> = None;` の直後に追加:
```rust
let mut parallel_cfg: Option<ParallelConfig> = None;
```

② `if trimmed == "[build]" { section = "build"; continue; }` の直後に追加:
```rust
if trimmed == "[parallel]" {
    section = "parallel";
    continue;
}
```

③ `"build" => { ... }` ブロックの直後に追加:
```rust
"parallel" => {
    let mut current = parallel_cfg.take().unwrap_or_default();
    if let Some((key, val)) = parse_kv(trimmed) {
        match key {
            "max_threads" => current.max_threads = val.parse::<usize>().unwrap_or(0),
            "queue_depth" => current.queue_depth = val.parse::<usize>().unwrap_or(256),
            _ => {}
        }
    }
    parallel_cfg = Some(current);
}
```

④ `FavToml { ... }` の構造体リテラル末尾（`build: build_cfg,` の後）に追加:
```rust
parallel: parallel_cfg,
```

### 4. `driver.rs` — `cmd_parallel_stats` 追加

`cmd_run_with_cache` の直後（または `cmd_incremental_cache_status` の近傍）に追加する:

```rust
/// v63.4.0: [parallel] 設定を解析し、有効スレッド数とキュー深度を返す。
/// toml_content が空の場合はデフォルト値（CPU コア数）を使用する。
pub fn cmd_parallel_stats(toml_content: &str) -> String {
    let cfg = crate::toml::parse_fav_toml_pub(toml_content);
    let p = cfg.parallel.unwrap_or_default();
    let effective_threads = if p.max_threads == 0 {
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1)
    } else {
        p.max_threads
    };
    format!(
        "parallel stats: max_threads={} (effective={}), queue_depth={}",
        p.max_threads, effective_threads, p.queue_depth
    )
}
```

### 5. `driver.rs` — `v63400_tests` 追加

`v63300_tests` の直前（ファイル先頭方向）に挿入する:

```rust
// -- v63400_tests (v63.4.0) -- par 動的スレッドプール・[parallel] fav.toml 設定 --
#[cfg(test)]
mod v63400_tests {
    #[test]
    fn parallel_toml_config_parsed() {
        let toml = "[package]\nname = \"test\"\nversion = \"0.1.0\"\n\n[parallel]\nmax_threads = 8\nqueue_depth = 1000\n";
        let config = crate::toml::parse_fav_toml_pub(toml);
        let p = config.parallel.expect("parallel config should be parsed");
        assert_eq!(p.max_threads, 8, "max_threads should be 8");
        assert_eq!(p.queue_depth, 1000, "queue_depth should be 1000");
    }

    #[test]
    fn parallel_stats_output() {
        let toml = "[package]\nname = \"test\"\nversion = \"0.1.0\"\n\n[parallel]\nmax_threads = 4\nqueue_depth = 512\n";
        let out = crate::driver::cmd_parallel_stats(toml);
        assert!(out.contains("max_threads=4"), "output should contain max_threads=4: {out}");
        assert!(out.contains("queue_depth=512"), "output should contain queue_depth=512: {out}");
        assert!(out.contains("effective=4"), "output should contain effective=4: {out}");
    }
}
```

---

## 完了条件

- `cargo build` エラーなし
- `cargo test v63400` で 2 件 PASS
  - `parallel_toml_config_parsed`
  - `parallel_stats_output`
- `cargo test -j 8 -- --test-threads=8` で 3414 tests passed, 0 failed
- `CHANGELOG.md`・`versions/current.md`・ロードマップに実績追記済み

---

## 非スコープ

- `vm.rs` の `par` 実行エンジンへの `ParallelConfig` 注入（スレッドプール制御）
- `fav run --parallel-stats` CLI フラグの実装（`main.rs` のフラグ解析追加）
- Tokio スレッドプールへの移行（現在 `std::thread::spawn` を使用）
- `site/` MDX ドキュメント追加（v63.x 以降）

---

## 技術ノート

### `max_threads = 0` のセマンティクス

`0` を「CPU コア数を自動検出」の特別値として扱う。
これは `rayon` / `tokio` の慣習（`0` = デフォルト）に準拠する。
`parse_fav_toml` では `0` をそのまま保存し、
`cmd_parallel_stats` / VM 実行時に `available_parallelism()` で解決する。

`[parallel]` セクション省略時は `parallel: None` となり、
`unwrap_or_default()` によって `max_threads=0` として処理される。
これは `max_threads = 0` を明示的に設定した場合と同一の出力（`effective=<CPU数>`）になる。
両者の区別は不要であり、これは意図した動作である。

### `BuildConfig` との対称性

`ParallelConfig` は `BuildConfig`（v62.7.0 追加）と同じパターンで実装する:
- 構造体定義 → `impl Default` → `FavToml` フィールド → `parse_fav_toml` セクション処理 → `FavToml` リテラル追加
- `parse_kv(trimmed)` で `key = val` 形式をパース（既存ヘルパー使用）

### `parse_fav_toml_pub` の利用

`parse_fav_toml` は `pub(crate)` ではなくプライベート関数のため、
テストからは `pub fn parse_fav_toml_pub` を使用する（v43.x で公開済み）。

### `cmd_parallel_stats` の位置

`cmd_run_with_cache` の直後（または近傍）に配置することで、
並列・キャッシュ関連の関数が局所化される。
