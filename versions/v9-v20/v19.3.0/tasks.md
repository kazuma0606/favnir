# v19.3.0 — インクリメンタルコンパイル タスク

## ステータス: COMPLETE

---

## タスク一覧

### T1: `fav/src/incremental/` ディレクトリ作成

**1-A: `FvcArtifact` の Serialize 対応（事前確認）**

- [x] `fav/src/backend/artifact.rs` を Grep して `#[derive(...)]` を確認
- [x] `FvcArtifact` / `FvcFunction` / `FvcGlobal` に `Serialize, Deserialize` が付いていない場合は追加
- [x] `TypeMeta` / `FieldMeta`（`ir.rs`）にも必要に応じて追加
- [x] `Constant`（`codegen.rs`）にも必要に応じて追加
- [x] `cargo build` でコンパイルエラーが 0 であることを確認

**1-B: `fingerprint.rs` 作成**

- [x] `fav/src/incremental/fingerprint.rs` を新規作成:
  - `pub fn file_hash(path: &std::path::Path) -> Result<String, String>`
  - `pub fn content_hash(bytes: &[u8]) -> String`
  - `sha2::{Digest, Sha256}` を使用（既存依存）

**1-C: `dep_graph.rs` 作成**

- [x] `fav/src/incremental/dep_graph.rs` を新規作成:
  - `pub struct DepGraph { edges: HashMap<String, Vec<String>> }`
  - `impl DepGraph { pub fn add_dep / transitive_deps / affected_by }`
  - `pub fn build_dep_graph(program: &Program, source_stem: &str) -> DepGraph`
  - `Program.uses: Vec<Vec<String>>` を走査（`use utils.{ f }` → `["utils", "f"]`）
  - rune import（単一セグメント `use json` 等）はスキップ

**1-D: `cache.rs` 作成**

- [x] `fav/src/incremental/cache.rs` を新規作成:
  - `pub struct IncrementalCache { root: PathBuf }`
  - `pub fn new(project_root: &Path) -> Self`:
    - `FAV_CACHE_DIR` env var → 指定値を root として使用
    - 未設定 → `$HOME/.fav/cache/<project-hash>/`（HOME は `HOME` / `USERPROFILE` env var）
    - `std::fs::create_dir_all(&root).ok()` でディレクトリを確保
  - `pub fn is_hit(&self, file_hash: &str) -> bool`
  - `pub fn read_artifact(&self, file_hash: &str) -> Result<FvcArtifact, String>`
    - `serde_json::from_slice` でデシリアライズ
  - `pub fn write_artifact(&self, file_hash: &str, artifact: &FvcArtifact) -> Result<(), String>`
    - `serde_json::to_vec` でシリアライズ → `.ir` ファイルに書き込み
  - `pub fn invalidate(&self, file_hash: &str)` — `.ir` ファイルを削除
  - `fn ir_path(&self, file_hash: &str) -> PathBuf` — `{root}/{hash}.ir`

**1-E: `mod.rs` 作成**

- [x] `fav/src/incremental/mod.rs` を新規作成:
  ```rust
  pub mod cache;
  pub mod dep_graph;
  pub mod fingerprint;
  ```

---

### T2: `fav/src/lib.rs` — モジュール登録

- [x] 既存の `pub mod` 宣言の末尾に追記:
  ```rust
  pub mod incremental;
  ```
- [x] `cargo build` でコンパイルエラーが 0 であることを確認

---

### T3: `fav/src/driver.rs` — キャッシュ統合

**3-A: `cmd_build` の `"fvc"` ターゲットにキャッシュ統合**

- [x] `use crate::incremental;` を driver.rs の use 宣言に追加
- [x] `cmd_build` の `"fvc"` ターゲット内:
  1. `file_hash` を `incremental::fingerprint::file_hash(path)` で取得
  2. `IncrementalCache::new(project_root)` でキャッシュ初期化
  3. `cache.is_hit(&file_hash)` → true なら `read_artifact` → 書き出し → return
  4. false なら従来通りコンパイル → `cache.write_artifact` → 書き出し

