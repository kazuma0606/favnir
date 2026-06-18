# v19.3.0 実装計画 — インクリメンタルコンパイル

## 実装順序

```
T1（incremental/ ディレクトリ作成）  ← 最初
T2（lib.rs にモジュール登録）        ← T1 完了後
T3（driver.rs 統合）                 ← T2 完了後
T4（Cargo.toml 確認）                ← T1 と並列可
T5（driver.rs テスト追加）           ← T3 完了後
T6（Cargo.toml バージョン）          ← T5 と並列可
T7（ドキュメント）                   ← T5 と並列可
```

---

## T1: `fav/src/incremental/` 作成

### `fingerprint.rs`

```rust
use sha2::{Digest, Sha256};
use std::path::Path;

/// ファイル内容の SHA-256 を小文字 hex 文字列で返す。
pub fn file_hash(path: &Path) -> Result<String, String> {
    let bytes = std::fs::read(path)
        .map_err(|e| format!("read error {}: {e}", path.display()))?;
    Ok(content_hash(&bytes))
}

/// バイト列の SHA-256 を小文字 hex 文字列で返す。
pub fn content_hash(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}
```

### `dep_graph.rs`

```rust
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct DepGraph {
    /// file_stem → 依存する file_stem のリスト（"pipeline" → ["utils"]）
    edges: HashMap<String, Vec<String>>,
}

impl DepGraph {
    pub fn new() -> Self { Self::default() }

    /// `from` が `to` に依存することを登録
    pub fn add_dep(&mut self, from: &str, to: &str) { ... }

    /// `file` が直接・推移的に依存するファイルをすべて返す
    pub fn transitive_deps(&self, file: &str) -> Vec<String> { ... }

    /// `changed` ファイルの変更で影響を受けるファイル（逆引き）
    pub fn affected_by(&self, changed: &str) -> Vec<String> { ... }
}

/// AST の `use` 宣言（`Program.uses`）から依存グラフを構築する。
/// rune import（`use json` 等のシングルセグメント）はスキップする。
pub fn build_dep_graph(program: &crate::ast::Program, source_stem: &str) -> DepGraph {
    let mut graph = DepGraph::new();
    for path in &program.uses {
        if path.len() >= 2 {
            // `use utils.{ format_date }` → path = ["utils", "format_date"]
            // first segment を依存ファイル名とみなす
            graph.add_dep(source_stem, &path[0]);
        }
    }
    graph
}
```

### `cache.rs`

```rust
use std::path::{Path, PathBuf};
use crate::backend::artifact::FvcArtifact;

pub struct IncrementalCache {
    root: PathBuf,  // ~/.fav/cache/<project-hash>/
}

impl IncrementalCache {
    pub fn new(project_root: &Path) -> Self {
        let project_hash = super::fingerprint::content_hash(
            project_root.to_string_lossy().as_bytes()
        );
        let root = dirs_or_home().join(".fav").join("cache").join(&project_hash);
        std::fs::create_dir_all(&root).ok();
        Self { root }
    }

    /// キャッシュの `.ir` ファイルが存在するか確認
    pub fn is_hit(&self, file_hash: &str) -> bool {
        self.ir_path(file_hash).exists()
    }

    /// キャッシュから FvcArtifact を読み込む
    pub fn read_artifact(&self, file_hash: &str) -> Result<FvcArtifact, String> {
        let bytes = std::fs::read(self.ir_path(file_hash))
            .map_err(|e| format!("cache read error: {e}"))?;
        serde_json::from_slice(&bytes)
            .map_err(|e| format!("cache deserialize error: {e}"))
    }

    /// FvcArtifact をキャッシュに書き込む
    pub fn write_artifact(&self, file_hash: &str, artifact: &FvcArtifact) -> Result<(), String> {
        let bytes = serde_json::to_vec(artifact)
            .map_err(|e| format!("cache serialize error: {e}"))?;
        std::fs::write(self.ir_path(file_hash), bytes)
            .map_err(|e| format!("cache write error: {e}"))
    }

    /// キャッシュエントリを削除（無効化）
    pub fn invalidate(&self, file_hash: &str) {
        std::fs::remove_file(self.ir_path(file_hash)).ok();
    }

    fn ir_path(&self, file_hash: &str) -> PathBuf {
        self.root.join(format!("{file_hash}.ir"))
    }
}
```

