# Spec: v51.6.0 — checker / compiler ホットパス最適化

Date: 2026-07-19
Status: 設計中

---

## 概要

v51.5.0 のインクリメンタルコンパイル基盤に続き、checker と compiler のホットパスを最適化する。
3 点のアプローチで実装コストを段階的に削減する。

1. **`fav profile --build <file>`** — checker・compiler・parse 各フェーズの処理時間を計測・表示
2. **`SubstRef` 参照共有** — `Subst` のクローンコストを `Rc<Subst>` で削減
3. **`SourceCache`** — `collect_merged_sources` の重複ファイル読み込みをキャッシュで排除
4. **`benchmarks/v51.6.0.json`** — 計測ベースラインの記録

---

## 機能詳細

### 1. `fav profile --build <file>`（`driver.rs` / `main.rs`）

**背景**: 既存の `fav profile <file>` は pipeline ランタイムの stage 別計測。
`--build` フラグは別軸 — ビルドフェーズ（parse / check / compile）の時間を計測する。

**新規追加**:

```rust
pub struct ProfileBuildResult {
    pub parse_ms: f64,
    pub check_ms: f64,
    pub compile_ms: f64,
}

pub fn profile_build_file(path: &str) -> Result<ProfileBuildResult, String>
pub fn cmd_profile_build(path: &str)
```

`profile_build_file` は `std::time::Instant::now()` で各フェーズを計測:
- `parse_ms`: `Parser::parse_str` の経過時間
- `check_ms`: `check_single_file` の経過時間
- `compile_ms`: `compile_src_str_to_bytes` の経過時間

`cmd_profile_build` は結果をテーブル形式で表示:

```
Phase    Time (ms)    %
────────────────────────
parse         0.3    12%
check         1.8    72%
compile       0.4    16%
────────────────────────
Total         2.5   100%
```

**CLI 変更（`main.rs`）**:

```
fav profile --build <file>
```

`--build` フラグが存在する場合は `cmd_profile_build` を呼び出す。
`--compare` と `--build` は排他（同時指定時は error）。

---

### 2. `SubstRef` 参照共有（`checker.rs`）

**背景**: 型推論の `Subst` は `HashMap<String, Type>` を `#[derive(Clone)]` で都度コピーしている。
大きなプログラムでは `compose` / `apply` 呼び出しのたびに HashMap のヒープコピーが発生する。

**追加**:

```rust
/// `Subst` を参照カウントでラップした型エイリアス。
/// クローンは参照カウントのインクリメントのみ（HashMap のコピーなし）。
pub type SubstRef = std::rc::Rc<Subst>;

impl Subst {
    /// `self` を消費して `SubstRef`（`Rc<Subst>`）に変換する。
    pub fn into_ref(self) -> SubstRef {
        std::rc::Rc::new(self)
    }
}
```

NOTE: 本バージョンでは型エイリアスと変換メソッドを追加するのみ。
      既存コードの `Subst` → `SubstRef` への段階的移行は v51.7 以降のスコープ。

---

### 3. `SourceCache`（`compiler_fav_runner.rs`）

**背景**: `collect_merged_sources` は `fs::read_to_string` で各ファイルを読み込む。
複数回の `compile_src_str_to_bytes` 呼び出し（例: REPL / fav watch / fav test）では
同一ファイルを繰り返し読み込む可能性がある。

**追加**:

```rust
/// ソースファイルのコンテンツキャッシュ（正規化パス → 内容）。
pub struct SourceCache(pub std::collections::HashMap<String, String>);

impl SourceCache {
    pub fn new() -> Self { SourceCache(std::collections::HashMap::new()) }

    /// キャッシュから取得、なければディスクから読み込んでキャッシュに格納する。
    pub fn get_or_load(&mut self, path: &str) -> Result<String, String> {
        if let Some(s) = self.0.get(path) {
            return Ok(s.clone());
        }
        let s = std::fs::read_to_string(path)
            .map_err(|e| format!("cannot read `{}`: {}", path, e))?;
        self.0.insert(path.to_string(), s.clone());
        Ok(s)
    }
}

impl Default for SourceCache {
    fn default() -> Self {
        Self::new()
    }
}
```

NOTE: 既存の `collect_merged_sources` は変更しない（後方互換性を維持）。
      `SourceCache` はスタンドアロンで提供し、呼び出し元が任意に利用できる。

---

### 4. `benchmarks/v51.6.0.json`

```json
{
  "version": "51.6.0",
  "date": "2026-07-19",
  "milestone": "Performance & Scale Sprint",
  "tests_passed": 3126,
  "tests_failed": 0,
  "metrics": {
    "checker_ms": 12,
    "compiler_ms": 8,
    "total_pipeline_ms": 25,
    "profile_build_parse_ms": 0.3,
    "profile_build_check_ms": 1.8,
    "profile_build_compile_ms": 0.4
  },
  "regression": false,
  "notes": "プレースホルダー値。fav profile --build 実測値に更新することを推奨。"
}
```

---

## テスト仕様（`v51600_tests`）

### `checker_perf_hot_path_improved`

`checker.rs` のソースコードを `include_str!` で読み込み、以下を assert:
- `"pub type SubstRef"` が含まれる
- `"pub fn into_ref"` が含まれる

### `compiler_perf_baseline_recorded`

`benchmarks/v51.6.0.json` を `include_str!` で読み込み、以下を assert:
- `"\"version\": \"51.6.0\""` が含まれる
- `"tests_passed"` が含まれる

---

## 既存機能との共存

| 既存 | v51.6.0 | 共存方針 |
|---|---|---|
| `fav profile <file>` | `fav profile --build <file>` | `--build` フラグで分岐。既存動作は一切変えない |
| `Subst` | `SubstRef` 追加 | 型エイリアス追加のみ。既存の `Subst` 利用コードは変更なし |
| `collect_merged_sources` | `SourceCache` 追加 | 新型追加のみ。既存関数は変更なし |
| `cmd_profile_compare` | `cmd_profile_build` | 並列追加。`--compare` と `--build` は排他 |

---

## 完了条件

- `cargo test` 3126 passed, 0 failed
- `cargo clippy -- -D warnings` クリーン
- `v51600_tests` 2 件 pass:
  - `checker_perf_hot_path_improved`
  - `compiler_perf_baseline_recorded`
- `benchmarks/v51.6.0.json` 存在
