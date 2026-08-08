# v64.2.0 Plan — パフォーマンスリグレッションテスト自動化

Version: 64.2.0
Status: 未着手

---

## 変更ファイル一覧

| ファイル | 変更内容 |
|---|---|
| `fav/src/toml.rs` | `BenchTomlConfig` 追加 + `FavToml` フィールド追加 + `parse_fav_toml` 更新 |
| `fav/src/driver.rs` | `parse_bench_mean_ms` helper + `cmd_bench_compare` 追加 + `v64200_tests` 追加 |

---

## 実装ステップ

### Step 1: `toml.rs` — `BenchTomlConfig` + `FavToml` フィールド追加

- `BackpressureConfig` 定義の直後に `BenchTomlConfig` 構造体を追加
- `FavToml` の `backpressure` フィールドの直後に `bench: Option<BenchTomlConfig>` を追加
- `parse_fav_toml` に以下を追加:
  - `let mut bench_cfg: Option<BenchTomlConfig> = None;`
  - `[backpressure]` セクション検出の直後に `[bench]` セクション検出を追加
  - `"backpressure"` アームの直後に `"bench"` アームを追加
  - `FavToml` 構造体リテラルに `bench: bench_cfg` を追加

### Step 2: `driver.rs` — `parse_bench_mean_ms` + `cmd_bench_compare` 追加

`cmd_bench_suite` の直後に追加:
- `fn parse_bench_mean_ms(json: &str, mode: &str) -> Option<f64>`: JSON 文字列から mean_ms を抽出
- `pub fn cmd_bench_compare(ref_a: &str, ref_b: &str) -> String`:
  - `ref_a`（ベース）・`ref_b`（現在）のAOT mean_ms を比較
  - 劣化率が 10% 超: `"Regression detected: AOT +X.X% slower (was Xms, now Yms) — exceeds threshold 10%"`
  - OK 時: `"No regression detected. AOT {pct:+.1}% (was Xms, now Yms)"` （改善時は `AOT -X.X%` になる）
  - parse 失敗時: `"bench_compare: could not parse ..."`

### Step 3: `driver.rs` — `v64200_tests` 追加

`v64100_tests` の直前に挿入:
- `bench_compare_detects_regression`: base=10ms/curr=15ms で Regression 検出。同一比較で No regression。
- `bench_toml_threshold`: `[bench]\nregression_threshold_pct = 5` を parse し `Some(5)` を確認。

### Step 4: ビルド・テスト全件確認

- `cargo build` エラーなし
- `cargo test --bin fav v64200_tests` で 2 件 PASS
- `cargo test -j 8 -- --test-threads=8` で 3435 tests passed, 0 failed

---

## テスト計画

| テスト名 | 確認内容 |
|---|---|
| `bench_compare_detects_regression` | リグレッション検出（+50%）+ No regression（同一） |
| `bench_toml_threshold` | `[bench]` セクションのパース、`regression_threshold_pct = 5` が `Some(5)` |

ベース: 3433 → 目標: 3435（+2）