**3-B: `--no-cache` / `--explain-cache` CLI フラグ追加**

- [x] `fav build --no-cache` → `FAV_NO_CACHE=1` env var か CLI 引数でキャッシュ無効化
- [x] `fav build --explain-cache` → キャッシュヒット / ミスを stderr に出力

---

### T4: `fav/Cargo.toml` 確認

- [x] `sha2 = "0.10"` が既存依存にあることを確認（追加不要）
- [x] `serde / serde_json` が既存依存にあることを確認（追加不要）

---

### T5: `fav/src/driver.rs` — `v193000_tests` 追加

- [x] `v192000_tests::version_is_19_2_0` に `#[ignore]` を追加
- [x] `v193000_tests` モジュールを追加（5件）:

  ```rust
  // ── v193000_tests (v19.3.0) — インクリメンタルコンパイル ──────────────────────
  #[cfg(test)]
  mod v193000_tests {
      use crate::incremental::{cache::IncrementalCache, fingerprint, dep_graph};
      use crate::frontend::parser::Parser;

      #[test]
      fn version_is_19_3_0() {
          let cargo = include_str!("../Cargo.toml");
          assert!(cargo.contains("19.3.0"), "Cargo.toml should have version 19.3.0");
      }

      #[test]
      fn cache_creates_on_first_build() {
          let dir = tempfile::tempdir().expect("tempdir");
          std::env::set_var("FAV_CACHE_DIR", dir.path().to_str().unwrap());
          let src = "fn main() -> Int { 1 }";
          let prog = Parser::parse_str(src, "test.fav").expect("parse");
          let artifact = super::build_artifact(&prog);
          let hash = fingerprint::content_hash(src.as_bytes());
          let cache = IncrementalCache::new(dir.path());
          cache.write_artifact(&hash, &artifact).expect("write");
          assert!(cache.is_hit(&hash), "cache should hit after write");
      }

      #[test]
      fn cache_hits_on_second_build() {
          let dir = tempfile::tempdir().expect("tempdir");
          std::env::set_var("FAV_CACHE_DIR", dir.path().to_str().unwrap());
          let src = "fn main() -> Int { 2 }";
          let prog = Parser::parse_str(src, "test.fav").expect("parse");
          let artifact = super::build_artifact(&prog);
          let hash = fingerprint::content_hash(src.as_bytes());
          let cache = IncrementalCache::new(dir.path());
          // 1 回目: 書き込み
          cache.write_artifact(&hash, &artifact).expect("write");
          // 2 回目: キャッシュヒット確認
          assert!(cache.is_hit(&hash), "cache should hit on second build");
          let read_back = cache.read_artifact(&hash).expect("read");
          assert_eq!(read_back.functions.len(), artifact.functions.len());
      }

      #[test]
      fn cache_invalidates_on_change() {
          let dir = tempfile::tempdir().expect("tempdir");
          std::env::set_var("FAV_CACHE_DIR", dir.path().to_str().unwrap());
          let src_v1 = "fn main() -> Int { 3 }";
          let src_v2 = "fn main() -> Int { 4 }";
          let prog = Parser::parse_str(src_v1, "test.fav").expect("parse");
          let artifact = super::build_artifact(&prog);
          let hash_v1 = fingerprint::content_hash(src_v1.as_bytes());
          let hash_v2 = fingerprint::content_hash(src_v2.as_bytes());
          let cache = IncrementalCache::new(dir.path());
          cache.write_artifact(&hash_v1, &artifact).expect("write");
          // v1 はキャッシュヒット、v2 はキャッシュミス
          assert!(cache.is_hit(&hash_v1), "v1 should hit");
          assert!(!cache.is_hit(&hash_v2), "v2 should miss (different content)");
      }

      #[test]
      fn dep_graph_propagates() {
          // pipeline.fav が utils を use → utils 変更で pipeline も影響を受ける
          let src = "use utils.{ format_date }\nfn main() -> Int { 1 }";
          let prog = Parser::parse_str(src, "pipeline.fav").expect("parse");
          let graph = dep_graph::build_dep_graph(&prog, "pipeline");
          let affected = graph.affected_by("utils");
          assert!(
              affected.contains(&"pipeline".to_string()),
              "pipeline should be affected by utils change, got: {:?}",
              affected
          );
      }
  }
  ```

