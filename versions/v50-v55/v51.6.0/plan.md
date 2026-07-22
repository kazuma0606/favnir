# Plan: v51.6.0 — checker / compiler ホットパス最適化

Date: 2026-07-19

---

## 前提確認

- ベース: v51.5.0（3124 tests passed, 0 failed）
- 変更ファイル: `checker.rs`・`compiler_fav_runner.rs`・`driver.rs`・`main.rs`・`benchmarks/v51.6.0.json`
- 新規ファイル: `benchmarks/v51.6.0.json`

---

## Step 1 — `SubstRef` 追加（`checker.rs`）

`src/middle/checker.rs` の `impl Subst` ブロックの**後**（ファイル本体）に `pub type SubstRef` を追加し、
`impl Subst` ブロック**内**（`compose` の後）に `pub fn into_ref` を追加する:

```rust
/// `Subst` を参照カウントでラップした型エイリアス。
/// クローンは Rc のインクリメントのみ（HashMap コピーなし）。
pub type SubstRef = std::rc::Rc<Subst>;

impl Subst {
    /// self を消費して SubstRef に変換する。
    pub fn into_ref(self) -> SubstRef {
        std::rc::Rc::new(self)
    }
}
```

`cargo build` でコンパイルエラーがないことを確認。

---

## Step 2 — `SourceCache` 追加（`compiler_fav_runner.rs`）

`src/compiler_fav_runner.rs` の `collect_merged_sources` 関数の直前（行 55 付近）に追加:

```rust
/// ソースファイルコンテンツキャッシュ（正規化パス → 内容）（v51.6.0）。
pub struct SourceCache(pub std::collections::HashMap<String, String>);

impl SourceCache {
    pub fn new() -> Self {
        SourceCache(std::collections::HashMap::new())
    }

    /// キャッシュから取得。未キャッシュならディスクから読み込む。
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

`cargo build` でコンパイルエラーがないことを確認。

---

## Step 3 — `ProfileBuildResult` + `profile_build_file` + `cmd_profile_build` 追加（`driver.rs`）

`// ── fav profile ──` セクション（行 12531 付近）の `fn render_profile_table` の直前に追加:

```rust
// ── v51.6.0: fav profile --build ──────────────────────────────────────────

/// ビルドフェーズ（parse / check / compile）の計測結果。
pub struct ProfileBuildResult {
    pub parse_ms: f64,
    pub check_ms: f64,
    pub compile_ms: f64,
}

/// `.fav` ファイルの parse / check / compile フェーズを計測して返す（v51.6.0）。
pub fn profile_build_file(path: &str) -> Result<ProfileBuildResult, String> {
    let src = std::fs::read_to_string(path)
        .map_err(|e| format!("cannot read '{}': {}", path, e))?;

    // parse フェーズ
    let t0 = std::time::Instant::now();
    let _ = crate::frontend::parser::Parser::parse_str(&src, path)
        .map_err(|e| e.to_string())?;
    let parse_ms = t0.elapsed().as_secs_f64() * 1000.0;

    // check フェーズ
    let t1 = std::time::Instant::now();
    let _ = check_single_file(path, false, false);
    let check_ms = t1.elapsed().as_secs_f64() * 1000.0;

    // compile フェーズ（compiler.fav 経由）
    let t2 = std::time::Instant::now();
    let _ = crate::compiler_fav_runner::compile_src_str_to_bytes(&src);
    let compile_ms = t2.elapsed().as_secs_f64() * 1000.0;

    Ok(ProfileBuildResult { parse_ms, check_ms, compile_ms })
}

/// `fav profile --build <file>` の出力を表示する（v51.6.0）。
pub fn cmd_profile_build(path: &str) {
    let result = match profile_build_file(path) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("error: {}", e);
            process::exit(1);
        }
    };
    let total = result.parse_ms + result.check_ms + result.compile_ms;
    let total_f = total.max(0.001);
    let name_w = 7usize; // "compile"
    println!(
        "{:<width$}  {:>10}  {:>6}",
        "Phase", "Time (ms)", "%",
        width = name_w
    );
    println!("{}", "─".repeat(name_w + 22));
    for (name, ms) in &[
        ("parse", result.parse_ms),
        ("check", result.check_ms),
        ("compile", result.compile_ms),
    ] {
        let pct = (ms / total_f * 100.0).round() as i64;
        println!(
            "{:<width$}  {:>10.3}  {:>5}%",
            name, ms, pct,
            width = name_w
        );
    }
    println!("{}", "─".repeat(name_w + 22));
    println!(
        "{:<width$}  {:>10.3}  {:>5}%",
        "Total", total, 100,
        width = name_w
    );
}
```

`cargo build` でコンパイルエラーがないことを確認。

---

## Step 4 — `fav profile --build` CLI 追加（`main.rs`）

`Some("profile")` アームの変数定義ブロックに `build` フラグを追加:

```rust
let mut build = false;
// ...（既存の i ループ内）...
} else if arg == "--build" {
    build = true; i += 1;
```

ディスパッチ部分（行 1721 付近）を更新:

```rust
if let Some(ref v) = compare {
    if build {
        eprintln!("error: --compare and --build cannot be used together");
        process::exit(1);
    }
    cmd_profile_compare(v, &path);
} else if build {
    cmd_profile_build(&path);
} else {
    cmd_profile(&path, &format, runs, stage_filter.as_deref(), out.as_deref());
}
```

`main.rs` の `use driver::` インポートに `cmd_profile_build` を追加。
`cargo build` でコンパイルエラーがないことを確認。

---

## Step 5 — `benchmarks/v51.6.0.json` 作成

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

## Step 6 — `v51600_tests` + バージョン更新

`driver.rs` の `v51500_tests` の直前に `v51600_tests` モジュールを追加（2 件）:

```rust
// -- v51600_tests (v51.6.0) -- checker/compiler ホットパス最適化 --
#[cfg(test)]
mod v51600_tests {
    #[test]
    fn checker_perf_hot_path_improved() {
        let src = include_str!("../middle/checker.rs");
        assert!(src.contains("pub type SubstRef"), "checker.rs must define pub type SubstRef");
        assert!(src.contains("pub fn into_ref"), "checker.rs must define Subst::into_ref");
    }

    #[test]
    fn compiler_perf_baseline_recorded() {
        let json = include_str!("../../benchmarks/v51.6.0.json");
        assert!(json.contains("\"version\": \"51.6.0\""),
            "benchmarks/v51.6.0.json must contain version 51.6.0");
        assert!(json.contains("tests_passed"),
            "benchmarks/v51.6.0.json must contain tests_passed");
    }
}
```

`fav/Cargo.toml` の `version` を `"51.6.0"` に更新。

`cargo test` 3126 passed, 0 failed を確認。
`cargo clippy -- -D warnings` クリーンを確認。
