# v63.4.0 Plan — `par` 動的スレッドプール・`[parallel]` fav.toml 設定

Version: 63.4.0
Status: 未着手

---

## 実装順序

### Step 1: `toml.rs` — `ParallelConfig` 構造体追加

`BuildConfig` 構造体の直前（`/// \`[build\]\` section of fav.toml (v62.7.0).` のコメント行の直前）に追加する:

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

`cargo build` でエラーなしを確認。

### Step 2: `toml.rs` — `FavToml` フィールド追加（1箇所）+ `parse_fav_toml` 内更新（3箇所）+ リテラル追加（1箇所）

計5箇所の更新（spec §2 が FavToml フィールド、spec §3 が parse_fav_toml 3箇所 + リテラル）。

**① `FavToml` 構造体** — `build: Option<BuildConfig>` の直後に追加:

```rust
/// Optional parallel execution configuration (v63.4.0).
pub parallel: Option<ParallelConfig>,
```

**② `parse_fav_toml` ローカル変数** — `let mut build_cfg: Option<BuildConfig> = None;` の直後:

```rust
let mut parallel_cfg: Option<ParallelConfig> = None;
```

**③ `parse_fav_toml` セクション検出** — `if trimmed == "[build]" { ... }` の直後:

```rust
if trimmed == "[parallel]" {
    section = "parallel";
    continue;
}
```

**④ `parse_fav_toml` セクション処理** — `"build" => { ... }` ブロックの直後:

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

**⑤ `FavToml { ... }` リテラル** — `build: build_cfg,` の直後:

```rust
parallel: parallel_cfg,
```

`cargo build` でエラーなしを確認。

### Step 3: `driver.rs` — `cmd_parallel_stats` 追加

`cmd_run_with_cache` の直後に追加する:

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

### Step 4: `driver.rs` — `v63400_tests` 追加

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

`cargo test v63400` で 2 件 PASS を確認。

### Step 5: 全テスト

`cargo test -j 8 -- --test-threads=8` で 3414 tests passed, 0 failed を確認。

### Step 6: ドキュメント更新

tasks.md T5 の各項目に従って更新する:
1. `CHANGELOG.md` 先頭に v63.4.0 エントリを追加
2. `versions/roadmap/roadmap-v63.1-v64.0.md` v63.4.0 セクションに実績追記
3. `versions/current.md` の「進行中」を v63.4.0（3414 tests）に更新
4. 最終ステップとして `tasks.md` を COMPLETE に更新（全チェックボックス `[x]`）

---

## 設計メモ

### 変更対象ファイル

| ファイル | 変更内容 |
|---|---|
| `fav/src/toml.rs` | `ParallelConfig` 構造体追加 + `FavToml` フィールド + `parse_fav_toml` 更新（4箇所） |
| `fav/src/driver.rs` | `cmd_parallel_stats` 追加 + `v63400_tests` 追加 |

### `FavToml` のフィールド追加ミス防止

`FavToml` 構造体定義とリテラル（`parse_fav_toml` 末尾）の **両方** を更新すること。
片方のみ追加するとコンパイルエラーになる。
過去の `build: build_cfg` 追加（v62.7.0）で同じパターンが確立されている。

### `parse_fav_toml_pub` の可視性

`parse_fav_toml` は crate プライベート関数だが、
`pub fn parse_fav_toml_pub` がテスト用ラッパーとして `toml.rs` に存在する（v43.x 追加）。
`v63400_tests` の両テストはこれを使用する（追加 import 不要）。

### `section` 変数との競合確認

`parse_fav_toml` の `section` は `&str` 型の mutable 変数。
`"parallel"` という文字列値は既存セクションと重複しないことを確認済み。