---

### T6: `fav/Cargo.toml` バージョン更新

- [x] `version = "19.2.0"` → `"19.3.0"` に変更

---

### T7: `site/content/docs/tools/incremental.mdx`（新規作成）

- [x] インクリメンタルコンパイルの概要
- [x] `--no-cache` / `--explain-cache` フラグの説明
- [x] キャッシュ構造（`~/.fav/cache/`）の解説
- [x] 依存グラフ追跡の仕組み
- [x] `FAV_CACHE_DIR` / `FAV_NO_CACHE` 環境変数の説明

---

## テスト（v193000_tests、5件）

| テスト名 | 内容 |
|---|---|
| `version_is_19_3_0` | Cargo.toml に `"19.3.0"` が含まれる |
| `cache_creates_on_first_build` | 初回ビルドでキャッシュが生成される（`is_hit` が true） |
| `cache_hits_on_second_build` | 2 回目でキャッシュヒット（`read_artifact` が Ok） |
| `cache_invalidates_on_change` | ファイル変更でキャッシュが無効化（別 hash → `is_hit` が false） |
| `dep_graph_propagates` | A が B を use → B 変更で A も影響を受ける |

---

## 完了条件チェックリスト

- [x] `FvcArtifact` が `serde::Serialize + Deserialize` を持つ
- [x] `fingerprint::file_hash` / `content_hash` が実装される
- [x] `dep_graph::build_dep_graph` / `affected_by` が実装される
- [x] `cache::IncrementalCache` の全メソッドが実装される
- [x] `FAV_CACHE_DIR` env var でキャッシュ先を上書きできる
- [x] `cmd_build --no-cache` でキャッシュを無効化できる
- [x] `cargo test v193000` — 5/5 PASS
- [x] `cargo test` — リグレッションなし
- [x] `site/content/docs/tools/incremental.mdx` が存在する

---

## 優先度

```
T1-A（FvcArtifact Serialize 対応）  ← 最初（キャッシュのシリアライズに必要）
T1-B（fingerprint.rs）              ← T1-A 完了後
T1-C（dep_graph.rs）                ← T1-A と並列可
T1-D（cache.rs）                    ← T1-B, T1-C 完了後
T1-E（mod.rs）                      ← T1-D と同時
T2（lib.rs モジュール登録）         ← T1 完了後
T3（driver.rs 統合）                ← T2 完了後
T4（Cargo.toml 確認）               ← T1 と並列可
T5（v193000_tests）                 ← T3 完了後
T6（Cargo.toml バージョン）         ← T5 と並列可
T7（ドキュメント）                  ← T5 と並列可
```

---

## 重要な技術ノート

### `FvcArtifact` の Serialize 確認コマンド

```bash
grep -n "derive" fav/src/backend/artifact.rs | head -20
grep -n "derive" fav/src/backend/codegen.rs | grep -i "const\|Constant" | head -10
```

### テスト内の `FAV_CACHE_DIR` 設定

`std::env::set_var` はスレッド安全ではないため、テストが並列実行される場合は `serial_test` クレートや
`tempfile::tempdir()` + `FAV_CACHE_DIR` の組み合わせで隔離する。
`cargo test v193000 -- --test-threads=1` でシリアル実行することで競合を避けられる。

### `Program.uses` の構造

`use utils.{ format_date }` → `program.uses` に `["utils", "format_date"]` が入る（実際の構造は Grep で確認）。
`use json` → `["json"]`（単一セグメント = rune import、dep_graph には含めない）。
