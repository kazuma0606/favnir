# v19.3.0 Spec — インクリメンタルコンパイル

## 概要

変更されたファイルのみを再コンパイルする。
現在は毎回全ファイルを再コンパイル（大きなプロジェクトで遅い）。
SHA-256 コンテンツハッシュによるフィンガープリントと依存グラフ追跡で
キャッシュヒット時の再コンパイルをスキップする。

**テーマ**: Production Performance シリーズ第3弾

---

## 動機

```
現在:
  fav build src/pipeline.fav   → 常に全ファイルを再コンパイル（50 ファイルで 10 秒）

v19.3.0 以降:
  fav build src/pipeline.fav   → 変更ファイルのみ再コンパイル（0.3〜1.2 秒）
```

---

## キャッシュ構造

```
~/.fav/cache/
  <project-content-hash>/
    <file-content-hash>.ast     # パース済み AST（bincode シリアライズ）
    <file-content-hash>.types   # 型チェック結果
    <file-content-hash>.ir      # コンパイル済み IR
```

- `project-content-hash`: プロジェクトルートパスの SHA-256
- `file-content-hash`: ファイル内容の SHA-256

---

## 依存グラフ追跡

```
src/
  pipeline.fav  ← use utils.{ format_date }   → utils.fav に依存
  utils.fav     ← use json                    → rune "json" に依存
  types.fav                                   → 他に依存しない

依存グラフ:
  pipeline.fav → utils.fav → rune:json
  types.fav（独立）

変更検出:
  utils.fav が変更 → utils.fav と pipeline.fav を再コンパイル
  types.fav が変更 → types.fav のみ再コンパイル
```

---

## ビルド時間の改善（目標）

| シナリオ | 現在 | v19.3.0 以降 |
|---|---|---|
| 初回ビルド（50 ファイル） | 10.0 秒 | 10.0 秒（同じ） |
| 変更なし 2 回目 | 10.0 秒 | 0.3 秒 |
| 1 ファイル変更 | 10.0 秒 | 1.2 秒 |

---

## 実装内容

### T1: `fav/src/incremental/` ディレクトリ作成

#### `fingerprint.rs`

- `pub fn file_hash(path: &Path) -> Result<String, String>` — ファイル内容の SHA-256 を hex 文字列で返す
- `pub fn content_hash(content: &[u8]) -> String` — バイト列の SHA-256

#### `dep_graph.rs`

- `pub struct DepGraph { edges: HashMap<String, Vec<String>> }` — `file → [depends_on]`
- `pub fn build_dep_graph(program: &Program) -> DepGraph` — AST の `use` 宣言から依存を収集
- `pub fn transitive_deps(graph: &DepGraph, file: &str) -> Vec<String>` — 推移的依存を列挙
- `pub fn affected_by(graph: &DepGraph, changed: &str) -> Vec<String>` — 変更ファイルの逆依存

#### `cache.rs`

- `pub struct IncrementalCache { root: PathBuf }` — キャッシュルート（`~/.fav/cache/`）
- `pub fn new(project_root: &Path) -> Self` — project hash をキャッシュルートサブディレクトリに使用
- `pub fn is_hit(&self, file_hash: &str) -> bool` — `.ir` ファイルの存在確認
- `pub fn read_artifact(&self, file_hash: &str) -> Result<FvcArtifact, String>` — キャッシュから読み込み
- `pub fn write_artifact(&self, file_hash: &str, artifact: &FvcArtifact) -> Result<(), String>` — キャッシュに書き込み
- `pub fn invalidate(&self, file_hash: &str)` — キャッシュエントリを削除

#### `mod.rs`

```rust
pub mod cache;
pub mod dep_graph;
pub mod fingerprint;
```

### T2: `fav/src/lib.rs` — `incremental` モジュール登録

```rust
pub mod incremental;
```

### T3: `fav/src/driver.rs` — インクリメンタルキャッシュ統合

- `cmd_build` / `cmd_check` に以下を追加:
  1. `IncrementalCache::new(project_root)` でキャッシュを初期化
  2. 入力ファイルの `file_hash` を計算
  3. `cache.is_hit(file_hash)` → `true` ならキャッシュから `FvcArtifact` を読み込み
  4. `false` なら通常通りコンパイルし、結果を `cache.write_artifact(file_hash, &artifact)` で保存

- `--no-cache` フラグ: `INCREMENTAL_CACHE=0` env var でも無効化可能
- `--explain-cache` フラグ: キャッシュヒット / ミスの詳細を stderr に出力

### T4: `fav/Cargo.toml` — sha2 依存確認

- `sha2 = "0.10"` は既存依存 → 追加不要
- `serde / serde_json` は既存 → キャッシュシリアライズに使用可能

### T5: `fav/src/driver.rs` — `v193000_tests` 追加

- `v192000_tests::version_is_19_2_0` に `#[ignore]` を追加
- `v193000_tests` モジュール（5件）

### T6: `fav/Cargo.toml` バージョン更新

- `version = "19.2.0"` → `"19.3.0"`

### T7: `site/content/docs/tools/incremental.mdx`（新規）

- インクリメンタルコンパイルの使い方ガイド
- `--no-cache` / `--explain-cache` フラグの説明
- キャッシュ構造の解説

---

## テスト（v193000_tests、5件）

| テスト名 | 内容 |
|---|---|
| `version_is_19_3_0` | Cargo.toml に `"19.3.0"` が含まれる |
| `cache_creates_on_first_build` | 初回ビルドでキャッシュファイルが生成される |
| `cache_hits_on_second_build` | 変更なし 2 回目でキャッシュヒット（`is_hit` が true） |
| `cache_invalidates_on_change` | ファイル変更でキャッシュが無効化される |
| `dep_graph_propagates` | A が B を use → B 変更で A も再コンパイル対象になる |

---

## 完了条件

- [ ] `fingerprint.rs` — `file_hash` / `content_hash` が実装される
- [ ] `dep_graph.rs` — `build_dep_graph` / `affected_by` が実装される
- [ ] `cache.rs` — `is_hit` / `read_artifact` / `write_artifact` / `invalidate` が実装される
- [ ] 初回ビルドでキャッシュファイルが `~/.fav/cache/` に生成される
- [ ] 変更なし 2 回目ビルドがキャッシュヒットする
- [ ] ファイル変更でキャッシュが無効化される
- [ ] 依存グラフが `use` 宣言から正しく構築される
- [ ] `--no-cache` で完全再ビルドが強制できる
- [ ] `cargo test v193000` — 5/5 PASS
- [ ] `cargo test` — リグレッションなし
- [ ] `site/content/docs/tools/incremental.mdx` が存在する

---

## 技術ノート

### sha2 クレート

`sha2 = "0.10"` は既存依存（ECDSA 署名等で使用）。`use sha2::{Sha256, Digest}` でそのまま使える。

### キャッシュのシリアライズ形式

`FvcArtifact` の `serde::Serialize` 実装状況を確認してから実装する。
`#[derive(Serialize, Deserialize)]` が付いていれば `serde_json` でキャッシュ可能。
ない場合は `bincode` クレートを追加するか、`serde` を derive する。

### `use` 宣言からの依存収集

`Program.uses: Vec<Vec<String>>` を走査する（`[["utils", "format_date"]]` → `utils.fav` 依存）。
rune import（`use json`）はグラフに含めない（ファイル依存ではないため）。

### キャッシュ無効化の粒度

v19.3.0 では **ファイル単位**でキャッシュする（関数単位は v19.5 以降）。
ファイル内容が 1 文字でも変わればハッシュが変わり、キャッシュミスになる。
