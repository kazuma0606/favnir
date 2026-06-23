# v22.1.0 仕様書 — Checkpoint / Resume（パイプライン永続化）

## 概要

長時間実行パイプラインの中断・再開を安全に行う機能を追加する。
`#[checkpoint]` アノテーションを stage に付けると、その stage の出力を `.fav_checkpoint/` ディレクトリに永続化する。
再実行時に `--resume <dir>` を指定すると、checkpoint 済み stage をスキップして次の stage から再開できる。

**テーマ**: 「失敗しても再開できるパイプライン」

---

## ロードマップ完了条件との対応

v22.1 は Distributed Scale ロードマップ（v22.1〜v23.0）の第一弾。
v23.0 の完了条件①「checkpoint 付きパイプラインが失敗後に再開できる」に対応。

---

## 機能仕様

### `#[checkpoint]` アノテーション

```favnir
#[checkpoint]
stage ProcessBatch: List<Row> -> List<Result> = |rows| { ... }

seq LongRunning = Load |> ProcessBatch |> Save
```

- `stage`（`TrfDef`）にのみ付与可能。`seq`（`FlwDef`）には付与しない。
- アノテーションがない stage は通常通り実行される。
- 複数の stage に付与可能。

### CLI フラグ

```bash
# checkpoint を保存しながら実行
fav run --checkpoint-dir /tmp/ckpt pipeline.fav

# 中断後に再開（checkpoint 済み stage をスキップ）
fav run --resume /tmp/ckpt pipeline.fav

# 両方指定（再開しつつ新しい checkpoint も保存）
fav run --checkpoint-dir /tmp/ckpt2 --resume /tmp/ckpt pipeline.fav
```

| フラグ | 型 | 説明 |
|---|---|---|
| `--checkpoint-dir <dir>` | `Option<String>` | 実行後に checkpoint ファイルを保存するディレクトリ |
| `--resume <dir>` | `Option<String>` | checkpoint ファイルを読み込んで該当 stage をスキップするディレクトリ |

### Checkpoint ファイル形式

```
<checkpoint-dir>/
  <stage_name>.ckpt     ← stage 出力のバイト列（v22.1.0 ではシリアライズ形式未定義）
```

- ファイル名: `{stage_name}.ckpt`（スペースやスラッシュはアンダースコアに変換）
- `<stage_name>.ckpt.sha`（SHA-256 チェックサム）は **v22.1.0 では作成しない**（v22.3 以降対応）
- `--resume` 時にチェックサム検証は行わない（v22.1.0 のシンプル実装）

---

## アーキテクチャ

### 追加する AST フィールド

`fav/src/ast.rs` の `TrfDef` に `checkpoint: bool` を追加:

```rust
pub struct TrfDef {
    // ...既存フィールド...
    pub stateful: bool,   // v19.1.0
    pub arrow: bool,      // v19.5.0
    pub checkpoint: bool, // v22.1.0: #[checkpoint] annotation
    pub span: Span,
}
```

### パーサー変更（`frontend/parser.rs`）

`#[stateful]` / `#[arrow]` を解析する既存ロジック（`frontend/parser.rs` の `parse_stateful_annotation` / `parse_arrow_annotation`）と同様のパターンで `#[checkpoint]` を解析:

```rust
fn parse_checkpoint_annotation(tokens: &[Token], pos: &mut usize) -> bool {
    // peek "[checkpoint]" → true; それ以外 → false
}
```

`parse_trf_def` で `stateful` / `arrow` を検出するループに `checkpoint` を追加。

### VM スレッドローカル（`backend/vm.rs`）

```rust
thread_local! {
    static CHECKPOINT_DIR:    RefCell<Option<PathBuf>>      = RefCell::new(None);
    static RESUME_DIR:        RefCell<Option<PathBuf>>      = RefCell::new(None);
    static CHECKPOINT_STAGES: RefCell<HashSet<String>>      = RefCell::new(HashSet::new());
}

pub fn set_checkpoint_dir(dir: Option<&str>)
pub fn set_resume_dir(dir: Option<&str>)
pub fn set_checkpoint_stages(names: HashSet<String>)
```

### `cmd_run` 拡張（`driver.rs`）

```rust
pub fn cmd_run(
    file: Option<&str>,
    db_url: Option<&str>,
    legacy: bool,
    verbose: bool,
    trace: bool,
    no_tap: bool,
    legacy_value_repr: bool,
    explain_pushdown: bool,
    checkpoint_dir: Option<&str>,  // v22.1.0 追加
    resume_dir: Option<&str>,      // v22.1.0 追加
)
```

`cmd_run` 内で:
1. Rust パーサーでソースを解析して `#[checkpoint]` stage 名を収集
2. `vm::set_checkpoint_stages(names)` を呼ぶ
3. `vm::set_checkpoint_dir(checkpoint_dir)` を呼ぶ
4. `vm::set_resume_dir(resume_dir)` を呼ぶ
5. 通常通り `run_with_favnir_pipeline` を呼ぶ

### Checkpoint ヘルパー関数（`driver.rs`）

```rust
pub fn stage_checkpoint_path(dir: &Path, stage_name: &str) -> PathBuf
pub fn write_stage_checkpoint(dir: &Path, stage_name: &str, data: &[u8]) -> std::io::Result<()>
pub fn read_stage_checkpoint(dir: &Path, stage_name: &str) -> Option<Vec<u8>>
```

