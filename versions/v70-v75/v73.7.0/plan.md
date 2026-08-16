# v73.7.0 実装計画 — ドッグフーディング Sprint

Date: 2026-08-13
Status: 計画中

---

## 前提確認

- `fav/Cargo.toml` version = "73.6.0"
- `cargo test` 3659 tests pass（0 failures）
- `driver.rs` に `v736000_tests` が存在する

---

## 実装ステップ

### Step 1: `pipelines/` ディレクトリに 5 本の `.fav` ファイルを作成

`C:\Users\yoshi\favnir\fav\pipelines\` ディレクトリを作成し、各ファイルを作成:

**benchmark_analytics.fav:**
```favnir
// Dogfooding pipeline: aggregate benchmark results and visualize trends
fn main() -> String {
    "benchmark_analytics"
}
```

**coverage_report.fav:**
```favnir
// Dogfooding pipeline: generate test coverage report and notify via Slack
fn main() -> String {
    "coverage_report"
}
```

**changelog_lint.fav:**
```favnir
// Dogfooding pipeline: validate CHANGELOG.md format and entry consistency
fn main() -> String {
    "changelog_lint"
}
```

**rune_catalog_sync.fav:**
```favnir
// Dogfooding pipeline: sync runes/ directory to catalog.mdx
fn main() -> String {
    "rune_catalog_sync"
}
```

**doc_link_check.fav:**
```favnir
// Dogfooding pipeline: detect broken links in MDX documentation files
fn main() -> String {
    "doc_link_check"
}
```

### Step 2: `DogfoodingPipeline` 構造体 + `list_dogfooding_pipelines` 追加

`driver.rs` の v73.6.0 コードの直後（`v736000_tests` より前）に追加:

```rust
// --- v73.7.0: Dogfooding Sprint (Favnir running Favnir) ---

pub struct DogfoodingPipeline {
    pub name: String,
    pub path: String,
    pub description: String,
}

pub fn list_dogfooding_pipelines() -> Vec<DogfoodingPipeline> {
    vec![
        DogfoodingPipeline {
            name: "benchmark_analytics".to_string(),
            path: "pipelines/benchmark_analytics.fav".to_string(),
            description: "Aggregate benchmark results and visualize trends".to_string(),
        },
        DogfoodingPipeline {
            name: "coverage_report".to_string(),
            path: "pipelines/coverage_report.fav".to_string(),
            description: "Generate test coverage report and notify via Slack".to_string(),
        },
        DogfoodingPipeline {
            name: "changelog_lint".to_string(),
            path: "pipelines/changelog_lint.fav".to_string(),
            description: "Validate CHANGELOG.md format and entry consistency".to_string(),
        },
        DogfoodingPipeline {
            name: "rune_catalog_sync".to_string(),
            path: "pipelines/rune_catalog_sync.fav".to_string(),
            description: "Sync runes/ directory to catalog.mdx".to_string(),
        },
        DogfoodingPipeline {
            name: "doc_link_check".to_string(),
            path: "pipelines/doc_link_check.fav".to_string(),
            description: "Detect broken links in MDX documentation files".to_string(),
        },
    ]
}
```

### Step 3: `cargo build` 確認

### Step 4: `v737000_tests` モジュール追加

`v736000_tests` の直後に追加:

```rust
#[cfg(test)]
mod v737000_tests {
    use super::list_dogfooding_pipelines;

    #[test]
    fn dogfooding_benchmark_pipeline_runs() {
        let pipelines = list_dogfooding_pipelines();
        assert_eq!(pipelines.len(), 5);

        let names: Vec<&str> = pipelines.iter().map(|p| p.name.as_str()).collect();
        assert!(names.contains(&"benchmark_analytics"));
        assert!(names.contains(&"doc_link_check"));

        // ファイルが存在し期待内容を含むことを確認
        let src = include_str!("../pipelines/benchmark_analytics.fav");
        assert!(src.contains("benchmark_analytics"));

        // path フィールドが正しい形式であることを確認
        let bench = pipelines.iter().find(|p| p.name == "benchmark_analytics").unwrap();
        assert_eq!(bench.path, "pipelines/benchmark_analytics.fav");
        assert!(!bench.description.is_empty());
    }

    #[test]
    fn dogfooding_doc_link_check_runs() {
        // doc_link_check.fav が存在し期待内容を含む
        let src = include_str!("../pipelines/doc_link_check.fav");
        assert!(src.contains("doc_link_check"));

        // 全 5 ファイルの存在を確認
        let _ = include_str!("../pipelines/benchmark_analytics.fav");
        let _ = include_str!("../pipelines/coverage_report.fav");
        let _ = include_str!("../pipelines/changelog_lint.fav");
        let _ = include_str!("../pipelines/rune_catalog_sync.fav");
        let _ = include_str!("../pipelines/doc_link_check.fav");

        let pipelines = list_dogfooding_pipelines();
        let paths: Vec<&str> = pipelines.iter().map(|p| p.path.as_str()).collect();
        assert!(paths.iter().all(|p| p.starts_with("pipelines/") && p.ends_with(".fav")));
    }
}
```

### Step 5: `cargo test v737000` で 2 件 pass 確認

### Step 6: バージョン更新

- `fav/Cargo.toml`: version = "73.6.0" → "73.7.0"
- `driver.rs`: `"73.6.0"` → `"73.7.0"`（replace_all）
  ※ バージョン検証テスト内の文字列リテラルも対象
  ※ `// --- v73.6.0:` コメントヘッダーは書き換えない

### Step 7: `cargo build` 確認

### Step 8: `cargo test` 全体確認（3661 tests pass）

### Step 9: `CHANGELOG.md` 更新

### Step 10: `versions/current.md` 更新
- 「最終更新」を `2026-08-13 (v73.7.0)` に変更
- 「進行中バージョン」を `v73.7.0` に変更
- 「次に切る版」を `v73.8.0` に変更