**注意**: `FvcArtifact` が `serde::Serialize + Deserialize` を derive していない場合、
先に `#[derive(Serialize, Deserialize)]` を追加する。

### `mod.rs`

```rust
pub mod cache;
pub mod dep_graph;
pub mod fingerprint;
```

---

## T2: `fav/src/lib.rs` — モジュール登録

```rust
pub mod incremental;
```

既存の `pub mod` 宣言の末尾に追加。

---

## T3: `fav/src/driver.rs` — `cmd_build` / `cmd_check` 統合

### `--no-cache` / `--explain-cache` フラグ

`cmd_build` / `cmd_run` の引数に追加:

```rust
pub fn cmd_build(file: Option<&str>, out: Option<&str>, target: Option<&str>,
                 no_cache: bool, explain_cache: bool) { ... }
```

または環境変数 `FAV_NO_CACHE=1` でも無効化できるようにする。

### キャッシュ統合ロジック

`cmd_build` の `"fvc"` ターゲット内:

```rust
let file_hash = incremental::fingerprint::file_hash(Path::new(&path))
    .unwrap_or_default();
let cache = incremental::cache::IncrementalCache::new(
    Path::new(&path).parent().unwrap_or(Path::new("."))
);

if !no_cache && !file_hash.is_empty() && cache.is_hit(&file_hash) {
    if explain_cache {
        eprintln!("[cache HIT]  {path}  ({file_hash:.8}...)");
    }
    // キャッシュから読み込み → そのまま書き出し
    if let Ok(cached) = cache.read_artifact(&file_hash) {
        write_artifact_to_path(&cached, &out_path).unwrap_or_else(...);
        println!("built {} (cached)", out_path.display());
        return;
    }
}

// キャッシュミス → 通常コンパイル
if explain_cache {
    eprintln!("[cache MISS] {path}  ({file_hash:.8}...)");
}
let artifact = build_artifact(&program);
cache.write_artifact(&file_hash, &artifact).ok();
write_artifact_to_path(&artifact, &out_path).unwrap_or_else(...);
```

---

## T4: `fav/Cargo.toml` 確認事項

- `sha2 = "0.10"` — **既存**（追加不要）
- `serde = { version = "1", features = ["derive"] }` — **既存**（追加不要）
- `serde_json = "1"` — **既存**（追加不要）

`FvcArtifact` に `#[derive(Serialize, Deserialize)]` が付いているか確認してから実装する。

---

## T5: `v193000_tests` 追加手順

`v192000_tests::version_is_19_2_0` に `#[ignore]` を追加後、末尾に追記:

```rust
mod v193000_tests {
    #[test]
    fn version_is_19_3_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("19.3.0"));
    }

    #[test]
    fn cache_creates_on_first_build() {
        // tempdir に main.fav を書き込み
        // IncrementalCache::new(dir) でキャッシュを初期化
        // build_artifact → write_artifact
        // is_hit が true になることを確認
    }

    #[test]
    fn cache_hits_on_second_build() {
        // 同上の手順で初回ビルド
        // 2 回目: is_hit が true → read_artifact が Ok
    }

    #[test]
    fn cache_invalidates_on_change() {
        // ファイル内容を変更 → 新しい file_hash を計算
        // 旧 hash のキャッシュは is_hit = false
    }

    #[test]
    fn dep_graph_propagates() {
        // "pipeline.fav" が "utils" を use するプログラムをパース
        // build_dep_graph → affected_by("utils") に "pipeline" が含まれることを確認
    }
}
```

---

## 注意点

### `FvcArtifact` の Serialize 対応

`artifact.rs` に `#[derive(serde::Serialize, serde::Deserialize)]` が付いていない場合、
`FvcArtifact` / `FvcFunction` / `FvcGlobal` / `TypeMeta` / `FieldMeta` 等すべての構造体に追加が必要。
追加前に Grep で対象を洗い出してから一括追加する。

### ホームディレクトリ取得

`std::env::var("HOME")` または `dirs` クレートで取得する。
`dirs` は新規依存になるため、まず `std::env::var("HOME").or(std::env::var("USERPROFILE"))` で対処する。

### キャッシュパスの環境変数オーバーライド

`FAV_CACHE_DIR` 環境変数でキャッシュ先を上書きできるようにする（テストで tempdir を指定するため）。

### テスト内での `FvcArtifact::fn_idx_by_name` の確認

テスト内で `artifact.fn_idx_by_name("main")` を使う場合、既存テストと同パターンを踏襲する。