### VM への hook（`backend/vm.rs`）

`call_builtin` に `"__checkpoint_wrap"` を追加:

```
引数: (stage_name: Str)
動作（v22.1.0 scope — lookup のみ）:
  1. RESUME_DIR に {stage_name}.ckpt が存在するか確認
  2. 存在する → read_checkpoint_bytes でデータを返す（VMValue::List等のシリアライズはv22.3以降）
  3. 存在しない → VMValue::Bool(false) を返す（"miss" シグナル）

※ stage_fn の実行ラップ（完全なwrapperとして動作させること）はスコープ外。
  v22.1.0 では lookup + hit/miss の判定のみ実装する。
```

> **注意（既存 `CheckpointBackend` との関係）**: vm.rs には `fav checkpoint` コマンド用の `CheckpointBackend` enum（File/Sqlite）が既に存在する。v22.1.0 の `CHECKPOINT_DIR` / `RESUME_DIR` スレッドローカルはパイプライン stage の再開用であり、`fav checkpoint` の増分チェックポイント（SQLite 管理）とは別個の機能として共存させる。命名衝突を避けるため、新規スレッドローカル名に `STAGE_` プレフィックスを付けること（例: `STAGE_CHECKPOINT_DIR`）を推奨する（T3 実装時に確認すること）。

### コンパイラへの hook（`frontend/parser.rs` + `driver.rs`）

v22.1.0 では FlwDef コンパイル時の `__checkpoint_wrap` IR 注入はスコープ外。
代わりに `cmd_run` が実行前に `#[checkpoint]` stage 名を収集し、vm.rs スレッドローカルに設定する。
実際の checkpoint lookup / save は vm.rs の `__checkpoint_wrap` builtin が担う。

---

## 変更ファイル一覧

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `fav/src/ast.rs` | 更新 | `TrfDef.checkpoint: bool` 追加 |
| `fav/src/frontend/parser.rs` | 更新 | `parse_checkpoint_annotation()` 追加、`parse_trf_def` の annotation ループ更新 |
| `fav/src/backend/vm.rs` | 更新 | スレッドローカル 3 個追加、`set_*` 関数 3 個追加、`__checkpoint_wrap` builtin 追加 |
| `fav/src/driver.rs` | 更新 | `cmd_run` シグネチャ更新、ヘルパー関数 3 個追加、`v221000_tests` 追加 |
| `fav/src/main.rs` | 更新 | `--checkpoint-dir` / `--resume` フラグ追加 |
| `fav/Cargo.toml` | 更新 | `version = "22.0.0"` → `"22.1.0"` |
| `CHANGELOG.md` | 更新 | v22.1.0 エントリ追加 |
| `site/content/docs/tools/dap.mdx` | 参照のみ | checkpoint はツールドキュメントではなく CLI ドキュメントに置く |
| `site/content/docs/cli/checkpoint.mdx` | 新規 | `fav run --checkpoint-dir` / `--resume` ドキュメント |

---

## テスト一覧（v221000_tests、5 件）

| テスト名 | 内容 |
|---|---|
| `version_is_22_1_0` | Cargo.toml に `"22.1.0"` が含まれる |
| `checkpoint_annotation_parsed` | `#[checkpoint] stage Foo: Int -> Int` で `TrfDef.checkpoint == true` |
| `write_and_read_stage_checkpoint` | `write_stage_checkpoint` → `read_stage_checkpoint` でデータが一致する |
| `resume_skips_if_checkpoint_exists` | resume_dir に checkpoint ファイルが存在する場合、`read_stage_checkpoint` が Some を返す |
| `changelog_has_v22_1_0` | CHANGELOG.md に `[v22.1.0]` が含まれる |

---

## スコープ外（v22.1 では実装しない）

- Favnir パイプラインパスでの `__checkpoint_wrap` 注入（compiler.fav 変更が必要）
- checkpoint ファイルの圧縮（gzip 等）
- 分散ストレージ（S3 / DynamoDB）への checkpoint 保存（v22.3 で対応）
- `fav checkpoint` サブコマンドの拡張（既存の incremental checkpoint とは別物）
- SHA-256 チェックサムの実装（`write_stage_checkpoint` / `read_stage_checkpoint` はシンプル実装）

---

## 完了条件

- [ ] `#[checkpoint]` アノテーションが `TrfDef.checkpoint = true` としてパースされる
- [ ] `fav run --checkpoint-dir <dir>` が受け付けられる（`cmd_run` シグネチャ更新）
- [ ] `fav run --resume <dir>` が受け付けられる（`cmd_run` シグネチャ更新）
- [ ] `write_stage_checkpoint` / `read_stage_checkpoint` ヘルパー関数が存在する
- [ ] `__checkpoint_wrap` が `call_builtin` に登録されている
- [ ] `cargo test v221000 --bin fav` — 5/5 PASS
- [ ] `cargo test --bin fav` — リグレッションなし（1842 件以上合格）
- [ ] `CHANGELOG.md` に v22.1.0 エントリ
- [ ] `site/content/docs/cli/checkpoint.mdx` 作成済み
